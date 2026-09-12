// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

error ZeroAmount();
error InvalidRecipient();
error NotController(address caller);
error AccountIsFrozen(address account);
error AlreadyFrozen(address account);
error NotFrozen(address account);
error DecimalMismatch(uint8 expected, uint8 actual);
