#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="baltimore-homicide-dashboard"
CONTAINER_NAME="baltimore-homicide-dashboard"
HOST_PORT="${HOST_PORT:-3838}"
CONTAINER_PORT="3838"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${SCRIPT_DIR}"

if [[ ! -f "homicide_data.csv" ]]; then
  echo "homicide_data.csv not found; scraping the latest data first..."
  Rscript scrape_data.R
fi

echo "Building Docker image: ${IMAGE_NAME}"
docker build -t "${IMAGE_NAME}" .

if docker ps -a --format '{{.Names}}' | grep -qx "${CONTAINER_NAME}"; then
  echo "Removing existing container: ${CONTAINER_NAME}"
  docker rm -f "${CONTAINER_NAME}" >/dev/null
fi

echo "Starting dashboard container on port ${HOST_PORT}"
echo "Dashboard running at http://localhost:${HOST_PORT}"
docker run --rm \
  --name "${CONTAINER_NAME}" \
  -p "${HOST_PORT}:${CONTAINER_PORT}" \
  "${IMAGE_NAME}"
