# State

- **Current phase:** 2 — Solana lifecycle
- **Status:** Phase 2 complete locally (`cd solana && yarn test` — 10 passing). Phase 1 complete (`forge test` green).
- **Granularity:** Coarse
- **Blocked on:** Nothing for local acceptance. Live Solana Devnet deploy is optional and skipped without a funded key/RPC.
- **Last updated:** 2026-09-12

## Completed

- [x] PRD committed (`PRD.md`)
- [x] Coarse planning docs (this directory)
- [x] Phase 1 EVM Foundry MVP (MockUSDC, MINUSD, IssuanceController)
- [x] Phase 1 Foundry tests green
- [x] Phase 1 README + `.env.example` + `.gitignore`

## In progress

- [x] `solana/` Anchor program (equivalent acquire / transfer / redeem / pause / freeze)
- [x] Anchor + TypeScript tests against local validator
- [x] EVM-versus-Solana note from this implementation

## Deferred (must not start in Phase 2)

- TypeScript product CLI, reconciler, Protocol Lab
- Simulated rewards / `claimYield`
- Collateral caps
- Trusted burn-and-mint bridge
- Full Solutions Architecture doc set (threat model, handoff template)
