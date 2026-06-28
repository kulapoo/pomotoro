# 🚀 Releasing Workflow

How to ship a new version of Pomotoro to Windows, macOS, and Linux. Releases are
**tag-triggered** — push a `v*.*.*` tag and CI builds and publishes all installers
automatically.

## TL;DR

From a clean `main`:

```bash
just release VERSION=0.2.0
# or equivalently:
just release 0.2.0
```

That single command bumps the version, commits, tags, pushes, and triggers the
release workflow. Artifacts appear on the GitHub Releases page in ~10–20 minutes.

---

## What ships

Only the **Tauri desktop app** (`apps/tauri-app`) is released. The `pomotoro-cli`
and `cosmic-de` crates are intentional stubs (see
[ROADMAP.md](../../ROADMAP.md)) and are **not** bundled.

| Platform    | Artifacts produced by CI                            |
| ----------- | --------------------------------------------------- |
| **Linux**   | `.AppImage` (portable), `.deb` (Debian/Ubuntu)      |
| **macOS**   | `.dmg` (universal — Intel + Apple Silicon in one)    |
| **Windows** | `.msi`, `.exe` (NSIS installer)                      |

All builds are currently **unsigned**. See the [README download section][dl]
for the user-facing trust prompts (macOS Gatekeeper, Windows SmartScreen).
Signing (Apple Developer ID + Windows Authenticode) is planned before 1.0.

[dl]: ../../README.md#-download-prebuilt-binaries

---

## Choosing the version number

Follow [Semantic Versioning](https://semver.org/):

| Change type                                | Bump          | Example                    |
| ------------------------------------------ | ------------- | -------------------------- |
| Bug fixes, refactors, performance          | **Patch**     | `0.1.0` → `0.1.1`          |
| New features (settings, tray options, UI)  | **Minor**     | `0.1.0` → `0.2.0`          |
| Breaking changes (config format, data...)  | **Major**     | `0.1.0` → `1.0.0`          |

Pre-release suffixes are allowed: `0.2.0-beta.1`, `1.0.0-rc.1`.

---

## Step-by-step: cutting a release

### 1. Make sure your changes are merged to `main`

All work for the release must be committed on `main` and pushed to `origin`.

```bash
git checkout main
git pull
git log --oneline -5        # confirm your changes are here
```

### 2. Verify the working tree is clean

`just release` refuses to run with uncommitted changes.

```bash
git status                  # must say "nothing to commit, working tree clean"
```

If you have release-notes-worthy changes, update [`CHANGELOG.md`](../../CHANGELOG.md)
under `## [Unreleased]` first and commit that.

### 3. Run the release command

```bash
just release VERSION=0.2.0
```

Under the hood, [`scripts/release.sh`](../../scripts/release.sh) does:

1. **Validates** — semver format, clean tree, tag doesn't already exist.
2. **Bumps version** in the three shipping-crate files only:
   - `apps/tauri-app/tauri.conf.json` (the source of truth)
   - `apps/tauri-app/Cargo.toml`
   - `apps/react-ui/package.json`
3. **Commits** with message `chore: release v0.2.0`.
4. **Tags** the commit as `v0.2.0`.
5. **Pushes** `main` and the `v0.2.0` tag to `origin`.

The CLI/Cosmic stub `Cargo.toml` files are deliberately left at `0.1.0` until
those apps become real.

### 4. Watch the workflow

The tag push triggers [`.github/workflows/release.yml`](../../.github/workflows/release.yml):

- **Test gate** — `cargo test --workspace` runs on all three OSes first. If any
  fails, the release aborts.
- **Build** — `tauri-apps/tauri-action` builds per-OS installers in parallel.
- **Publish** — artifacts are attached to a GitHub Release named
  `Pomotoro v0.2.0`, body linking to CHANGELOG.md.

Live progress: **https://github.com/kulapoo/pomotoro/actions**

### 5. Verify the release

Once all jobs are green, open the release page and confirm the expected
artifacts are attached:

- `pomotoro_0.2.0_amd64.AppImage`
- `pomotoro_0.2.0_amd64.deb`
- `Pomotoro_0.2.0_universal.dmg`
- `pomotoro_0.2.0_x64-setup.exe` / `pomotoro_0.2.0_x64_en-US.msi`

Smoke-test at least one artifact on its target OS before announcing.

---

## Updating CHANGELOG.md

`just release` does **not** auto-edit changelog entries — it only stages
whatever you've already written. Before releasing, move items from
`## [Unreleased]` into a new section:

```markdown
## [0.2.0] - 2026-07-15

### Added
- Statistics dashboard with daily/weekly summaries.

### Fixed
- Tray countdown freezing on macOS when switching spaces.
```

Keep an empty `## [Unreleased]` above it for the next cycle. The compare links
at the bottom of the file should be updated to point at the new tag.

---

## Recovery: when a release fails

### The build broke after tagging

Fix the issue on `main`, then re-tag. Two options:

**Option A — overwrite the tag** (only if no one has downloaded it yet):

```bash
git tag -d v0.2.0                         # delete local tag
git push origin :refs/tags/v0.2.0         # delete remote tag
# (also delete the draft release on GitHub if it was created)
# ...fix the issue, commit...
just release VERSION=0.2.0                # re-run
```

**Option B — bump to the next patch** (cleaner public history):

```bash
# ...fix the issue, commit...
just release VERSION=0.2.1
```

### You tagged the wrong version

Same as Option A above: delete the local + remote tag, delete the GitHub release
if created, then re-run with the correct version.

---

## Manual dispatch (no tag)

The release workflow can also be triggered manually from the Actions tab
("Run workflow"), but this is rarely needed — the tag flow is the canonical path.

---

## How the pipeline is wired

| Piece                              | File                                              | Purpose                                              |
| ---------------------------------- | ------------------------------------------------- | ---------------------------------------------------- |
| Release script                     | [`scripts/release.sh`](../../scripts/release.sh)  | Version bump + commit + tag + push                   |
| `just` recipe                      | [`justfile`](../../justfile) `release`            | Thin wrapper that invokes the script                 |
| CI (every push/PR)                 | `.github/workflows/ci.yml`                        | Lint + test + smoke-build on 3 OSes                  |
| Release (on tag)                   | `.github/workflows/release.yml`                   | Test gate + `tauri-action` build + GitHub Release    |
| Toolchain pin                      | [`rust-toolchain.toml`](../../rust-toolchain.toml) | Stable Rust + clippy/rustfmt                         |
| Node pin                          | [`.nvmrc`](../../.nvmrc)                          | Node LTS                                             |

The system is fully automated end-to-end: after this is set up, every release is
a one-command operation.
