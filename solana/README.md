# MinUSD Solana program

MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.

Phase 2 lifecycle program. See the [root README](../README.md) for pinned toolchain versions and the EVM comparison.

```bash
yarn install
yarn test
# or: ./scripts/test.sh
```

`yarn test` runs `scripts/prepare-keys.sh` (gitignored local program keypair + `anchor keys sync`) then `anchor test`. Never commit `*-keypair.json`.
