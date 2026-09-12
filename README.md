# MinUSD

MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.

Branded-dollar prototype for a fictional wallet (`MinWallet`). Phase 1 is the EVM lifecycle on local Anvil and optionally Base Sepolia.

- Product spec: [`PRD.md`](PRD.md)
- Coarse roadmap: [`.planning/ROADMAP.md`](.planning/ROADMAP.md)
- Phase 1 plan: [`.planning/phases/01-evm-lifecycle/PLAN.md`](.planning/phases/01-evm-lifecycle/PLAN.md)

## Phase 1 includes

- `MockUSDC` — 6-decimal test faucet (no monetary value)
- `MINUSD` — 6-decimal ERC-20; mint/burn only via the issuance controller
- `IssuanceController` — holds MockUSDC; acquire and redeem 1:1
- Pause of acquire/redeem only (transfers still work)
- Freeze/unfreeze via a compliance role
- Foundry tests and a deploy script

## Phase 1 does not include

- Solana / Devnet
- TypeScript CLI, reconciler, or Protocol Lab UI
- Simulated rewards / `claimYield`
- Collateral caps or a separate vault contract
- Trusted burn-and-mint bridge

## Install Foundry and run tests

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
cd evm
forge test
```

Verbose:

```bash
cd evm
forge test -vvv
```

## Local deploy (Anvil)

```bash
# terminal 1
anvil

# terminal 2 — Anvil account #0 is a well-known local test key, not a secret
cd evm
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
```

## Base Sepolia (optional)

Live deploy is **not** required. If `PRIVATE_KEY` or `BASE_SEPOLIA_RPC_URL` is unset, skip it.

```bash
cp .env.example .env   # fill placeholders with a dedicated testnet wallet
cd evm
source ../.env
forge script script/Deploy.s.sol --rpc-url "$BASE_SEPOLIA_RPC_URL" --broadcast
```

Never commit `.env`, keystores, or funded keys.

## How the lifecycle works

1. Mint MockUSDC from the faucet.
2. `MockUSDC.approve(controller, amount)`.
3. `controller.acquire(recipient, amount)` — controller pulls MockUSDC and mints the same nominal MINUSD.
4. Transfer MINUSD with the standard ERC-20 `transfer`. Pause does not block this. Frozen sender or recipient reverts.
5. `controller.redeem(recipient, amount)` — burns caller MINUSD and returns the same nominal MockUSDC.
