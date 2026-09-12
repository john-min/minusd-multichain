# Roadmap

Coarse five-phase plan. Implementation is sequential: EVM first, Solana later.

**No-leak constraint:** CLI, reconciler, Protocol Lab, simulated rewards, and the trusted bridge **must not leak into Phase 1 or Phase 2**.

## Phase 1 — EVM lifecycle (complete)

**Scope:** EVM lifecycle on **Base Sepolia / local Anvil**.

- MockUSDC faucet + MINUSD ERC-20 (6/6 decimals).
- Single issuance controller that holds MockUSDC.
- Acquire, transfer, redeem, pause (acquire/redeem only), freeze.
- Foundry unit, negative-path, and invariant tests.
- Deploy script + README. Live Base Sepolia deploy is optional if no funded key/RPC is present.

**Not in this phase:** Solana, CLI, Lab, rewards, caps, vault contract, bridge.

**Status:** Complete locally (`cd evm && forge test`).

## Phase 2 — Solana lifecycle (now)

**Scope:** Solana lifecycle on **local validator**; Devnet after tests pass (optional if no funded key/RPC).

- Native Anchor program with equivalent product flows (acquire, transfer, redeem, pause, freeze).
- Original SPL Token program (not Token-2022). Program is MINUSD mint freeze authority so frozen ATAs cannot transfer.
- Local-validator Anchor + TypeScript tests. Compare account/CPI model with the EVM implementation.

**Not in this phase:** TypeScript product CLI, Lab, reconciler, rewards, caps, bridge.

## Phase 3 — Operator surface

**Scope:** TypeScript CLI + thin Lab v1 static case study.

- CLI for balances, faucet, acquire, transfer, redeem on the chains that already exist.
- Thin Protocol Lab v1: static case study from documented evidence, not a consumer wallet.
- No requirement to finish a live Lab or reconciler service here beyond what the CLI needs.

## Phase 4 — Trusted burn-and-mint

**Scope:** Trusted burn-and-mint **only after Phases 1–2 are done**.

- Source burn + message, destination mint, replay protection.
- Explicitly trusted relayer. Not a production bridge.

## Phase 5 — Portfolio polish

- README/demo polish, representative testnet evidence, narrated walkthrough.
- Guided Lab improvements that do not reopen Phase 1 contract scope.
- Optional later: simulated rewards, caps, DeFi vault — not required to close Phase 5.

## Dependency order

```text
Phase 1 (EVM) ──► Phase 2 (Solana) ──► Phase 4 (trusted bridge)
        \              /
         └──► Phase 3 (CLI + thin Lab)
                      │
                      ▼
                 Phase 5 (polish)
```
