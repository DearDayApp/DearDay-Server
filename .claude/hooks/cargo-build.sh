#!/usr/bin/env bash
set -uo pipefail

PROJECT_ROOT="/Users/jiwon/Desktop/projects/dearday"
CARGO="$HOME/.cargo/bin/cargo"

STDIN=$(cat)
if [ "$(echo "$STDIN" | jq -r '.stop_hook_active // false')" = "true" ]; then
  exit 0
fi

cd "$PROJECT_ROOT"

set -a
[ -f .env ] && . ./.env
set +a

"$CARGO" fmt --quiet 2>/dev/null || true

if ! OUTPUT=$("$CARGO" clippy --all-targets --color never -- -D warnings 2>&1); then
  jq -Rn --arg out "$OUTPUT" '{decision: "block", reason: ("cargo clippy failed:\n" + $out)}'
  exit 0
fi

if ! OUTPUT=$("$CARGO" build --color never 2>&1); then
  jq -Rn --arg out "$OUTPUT" '{decision: "block", reason: ("cargo build failed:\n" + $out)}'
  exit 0
fi

if ! OUTPUT=$("$CARGO" test --color never --no-fail-fast 2>&1); then
  jq -Rn --arg out "$OUTPUT" '{decision: "block", reason: ("cargo test failed:\n" + $out)}'
  exit 0
fi

echo "cargo fmt + clippy + build + test: ok"
