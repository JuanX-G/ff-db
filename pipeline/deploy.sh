#!/usr/bin/env bash
set -euo pipefail

PROJECT="flat-file-db"
TAG="${1:?release tag required}"

PI_HOST="$RELEASE_REMOTE_USER@$RELEASE_REMOTE_ADDR"
BASE_DIR="/var/www/releases/$PROJECT"
RELEASE_DIR="$BASE_DIR/$TAG"
CURRENT_LINK="$BASE_DIR/latest"

echo "Deploying $PROJECT release $TAG"

# create release dir on Pi
ssh "$PI_HOST" "mkdir -p '$RELEASE_DIR'"

# upload artifacts
rsync -avz --delete \
  src/ \
  "$PI_HOST:$RELEASE_DIR/"

# atomically update 'latest' symlink
ssh "$PI_HOST" <<EOF
ln -sfn "$RELEASE_DIR" "$CURRENT_LINK"
EOF

echo "Release $TAG deployed successfully"
