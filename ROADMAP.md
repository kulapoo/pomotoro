# Roadmap

Planned features and direction for Pomotoro. Items are exploratory and may shift
based on feedback and contributor interest.

---

## 📊 Statistics

Deeper insight into your focus patterns.

- **Session history** — timeline of completed focus / break sessions with phase breakdown.
- **Daily / weekly / monthly summaries** — total focus time, sessions completed, tasks finished.
- **Streaks** — consecutive focused days and consistency tracking.
- **Task breakdowns** — time spent per task, per tag, and per project.
- **Charts & trends** — heatmaps, bar charts, and rolling averages.
- **Export** — CSV / JSON export for external analysis and spreadsheet tooling.
- **Goals** — daily or weekly focus-time targets with progress indicators.

---

## 💻 Command-line interface (CLI)

Drive Pomotoro from the terminal for power users and scripting.

- **Status commands** — `pomotoro status`, `pomotoro current`, `pomotoro next-task`.
- **Timer control** — `pomotoro start`, `pomotoro pause`, `pomotoro reset`, `pomotoro skip`.
- **Task management** — `pomotoro task add|list|done|remove` for headless workflows.
- **Config access** — read and write settings from the command line.
- **Headless mode** — run the engine without the GUI (background daemon + IPC).
- **Shell completion** — generated completions for bash, zsh, fish, and PowerShell.

---

## ⚡ Command hooks (event-driven shell actions)

Run user-defined shell commands in response to Pomotoro lifecycle events.

- **Event types** — `timer.started`, `timer.paused`, `timer.resumed`, `timer.reset`,
  `timer.expired`, `task.started`, `task.completed`, `phase.changed`.
- **Per-event commands** — configure a shell command per event (e.g. on
  `timer.started` → `bash /path/to/start-focus-music.sh`).
- **Templated arguments** — pass context to the command via placeholders such as
  `{phase}`, `{task}`, `{duration_seconds}`, `{session_index}`.
- **Safe defaults** — disabled by default; explicit opt-in per event with a dry-run
  preview of the resolved command.
- **Logging & diagnostics** — capture stdout/stderr and exit codes for review in a
  dedicated hooks log view.

---

## 🔭 Longer-term explorations

- **Cloud sync** — optional end-to-end-encrypted sync across devices.
- **Mobile companion** — read-only status + push notifications.
- **Themes & customization** — user-built color schemes and timer visualizations.
- **Plugin system** — extend Pomotoro with user-authored event handlers and widgets.
- **Focus metrics API** — local HTTP/socket endpoint for integration with other tools.
