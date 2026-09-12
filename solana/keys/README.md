# Program identity keypair (local only)

`keys/minusd-keypair.json` is **gitignored**. Create it with:

```bash
./scripts/prepare-keys.sh
```

That script generates a keypair if missing, copies it to `target/deploy/`, and runs `anchor keys sync` so `declare_id!` matches.

Never commit `*-keypair.json`, `~/.config/solana/id.json`, `.env`, or any keypair that holds funds. The previous committed identity (`fLsZq7…NRw`) is retired — its secret was published and must not be used on Devnet or any shared cluster. For Devnet, generate a fresh keypair offline and keep it out of git.
