# Phase 2 plan — Solana lifecycle

## Outcome

A reviewer can install the pinned Anchor/Solana toolchain, run `cd solana && yarn test` (or `./scripts/test.sh`), and see acquire → transfer → redeem plus pause/freeze/failure paths with the same product rules as Phase 1. No TypeScript product CLI, Lab, reconciler, rewards, caps, or bridge.

## Layout

```text
solana/
  Anchor.toml
  Cargo.toml
  package.json
  programs/minusd/src/   # lib.rs + state.rs + errors.rs
  tests/minusd.ts        # Anchor + TypeScript test client (not the Phase 3 CLI)
  keys/minusd-keypair.json
  scripts/test.sh
docs/adr/0001-spl-token-not-token-2022.md
docs/evm-vs-solana.md
```

## Program

Native Anchor program. Persistent state lives in PDAs and SPL token accounts; the program itself is stateless.

### Accounts

| Account | Seeds / notes |
| --- | --- |
| Config PDA | `[b"config"]` — admin, pauser, compliance, pause flag, mint/vault pubkeys, bumps |
| Vault authority PDA | `[b"vault_authority"]` — signer only (mint + freeze + vault authority). No data. |
| MockUSDC mint | `[b"mock_usdc_mint"]` — 6 decimals; mint authority = vault authority |
| MINUSD mint | `[b"minusd_mint"]` — 6 decimals; mint + freeze authority = vault authority |
| Vault token account | `[b"vault"]` — MockUSDC; authority = vault authority |
| FrozenOwner PDA | `[b"frozen", owner]` — owner-level flag used on acquire/redeem |

### Instructions

- `initialize(admin, pauser, compliance)` — one-time setup; creates mints and vault.
- `mint_mock_usdc(amount)` — public test faucet. No monetary value.
- `acquire(amount)` — caller’s MockUSDC token account → vault (CPI transfer); program mints the same nominal MINUSD to the recipient token account. **No ERC-20 approve.** Caller is the MockUSDC token-account owner and signs.
- `redeem(amount)` — burn caller MINUSD, then CPI-transfer the same nominal MockUSDC from the vault.
- `pause` / `unpause` — pauser only. Blocks acquire and redeem only.
- `freeze` / `unfreeze` — compliance only. Sets `FrozenOwner` **and** CPIs `spl_token::freeze_account` / `thaw_account` on the supplied MINUSD token account.
- `set_pauser` / `set_compliance` — admin only (role updates; no OpenZeppelin AccessControl equivalent).

Ordinary MINUSD transfers use the **original SPL Token program** (`spl_token::transfer`). They never enter this program. Pause must not freeze token accounts.

### Freeze (honest)

Classic SPL transfers do not call this program. A PDA flag alone would not stop `spl_token::transfer`.

**Chosen design (recommended path):**

1. The program is the MINUSD mint **freeze authority**.
2. `freeze` writes `FrozenOwner.frozen = true` (owner-keyed, matches EVM `_frozen`) **and** freezes the MINUSD token account passed in.
3. Acquire/redeem always take the caller and recipient `FrozenOwner` PDAs (empty = not frozen) and reject if `frozen`.
4. Raw SPL transfers of a frozen token account fail inside the Token program (`AccountFrozen`). That is how “frozen accounts cannot transfer MINUSD” holds for the ATA we froze.

**Limitation:** SPL freeze is per token account. A later-created MINUSD ATA for the same owner is not auto-frozen. Acquire/redeem still fail via `FrozenOwner`. Compliance can freeze another ATA only after unfreeze, or we would need an extra “freeze additional ATA” ix (not in this phase). Tests use the ATA frozen at `freeze` time.

Do not use Token-2022 transfer hooks. See `docs/adr/0001-spl-token-not-token-2022.md`.

### Product rules (must match Phase 1)

- 6 decimals on both mints. One whole unit is `1_000_000`.
- Pause stops acquire/redeem; transfers still succeed.
- Frozen caller or recipient fails acquire, transfer (frozen ATA), and redeem.
- Unauthorized privileged ix fail.
- Zero amount and invalid recipient/accounts fail.
- After acquire/redeem/transfer sequences: vault MockUSDC amount == MINUSD `supply`.
- Failed instructions leave no partial accounting (Solana tx atomicity; no pre-ix writes that survive failure).
- Custom errors + `msg!` / events sufficient to diagnose a failure.

### Deviations from the suggested sketch

- Mints and vault are PDAs created in `initialize` (deterministic; no client-created mint keys).
- Transfers are raw SPL Token transfers, not a program `transfer_minusd`. Freeze uses mint freeze authority so that path is actually blocked.
- A `FrozenOwner` PDA is kept **in addition** to SPL freeze so acquire/redeem match EVM’s owner-level mapping even when an ATA is missing or substituted.
- Instruction handlers and `#[derive(Accounts)]` structs live in `lib.rs` (not split instruction modules). Anchor 0.31’s `#[program]` client-account imports resolve to `crate::<ix_name>`; a split module layout fought that. `state.rs` / `errors.rs` remain separate.

## Tests (`anchor test` / `yarn test` must pass)

| Case | Expect |
| --- | --- |
| Happy acquire / transfer / redeem | Balances match; 1:1 nominal; peg holds |
| Unauthorized admin / pause / freeze | Fail |
| Paused acquire and redeem | Fail; **SPL transfers still succeed** |
| Frozen sender and recipient | Acquire, SPL transfer, redeem fail |
| Insufficient MINUSD / MockUSDC | Fail; balances unchanged |
| Zero amount and invalid accounts | Fail |
| Decimals | Both mints report 6; `1e6` is one whole unit; not 9 |
| Invariant | Vault MockUSDC == MINUSD supply after mixed flows |
| Failed ix | Snapshot balances unchanged |

The TypeScript file under `solana/tests/` is a **test client**, not the Phase 3 product CLI.

## Tooling

- Pin **Anchor 0.31.1** and **Solana CLI 2.1.0** in README and `Anchor.toml`.
- GitHub Actions workflow for `solana/` if it can be made reliable (pinned versions, cargo/solana cache).
- Never commit funded keypairs, `.env`, or `id.json`. The program identity keypair in `solana/keys/` has no funds and is committed so the program ID stays stable.
- Devnet deploy is optional. If no funded key/RPC, skip it. Local `anchor test` is the acceptance bar.

## Explicitly out of Phase 2

- TypeScript product CLI, reconciler, Protocol Lab UI
- Trusted burn-and-mint / bridge
- Simulated rewards / `claimYield`
- Collateral caps
- Full Solutions Architecture doc set (threat model, handoff template)

## Acceptance

- `cd solana && ./scripts/test.sh` (or `yarn test`) exits 0.
- Equivalent acquire / transfer / redeem / pause / freeze behavior, with the freeze story above.
- Planning files mark Phase 2 in progress and Phase 1 complete.
- `docs/evm-vs-solana.md` is written from this implementation.
- No secrets in git.
- Phase 3+ not started.
