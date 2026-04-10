#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${1:?usage: push-image.sh <image-name> <image-tag>}"
IMAGE_TAG="${2:?usage: push-image.sh <image-name> <image-tag>}"

docker push "${IMAGE_NAME}:${IMAGE_TAG}"
