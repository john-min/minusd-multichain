# Roadmap

Coarse five-phase plan. Implementation is sequential: EVM first, Solana later.

**No-leak constraint:** CLI, reconciler, Protocol Lab, simulated rewards, and the trusted bridge **must not leak into Phase 1**.

## Phase 1 — EVM lifecycle (now)

**Scope:** EVM lifecycle on **Base Sepolia / local Anvil**.

- MockUSDC faucet + MINUSD ERC-20 (6/6 decimals).
- Single issuance controller that holds MockUSDC.
- Acquire, transfer, redeem, pause (acquire/redeem only), freeze.
- Foundry unit, negative-path, and invariant tests.
- Deploy script + README. Live Base Sepolia deploy is optional if no funded key/RPC is present.

**Not in this phase:** Solana, CLI, Lab, rewards, caps, vault contract, bridge.

## Phase 2 — Solana lifecycle

**Scope:** Solana lifecycle on **Devnet** (do not build it in Phase 1).

- Native Anchor (or documented equivalent) program with equivalent product flows.
- Devnet after local-validator tests pass.
- Compare account/CPI model with the EVM implementation.

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
