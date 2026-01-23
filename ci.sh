#!/usr/bin/env bash
set -e

TAG="$1"

echo "Building release $TAG"

./pipeline/test.sh
./pipeline/deploy.sh "$TAG"
