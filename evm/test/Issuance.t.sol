// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";

import {
    AccountIsFrozen,
    CollateralAmountMismatch,
    DecimalMismatch,
    InvalidRecipient,
    NotController,
    ZeroAmount
} from "../src/Errors.sol";
import {IssuanceController} from "../src/IssuanceController.sol";
import {MinUSD} from "../src/MinUSD.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract EighteenDecimalToken is ERC20 {
    constructor() ERC20("Eighteen", "E18") {}
}

/// @dev Six-decimal ERC-20 that silently keeps 1% of each transfer.
contract FeeOnTransferSixDecimal is ERC20 {
    uint256 internal constant FEE_BPS = 100; // 1%

    constructor() ERC20("Fee USDC", "fUSDC") {}

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function transfer(address to, uint256 amount) public override returns (bool) {
        uint256 fee = (amount * FEE_BPS) / 10_000;
        uint256 sendAmount = amount - fee;
        _transfer(msg.sender, to, sendAmount);
        if (fee > 0) {
            _transfer(msg.sender, address(this), fee);
        }
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public override returns (bool) {
        _spendAllowance(from, msg.sender, amount);
        uint256 fee = (amount * FEE_BPS) / 10_000;
        uint256 sendAmount = amount - fee;
        _transfer(from, to, sendAmount);
        if (fee > 0) {
            _transfer(from, address(this), fee);
        }
        return true;
    }
}

contract IssuanceTest is Test {
    uint256 internal constant UNIT = 1e6;

    MockUSDC internal usdc;
    IssuanceController internal controller;
    MinUSD internal minusd;

    address internal admin = makeAddr("admin");
    address internal pauser = makeAddr("pauser");
    address internal compliance = makeAddr("compliance");
    address internal alice = makeAddr("alice");
    address internal bob = makeAddr("bob");
    address internal stranger = makeAddr("stranger");

    function setUp() public {
        usdc = new MockUSDC();
        controller = new IssuanceController(address(usdc), admin);
        minusd = controller.minusd();

        vm.startPrank(admin);
        controller.grantRole(controller.PAUSER_ROLE(), pauser);
        controller.grantRole(controller.COMPLIANCE_ROLE(), compliance);
        vm.stopPrank();

        usdc.mint(alice, 1_000 * UNIT);
    }

    function _acquire(address user, address to, uint256 amount) internal {
        vm.startPrank(user);
        usdc.approve(address(controller), amount);
        controller.acquire(to, amount);
        vm.stopPrank();
    }

    function _approve(address user, uint256 amount) internal {
        vm.prank(user);
        usdc.approve(address(controller), amount);
    }

    function _assertPeg() internal view {
        assertEq(usdc.balanceOf(address(controller)), minusd.totalSupply(), "controller USDC != MINUSD supply");
    }

    function test_AcquireTransferRedeem() public {
        uint256 acquireAmount = 100 * UNIT;

        _approve(alice, acquireAmount);
        vm.expectEmit(true, true, false, true, address(controller));
        emit IssuanceController.Acquired(alice, alice, acquireAmount);
        vm.prank(alice);
        controller.acquire(alice, acquireAmount);

        assertEq(minusd.balanceOf(alice), acquireAmount);
        assertEq(usdc.balanceOf(alice), 900 * UNIT);
        assertEq(usdc.balanceOf(address(controller)), acquireAmount);
        _assertPeg();

        vm.prank(alice);
        minusd.transfer(bob, 40 * UNIT);
        assertEq(minusd.balanceOf(alice), 60 * UNIT);
        assertEq(minusd.balanceOf(bob), 40 * UNIT);
        assertEq(minusd.totalSupply(), acquireAmount);
        assertEq(usdc.balanceOf(address(controller)), acquireAmount);

        uint256 redeemAmount = 25 * UNIT;
        vm.expectEmit(true, true, false, true, address(controller));
        emit IssuanceController.Redeemed(alice, alice, redeemAmount);
        vm.prank(alice);
        controller.redeem(alice, redeemAmount);

        assertEq(minusd.balanceOf(alice), 35 * UNIT);
        assertEq(usdc.balanceOf(alice), 925 * UNIT);
        assertEq(minusd.totalSupply(), 75 * UNIT);
        _assertPeg();
    }

    function test_UnauthorizedAdminPauseFreeze() public {
        bytes32 adminRole = controller.DEFAULT_ADMIN_ROLE();
        bytes32 pauserRole = controller.PAUSER_ROLE();
        bytes32 complianceRole = controller.COMPLIANCE_ROLE();

        vm.prank(stranger);
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, stranger, adminRole)
        );
        controller.grantRole(pauserRole, stranger);

        vm.prank(stranger);
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, stranger, pauserRole)
        );
        controller.pause();

        vm.prank(stranger);
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, stranger, complianceRole)
        );
        controller.freeze(alice);

        vm.prank(stranger);
        vm.expectRevert(abi.encodeWithSelector(NotController.selector, stranger));
        minusd.mint(stranger, UNIT);

        vm.prank(stranger);
        vm.expectRevert(abi.encodeWithSelector(NotController.selector, stranger));
        minusd.burn(alice, UNIT);
    }

    function test_PauseBlocksAcquireAndRedeemButNotTransfer() public {
        _acquire(alice, alice, 50 * UNIT);

        vm.prank(pauser);
        controller.pause();

        vm.startPrank(alice);
        usdc.approve(address(controller), 10 * UNIT);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        controller.acquire(alice, 10 * UNIT);

        vm.expectRevert(Pausable.EnforcedPause.selector);
        controller.redeem(alice, 10 * UNIT);

        minusd.transfer(bob, 10 * UNIT);
        vm.stopPrank();

        assertEq(minusd.balanceOf(bob), 10 * UNIT);
        assertEq(minusd.totalSupply(), 50 * UNIT);
        _assertPeg();

        vm.prank(pauser);
        controller.unpause();
        _acquire(alice, alice, 5 * UNIT);
        assertEq(minusd.totalSupply(), 55 * UNIT);
        _assertPeg();
    }

    function test_FrozenSenderAndRecipient() public {
        _acquire(alice, alice, 80 * UNIT);

        vm.prank(compliance);
        controller.freeze(alice);

        vm.startPrank(alice);
        usdc.approve(address(controller), 10 * UNIT);
        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, alice));
        controller.acquire(bob, 10 * UNIT);

        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, alice));
        minusd.transfer(bob, UNIT);

        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, alice));
        controller.redeem(alice, UNIT);
        vm.stopPrank();

        vm.prank(compliance);
        controller.unfreeze(alice);
        _acquire(alice, alice, 10 * UNIT);

        vm.prank(compliance);
        controller.freeze(bob);

        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, bob));
        minusd.transfer(bob, UNIT);

        vm.startPrank(alice);
        usdc.approve(address(controller), 10 * UNIT);
        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, bob));
        controller.acquire(bob, 10 * UNIT);

        vm.expectRevert(abi.encodeWithSelector(AccountIsFrozen.selector, bob));
        controller.redeem(bob, UNIT);
        vm.stopPrank();
    }

    function test_InsufficientMinUSD() public {
        _acquire(alice, alice, 10 * UNIT);
        vm.prank(alice);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, alice, 10 * UNIT, 11 * UNIT)
        );
        controller.redeem(alice, 11 * UNIT);
        assertEq(minusd.balanceOf(alice), 10 * UNIT);
        _assertPeg();
    }

    function test_InsufficientMockUSDC() public {
        vm.startPrank(alice);
        usdc.approve(address(controller), 2_000 * UNIT);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, alice, 1_000 * UNIT, 1_001 * UNIT)
        );
        controller.acquire(alice, 1_001 * UNIT);
        vm.stopPrank();
        assertEq(minusd.totalSupply(), 0);
        assertEq(usdc.balanceOf(address(controller)), 0);
    }

    function test_InsufficientAllowance() public {
        vm.startPrank(alice);
        usdc.approve(address(controller), 5 * UNIT);
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientAllowance.selector, address(controller), 5 * UNIT, 6 * UNIT
            )
        );
        controller.acquire(alice, 6 * UNIT);
        vm.stopPrank();
        assertEq(minusd.totalSupply(), 0);
    }

    function test_ZeroAmountAndInvalidRecipient() public {
        vm.startPrank(alice);
        usdc.approve(address(controller), 10 * UNIT);

        vm.expectRevert(ZeroAmount.selector);
        controller.acquire(alice, 0);

        vm.expectRevert(InvalidRecipient.selector);
        controller.acquire(address(0), UNIT);

        vm.expectRevert(ZeroAmount.selector);
        controller.redeem(alice, 0);

        vm.expectRevert(InvalidRecipient.selector);
        controller.redeem(address(0), UNIT);
        vm.stopPrank();

        vm.expectRevert(InvalidRecipient.selector);
        usdc.mint(address(0), UNIT);
        vm.expectRevert(ZeroAmount.selector);
        usdc.mint(alice, 0);
    }

    function test_DecimalsAreSix() public {
        assertEq(usdc.decimals(), 6);
        assertEq(minusd.decimals(), 6);
        assertEq(controller.TOKEN_DECIMALS(), 6);

        _acquire(alice, alice, 1 * UNIT);
        assertEq(minusd.balanceOf(alice), 1_000_000);
        assertEq(usdc.balanceOf(address(controller)), 1_000_000);
        assertNotEq(minusd.balanceOf(alice), 1e18);
    }

    function test_ConstructorRejectsNonSixDecimalCollateral() public {
        EighteenDecimalToken e18 = new EighteenDecimalToken();
        vm.expectRevert(abi.encodeWithSelector(DecimalMismatch.selector, uint8(6), uint8(18)));
        new IssuanceController(address(e18), admin);
    }

    function test_AcquireRejectsFeeOnTransferCollateral() public {
        FeeOnTransferSixDecimal feeToken = new FeeOnTransferSixDecimal();
        IssuanceController feeController = new IssuanceController(address(feeToken), admin);
        uint256 amount = 100 * UNIT;
        feeToken.mint(alice, amount);

        vm.startPrank(alice);
        feeToken.approve(address(feeController), amount);
        uint256 expectedReceived = amount - (amount / 100);
        vm.expectRevert(abi.encodeWithSelector(CollateralAmountMismatch.selector, amount, expectedReceived));
        feeController.acquire(alice, amount);
        vm.stopPrank();

        assertEq(feeController.minusd().totalSupply(), 0);
        assertEq(feeToken.balanceOf(address(feeController)), 0);
    }

    function test_InvariantHoldsAfterMixedSequence() public {
        _acquire(alice, alice, 100 * UNIT);
        vm.prank(alice);
        minusd.transfer(bob, 40 * UNIT);
        _acquire(alice, alice, 25 * UNIT);
        vm.prank(alice);
        controller.redeem(alice, 30 * UNIT);
        vm.prank(bob);
        minusd.transfer(alice, 10 * UNIT);
        vm.prank(alice);
        controller.redeem(bob, 20 * UNIT);

        assertEq(minusd.totalSupply(), 75 * UNIT);
        assertEq(usdc.balanceOf(address(controller)), 75 * UNIT);
        assertEq(minusd.balanceOf(alice), 45 * UNIT);
        assertEq(minusd.balanceOf(bob), 30 * UNIT);
        _assertPeg();
    }

    function test_FailedTxDoesNotPersistState() public {
        _acquire(alice, alice, 20 * UNIT);

        uint256 aliceUsdc = usdc.balanceOf(alice);
        uint256 aliceMin = minusd.balanceOf(alice);
        uint256 bobMin = minusd.balanceOf(bob);
        uint256 supply = minusd.totalSupply();
        uint256 vault = usdc.balanceOf(address(controller));

        vm.startPrank(alice);
        usdc.approve(address(controller), 2_000 * UNIT);
        vm.expectRevert();
        controller.acquire(alice, 2_000 * UNIT);

        vm.expectRevert();
        controller.redeem(alice, 21 * UNIT);

        vm.expectRevert();
        minusd.transfer(bob, 21 * UNIT);
        vm.stopPrank();

        vm.prank(compliance);
        controller.freeze(bob);
        vm.prank(alice);
        vm.expectRevert();
        minusd.transfer(bob, UNIT);

        assertEq(usdc.balanceOf(alice), aliceUsdc);
        assertEq(minusd.balanceOf(alice), aliceMin);
        assertEq(minusd.balanceOf(bob), bobMin);
        assertEq(minusd.totalSupply(), supply);
        assertEq(usdc.balanceOf(address(controller)), vault);
    }

    function test_InsufficientControllerCollateralRevertsWithoutBurn() public {
        _acquire(alice, alice, 10 * UNIT);
        // Mint extra MINUSD without depositing MockUSDC to force an insolvent redeem.
        vm.prank(address(controller));
        minusd.mint(alice, 5 * UNIT);

        uint256 aliceMin = minusd.balanceOf(alice);
        uint256 aliceUsdc = usdc.balanceOf(alice);
        uint256 vault = usdc.balanceOf(address(controller));

        vm.prank(alice);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(controller), vault, aliceMin)
        );
        controller.redeem(alice, aliceMin);

        assertEq(minusd.balanceOf(alice), aliceMin);
        assertEq(usdc.balanceOf(alice), aliceUsdc);
        assertEq(usdc.balanceOf(address(controller)), vault);
    }

    function testFuzz_AcquireThenRedeem(uint256 amount) public {
        amount = bound(amount, 1, 1_000 * UNIT);
        _acquire(alice, alice, amount);
        _assertPeg();
        vm.prank(alice);
        controller.redeem(alice, amount);
        assertEq(minusd.totalSupply(), 0);
        assertEq(usdc.balanceOf(address(controller)), 0);
        assertEq(usdc.balanceOf(alice), 1_000 * UNIT);
    }
}
