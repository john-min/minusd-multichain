# State

- **Current phase:** 1 — EVM lifecycle
- **Status:** Phase 1 complete locally (`forge test` green)
- **Granularity:** Coarse
- **Blocked on:** Nothing for local `forge test`. Live Base Sepolia deploy is optional and skipped without `PRIVATE_KEY` / RPC.
- **Last updated:** 2026-09-12

## Completed

- [x] PRD committed (`PRD.md`)
- [x] Coarse planning docs (this directory)

## In progress

- [x] `evm/` Foundry MVP (MockUSDC, MINUSD, IssuanceController)
- [x] Foundry tests green
- [x] README + `.env.example` + `.gitignore`

## Deferred (must not start in Phase 1)

- Solana program / Devnet
- TypeScript CLI, reconciler, Protocol Lab
- Simulated rewards / `claimYield`
- Collateral caps
- Separate vault contract
- Trusted burn-and-mint bridge
