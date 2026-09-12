// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

import {AccountIsFrozen, AlreadyFrozen, InvalidRecipient, NotController, NotFrozen, ZeroAmount} from "./Errors.sol";

/// @title MinUSD
/// @notice 6-decimal branded dollar. Mint/burn/freeze only via the issuance controller.
/// @dev Transfers are not paused by the controller. Frozen sender or recipient always fails.
contract MinUSD is ERC20 {
    address public immutable controller;

    mapping(address account => bool frozen) private _frozen;

    event AccountFrozen(address indexed account, address indexed operator);
    event AccountUnfrozen(address indexed account, address indexed operator);

    modifier onlyController() {
        if (msg.sender != controller) revert NotController(msg.sender);
        _;
    }

    constructor(address controller_) ERC20("MinUSD", "MINUSD") {
        if (controller_ == address(0)) revert InvalidRecipient();
        controller = controller_;
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function isFrozen(address account) external view returns (bool) {
        return _frozen[account];
    }

    function mint(address to, uint256 amount) external onlyController {
        if (to == address(0)) revert InvalidRecipient();
        if (amount == 0) revert ZeroAmount();
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyController {
        if (from == address(0)) revert InvalidRecipient();
        if (amount == 0) revert ZeroAmount();
        _burn(from, amount);
    }

    function freeze(address account) external onlyController {
        if (account == address(0)) revert InvalidRecipient();
        if (_frozen[account]) revert AlreadyFrozen(account);
        _frozen[account] = true;
        emit AccountFrozen(account, msg.sender);
    }

    function unfreeze(address account) external onlyController {
        if (account == address(0)) revert InvalidRecipient();
        if (!_frozen[account]) revert NotFrozen(account);
        _frozen[account] = false;
        emit AccountUnfrozen(account, msg.sender);
    }

    function _update(address from, address to, uint256 value) internal override {
        if (from != address(0) && _frozen[from]) revert AccountIsFrozen(from);
        if (to != address(0) && _frozen[to]) revert AccountIsFrozen(to);
        super._update(from, to, value);
    }
}
