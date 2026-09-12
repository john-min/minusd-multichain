// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {StdInvariant} from "forge-std/StdInvariant.sol";

import {IssuanceController} from "../src/IssuanceController.sol";
import {MinUSD} from "../src/MinUSD.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract LifecycleHandler is Test {
    uint256 internal constant UNIT = 1e6;

    IssuanceController public controller;
    MockUSDC public usdc;
    MinUSD public minusd;
    address public compliance;

    address[] public users;

    function userCount() external view returns (uint256) {
        return users.length;
    }

    uint256 public ghostSupply;
    uint256 public ghostCollateral;

    constructor(IssuanceController controller_, MockUSDC usdc_, address compliance_) {
        controller = controller_;
        usdc = usdc_;
        minusd = controller_.minusd();
        compliance = compliance_;

        users.push(makeAddr("inv-alice"));
        users.push(makeAddr("inv-bob"));
        users.push(makeAddr("inv-carol"));

        for (uint256 i = 0; i < users.length; i++) {
            usdc.mint(users[i], 1_000_000 * UNIT);
        }
    }

    function acquire(uint256 userSeed, uint256 amount) external {
        address user = users[userSeed % users.length];
        uint256 maxAmount = usdc.balanceOf(user);
        if (maxAmount == 0) return;
        amount = bound(amount, 1, maxAmount);

        vm.startPrank(user);
        usdc.approve(address(controller), amount);
        try controller.acquire(user, amount) {
            ghostSupply += amount;
            ghostCollateral += amount;
        } catch {}
        vm.stopPrank();
    }

    function redeem(uint256 userSeed, uint256 amount) external {
        address user = users[userSeed % users.length];
        uint256 maxAmount = minusd.balanceOf(user);
        if (maxAmount == 0) return;
        amount = bound(amount, 1, maxAmount);

        vm.startPrank(user);
        try controller.redeem(user, amount) {
            ghostSupply -= amount;
            ghostCollateral -= amount;
        } catch {}
        vm.stopPrank();
    }

    function transfer(uint256 fromSeed, uint256 toSeed, uint256 amount) external {
        address from = users[fromSeed % users.length];
        address to = users[toSeed % users.length];
        uint256 maxAmount = minusd.balanceOf(from);
        if (maxAmount == 0 || from == to) return;
        amount = bound(amount, 1, maxAmount);

        vm.prank(from);
        try minusd.transfer(to, amount) {} catch {}
    }
}

contract InvariantTest is StdInvariant, Test {
    MockUSDC internal usdc;
    IssuanceController internal controller;
    MinUSD internal minusd;
    LifecycleHandler internal handler;

    address internal admin = makeAddr("inv-admin");
    address internal compliance = makeAddr("inv-compliance");

    function setUp() public {
        usdc = new MockUSDC();
        controller = new IssuanceController(address(usdc), admin);
        minusd = controller.minusd();

        vm.prank(admin);
        controller.grantRole(controller.COMPLIANCE_ROLE(), compliance);

        handler = new LifecycleHandler(controller, usdc, compliance);
        targetContract(address(handler));
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
