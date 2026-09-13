# Requirements

Phase 1 (EVM) is complete. This file now covers **Phase 1 (closed) + Phase 2 (Solana lifecycle)**. Later-phase requirements still live in `PRD.md` and must not be implemented now.

Phase 1 no-leak rules stay in force for the EVM tree: CLI, reconciler, Protocol Lab, simulated rewards, collateral caps, a separate vault contract, and the trusted bridge **must not leak into Phase 1**. Solana is Phase 2 only.

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

---

## Phase 2 — Solana lifecycle

Phase 2 is the **Solana token lifecycle** on a local validator and (optionally) Devnet. It must match Phase 1 product semantics. The TypeScript product CLI, reconciler, Protocol Lab, simulated rewards, collateral caps, and trusted bridge **must not leak into Phase 2**. A TypeScript **test client** under `solana/tests/` is required and is not the Phase 3 CLI.

### Assets

- **MockUSDC mint:** Freely mintable test faucet. 6 decimals. No monetary value.
- **MINUSD mint:** 6 decimals. Mint/burn authority is the program (PDA), not users.

### Issuance

- Program-controlled vault token account holds MockUSDC.
- **Acquire:** User sends MockUSDC into the vault (SPL transfer CPI; user is token-account owner). Program mints the same nominal MINUSD to the recipient. No ERC-20 approve.
- **Redeem:** Burn caller MINUSD and release the same nominal MockUSDC from the vault.
- **Transfer:** Ordinary SPL Token transfer of MINUSD. Pause does **not** block transfers.

### Controls

- Roles on a Config PDA: admin, pauser, compliance — separate where practical.
- Pause/unpause: authorized pauser only. Pause stops acquire and redeem only.
- Freeze/unfreeze: authorized compliance operator only. Frozen sender or recipient cannot acquire, transfer, or redeem MINUSD.
- Freeze implementation: program is the MINUSD mint freeze authority; `freeze`/`unfreeze` CPI into classic SPL Token **and** a `FrozenOwner` PDA keyed by owner so acquire/redeem match EVM’s `_frozen` mapping. Do not use Token-2022 transfer hooks.
- Unauthorized privileged instructions fail.

### Safety properties

- Custom errors and logs sufficient to diagnose a failure.
- Events (or equivalent Anchor events) for acquire, redeem, pause, freeze.
- Failed instructions leave no partial accounting (Solana transaction atomicity).

### Invariants

- After acquire/redeem sequences, circulating MINUSD supply equals MockUSDC held in the vault. (Rewards do not exist yet.)
- Transfers do not change total supply or vault collateral.
- Failed operations do not persist accounting changes.

### Tooling

- Rust + Anchor under `solana/` (`programs/`, `tests/`).
- Original SPL Token program, not Token-2022.
- Local `anchor test` is the acceptance bar. Devnet deploy is optional.
- No TypeScript product CLI in this phase.

### Explicitly out of Phase 2

- TypeScript product CLI, reconciler, Protocol Lab UI
- Trusted burn-and-mint / bridge
- `claimYield` / simulated rewards / reward manager
- Collateral caps
- Full Solutions Architecture doc set (threat model, handoff template)
