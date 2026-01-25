#!/usr/bin/env bash
set -euo pipefail

PROJECT="flat-file-db"
TAG="${1:?release tag required}"

BASE_DIR="/home/macia/releases/$PROJECT"
RELEASE_DIR="$BASE_DIR/$TAG"

echo "Deploying $PROJECT release $TAG"

mkdir -p $RELEASE_DIR

# move artifacts
mv src/ $RELEASE_DIR

echo "Release $TAG deployed successfully"
