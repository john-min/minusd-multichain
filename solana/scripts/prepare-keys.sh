#!/usr/bin/env bash
# Generate a gitignored local program keypair (if missing), copy it for Anchor deploy,
# and sync declare_id! / Anchor.toml. Never commit *-keypair.json.
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p keys target/deploy
KEYPAIR="keys/minusd-keypair.json"

if [[ ! -f "$KEYPAIR" ]]; then
  if ! command -v solana-keygen >/dev/null 2>&1; then
    echo "solana-keygen is required to create a local program keypair" >&2
    exit 1
  fi
  solana-keygen new --no-bip39-passphrase -o "$KEYPAIR" --force --silent
  echo "Generated gitignored program keypair at $KEYPAIR"
fi

cp "$KEYPAIR" target/deploy/minusd-keypair.json

if command -v anchor >/dev/null 2>&1; then
  anchor keys sync
  echo "Synced program ID to $(solana-keygen pubkey "$KEYPAIR")"
else
  echo "warning: anchor CLI not found; skipped keys sync" >&2
fi
