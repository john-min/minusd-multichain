# ADR 0001 — Use original SPL Token, not Token-2022

**Status:** Accepted (Phase 2)
**Date:** 2026-09-12

## Context

PRD §11.2 and the Phase 2 closed decisions require a documented choice between the original SPL Token program and Token-2022. Phase 1 freeze is an owner-level mapping on `MinUSD` that rejects frozen senders and recipients on every ERC-20 movement. On Solana, the equivalent product rule is “frozen accounts cannot acquire, transfer, or redeem MINUSD.”

## Decision

Use the **original SPL Token program** (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`) for MockUSDC and MINUSD.

Do **not** use Token-2022, transfer hooks, or default-account-state extensions in Phase 2.

## Why

1. **Equivalent product behavior does not need Token-2022.** Mint, burn, transfer, and mint freeze/thaw are enough. Transfer hooks would let the program see every transfer, but they add another program, extra accounts, and a different failure surface that Phase 1 does not have.
2. **Freeze can match EVM without hooks.** The program is the MINUSD mint freeze authority. `freeze` / `unfreeze` CPI into `spl_token::freeze_account` / `thaw_account` on the user’s MINUSD token account, and a `FrozenOwner` PDA keyed by owner covers acquire/redeem the way `_frozen` does on EVM.
3. **Classic Token is the well-trodden test path.** `@solana/spl-token` + `anchor-spl::token` + local validator coverage is simpler and less flaky than Token-2022 extensions for a first Solana slice.
4. **Honest limitation is acceptable.** Freeze is per token account. A later-created ATA is not auto-frozen. That is documented in the Phase 2 plan. Token-2022 transfer hooks would close that gap; they are a future consideration, not a Phase 2 requirement.

## Consequences

- Ordinary MINUSD transfers are raw `spl_token::transfer` and never enter the MinUSD program. Pause cannot intercept them (and must not, per the pause-scope decision).
- Tests must freeze the ATA they later transfer, and must pass `FrozenOwner` PDAs into acquire/redeem.
- A future Token-2022 migration would change mint addresses, client account lists, and freeze semantics; it is out of Phase 2.

## Alternatives considered

| Option | Why not now |
| --- | --- |
| Token-2022 + transfer hook | Would intercept raw transfers in-program, but is extra surface and not needed if mint freeze authority is used. |
| Program `transfer_minusd` only | Weaker: raw SPL transfers would bypass policy. Only acceptable if documented as such. Rejected so the invariant matches EVM. |
| PDA freeze list only | Users can still `spl_token::transfer`. Does not satisfy “frozen accounts cannot transfer MINUSD.” |
