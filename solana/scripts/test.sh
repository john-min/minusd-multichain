#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/prepare-keys.sh
yarn test
