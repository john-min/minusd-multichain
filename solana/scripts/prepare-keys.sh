#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/deploy
cp keys/minusd-keypair.json target/deploy/minusd-keypair.json
echo "Copied program keypair to target/deploy/minusd-keypair.json"
