# MinUSD

MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.

Branded-dollar prototype for a fictional wallet (`MinWallet`). Phase 1 is the EVM lifecycle on local Anvil (optionally Base Sepolia). Phase 2 is the equivalent Solana lifecycle on a local validator (optionally Devnet).

- Product spec: [`PRD.md`](PRD.md)
- Coarse roadmap: [`.planning/ROADMAP.md`](.planning/ROADMAP.md)
- Phase 1 plan: [`.planning/phases/01-evm-lifecycle/PLAN.md`](.planning/phases/01-evm-lifecycle/PLAN.md)
- Phase 2 plan: [`.planning/phases/02-solana-lifecycle/PLAN.md`](.planning/phases/02-solana-lifecycle/PLAN.md)
- EVM versus Solana (from this repo): [`docs/evm-vs-solana.md`](docs/evm-vs-solana.md)

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

## Phase 2 includes

- Anchor program under `solana/programs/minusd` with equivalent acquire / redeem / pause / freeze
- MockUSDC and MINUSD mints at **6 decimals**; program PDA is mint (and MINUSD freeze) authority
- Acquire via SPL transfer CPI into a program vault — **no ERC-20 approve**
- Ordinary MINUSD transfers via the original SPL Token program (pause does not block them)
- Freeze: `FrozenOwner` PDA **plus** `spl_token::freeze_account` on the supplied MINUSD ATA
- TypeScript **test client** under `solana/tests/` (not the Phase 3 product CLI)

## Phase 2 does not include

- TypeScript product CLI, reconciler, or Protocol Lab UI
- Simulated rewards / `claimYield`
- Collateral caps
- Trusted burn-and-mint bridge
- Token-2022 / transfer hooks

## Install Foundry and run EVM tests

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

## Local EVM deploy (Anvil)

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

## How the EVM lifecycle works

1. Mint MockUSDC from the faucet.
2. `MockUSDC.approve(controller, amount)`.
3. `controller.acquire(recipient, amount)` — controller pulls MockUSDC and mints the same nominal MINUSD.
4. Transfer MINUSD with the standard ERC-20 `transfer`. Pause does not block this. Frozen sender or recipient reverts.
5. `controller.redeem(recipient, amount)` — burns caller MINUSD and returns the same nominal MockUSDC.

## Solana toolchain (pinned)

| Tool | Version |
| --- | --- |
| Anchor CLI / `anchor-lang` | **0.31.1** |
| Solana / Agave CLI | **2.1.0** |
| Node | 20+ (tests use Yarn) |

```bash
# Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.0/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Anchor CLI (prebuilt)
mkdir -p "$HOME/.local/bin"
curl -sSfL https://github.com/solana-foundation/anchor/releases/download/v0.31.1/anchor-0.31.1-x86_64-unknown-linux-gnu \
  -o "$HOME/.local/bin/anchor"
chmod +x "$HOME/.local/bin/anchor"
export PATH="$HOME/.local/bin:$PATH"

solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json   # local fee-payer only
solana --version   # solana-cli 2.1.0
anchor --version   # anchor-cli 0.31.1
```

macOS / other arches: install Anchor via `avm` (`avm install 0.31.1 && avm use 0.31.1`) if there is no matching prebuilt binary.

## Run Solana tests

```bash
cd solana
yarn install
yarn test
# equivalent: ./scripts/test.sh
# equivalent: mkdir -p target/deploy && cp keys/minusd-keypair.json target/deploy/minusd-keypair.json && anchor test
```

`yarn test` copies the committed program identity keypair into `target/deploy/` (gitignored) so `declare_id!` matches, then runs `anchor test` against `solana-test-validator`.

## How the Solana lifecycle works

1. `initialize` creates the MockUSDC mint, MINUSD mint, vault token account, and config PDA.
2. `mint_mock_usdc` is the public test faucet.
3. `acquire` — the caller signs; the program CPIs an SPL transfer of MockUSDC into the vault and mints the same nominal MINUSD. No approve instruction.
4. Transfer MINUSD with a normal `spl_token::transfer`. Pause does not block this.
5. `redeem` burns caller MINUSD and releases the same nominal MockUSDC from the vault.
6. `freeze` (compliance) sets a `FrozenOwner` PDA **and** freezes the supplied MINUSD token account via the mint freeze authority. Raw SPL transfers of that ATA fail inside the Token program.

## Solana Devnet (optional)

Live deploy is **not** required. Local `anchor test` is the acceptance bar. If you have a dedicated Devnet wallet and RPC:

```bash
solana config set --url devnet
solana airdrop 2
cd solana
yarn prepare-keys
anchor deploy --provider.cluster devnet
```

Skip this if `id.json` is missing or unfunded. Never commit that keypair.

## How freeze actually works

Classic SPL transfers never enter the MinUSD program. A PDA flag alone would not stop `spl_token::transfer`. This implementation:

1. Makes the program the MINUSD mint freeze authority.
2. Records `FrozenOwner` (owner-keyed) so acquire/redeem match EVM’s `_frozen` mapping.
3. CPIs `spl_token::freeze_account` on the ATA passed to `freeze`.

A later-created MINUSD ATA for the same owner is not auto-frozen. Acquire/redeem still fail via `FrozenOwner`. See [`docs/adr/0001-spl-token-not-token-2022.md`](docs/adr/0001-spl-token-not-token-2022.md).
