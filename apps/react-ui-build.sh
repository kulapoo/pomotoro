#!/bin/bash
# Builds the React UI frontend for production.
# Run from apps/tauri-app/ (where cargo tauri build is invoked).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/react-ui"

# Install deps if node_modules is missing
if [ ! -d node_modules ]; then
  npm install --silent
fi

exec npm run build
