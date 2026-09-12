// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";

import {IssuanceController} from "../src/IssuanceController.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

/// @notice Deploys MockUSDC + IssuanceController (which deploys MINUSD).
/// @dev Requires PRIVATE_KEY. Skip live deploy if the key or RPC is unset.
contract Deploy is Script {
    uint256 internal constant FAUCET_AMOUNT = 1_000_000e6;

    function run() external {
        uint256 pk = vm.envUint("PRIVATE_KEY");
        address admin = vm.envOr("ADMIN_ADDRESS", vm.addr(pk));

        vm.startBroadcast(pk);
        MockUSDC usdc = new MockUSDC();
        IssuanceController controller = new IssuanceController(address(usdc), admin);
        usdc.mint(admin, FAUCET_AMOUNT);
        vm.stopBroadcast();

        console.log("MockUSDC", address(usdc));
        console.log("IssuanceController", address(controller));
        console.log("MINUSD", address(controller.minusd()));
        console.log("admin", admin);
    }
}
