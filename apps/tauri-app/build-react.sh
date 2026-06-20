#!/bin/bash
# Builds the React UI for production.
# Tauri runs this from the tauri-app/ directory.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/../react-ui"

if [ ! -d node_modules ]; then
  npm install --silent
fi

exec npm run build
