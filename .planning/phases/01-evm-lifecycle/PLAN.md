# Phase 1 plan — EVM lifecycle

## Outcome

A reviewer can clone the repo, install Foundry, run `forge test` in `evm/`, and see acquire → transfer → redeem plus pause/freeze/failure paths. No Solana, CLI, Lab, rewards, caps, vault, or bridge code.

## Layout

```text
evm/
  foundry.toml
  remappings.txt
  src/
    MockUSDC.sol
    MinUSD.sol
    IssuanceController.sol
  test/
    Issuance.t.sol
    Invariants.t.sol
  script/
    Deploy.s.sol
```

Planning docs stay at repo-root `.planning/`.

## Contracts

### MockUSDC

- ERC-20, 6 decimals, name/symbol suitable for a test faucet (`Mock USDC` / `USDC` or `MockUSDC`).
- Public `mint(to, amount)` for tests and scripts. No monetary value.

### MinUSD

- ERC-20, 6 decimals, symbol `MINUSD`.
- Mint and burn callable only by the issuance controller.
- Freeze mapping enforced in `_update` so frozen sender **or** recipient cannot transfer, and frozen accounts cannot be minted to or burned from (covers acquire/redeem).
- Pause is **not** implemented on the token. Transfers work while the controller is paused.

### IssuanceController

- Deploys `MinUSD` in its constructor (avoids circular addresses) and takes an existing MockUSDC.
- Holds all MockUSDC collateral. No vault contract.
- Roles (`AccessControl`):
  - `DEFAULT_ADMIN_ROLE` — grant/revoke roles
  - `PAUSER_ROLE` — pause/unpause acquire and redeem
  - `COMPLIANCE_ROLE` — freeze/unfreeze (forwards to MinUSD)
- `acquire(recipient, amount)`: `nonReentrant` + `whenNotPaused`; `transferFrom` MockUSDC from caller, then mint the same nominal MINUSD. Require amount > 0 and recipient ≠ 0.
- `redeem(recipient, amount)`: `nonReentrant` + `whenNotPaused`; burn caller MINUSD first (effect), then transfer the same nominal MockUSDC (interaction).
- Custom errors; events `Acquired`, `Redeemed`, plus OZ `Paused`/`Unpaused` and freeze events.
- Constructor checks collateral decimals == 6 so 6/6 cannot silently become 18.

## Acquire / redeem ordering

- Guard with `ReentrancyGuard`.
- Redeem: burn MINUSD (effect) then `safeTransfer` MockUSDC (interaction).
- Acquire: pull MockUSDC then mint MINUSD, or mint then pull, inside the reentrancy lock. Prefer pull-then-mint so insolvent pulls fail before supply changes; a revert still rolls back both.
- One transaction, all-or-nothing.

## Tests (`forge test` must pass)

| Case | Expect |
| --- | --- |
| Happy acquire / transfer / redeem | Balances and events match; 1:1 nominal |
| Unauthorized admin / pause / freeze | Revert |
| Paused acquire and redeem | Revert; **transfers still succeed** |
| Frozen sender and recipient | Acquire, transfer, redeem revert |
| Insufficient MINUSD / MockUSDC / allowance | Revert |
| Zero amount and invalid recipient | Revert |
| Decimals | Both tokens report 6; `1e6` is one whole unit |
| Invariant | Controller MockUSDC == MINUSD `totalSupply` after mixed acquire/redeem/transfer |
| Failed tx | Snapshot balances unchanged after revert |

Use `vm.expectRevert` and `vm.snapshot` / try-catch style checks so failed calls prove no partial writes.

## Scripts and docs

- `Deploy.s.sol`: deploy MockUSDC + controller (which deploys MINUSD); mint a faucet amount to the deployer on local/testnet.
- README: disclaimer, Foundry install, `forge test`, Phase 1 includes/excludes, pointers to `PRD.md` and `.planning/ROADMAP.md`.
- Document Base Sepolia commands. If `PRIVATE_KEY` or RPC is missing, skip live deploy.

## Acceptance

- `cd evm && forge test` exits 0.
- No secrets in git.
- Phase 1 files do not add `solana/`, `client/`, `portfolio/`, rewards, caps, or bridge contracts.
