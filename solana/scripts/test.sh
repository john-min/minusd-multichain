#!/usr/bin/env bash
# Run local-validator Anchor tests. Pins are in Anchor.toml / README.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/deploy
cp keys/minusd-keypair.json target/deploy/minusd-keypair.json
exec anchor test "$@"
