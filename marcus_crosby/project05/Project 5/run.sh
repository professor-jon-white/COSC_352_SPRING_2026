#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/output"

mkdir -p "$OUTPUT_DIR"
cd "$SCRIPT_DIR"

docker build . -t homicide-analysis
docker run --rm -v "$OUTPUT_DIR:/app/output" homicide-analysis 2>&1 | tee "$OUTPUT_DIR/run.log"

echo
echo "Artifacts written to $OUTPUT_DIR"
