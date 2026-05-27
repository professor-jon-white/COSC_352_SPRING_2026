#!/usr/bin/env bash
set -euo pipefail

cargo run -- sample_data/employee_quality.csv --percentiles --histogram
