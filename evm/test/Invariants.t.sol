// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {StdInvariant} from "forge-std/StdInvariant.sol";
import {StdUtils} from "forge-std/StdUtils.sol";
import {Vm} from "forge-std/Vm.sol";

import {IssuanceController} from "../src/IssuanceController.sol";
import {MinUSD} from "../src/MinUSD.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract LifecycleHandler is StdUtils {
    uint256 internal constant UNIT = 1e6;
    Vm internal constant vm = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));

    IssuanceController public controller;
    MockUSDC public usdc;
    MinUSD public minusd;

    address[] public users;

    uint256 public ghostSupply;
    uint256 public ghostCollateral;

    constructor(IssuanceController controller_, MockUSDC usdc_) {
        controller = controller_;
        usdc = usdc_;
        minusd = controller_.minusd();

        users.push(address(uint160(uint256(keccak256("inv-alice")))));
        users.push(address(uint160(uint256(keccak256("inv-bob")))));
        users.push(address(uint160(uint256(keccak256("inv-carol")))));

        for (uint256 i = 0; i < users.length; i++) {
            usdc.mint(users[i], 1_000_000 * UNIT);
        }
    }

    function userCount() external view returns (uint256) {
        return users.length;
    }

    function acquire(uint256 userSeed, uint256 amount) external {
        address user = users[userSeed % users.length];
        uint256 maxAmount = usdc.balanceOf(user);
        if (maxAmount == 0) return;
        amount = bound(amount, 1, maxAmount);

        // Valid calls must succeed. Unexpected reverts fail the invariant campaign
        // because foundry.toml sets fail_on_revert = true.
        vm.startPrank(user);
        usdc.approve(address(controller), amount);
        controller.acquire(user, amount);
        vm.stopPrank();

        ghostSupply += amount;
        ghostCollateral += amount;
    }

    function redeem(uint256 userSeed, uint256 amount) external {
        address user = users[userSeed % users.length];
        uint256 maxAmount = minusd.balanceOf(user);
        if (maxAmount == 0) return;
        amount = bound(amount, 1, maxAmount);

        vm.startPrank(user);
        controller.redeem(user, amount);
        vm.stopPrank();

        ghostSupply -= amount;
        ghostCollateral -= amount;
    }

    function transferTo(uint256 fromSeed, uint256 toSeed, uint256 amount) external {
        address from = users[fromSeed % users.length];
        address to = users[toSeed % users.length];
        uint256 maxAmount = minusd.balanceOf(from);
        if (maxAmount == 0 || from == to) return;
        amount = bound(amount, 1, maxAmount);

        vm.prank(from);
        minusd.transfer(to, amount);
    }
}

contract InvariantTest is StdInvariant, Test {
    MockUSDC internal usdc;
    IssuanceController internal controller;
    MinUSD internal minusd;
    LifecycleHandler internal handler;

    address internal admin;
    address internal compliance;

    function setUp() public {
        admin = makeAddr("inv-admin");
        compliance = makeAddr("inv-compliance");

        usdc = new MockUSDC();
        controller = new IssuanceController(address(usdc), admin);
        minusd = controller.minusd();

        bytes32 complianceRole = controller.COMPLIANCE_ROLE();
        vm.prank(admin);
        controller.grantRole(complianceRole, compliance);

        handler = new LifecycleHandler(controller, usdc);
        targetContract(address(handler));

        bytes4[] memory selectors = new bytes4[](3);
        selectors[0] = LifecycleHandler.acquire.selector;
        selectors[1] = LifecycleHandler.redeem.selector;
        selectors[2] = LifecycleHandler.transferTo.selector;
        targetSelector(FuzzSelector({addr: address(handler), selectors: selectors}));
    }

    function invariant_controllerCollateralEqualsTotalSupply() public view {
        assertEq(usdc.balanceOf(address(controller)), minusd.totalSupply());
        assertEq(handler.ghostSupply(), minusd.totalSupply());
        assertEq(handler.ghostCollateral(), usdc.balanceOf(address(controller)));
    }

    function invariant_userBalancesSumToSupply() public view {
        uint256 sum;
        uint256 n = handler.userCount();
        for (uint256 i = 0; i < n; i++) {
            sum += minusd.balanceOf(handler.users(i));
        }
        assertEq(sum, minusd.totalSupply());
    }
}
