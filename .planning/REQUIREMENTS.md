# Requirements

Trimmed for **Phase 1 only**. Later-phase requirements live in `PRD.md` and must not be implemented now.

Phase 1 is the **EVM token lifecycle** on local Anvil and (optionally) Base Sepolia. CLI, reconciler, Protocol Lab, simulated rewards, collateral caps, a separate vault contract, Solana, and the trusted bridge **must not leak into Phase 1**.

## Safety (always on)

- Every user-facing document includes: “MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.”
- Never commit private keys, seed phrases, RPC tokens, or `.env`.
- `.gitignore` covers `.env`, keystores, `out/`, `cache/`, `broadcast/` secrets, and local validator state.
- `.env.example` contains placeholders only.

## Assets

- **MockUSDC:** Freely mintable test ERC-20 faucet for tests and scripts. 6 decimals. No monetary value.
- **MINUSD:** ERC-20 branded dollar. 6 decimals. Mint and burn only via the issuance controller.

## Issuance

- One issuance controller contract holds MockUSDC. No separate vault.
- **Acquire:** User approves the controller, then calls acquire/mint. Controller `transferFrom`s MockUSDC in and mints the same nominal MINUSD amount to the recipient.
- **Redeem:** Controller burns the caller’s MINUSD and returns the same nominal MockUSDC to the recipient.
- **Transfer:** Standard ERC-20. Pause does **not** block transfers.

## Controls

- Roles via OpenZeppelin AccessControl: admin, pauser, compliance operator.
- Pause/unpause: authorized pauser only. Pause stops acquire and redeem only.
- Freeze/unfreeze: authorized compliance operator only. Frozen sender or recipient cannot acquire, transfer, or redeem.
- Unauthorized privileged calls revert.

## Safety properties

- Custom errors or clear revert reasons.
- Domain events for acquire, redeem, pause, freeze (plus standard ERC-20 `Transfer`).
- Reentrancy-safe acquire/redeem (checks-effects-interactions).
- A reverted call leaves no partial state.

## Invariants

- After acquire/redeem sequences, circulating MINUSD `totalSupply` equals MockUSDC held by the controller. (Rewards do not exist yet.)
- Transfers do not change total supply or controller collateral.
- Failed operations do not persist accounting changes.

## Tooling

- Solidity + Foundry under `evm/` (`src/`, `test/`, `script/`).
- Foundry script for local / Base Sepolia deploy. Live deploy is optional; `forge test` is the acceptance bar.
- No TypeScript CLI in this phase.

## Explicitly out of Phase 1

- Solana program or Devnet work
- TypeScript CLI, reconciler, Protocol Lab UI
- Trusted burn-and-mint / bridge
- `claimYield` / simulated rewards / reward manager
- Collateral caps
- Separate vault contract
- Solutions Architecture doc set (threat model, handoff template, full ADRs)
