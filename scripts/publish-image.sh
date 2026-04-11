#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${1:?usage: publish-image.sh <image-name> <image-tag> [alias-tag]}"
IMAGE_TAG="${2:?usage: publish-image.sh <image-name> <image-tag> [alias-tag]}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/build-image.sh" "$IMAGE_NAME" "$IMAGE_TAG"
"$ROOT_DIR/scripts/push-image.sh" "$IMAGE_NAME" "$IMAGE_TAG"

shift 2

for ALIAS_TAG in "$@"; do
  docker tag "${IMAGE_NAME}:${IMAGE_TAG}" "${IMAGE_NAME}:${ALIAS_TAG}"
  "$ROOT_DIR/scripts/push-image.sh" "$IMAGE_NAME" "$ALIAS_TAG"
done
