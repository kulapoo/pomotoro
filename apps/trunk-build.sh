#!/bin/bash
# Wrapper script to run trunk build from project root

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the project root (one level up from apps/)
cd "$SCRIPT_DIR/.."

# Run trunk build
exec trunk build
