#!/bin/bash
# Starts the Vite dev server for the React UI frontend.
# Run from apps/tauri-app/ (where cargo tauri dev is invoked).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/react-ui"

# Install deps if node_modules is missing
if [ ! -d node_modules ]; then
  npm install --silent
fi

exec npm run dev
