// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

import {AccountIsFrozen, CollateralAmountMismatch, DecimalMismatch, InvalidRecipient, ZeroAmount} from "./Errors.sol";
import {MinUSD} from "./MinUSD.sol";

/// @title IssuanceController
/// @notice Holds MockUSDC and mints/burns MINUSD 1:1. Pause blocks acquire/redeem only.
contract IssuanceController is AccessControl, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    bytes32 public constant COMPLIANCE_ROLE = keccak256("COMPLIANCE_ROLE");

    uint8 public constant TOKEN_DECIMALS = 6;

    IERC20 public immutable collateral;
    MinUSD public immutable minusd;

    event Acquired(address indexed caller, address indexed recipient, uint256 amount);
    event Redeemed(address indexed caller, address indexed recipient, uint256 amount);
    event AccountFrozen(address indexed account, address indexed operator);
    event AccountUnfrozen(address indexed account, address indexed operator);

    constructor(address collateral_, address admin_) {
        if (collateral_ == address(0) || admin_ == address(0)) revert InvalidRecipient();

        uint8 decimals_ = IERC20Metadata(collateral_).decimals();
        if (decimals_ != TOKEN_DECIMALS) revert DecimalMismatch(TOKEN_DECIMALS, decimals_);

        collateral = IERC20(collateral_);
        minusd = new MinUSD(address(this));

        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(PAUSER_ROLE, admin_);
        _grantRole(COMPLIANCE_ROLE, admin_);
    }

    /// @notice Deposit MockUSDC and mint the same nominal MINUSD amount.
    function acquire(address recipient, uint256 amount) external nonReentrant whenNotPaused {
        if (amount == 0) revert ZeroAmount();
        if (recipient == address(0)) revert InvalidRecipient();
        _requireNotFrozen(msg.sender);
        _requireNotFrozen(recipient);

        // Pull collateral first so a failed transfer cannot mint. Require the
        // observed balance delta equals `amount` so fee-on-transfer tokens cannot
        // mint undercollateralized MINUSD.
        uint256 beforeBalance = collateral.balanceOf(address(this));
        collateral.safeTransferFrom(msg.sender, address(this), amount);
        uint256 received = collateral.balanceOf(address(this)) - beforeBalance;
        if (received != amount) revert CollateralAmountMismatch(amount, received);

        minusd.mint(recipient, amount);
        emit Acquired(msg.sender, recipient, amount);
    }

    /// @notice Burn caller MINUSD and return the same nominal MockUSDC amount.
    function redeem(address recipient, uint256 amount) external nonReentrant whenNotPaused {
        if (amount == 0) revert ZeroAmount();
        if (recipient == address(0)) revert InvalidRecipient();
        _requireNotFrozen(msg.sender);
        _requireNotFrozen(recipient);

        // Burn before releasing collateral (checks-effects-interactions). Require
        // the recipient's observed balance increase equals `amount`.
        minusd.burn(msg.sender, amount);
        uint256 beforeBalance = collateral.balanceOf(recipient);
        collateral.safeTransfer(recipient, amount);
        uint256 delivered = collateral.balanceOf(recipient) - beforeBalance;
        if (delivered != amount) revert CollateralAmountMismatch(amount, delivered);

        emit Redeemed(msg.sender, recipient, amount);
    }

    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    function freeze(address account) external onlyRole(COMPLIANCE_ROLE) {
        minusd.freeze(account);
        emit AccountFrozen(account, msg.sender);
    }

    function unfreeze(address account) external onlyRole(COMPLIANCE_ROLE) {
        minusd.unfreeze(account);
        emit AccountUnfrozen(account, msg.sender);
    }

    function isFrozen(address account) external view returns (bool) {
        return minusd.isFrozen(account);
    }

    function collateralBalance() external view returns (uint256) {
        return collateral.balanceOf(address(this));
    }

    function _requireNotFrozen(address account) internal view {
        if (minusd.isFrozen(account)) revert AccountIsFrozen(account);
    }
}
