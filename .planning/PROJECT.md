# MinUSD — Project

**Product:** MinUSD (`MINUSD`)
**Repo:** `minusd-multichain`
**Status:** Phase 1 in progress
**Networks (eventual):** Base Sepolia and Solana Devnet
**Source of truth:** [`PRD.md`](../PRD.md)

## What this is

MinUSD is an educational, testnet-only branded-dollar prototype. It shows how a fictional wallet (`MinWallet`) could issue a programmable dollar against mock collateral, enforce pause/freeze controls, and later compare equivalent EVM and Solana implementations.

MinUSD is not a real stablecoin. It has no monetary value, is not backed by reserves, is unaudited, and is not redeemable for fiat.

## Goals

- Ship a correct single-chain EVM lifecycle first (acquire, transfer, redeem, pause, freeze).
- Later ship a native Solana equivalent, then a thin operator surface, then a trusted burn-and-mint demo, then portfolio polish.
- Keep each phase independently reviewable. Do not leak later-phase work into earlier phases.

## Non-goals (whole project)

- Real fiat, USDC, or any asset with monetary value.
- Mainnet deployment.
- Production bridges, oracles, KYC, AMMs, or proof of reserves.
- Implying affiliation with M0 or any issuer.

## Phase 1 closed decisions

Recorded here so implementation does not reopen them:

| Decision | Choice |
| --- | --- |
| Decimals | MockUSDC and MINUSD both use **6 decimals**. No 18-decimal hidden conversion. |
| Pause scope | Pause stops **acquisition and redemption only**. Ordinary transfers still work. |
| Collateral custody | **One issuance controller** holds MockUSDC. No separate vault contract. |
| Rewards | **Skip** `claimYield` / simulated rewards in Phase 1. |
| Caps | **Skip** collateral caps in Phase 1. |
| Operator UI | **No CLI / Lab / reconciler** in Phase 1. Foundry scripts + README are enough. |

## Stack (Phase 1)

- Solidity + Foundry under `evm/`
- OpenZeppelin ERC-20, AccessControl, Pausable, ReentrancyGuard
- Local Anvil for development; Base Sepolia deploy script (live deploy is optional)

## Safety

Never commit private keys, seed phrases, RPC tokens, or `.env`. Every user-facing document must carry the educational disclaimer.
