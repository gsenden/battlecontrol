#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="${1:-battlecontrol}"
IMAGE_TAG="${2:-local}"

cd "$ROOT_DIR"

docker build \
  --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
  .
