# EVM versus Solana — notes from this repo

MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.

This is not a generic architecture essay. It is what differed while implementing Phase 1 (`evm/`) and Phase 2 (`solana/programs/minusd`) against the same product rules.

## Approve + `transferFrom` versus CPI / token accounts

On EVM, acquire is two user actions: `MockUSDC.approve(controller, amount)` then `controller.acquire`. The controller is a contract address that already exists; `transferFrom` pulls tokens because the user granted an allowance.

On Solana there is no allowance for this flow. The caller **owns** a MockUSDC token account and **signs** `acquire`. The program CPIs `spl_token::transfer_checked` with that signer as authority, then CPIs `mint_to_checked` with the vault-authority PDA. One transaction, no approve instruction. The “vault” is not the program’s key — it is a token account whose authority is a PDA.

That is why Phase 2 tests never call `approve`. If you look for it, you are still thinking in ERC-20.

## Contract storage versus PDAs

`IssuanceController` stores admin/pauser/compliance, the pause flag, and token addresses in contract storage. `MinUSD` stores `_frozen[address]`.

The Solana program stores nothing in itself. `Config` is a PDA (`seeds = ["config"]`). Freeze-by-owner is another PDA (`seeds = ["frozen", owner]`). Mints and the vault are also PDAs so tests and clients can derive them. Every instruction must be handed the accounts it will read or write; a missing or substituted account fails a constraint (`has_one`, `token::mint`, seeds) before our custom errors run.

## Pause and freeze

Pause is similar in spirit: a flag that gates acquire/redeem only. On EVM that flag lives on the controller (`Pausable`). On Solana it lives on `Config`. Ordinary MINUSD transfers never consult it — ERC-20 `transfer` does not call the controller, and `spl_token::transfer` does not call our program.

Freeze is where the models diverge.

- EVM: `_update` on `MinUSD` sees every mint, burn, and transfer. Frozen sender **or** recipient reverts. One mapping, one code path.
- Solana, classic Token: a raw transfer never reaches our program. A `FrozenOwner` PDA is enough for `acquire` / `redeem` (we require those PDAs and reject if `frozen`). It is **not** enough for transfers. So the program is also the MINUSD mint freeze authority and `freeze` CPIs `spl_token::freeze_account` on the ATA we are given. The Token program then rejects transfers of that account (`AccountFrozen` / `0x11`).

SPL freeze is per token account, not per wallet. EVM freeze is per address. Tests freeze the ATA they later transfer. A later-created ATA for the same owner would not be frozen until compliance passes it in (the owner-level PDA still blocks acquire/redeem). That limitation is real; we did not paper over it with Token-2022 transfer hooks.

## One thing that surprised me

I expected “put a freeze bit on a PDA” to be the Solana equivalent of `_frozen`. It is, for instructions we write. It is not, for the transfer path users actually use. Until the mint freeze-authority CPI was in place, a frozen owner could still `spl_token::transfer` and the tests would have been lying if they only called a program `transfer_minusd`. Matching Phase 1 meant freezing the token account the Token program already knows how to stop — and documenting that this is per-ATA, not per-wallet.
