// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

import {InvalidRecipient, ZeroAmount} from "./Errors.sol";

/// @title MockUSDC
/// @notice Freely mintable 6-decimal test collateral. No monetary value.
contract MockUSDC is ERC20 {
    constructor() ERC20("Mock USDC", "USDC") {}

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    /// @notice Test faucet. Anyone may mint to any non-zero recipient.
    function mint(address to, uint256 amount) external {
        if (to == address(0)) revert InvalidRecipient();
        if (amount == 0) revert ZeroAmount();
        _mint(to, amount);
    }
}
