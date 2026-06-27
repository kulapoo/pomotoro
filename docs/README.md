# Pomotoro Documentation

Reference index for the Pomotoro codebase — a native Pomodoro focus timer built
with Rust + Tauri on **Clean Architecture + DDD**. For the project pitch and
end-user info, see the [root README](../README.md). For why the project exists,
see [MOTIVATION.md](./MOTIVATION.md).

## Start here

- **[Getting Started](./getting-started.md)** — set up your environment and make your first change.
- **[Architecture Overview](./architecture/overview.md)** — the four layers and the dependency rule.

## Architecture — *understand the system*

How the codebase is structured and how data moves through it.

- [Overview](./architecture/overview.md) — Clean Architecture layers & tech stack
- [Domain Layer](./architecture/domain-layer.md) — entities, value objects, aggregates, events
- [Use Cases Layer](./architecture/usecases-layer.md) — application services & DTOs
- [Infrastructure Layer](./architecture/infra-layer.md) — adapters, SQLite repos, event bus
- [Data Flow](./architecture/data-flow.md) — tracing a request UI → Domain → persistence
- [Event System](./architecture/events.md) — domain events, handlers, choreography
- [Dependencies & Boundaries](./architecture/dependencies.md) — the dependency rule, enforced

## Reference — *look things up*

Precise, lookup-oriented material.

- [Module Map](./reference/module-map.md) — file-by-file tour of `core/` + `apps/`
- [API Index](./reference/api-index.md) — key functions, traits, and Tauri commands
- [Domain Model](./reference/domain-model.md) — entities, value objects, events, aggregate boundaries
- [Feature docs](./reference/features/) — per-feature design notes
  - [Screen Blocker](./reference/features/screen-blocker.md)
  - [Task Reset](./reference/features/task-reset-status.md)

## Standards — *how we write code*

Conventions every contribution should follow.

- [Style Guide](./standards/style-guide.md) — the canonical development guidelines
- [Naming Convention](./standards/naming.md) — context-aware, no-stutter naming
- [File Structure](./standards/file-structure.md) — domain-centric file layout rules
- [Code Templates](./standards/code-templates.md) — copy-paste entity / repository / use-case skeletons
- [Domain Services](./standards/domain-services.md) — what they are and aren't
- [Patterns](./standards/patterns.md) — repository, builder, event-driven, DI, type-safety patterns

## Workflows — *how we do tasks*

Step-by-step guides for common activities.

- [Development Workflow](./workflows/development.md) — git branches, commits, CI, release process
- [Adding a Feature](./workflows/adding-a-feature.md) — full Clean-Architecture feature walkthrough
- [Fixing a Bug](./workflows/fixing-a-bug.md) — reproduce → failing test → root cause → fix
- [Testing](./workflows/testing.md) — test pyramid, layer-specific tests, commands
- [Code Review](./workflows/code-review.md) — PR template, review checklist, review levels

## Quick reference

All build/test/dev commands run through [`just`](../justfile) — run `just` with
no arguments to see them all. The essentials:

```bash
just dev          # dev server (Vite + Tauri, hot-reload)
just test         # all workspace tests
just ci           # full gate: test, check, fmt, clippy, react lint+typecheck
```

Per-layer tests: `just test-domain` · `just test-usecases` · `just test-infra`.
See the root [README's Development section](../README.md#-development) for the
full command table and database-migration commands.
