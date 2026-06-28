# Pomotoro Landing Page — Design Spec

**Date:** 2026-06-29
**Status:** Approved (user pre-approved design 2026-06-29 with "whatever is recommended. do it")
**Target URL:** https://kulapoo.github.io/pomotoro/

## Goal

A single-page marketing site that serves as the public face of Pomotoro, replacing the README as the first thing visitors see. The README remains the source of truth; the site links into it (and other docs) for depth.

## Audience

People who land on the GitHub repo link from a share, search, or the README badge — potential users deciding whether to download, and potential contributors deciding whether to dig in.

## Non-goals

- Documentation site (the `/docs` folder and GitHub rendering handle this).
- Blog, search, accounts, analytics dashboards.
- Per-release dynamic content (download links point at the static Releases page).
- Custom domain (deferred — drop in a `CNAME` later if desired).

## Stack

- **Static HTML + CSS + vanilla JS.** Zero build step, zero dependencies, instant load.
- **Hosting:** GitHub Pages via the modern "Source: GitHub Actions" mechanism.
- **No framework.** A landing page does not need React/Astro/Vite. Reusing the project's Vite/React toolchain would ship more JS than the page needs and double the frontend surface area.

## Repo Layout

```
pomotoro/
├── website/                       # NEW — pure static site
│   ├── index.html
│   ├── styles.css
│   ├── main.js
│   ├── assets/
│   │   ├── demo.gif               # copy of repo-root demo.gif
│   │   ├── demo.mp4               # copy of repo-root demo.mp4
│   │   └── logo.svg               # NEW — simple tomato-with-horns mark
│   └── (no _config, no build)
└── .github/workflows/
    └── deploy-website.yml         # NEW — deploys website/ to Pages
```

**Why `/website/` and not `/docs/`:** the existing `/docs` folder holds project documentation; mixing the marketing page in would collide.

## Deployment

Workflow `.github/workflows/deploy-website.yml`:

- **Trigger:** push to `main` with paths filter `website/**` (also workflow `workflow_dispatch` for manual runs).
- **Permissions:** `contents: read`, `pages: write`, `id-token: write`.
- **Steps:**
  1. Checkout.
  2. `actions/configure-pages` — derives the Pages URL.
  3. `actions/upload-pages-artifact` with `path: website/`.
  4. `actions/deploy-pages`.
- Concurrency group so only the latest run deploys.

**One-time manual step (cannot be automated):** repo Settings → Pages → Source = "GitHub Actions".

## Page Structure (top to bottom)

### 1. Hero
- `🍅🐂 Pomotoro` wordmark + `logo.svg`.
- Headline: **"Charge through your focus sessions."**
- Subhead: native Pomodoro focus timer with task queues, ambient audio, and screen-blocking breaks. Built fast and private with Rust + Tauri.
- CTAs:
  - **Download for {Mac/Win/Linux}** — JS auto-detects OS via `navigator.userAgent`, sets the button label + links to the right asset pattern on the Releases page. Falls back to the Releases index if unknown.
  - **Other platforms ↓** — discreet text toggle under the primary CTA; expands to show the other two platforms inline.
  - **Star on GitHub** — secondary button, links to the repo.
- Hero visual: `demo.gif` autoplaying in a rounded framed card; clicking it opens `demo.mp4` in a new tab (or inline `<video>` with controls). Caption underneath: "Watch the full 1m40s demo".

### 2. Features grid
10 cards in a responsive grid (1 col mobile, 2 col tablet, 3-4 col desktop). Each card: large emoji, title, one-line description.

Cards (from README feature table):
- 🍅 **Pomodoro engine** — Configurable focus/break cycles with smooth ring and session dots.
- ✅ **Task management** — Multi-session tasks with tags, search, and live progress.
- 🎵 **Focus audio** — Rain, forest, ocean, white noise, café, fireplace, thunderstorm, brown noise.
- 🔔 **Smart notifications** — Desktop + sound alerts with position and auto-dismiss control.
- 🚫 **Screen blocker** — Full-screen overlay that actually forces your break.
- 🪟 **System tray** — Minimize to tray with a live countdown baked into the icon.
- ⌨️ **Keyboard shortcuts** — Cycle tasks instantly with `Ctrl/Meta+Tab`.
- 🎛️ **Deeply configurable** — Timer, automation, audio, appearance, window, blocking.
- ⚡ **Native speed** — Tauri + Rust core with SQLite persistence. Minimal resources.
- 🖥️ **Cross-platform** — Windows, macOS, and Linux.

### 3. How it works
Three numbered steps in a horizontal row (stacks on mobile):
1. **Start** the timer — a default focus session is ready out of the box.
2. **Add tasks** — quick-add or detail modal with tags and session counts.
3. **Focus** — pick a task, hit play, cycle with `Ctrl/Meta+Tab`.

### 4. Why Pomotoro
Four-up grid of value props with one-sentence each:
- **Native** — Tauri + Rust, not Electron. Tiny footprint, instant response.
- **Private** — Everything runs locally. Your data never leaves your machine.
- **Free** — MIT-licensed open source. No freemium, no upsells.
- **Yours** — Built to be the focus tool I wanted and couldn't find.

Inline link: "Read the full story →" pointing at `docs/MOTIVATION.md` (raw GitHub URL).

### 5. Roadmap teaser
Three cards teasing what's planned, with a "See full roadmap →" link to `ROADMAP.md`:
- 📊 **Statistics** — Session timelines, streaks, per-task/tag breakdowns, charts, export.
- 💻 **CLI** — Drive Pomotoro from the terminal: status, timer, tasks, config.
- ⚡ **Command hooks** — Run shell commands on lifecycle events with templated args.

### 6. Footer
- Centered logo + tagline.
- Three columns:
  - **Product:** Download, Releases, Roadmap
  - **Develop:** GitHub repo, Docs, Contributing
  - **Project:** License (MIT), Motivation, Acknowledgments
- Bottom line: "Built with Tauri + React · Powered by Rust · MIT License".

## Visual Design — "Playful & warm"

### Color tokens
| Token      | Value     | Use                            |
| ---------- | --------- | ------------------------------ |
| Background | `#FDF6E3` | page bg (warm cream)           |
| Surface    | `#FFFFFF` | cards                          |
| Primary    | `#E63946` | CTAs, key accents, logo (tomato red) |
| Primary-dark | `#C1121F` | CTA hover                      |
| Secondary  | `#FFB703` | highlights, dots (sunny yellow) |
| Text       | `#2D2A26` | body (warm dark brown)         |
| Muted      | `#8A8278` | secondary text                 |
| Border     | `#F0E6D2` | card borders                   |

All text/bg combinations meet WCAG AA contrast.

### Typography
- **Family:** Inter (via Google Fonts CDN, with `system-ui` fallback), weights 400/500/600/700.
- **Headlines:** Inter 700, sizes scale fluidly via `clamp()` (e.g. hero `clamp(2.5rem, 6vw, 4.5rem)`).
- **Body:** Inter 400/500, base `1.0625rem` (17px), `line-height: 1.6`.
- No display/serif font. Friendly + clean.

### Components
- **Cards:** white surface, `border: 1px solid var(--border)`, `border-radius: 16px`, `box-shadow: 0 8px 32px rgba(0,0,0,0.06)`, `padding: 28px`.
- **Buttons:** primary = filled tomato red, white text, `border-radius: 12px`, `padding: 14px 28px`, hover lifts to `translateY(-1px)` + slightly darker bg. Secondary = outlined, transparent bg, border in muted tone.
- **Container:** `max-width: 1100px`, centered, `padding: 0 24px`.
- **Section rhythm:** `padding: 96px 0` between sections on desktop, `64px 0` on mobile.

### Motion
- Section fade-in-up on scroll via `IntersectionObserver` (`opacity 0 → 1`, `translateY(16px) → 0`, `400ms ease`).
- Card hover: `transform: translateY(-4px)`, transition `200ms`.
- **`prefers-reduced-motion: reduce`:** disable all transitions and reveal animations; show everything immediately.

### Responsive
- Mobile-first CSS.
- Breakpoints: `640px` (tablet), `1024px` (desktop).
- Nav (if any): no top nav on mobile, simple footer-only nav.
- Hero stacks: text first, demo gif below on mobile.

## Accessibility
- Semantic HTML5 (`<header>`, `<main>`, `<section>`, `<footer>`, proper heading order).
- All images have `alt` text.
- Buttons are `<a>`/`<button>`, keyboard-focusable, visible focus ring.
- Color contrast ≥ AA on all text.
- `prefers-reduced-motion` respected.
- `<html lang="en">`, descriptive `<title>`, meta description.

## Assets

- **`demo.gif`, `demo.mp4`:** committed copies inside `website/assets/`. Rationale: simpler than copying at deploy time, portable for local testing, ~few MB is negligible in a repo that already ships binaries. Re-sync from repo root if the originals change.
- **`logo.svg`:** new asset. A simple flat tomato shape with bull horns rendered in the primary red, scales crisply. Built by hand in SVG; no icon library.
- **Feature icons:** emoji used directly (matches the README voice, zero asset weight).

## Copy Approach
Rewritten punchier for the web, not pasted from the README. README remains canonical; the site links into it for depth (Quick Start, Configuration Reference, Architecture, etc.).

## Verification (post-implementation)
1. `python3 -m http.server` (or similar) from `website/` — load locally and click through every section.
2. Mobile viewport (DevTools) — confirm responsive stacking and touch-target sizes.
3. Lighthouse pass: Performance ≥ 95, Accessibility ≥ 95, Best Practices ≥ 95.
4. Verify all asset paths resolve under `/pomotoro/` base (relative paths everywhere).
5. After first deploy to Pages: confirm URL loads, demo gif autoplays, download CTA picks correct OS on at least Mac + Windows + Linux user-agents.

## Out-of-scope / Future
- Custom domain (drop in `CNAME` when ready).
- Auto-pull latest release version into the download button (currently links to Releases index).
- Blog/changelog rendered from `CHANGELOG.md`.
- Screenshot gallery beyond the demo gif.
- Internationalization.
