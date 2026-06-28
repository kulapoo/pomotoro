#!/usr/bin/env bash
# Cut a new release of Pomotoro.
#
# Usage: scripts/release.sh <version>
#   version: semver string without leading 'v', e.g. 0.2.0 or 1.0.0-beta.1
#
# This script:
#   1. Validates the working tree is clean and the tag does not exist.
#   2. Bumps the version in the shipping crates only:
#        - apps/tauri-app/tauri.conf.json
#        - apps/tauri-app/Cargo.toml
#        - apps/react-ui/package.json
#      (pomotoro-cli and cosmic-de are intentional stubs and stay at 0.1.0.)
#   3. Commits, tags vX.Y.Z, and pushes main + the tag to origin.
#   4. The tag push triggers .github/workflows/release.yml, which builds
#      Windows, macOS (universal), and Linux installers automatically.
#
# Run via: just release VERSION=x.y.z

set -euo pipefail

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>   (e.g. 0.2.0)"
    exit 1
fi

if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    echo "Invalid version '$VERSION'. Expected semver like 0.2.0 or 1.0.0-beta.1"
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAG="v${VERSION}"

if [ -n "$(git -C "$ROOT" status --porcelain)" ]; then
    echo "Working tree is not clean. Commit or stash changes first."
    git -C "$ROOT" status --short
    exit 1
fi

if git -C "$ROOT" rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "Tag $TAG already exists."
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to bump version files."
    exit 1
fi

echo "Bumping version to $VERSION in shipping crates..."

# apps/tauri-app/tauri.conf.json — update the top-level "version" field.
TAURI_CONF="$ROOT/apps/tauri-app/tauri.conf.json"
python3 - "$TAURI_CONF" "$VERSION" <<'PY'
import json, sys
path, version = sys.argv[1], sys.argv[2]
with open(path) as f:
    data = json.load(f)
data["version"] = version
with open(path, "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(f"  updated {path}")
PY

# apps/tauri-app/Cargo.toml — update the first 'version = "..."' line.
CARGO_TOML="$ROOT/apps/tauri-app/Cargo.toml"
python3 - "$CARGO_TOML" "$VERSION" <<'PY'
import re, sys
path, version = sys.argv[1], sys.argv[2]
with open(path) as f:
    content = f.read()
content = re.sub(
    r'^version = "[^"]+"',
    f'version = "{version}"',
    content,
    count=1,
    flags=re.MULTILINE,
)
with open(path, "w") as f:
    f.write(content)
print(f"  updated {path}")
PY

# apps/react-ui/package.json — update the "version" field.
REACT_PKG="$ROOT/apps/react-ui/package.json"
python3 - "$REACT_PKG" "$VERSION" <<'PY'
import json, sys
path, version = sys.argv[1], sys.argv[2]
with open(path) as f:
    data = json.load(f)
data["version"] = version
with open(path, "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(f"  updated {path}")
PY

echo
echo "Staging release commit..."
git -C "$ROOT" add \
    apps/tauri-app/tauri.conf.json \
    apps/tauri-app/Cargo.toml \
    apps/react-ui/package.json

echo
echo "Summary of version changes:"
git -C "$ROOT" diff --cached --stat

echo
echo "Committing and tagging $TAG..."
git -C "$ROOT" commit -m "chore: release $TAG"
git -C "$ROOT" tag -a "$TAG" -m "Release $TAG"

echo
echo "Pushing main and $TAG to origin..."
git -C "$ROOT" push origin main
git -C "$ROOT" push origin "$TAG"

echo
echo "Done. The release workflow is now running at:"
echo "  https://github.com/kulapoo/pomotoro/actions"
echo "Artifacts will appear on the $TAG release page once all jobs finish."
