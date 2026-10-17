#!/usr/bin/env bash
set -euo pipefail

OUTPUT_PATH="target/llvm-cov-summary.json"

cargo llvm-cov \
  --json \
  --summary-only \
  --output-path "$OUTPUT_PATH" \
  --fail-under-lines 60

LINES=$(jq -r '.data[0].totals.lines.percent' "$OUTPUT_PATH")
BRANCHES=$(jq -r '.data[0].totals.branches.percent' "$OUTPUT_PATH")

echo "Coverage: ${LINES}% lines, ${BRANCHES}% branches"
