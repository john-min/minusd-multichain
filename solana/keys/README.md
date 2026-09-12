# Program identity keypair

`minusd-keypair.json` is the **program identity** used so `declare_id!` stays stable across `anchor build` / `anchor test`. It is not a funded wallet and must not be reused as `id.json` or a Devnet fee-payer.

Never commit `~/.config/solana/id.json`, `.env`, or any keypair that holds funds.
