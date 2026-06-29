# Changelog

All notable changes to Pomotoro are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6]
### Fixed

- fix task description not accepting empty value on edit

## [0.1.5]
### Fixed

- Fix hide window behavior with screen blocker enabled settings

## [0.1.4] - 2026-06-29

### Changed
- Map appname into notification title for Linux

## [0.1.3] - 2026-06-29

### Fixed
- switching task when auto advance is disabled triggers pause_timer error

## [0.1.1] - 2026-06-29

### Fixed
- App icon now uses the toro brand mark (was Tauri default).
- macOS window opens centered at 490x844 instead of filling the screen.

## [0.1.0] - 2026-06-29

### Added
- Pomodoro focus engine with configurable focus / short-break / long-break cycles.
- Task management with multi-session tasks, tags, search, status filters, and live progress.
- Ambient focus audio (rain, forest, ocean, white noise, café, fireplace, thunderstorm) plus work/break chimes.
- Desktop notifications, sound alerts, phase-transition and task-completion alerts.
- Optional full-screen screen blocker that enforces breaks.
- System tray with live countdown baked into the icon and tooltip.
- Keyboard shortcut (`Ctrl/Meta+Tab`) to cycle incomplete tasks.
- Settings hub for timer durations, automation, appearance, window, audio, and storage.
- Cross-platform desktop builds for Windows, macOS (universal), and Linux.
- Tag-triggered release workflow producing `.deb`, `.AppImage`, `.dmg`, `.msi`, and `.exe` installers.
- Continuous integration matrix across Windows, macOS, and Linux.

[Unreleased]: https://github.com/kulapoo/pomotoro/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/kulapoo/pomotoro/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kulapoo/pomotoro/releases/tag/v0.1.0
