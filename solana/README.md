# MinUSD Solana program

MinUSD is an educational, testnet-only prototype. It is unaudited, has no monetary value, is not backed by real reserves, and is not redeemable for fiat.

Phase 2 lifecycle program. See the [root README](../README.md) for pinned toolchain versions and the EVM comparison.

```bash
yarn install
yarn test
# or: ./scripts/test.sh
```

`yarn test` copies `keys/minusd-keypair.json` to `target/deploy/` then runs `anchor test`.
