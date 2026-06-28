# Pomotoro Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a single-page marketing site at `https://kulapoo.github.io/pomotoro/` that serves as the public face of Pomotoro.

**Architecture:** Pure static HTML/CSS/vanilla JS in `website/` at the repo root. No build step. Deployed to GitHub Pages via a dedicated Actions workflow that uploads `website/` as a Pages artifact.

**Tech Stack:** HTML5, CSS3 (custom properties, grid, flexbox), vanilla ES6+, GitHub Actions (`actions/configure-pages`, `actions/upload-pages-artifact`, `actions/deploy-pages`), Inter font via Google Fonts CDN.

## Global Constraints

- **All asset paths MUST be relative** (`assets/demo.gif`, not `/assets/demo.gif`) — site lives at `/pomotoro/`, not domain root.
- **No build step, no framework, no npm deps for the site.** The repo already has a Vite/React app for the desktop UI; the website is intentionally separate and tooling-free.
- **Color tokens (verbatim from spec):** bg `#FDF6E3`, surface `#FFFFFF`, primary `#E63946`, primary-dark `#C1121F`, secondary `#FFB703`, text `#2D2A26`, muted `#8A8278`, border `#F0E6D2`.
- **Type:** Inter 400/500/600/700 via Google Fonts, `system-ui` fallback.
- **Radii:** cards `16px`, buttons `12px`.
- **Container:** `max-width: 1100px` centered, `padding: 0 24px`.
- **Section rhythm:** desktop `padding: 96px 0`, mobile `64px 0`.
- **WCAG AA contrast** on all text/bg combos.
- **`prefers-reduced-motion: reduce`** must disable all transitions and reveal animations.
- **Repo URLs** (use throughout):
  - GitHub: `https://github.com/kulapoo/pomotoro`
  - Releases: `https://github.com/kulapoo/pomotoro/releases`
  - Releases latest: `https://github.com/kulapoo/pomotoro/releases/latest`
  - Motivation doc (raw): `https://github.com/kulapoo/pomotoro/blob/main/docs/MOTIVATION.md`
  - Roadmap doc: `https://github.com/kulapoo/pomotoro/blob/main/ROADMAP.md`
  - Docs index: `https://github.com/kulapoo/pomotoro/blob/main/docs/README.md`
  - README quick start: `https://github.com/kulapoo/pomotoro#-quick-start`
- **Verification per task:** there is no test framework for static HTML/CSS. Each task's "test" is loading `website/index.html` in a browser (via `python3 -m http.server` from `website/`) and confirming the relevant section renders. Lighthouse audit is the final verification.

---

### Task 1: Scaffold `website/` and commit assets + logo

**Files:**
- Create: `website/assets/demo.gif` (copy of repo-root `demo.gif`)
- Create: `website/assets/demo.mp4` (copy of repo-root `demo.mp4`)
- Create: `website/assets/logo.svg`
- Create: `website/index.html` (placeholder — just enough to verify assets load)
- Create: `website/.gitignore` (empty or none — not needed)

**Interfaces:**
- Produces: `website/assets/logo.svg` (used by `index.html` favicon + header wordmark in Task 2), `website/assets/demo.gif` and `website/assets/demo.mp4` (referenced by hero in Task 2).

- [ ] **Step 1: Create the directory structure**

```bash
mkdir -p website/assets
```

- [ ] **Step 2: Copy demo assets into `website/assets/`**

```bash
cp demo.gif website/assets/demo.gif
cp demo.mp4 website/assets/demo.mp4
ls -lh website/assets/
```

Expected: both files listed with non-zero sizes matching repo-root originals.

- [ ] **Step 3: Write `website/assets/logo.svg`**

A flat tomato-with-horns mark in the primary red, 64×64 viewBox. Used as favicon and header wordmark.

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" role="img" aria-label="Pomotoro logo">
  <title>Pomotoro</title>
  <!-- Bull horns (sunny yellow) -->
  <path d="M14 18 Q 9 8, 18 6 Q 21 13, 23 19 Z" fill="#FFB703"/>
  <path d="M50 18 Q 55 8, 46 6 Q 43 13, 41 19 Z" fill="#FFB703"/>
  <!-- Leaves (green) -->
  <path d="M32 10 L 22 18 L 32 24 L 42 18 Z" fill="#588157"/>
  <!-- Tomato body (red) -->
  <circle cx="32" cy="38" r="20" fill="#E63946"/>
  <!-- Highlight -->
  <ellipse cx="25" cy="32" rx="5" ry="3" fill="#FFFFFF" opacity="0.35"/>
</svg>
```

- [ ] **Step 4: Write a minimal placeholder `website/index.html` to verify assets resolve**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Pomotoro — scaffold check</title>
</head>
<body>
  <h1> scaffold</h1>
  <img src="assets/logo.svg" alt="logo" height="64">
  <img src="assets/demo.gif" alt="demo" height="200">
</body>
</html>
```

- [ ] **Step 5: Verify locally**

```bash
cd website && python3 -m http.server 8080
```

Open `http://localhost:8080/` — confirm both the logo SVG and demo gif render. Ctrl-C to stop the server.

- [ ] **Step 6: Commit**

```bash
git add website/
git commit -m "feat(website): scaffold website/ with logo and demo assets"
```

---

### Task 2: Build complete `website/index.html`

**Files:**
- Modify (replace): `website/index.html`

**Interfaces:**
- Consumes: `website/assets/logo.svg`, `website/assets/demo.gif`, `website/assets/demo.mp4` (from Task 1)
- Produces: `website/index.html` with semantic structure and IDs that Task 3 (CSS) and Task 4 (JS) will hook into. Key IDs/classes the later tasks depend on:
  - `#download-btn` — primary download `<a>` element; JS sets its label + href based on `navigator.userAgent`.
  - `#other-platforms-toggle` — button that toggles `.is-expanded` on `#other-platforms`.
  - `#other-platforms` — the expandable container; CSS hides it by default, shows when `.is-expanded`.
  - `.reveal` — any element with this class gets IntersectionObserver fade-in from Task 4.
  - `#features`, `#how`, `#why`, `#roadmap` — section anchors.

- [ ] **Step 1: Write the complete `website/index.html`**

Replace the placeholder with the full page:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Pomotoro — Native Pomodoro focus timer</title>
  <meta name="description" content="Pomotoro is a native Pomodoro focus timer with task queues, ambient audio, and screen-blocking break enforcement. Built fast and private with Rust + Tauri.">
  <link rel="icon" href="assets/logo.svg" type="image/svg+xml">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <header class="hero">
    <div class="container hero__inner">
      <a class="wordmark" href="#">
        <img src="assets/logo.svg" alt="" width="40" height="40">
        <span>Pomotoro</span>
      </a>

      <div class="hero__content reveal">
        <h1 class="hero__headline">Charge through your focus sessions.</h1>
        <p class="hero__subhead">
          Native Pomodoro focus timer with task queues, ambient audio, and
          screen-blocking breaks. Built fast and private with Rust + Tauri.
        </p>

        <div class="hero__cta">
          <a id="download-btn" class="btn btn--primary"
             href="https://github.com/kulapoo/pomotoro/releases/latest">
            Download
          </a>
          <button id="other-platforms-toggle" class="btn btn--link" type="button"
                  aria-expanded="false" aria-controls="other-platforms">
            Other platforms <span aria-hidden="true">↓</span>
          </button>
          <a class="btn btn--secondary"
             href="https://github.com/kulapoo/pomotoro">
            ★ Star on GitHub
          </a>
        </div>

        <div id="other-platforms" class="hero__platforms">
          <a class="platform-link" data-os="macos"
             href="https://github.com/kulapoo/pomotoro/releases/latest">
            macOS (.dmg)
          </a>
          <a class="platform-link" data-os="windows"
             href="https://github.com/kulapoo/pomotoro/releases/latest">
            Windows (.msi / .exe)
          </a>
          <a class="platform-link" data-os="linux"
             href="https://github.com/kulapoo/pomotoro/releases/latest">
            Linux (.AppImage / .deb)
          </a>
        </div>
      </div>

      <div class="hero__demo reveal">
        <a href="assets/demo.mp4" target="_blank" rel="noopener"
           title="Watch the full 1m40s demo">
          <img src="assets/demo.gif" alt="Pomotoro in action — timer phases, task queues, ambient audio, notifications, and the break screen blocker.">
        </a>
        <p class="hero__demo-caption">
          A quick loop of Pomotoro in action —
          <a href="assets/demo.mp4" target="_blank" rel="noopener">watch the full demo</a> (1m40s).
        </p>
      </div>
    </div>
  </header>

  <main>
    <section id="features" class="section">
      <div class="container">
        <h2 class="section__title reveal">Everything you need to focus</h2>
        <p class="section__lede reveal">A complete Pomodoro toolkit that gets out of your way.</p>

        <ul class="feature-grid">
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🍅</div>
            <h3 class="feature-card__title">Pomodoro engine</h3>
            <p class="feature-card__text">Configurable focus and break cycles with a smooth ring and session dots.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">✅</div>
            <h3 class="feature-card__title">Task management</h3>
            <p class="feature-card__text">Multi-session tasks with tags, search, and live progress.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🎵</div>
            <h3 class="feature-card__title">Focus audio</h3>
            <p class="feature-card__text">Rain, forest, ocean, white noise, café, fireplace, thunderstorm, brown noise.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🔔</div>
            <h3 class="feature-card__title">Smart notifications</h3>
            <p class="feature-card__text">Desktop and sound alerts with position and auto-dismiss control.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🚫</div>
            <h3 class="feature-card__title">Screen blocker</h3>
            <p class="feature-card__text">A full-screen overlay that actually forces you to take your break.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🪟</div>
            <h3 class="feature-card__title">System tray</h3>
            <p class="feature-card__text">Minimize to tray with the live countdown baked into the icon.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">⌨️</div>
            <h3 class="feature-card__title">Keyboard shortcuts</h3>
            <p class="feature-card__text">Cycle tasks instantly with <kbd>Ctrl</kbd>/<kbd>Meta</kbd>+<kbd>Tab</kbd>. No mouse needed.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🎛️</div>
            <h3 class="feature-card__title">Deeply configurable</h3>
            <p class="feature-card__text">Timer, automation, audio, appearance, window, and blocking settings.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">⚡</div>
            <h3 class="feature-card__title">Native speed</h3>
            <p class="feature-card__text">Tauri + Rust core with SQLite persistence. Minimal resources, instant response.</p>
          </li>
          <li class="feature-card reveal">
            <div class="feature-card__icon" aria-hidden="true">🖥️</div>
            <h3 class="feature-card__title">Cross-platform</h3>
            <p class="feature-card__text">Windows, macOS, and Linux. The same app everywhere you work.</p>
          </li>
        </ul>
      </div>
    </section>

    <section id="how" class="section section--alt">
      <div class="container">
        <h2 class="section__title reveal">How it works</h2>
        <p class="section__lede reveal">Three steps from install to flow.</p>

        <ol class="steps">
          <li class="step reveal">
            <div class="step__number" aria-hidden="true">1</div>
            <h3 class="step__title">Start</h3>
            <p class="step__text">Hit play. A default focus session is ready out of the box — no setup, no sign-in.</p>
          </li>
          <li class="step reveal">
            <div class="step__number" aria-hidden="true">2</div>
            <h3 class="step__title">Add tasks</h3>
            <p class="step__text">Quick-add a task, or open the detail modal for tags, descriptions, and a custom session count.</p>
          </li>
          <li class="step reveal">
            <div class="step__number" aria-hidden="true">3</div>
            <h3 class="step__title">Focus</h3>
            <p class="step__text">Pick a task, hit play, and let ambient audio keep you in flow. Cycle tasks with <kbd>Ctrl</kbd>/<kbd>Meta</kbd>+<kbd>Tab</kbd>.</p>
          </li>
        </ol>
      </div>
    </section>

    <section id="why" class="section">
      <div class="container">
        <h2 class="section__title reveal">Why Pomotoro?</h2>
        <p class="section__lede reveal">Built to be the focus tool I wanted and couldn't find.</p>

        <ul class="value-grid">
          <li class="value-card reveal">
            <div class="value-card__icon" aria-hidden="true">🦀</div>
            <h3 class="value-card__title">Native</h3>
            <p class="value-card__text">Tauri + Rust, not Electron. Tiny footprint, instant response.</p>
          </li>
          <li class="value-card reveal">
            <div class="value-card__icon" aria-hidden="true">🔒</div>
            <h3 class="value-card__title">Private</h3>
            <p class="value-card__text">Everything runs locally. Your data never leaves your machine.</p>
          </li>
          <li class="value-card reveal">
            <div class="value-card__icon" aria-hidden="true">🎁</div>
            <h3 class="value-card__title">Free</h3>
            <p class="value-card__text">MIT-licensed open source. No freemium tiers, no upsells, no tracking.</p>
          </li>
          <li class="value-card reveal">
            <div class="value-card__icon" aria-hidden="true">💚</div>
            <h3 class="value-card__title">Yours</h3>
            <p class="value-card__text">Hack on it, learn from it, bend it to your workflow. It's yours.</p>
          </li>
        </ul>

        <p class="section__link reveal">
          <a href="https://github.com/kulapoo/pomotoro/blob/main/docs/MOTIVATION.md" target="_blank" rel="noopener">
            Read the full story →
          </a>
        </p>
      </div>
    </section>

    <section id="roadmap" class="section section--alt">
      <div class="container">
        <h2 class="section__title reveal">What's coming</h2>
        <p class="section__lede reveal">Pomotoro is actively developed. A peek at the roadmap:</p>

        <ul class="roadmap-grid">
          <li class="roadmap-card reveal">
            <div class="roadmap-card__icon" aria-hidden="true">📊</div>
            <h3 class="roadmap-card__title">Statistics</h3>
            <p class="roadmap-card__text">Session timelines, streaks, per-task and per-tag breakdowns, charts, and export.</p>
          </li>
          <li class="roadmap-card reveal">
            <div class="roadmap-card__icon" aria-hidden="true">💻</div>
            <h3 class="roadmap-card__title">CLI</h3>
            <p class="roadmap-card__text">Drive Pomotoro from the terminal: status, timer control, tasks, and config.</p>
          </li>
          <li class="roadmap-card reveal">
            <div class="roadmap-card__icon" aria-hidden="true">⚡</div>
            <h3 class="roadmap-card__title">Command hooks</h3>
            <p class="roadmap-card__text">Run shell commands on lifecycle events with templated arguments.</p>
          </li>
        </ul>

        <p class="section__link reveal">
          <a href="https://github.com/kulapoo/pomotoro/blob/main/ROADMAP.md" target="_blank" rel="noopener">
            See the full roadmap →
          </a>
        </p>
      </div>
    </section>
  </main>

  <footer class="footer">
    <div class="container footer__inner">
      <div class="footer__brand">
        <a class="wordmark wordmark--footer" href="#">
          <img src="assets/logo.svg" alt="" width="32" height="32">
          <span>Pomotoro</span>
        </a>
        <p class="footer__tagline">🍅🐂 Charge through your focus sessions.</p>
      </div>

      <nav class="footer__nav" aria-label="Footer">
        <div class="footer__col">
          <h4 class="footer__heading">Product</h4>
          <ul>
            <li><a href="https://github.com/kulapoo/pomotoro/releases/latest">Download</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro/releases">Releases</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro/blob/main/ROADMAP.md">Roadmap</a></li>
          </ul>
        </div>
        <div class="footer__col">
          <h4 class="footer__heading">Develop</h4>
          <ul>
            <li><a href="https://github.com/kulapoo/pomotoro">GitHub repo</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro/blob/main/docs/README.md">Docs</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro#-contributing">Contributing</a></li>
          </ul>
        </div>
        <div class="footer__col">
          <h4 class="footer__heading">Project</h4>
          <ul>
            <li><a href="https://github.com/kulapoo/pomotoro/blob/main/LICENSE">License (MIT)</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro/blob/main/docs/MOTIVATION.md">Motivation</a></li>
            <li><a href="https://github.com/kulapoo/pomotoro#-acknowledgments">Acknowledgments</a></li>
          </ul>
        </div>
      </nav>
    </div>

    <div class="container footer__bottom">
      <p>Built with <a href="https://tauri.app/">Tauri</a> + <a href="https://react.dev/">React</a> · Powered by Rust · MIT License.</p>
    </div>
  </footer>

  <script src="main.js"></script>
</body>
</html>
```

- [ ] **Step 2: Verify the page renders (unstyled but complete)**

```bash
cd website && python3 -m http.server 8080
```

Open `http://localhost:8080/` in a browser. Confirm: all six sections visible (hero, features, how, why, roadmap, footer), logo and demo gif load, all links point at valid URLs. Page will look unstyled — that's expected; CSS comes next. Ctrl-C to stop.

- [ ] **Step 3: Commit**

```bash
git add website/index.html
git commit -m "feat(website): full semantic HTML for all six sections"
```

---

### Task 3: Build `website/styles.css` (design system + all components + responsive)

**Files:**
- Create: `website/styles.css`

**Interfaces:**
- Consumes: every class and ID from Task 2's HTML.
- Produces: the "Playful & warm" visual treatment. No JS interaction here.

- [ ] **Step 1: Write the complete `website/styles.css`**

```css
/* ==========================================================================
   Pomotoro landing — Playful & warm
   ========================================================================== */

/* ----- Design tokens ----- */
:root {
  --bg: #FDF6E3;
  --surface: #FFFFFF;
  --primary: #E63946;
  --primary-dark: #C1121F;
  --secondary: #FFB703;
  --text: #2D2A26;
  --muted: #8A8278;
  --border: #F0E6D2;

  --radius-card: 16px;
  --radius-btn: 12px;
  --shadow-card: 0 8px 32px rgba(0, 0, 0, 0.06);
  --shadow-card-hover: 0 12px 40px rgba(0, 0, 0, 0.10);

  --container: 1100px;
  --section-pad: 96px;
  --section-pad-mobile: 64px;

  --font: 'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
}

/* ----- Reset ----- */
*, *::before, *::after { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body {
  margin: 0;
  font-family: var(--font);
  font-size: 1.0625rem;
  line-height: 1.6;
  color: var(--text);
  background: var(--bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
img, svg, video { max-width: 100%; display: block; }
a { color: var(--primary); text-decoration: none; transition: color 150ms ease; }
a:hover { color: var(--primary-dark); }
ul, ol { list-style: none; padding: 0; margin: 0; }
kbd {
  font-family: var(--font);
  font-size: 0.85em;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 1px 6px;
  box-shadow: 0 1px 0 var(--border);
}

/* ----- Layout ----- */
.container {
  max-width: var(--container);
  margin: 0 auto;
  padding: 0 24px;
}

.section {
  padding: var(--section-pad) 0;
}
.section--alt {
  background: rgba(255, 183, 3, 0.06);
}
.section__title {
  font-size: clamp(1.875rem, 4vw, 2.75rem);
  font-weight: 700;
  letter-spacing: -0.02em;
  margin: 0 0 12px;
  text-align: center;
}
.section__lede {
  font-size: 1.125rem;
  color: var(--muted);
  text-align: center;
  max-width: 640px;
  margin: 0 auto 56px;
}
.section__link {
  text-align: center;
  margin-top: 48px;
  font-weight: 500;
}
.section__link a {
  font-size: 1.0625rem;
}

/* ----- Wordmark ----- */
.wordmark {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text);
  font-weight: 700;
  font-size: 1.25rem;
  letter-spacing: -0.01em;
}
.wordmark:hover { color: var(--text); }
.wordmark img { width: 40px; height: 40px; }
.wordmark--footer img { width: 32px; height: 32px; }

/* ----- Buttons ----- */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-family: var(--font);
  font-size: 1rem;
  font-weight: 600;
  line-height: 1;
  padding: 14px 28px;
  border-radius: var(--radius-btn);
  border: 1px solid transparent;
  cursor: pointer;
  text-decoration: none;
  transition: transform 150ms ease, background 150ms ease, color 150ms ease, box-shadow 150ms ease;
}
.btn:hover { transform: translateY(-1px); }
.btn:focus-visible {
  outline: 3px solid var(--secondary);
  outline-offset: 2px;
}
.btn--primary {
  background: var(--primary);
  color: #fff;
}
.btn--primary:hover { background: var(--primary-dark); color: #fff; }
.btn--secondary {
  background: transparent;
  color: var(--text);
  border-color: var(--border);
}
.btn--secondary:hover { background: var(--surface); color: var(--text); }
.btn--link {
  background: transparent;
  color: var(--muted);
  padding: 14px 8px;
  border: none;
  font-weight: 500;
}
.btn--link:hover { color: var(--text); transform: none; }

/* ----- Hero ----- */
.hero {
  padding: 48px 0 var(--section-pad);
  background:
    radial-gradient(ellipse at top, rgba(255, 183, 3, 0.12), transparent 60%),
    radial-gradient(ellipse at bottom right, rgba(230, 57, 70, 0.08), transparent 55%);
}
.hero__inner {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 64px;
}
.hero__content {
  text-align: center;
  max-width: 720px;
  margin: 32px auto 0;
}
.hero__headline {
  font-size: clamp(2.5rem, 6vw, 4.5rem);
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.05;
  margin: 0 0 20px;
}
.hero__subhead {
  font-size: clamp(1.0625rem, 2vw, 1.375rem);
  color: var(--muted);
  margin: 0 auto 36px;
  max-width: 580px;
}
.hero__cta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: center;
  align-items: center;
}
.hero__platforms {
  display: none;
  flex-wrap: wrap;
  gap: 20px;
  justify-content: center;
  margin-top: 20px;
  font-size: 0.9375rem;
}
.hero__platforms.is-expanded { display: flex; }
.platform-link {
  color: var(--muted);
  font-weight: 500;
}
.platform-link:hover { color: var(--primary); }

.hero__demo {
  max-width: 720px;
  margin: 0 auto;
}
.hero__demo a {
  display: block;
  border-radius: var(--radius-card);
  overflow: hidden;
  box-shadow: var(--shadow-card);
  border: 1px solid var(--border);
  background: var(--surface);
  transition: transform 200ms ease, box-shadow 200ms ease;
}
.hero__demo a:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-card-hover);
}
.hero__demo img { width: 100%; height: auto; }
.hero__demo-caption {
  text-align: center;
  color: var(--muted);
  font-size: 0.9375rem;
  margin: 16px 0 0;
}

/* ----- Feature grid ----- */
.feature-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}
.feature-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 28px;
  box-shadow: var(--shadow-card);
  transition: transform 200ms ease, box-shadow 200ms ease;
}
.feature-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-card-hover);
}
.feature-card__icon {
  font-size: 2rem;
  line-height: 1;
  margin-bottom: 12px;
}
.feature-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0 0 6px;
}
.feature-card__text {
  margin: 0;
  color: var(--muted);
  font-size: 0.9375rem;
  line-height: 1.55;
}

/* ----- Steps ----- */
.steps {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
  counter-reset: step;
}
.step {
  text-align: center;
  padding: 24px;
}
.step__number {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--primary);
  color: #fff;
  font-size: 1.5rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px;
  box-shadow: 0 6px 18px rgba(230, 57, 70, 0.25);
}
.step__title {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0 0 8px;
}
.step__text {
  color: var(--muted);
  margin: 0;
  font-size: 1rem;
}

/* ----- Value grid (Why Pomotoro) ----- */
.value-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
}
.value-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 28px;
  text-align: center;
  box-shadow: var(--shadow-card);
  transition: transform 200ms ease, box-shadow 200ms ease;
}
.value-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-card-hover);
}
.value-card__icon {
  font-size: 2rem;
  margin-bottom: 12px;
}
.value-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0 0 6px;
}
.value-card__text {
  margin: 0;
  color: var(--muted);
  font-size: 0.9375rem;
}

/* ----- Roadmap ----- */
.roadmap-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}
.roadmap-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-card);
  padding: 28px;
  box-shadow: var(--shadow-card);
  transition: transform 200ms ease, box-shadow 200ms ease;
}
.roadmap-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-card-hover);
}
.roadmap-card__icon {
  font-size: 2rem;
  margin-bottom: 12px;
}
.roadmap-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0 0 6px;
}
.roadmap-card__text {
  margin: 0;
  color: var(--muted);
  font-size: 0.9375rem;
}

/* ----- Footer ----- */
.footer {
  background: var(--text);
  color: #F5EFE0;
  padding: 64px 0 32px;
  margin-top: var(--section-pad);
}
.footer__inner {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 48px;
  padding-bottom: 48px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}
.wordmark--footer { color: #fff; }
.wordmark--footer:hover { color: #fff; }
.footer__tagline {
  margin: 16px 0 0;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.9375rem;
}
.footer__nav {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 24px;
}
.footer__heading {
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--secondary);
  margin: 0 0 12px;
}
.footer__col ul {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.footer__col a {
  color: rgba(255, 255, 255, 0.8);
  font-size: 0.9375rem;
}
.footer__col a:hover { color: #fff; }
.footer__bottom {
  padding-top: 24px;
  text-align: center;
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.875rem;
}
.footer__bottom a { color: rgba(255, 255, 255, 0.85); }
.footer__bottom a:hover { color: #fff; }

/* ----- Reveal animation (IntersectionObserver adds .is-visible) ----- */
.reveal {
  opacity: 0;
  transform: translateY(16px);
  transition: opacity 400ms ease, transform 400ms ease;
}
.reveal.is-visible {
  opacity: 1;
  transform: none;
}

/* ----- Responsive ----- */
@media (max-width: 1024px) {
  .feature-grid { grid-template-columns: repeat(2, 1fr); }
  .value-grid { grid-template-columns: repeat(2, 1fr); }
  .roadmap-grid { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 768px) {
  :root { --section-pad: var(--section-pad-mobile); }
  .hero { padding: 32px 0 var(--section-pad); }
  .hero__inner { gap: 40px; }
  .hero__cta { flex-direction: column; align-items: stretch; }
  .hero__cta .btn { width: 100%; }
  .btn--link { width: auto !important; align-self: center; }
  .steps { grid-template-columns: 1fr; gap: 16px; }
  .footer__inner { grid-template-columns: 1fr; gap: 32px; text-align: center; }
  .wordmark--footer { justify-content: center; }
  .footer__tagline { text-align: center; }
}

@media (max-width: 560px) {
  .feature-grid,
  .value-grid,
  .roadmap-grid { grid-template-columns: 1fr; }
  .footer__nav { grid-template-columns: 1fr; gap: 32px; }
}

/* ----- Reduced motion ----- */
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.001ms !important;
    transition-duration: 0.001ms !important;
    animation-iteration-count: 1 !important;
    scroll-behavior: auto !important;
  }
  .reveal { opacity: 1; transform: none; }
}
```

- [ ] **Step 2: Verify locally**

```bash
cd website && python3 -m http.server 8080
```

Open `http://localhost:8080/`. Confirm:
- Cream background, tomato red CTAs, Inter font loading.
- Hero, features grid (3 cols on desktop), steps, value cards, roadmap cards all visible.
- Hover effects on cards work (lift + stronger shadow).
- DevTools → toggle device toolbar → iPhone: layout stacks to single column, no horizontal scroll.
- DevTools → Rendering → "Emulate CSS media feature prefers-reduced-motion: reduce" → all transitions stop, content visible (not stuck invisible).

- [ ] **Step 3: Commit**

```bash
git add website/styles.css
git commit -m "feat(website): playful-and-warm design system with responsive layout"
```

---

### Task 4: Build `website/main.js`

**Files:**
- Create: `website/main.js`

**Interfaces:**
- Consumes from HTML: `#download-btn` (sets text + href), `#other-platforms-toggle` (click handler), `#other-platforms` (toggle class), `.reveal` elements (IntersectionObserver targets).
- Produces: dynamic copy + reveal animations. No exports — pure side-effect script.

- [ ] **Step 1: Write the complete `website/main.js`**

```js
(function () {
  'use strict';

  // ----- OS detection for the primary download button -----
  function detectOS() {
    const ua = navigator.userAgent || '';
    if (/Mac/i.test(ua)) return 'macos';
    if (/Windows/i.test(ua)) return 'windows';
    if (/Linux/i.test(ua)) return 'linux';
    return null;
  }

  const PLATFORM_LABELS = {
    macos: 'Download for macOS',
    windows: 'Download for Windows',
    linux: 'Download for Linux'
  };

  const PLATFORM_HREFS = {
    macos: 'https://github.com/kulapoo/pomotoro/releases/latest',
    windows: 'https://github.com/kulapoo/pomotoro/releases/latest',
    linux: 'https://github.com/kulapoo/pomotoro/releases/latest'
  };

  function initDownloadButton() {
    const btn = document.getElementById('download-btn');
    if (!btn) return;
    const os = detectOS();
    if (os && PLATFORM_LABELS[os]) {
      btn.textContent = PLATFORM_LABELS[os];
      btn.href = PLATFORM_HREFS[os];
    }
  }

  // ----- "Other platforms" expandable -----
  function initOtherPlatforms() {
    const toggle = document.getElementById('other-platforms-toggle');
    const panel = document.getElementById('other-platforms');
    if (!toggle || !panel) return;

    toggle.addEventListener('click', function () {
      const isOpen = panel.classList.toggle('is-expanded');
      toggle.setAttribute('aria-expanded', String(isOpen));
    });
  }

  // ----- Reveal on scroll (respects reduced motion) -----
  function initReveal() {
    const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const items = document.querySelectorAll('.reveal');

    if (prefersReduced || !('IntersectionObserver' in window)) {
      items.forEach(function (el) { el.classList.add('is-visible'); });
      return;
    }

    const observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
          observer.unobserve(entry.target);
        }
      });
    }, { rootMargin: '0px 0px -10% 0px', threshold: 0.1 });

    items.forEach(function (el) { observer.observe(el); });
  }

  // ----- Run on DOM ready -----
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () {
      initDownloadButton();
      initOtherPlatforms();
      initReveal();
    });
  } else {
    initDownloadButton();
    initOtherPlatforms();
    initReveal();
  }
})();
```

- [ ] **Step 2: Verify locally**

```bash
cd website && python3 -m http.server 8080
```

Open `http://localhost:8080/`. Confirm:
- The Download button label shows your current OS (e.g. "Download for Linux").
- Clicking "Other platforms ↓" reveals the three platform links; clicking again collapses them.
- Scrolling down: sections fade in smoothly.
- DevTools → Rendering → "Emulate prefers-reduced-motion: reduce" → reload → all content visible immediately, no fade animation.

- [ ] **Step 3: Commit**

```bash
git add website/main.js
git commit -m "feat(website): OS-aware download CTA, expandable platforms, scroll reveals"
```

---

### Task 5: Create the Pages deploy workflow

**Files:**
- Create: `.github/workflows/deploy-website.yml`

**Interfaces:**
- Consumes: `website/` directory (deployed wholesale).
- Produces: a published GitHub Pages site on every push to `main` that touches `website/**`.

- [ ] **Step 1: Write `.github/workflows/deploy-website.yml`**

```yaml
name: Deploy website

on:
  push:
    branches: [main]
    paths: ['website/**', '.github/workflows/deploy-website.yml']
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

# Allow only one concurrent deployment, don't cancel in-progress runs.
concurrency:
  group: deploy-website
  cancel-in-progress: false

jobs:
  deploy:
    name: Deploy to GitHub Pages
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Pages
        uses: actions/configure-pages@v5

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: website/

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Lint the YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-website.yml'))" && echo "OK"
```

Expected: prints `OK`. If yaml module isn't available, fall back to visual inspection — confirm indentation (2-space), action versions, and the four-step structure.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-website.yml
git commit -m "ci(website): deploy website/ to GitHub Pages on push to main"
```

---

### Task 6: Final local verification + push

**Files:** none modified.

- [ ] **Step 1: Full local verification pass**

```bash
cd website && python3 -m http.server 8080
```

Open `http://localhost:8080/`. Walk through this checklist:

| Check | Expected |
|---|---|
| Page loads without 404s in DevTools Network tab | All assets 200 |
| Hero headline copy | "Charge through your focus sessions." |
| Hero demo gif autoplays | Yes |
| Click hero gif → opens demo.mp4 | Yes |
| Download button reflects your OS | Yes (e.g. "Download for Linux") |
| Click "Other platforms ↓" | Reveals 3 platform links |
| Features grid shows 10 cards | Yes |
| How it works shows 3 numbered steps | Yes |
| Why Pomotoro shows 4 value cards | Yes |
| Roadmap shows 3 cards + "See full roadmap" link | Yes |
| Footer has 3 nav columns + bottom line | Yes |
| Resize to mobile width | Stacks cleanly, no horizontal scroll |
| Emulate prefers-reduced-motion | All content visible, no animations |
| Lighthouse Audit (Chrome DevTools) | Performance ≥ 90, Accessibility ≥ 95 |

- [ ] **Step 2: Verify no broken internal links**

In DevTools Console:
```js
Array.from(document.querySelectorAll('a[href^="assets/"]')).forEach(a => fetch(a.href).then(r => console.log(r.status, a.href)));
```

All should be 200.

- [ ] **Step 3: Update README to mention the website (optional polish)**

Skip unless requested — the website links to the README, not the other way around yet.

- [ ] **Step 4: Push and trigger the first deploy**

```bash
git push origin main
```

Then visit the **Actions** tab on GitHub and watch the "Deploy website" run. When it finishes, the workflow's job summary will show the deployed URL.

- [ ] **Step 5: Enable Pages (one-time manual step)**

If this is the first deploy, GitHub may require:
1. Repo **Settings → Pages → Build and deployment → Source** = **"GitHub Actions"**.
2. Re-run the workflow via the Actions tab if the first run failed because this wasn't set yet.

- [ ] **Step 6: Verify production URL**

Open `https://kulapoo.github.io/pomotoro/`. Walk the Step 1 checklist again on production. Confirm:
- All asset paths resolve under `/pomotoro/` (no broken images or links).
- Download CTA works on at least one OS you can test.
- Lighthouse production score ≥ 90/95.

**Done.**

---

## Plan Self-Review (executed inline)

**1. Spec coverage:**
- ✅ Stack: static HTML/CSS/JS — Task 1-4.
- ✅ Repo layout `website/` — Task 1.
- ✅ Deployment via Actions — Task 5.
- ✅ Hero with demo + OS-aware CTA + Other platforms expander + Star — Task 2 (HTML), Task 4 (JS).
- ✅ Features grid, 10 cards — Task 2.
- ✅ How it works (3 steps) — Task 2.
- ✅ Why Pomotoro (4 cards + Motivation link) — Task 2.
- ✅ Roadmap teaser (3 cards + link) — Task 2.
- ✅ Footer (3 columns + bottom line) — Task 2.
- ✅ Color tokens, Inter, 16/12 radii, container, section rhythm — Task 3.
- ✅ Card hover lift — Task 3.
- ✅ IntersectionObserver reveal + reduced-motion — Task 3 (CSS) + Task 4 (JS).
- ✅ Responsive breakpoints — Task 3.
- ✅ Accessibility (semantic HTML, alt text, focus, AA) — Task 2 + Task 3.
- ✅ Assets committed into `website/assets/` — Task 1.
- ✅ Logo SVG (tomato + horns) — Task 1.
- ✅ Headline "Charge through your focus sessions." — Task 2.
- ✅ Copy rewritten punchier (not pasted) — Task 2.
- ✅ Relative asset paths — Task 2 + Task 3 verification.
- ✅ Verification: local http.server + Lighthouse + production check — Task 6.

**2. Placeholder scan:** no TBDs, TODOs, "implement later", "similar to Task N", or "add appropriate handling". All steps contain concrete code or commands.

**3. Type/name consistency:** IDs referenced by JS (`#download-btn`, `#other-platforms-toggle`, `#other-platforms`, `.reveal`) match exactly between Task 2 HTML and Task 4 JS. Class names referenced by CSS (`.is-expanded`, `.is-visible`, `.btn--primary`, etc.) match between Task 2 HTML and Task 3 CSS.
