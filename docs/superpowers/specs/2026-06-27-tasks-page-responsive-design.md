# Tasks Page Responsive Layout — Design

**Date:** 2026-06-27
**Status:** Approved (pending implementation plan)
**Scope:** `apps/react-ui/src/pages/tasks/`

## Goal

Make `TasksPage` reflow gracefully at narrow window widths so nothing overflows or gets cramped, while keeping the current layout on wider windows. The trigger: the app's **default window is 490×844** (`apps/tauri-app/tauri.conf.json:18`), so "narrow" is not an edge case — it is the primary viewport.

## Problem

`TasksPage` is centered at `max-w-2xl` and is composed of several top bars whose controls sit in single horizontal rows designed for wide screens. At the default window width the content area is only ~378px (490px window − 64px sidebar `w-16` − 48px `main` padding), and the bars break down:

- **`QuickAddBar`** — four controls in one `flex` row: title input (`flex-1`) + sessions number (`w-16`) + "Add" + "Detail". The two buttons and the fixed-width number input consume ~200px, squeezing the title input to near-nothing.
- **`TasksHeader`** — title + "Reset all" button on the left, three `StatBadge`s on the right. The three badges are crowded against the title at ~378px.
- **`BulkSelectToolbar`** — fits but tight; the Cancel/Reset group can collide with the "selected" count.

The whole codebase has exactly one responsive class (`md:p-10` on `<main>` in `App.tsx:50`), so there is no existing responsive convention to follow — this spec establishes the mobile-first one.

## Chosen Behavior

**Mobile-first Tailwind utilities.** Each affected component's **base** classes produce the stacked/narrow layout, and `sm:`/`md:` variants relax back toward the current wide layout. Pure CSS — no JS, no new props, no API changes, no new hooks.

### Breakpoint map

The sidebar is a fixed `w-16` (64px), so Tailwind's viewport breakpoints track the content width cleanly:

| Window width                | Content area*            | Tailwind | Layout                |
| --------------------------- | ------------------------ | -------- | --------------------- |
| < 640px (incl. default 490) | ~378–574px               | base     | stacked               |
| ≥ 640px                     | ~528px+                  | `sm:`    | stacking relaxed      |
| ≥ 768px                     | ~656px (`max-w-2xl` binds) | `md:`  | full current layout   |

*content ≈ window − 64px sidebar − 48px padding (`p-6`).

## Design

### Change 1 — `TasksHeader` (`components/TasksHeader.tsx`)

Stats wrap below the title on narrow widths.

- Container (line 22): `flex items-start justify-between` → `flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between`.
- The stats `<div className="flex gap-2">` group is unchanged (it sits on its own row when the parent is `flex-col`).

### Change 2 — `QuickAddBar` (`components/QuickAddBar.tsx`)

Title input on its own row; sessions + Add + Detail wrap to a second row (per agreed choice: "input over actions").

- Container (line 29): `flex gap-2` → `flex flex-col gap-2 sm:flex-row sm:gap-2`.
- Wrap the three trailing controls (sessions `<input>`, "Add" `<button>`, "Detail" `<button>`, lines 38–63) in a new `<div className="flex gap-2">`. The title input (lines 30–37) becomes the first (full-width) child of the column.
- On `sm:+` the column collapses back to the current single row; the new wrapper `<div>` then behaves as an inline group.

### Change 3 — `TaskSearchBar` (`components/TaskSearchBar.tsx`)

Already fine at narrow widths (search `flex-1` + compact `<select>`). Defensive-only:

- Container (line 18): `flex gap-2` → `flex flex-wrap gap-2` so the `<select>` can never force horizontal overflow. No structural change.

### Change 4 — `BulkSelectToolbar` (`components/BulkSelectToolbar.tsx`)

Allow the action group to drop to a second row if needed.

- Container (line 21): `flex items-center justify-between gap-3` → `flex flex-wrap items-center justify-between gap-3 gap-y-2`. No behavior change; pure safety net.

### Change 5 — `TaskRow` (`components/TaskRow.tsx`): no change

Already `flex flex-wrap` (line 48) with the title at `min-w-0 flex-1 basis-40` and the action group at `shrink-0 ml-auto`. Verified: title basis (~160px) + action group (~200px) fits within ~378px minus the row's `px-4` padding. No edit.

## Architectural Decisions (the "no"s)

1. **No container queries.** `@container` would be more "correct" (react to the page's own width, sidebar-independent), but the sidebar is a fixed 64px and the codebase has zero container-query usage. Viewport breakpoints track content width closely enough; introducing a new query mechanism here is unjustified novelty.

2. **No JS `useMediaQuery` hook.** A hook could change component *composition* per breakpoint, but every change here is pure layout reflow — exactly what CSS does. A hook would add re-renders and a new dependency for no benefit.

3. **No change to `App.tsx` / `<main>` / `Sidebar`.** Out of scope. `main`'s `p-6 md:p-10` already adapts; the sidebar is fixed. The `max-w-2xl` cap on `TasksPage` is deliberately kept (the agreed goal: stack down on narrow, keep the cap on desktop).

4. **Mobile-first ordering (base = stacked, not `max-sm:` = stacked).** The default window is *below* `sm`, so the stacked layout must be the base. Using `max-sm:` for stacking would invert this and fight Tailwind's mobile-first convention; base-stacked is idiomatic and matches `App.tsx`'s `md:` escalation pattern.

## Verification

No unit tests apply (pure className/CSS). Verification is typecheck + lint + manual visual sweep:

- `pnpm --filter react-ui lint` and `pnpm --filter react-ui typecheck` (or repo equivalent) after edits.
- Visual check at three window widths:
  - **390px** (default-ish, below `sm`) — all top bars stacked, nothing overflows.
  - **640px** (`sm` boundary) — stacking relaxes to rows.
  - **900px** (≥ `md`) — current desktop layout intact, `max-w-2xl` centered.

## Out of Scope / Follow-ups

- Establishing a project-wide responsive convention / design tokens for breakpoints (this spec uses Tailwind defaults).
- `TaskFormModal` responsive behavior (modal, separate concern).
- Phone-width (<400px) targeted layouts — the default 490px window is the floor; no narrower target identified.
- Container-query refactor if a future non-fixed sidebar makes viewport breakpoints inaccurate.

## References

- `apps/react-ui/src/pages/tasks/TasksPage.tsx:78` — page container (`max-w-2xl`, kept).
- `apps/react-ui/src/app/App.tsx:50` — `<main className="p-6 md:p-10">`, the only existing responsive class.
- `apps/react-ui/src/components/layout/Sidebar.tsx:18` — fixed `w-16` sidebar.
- `apps/tauri-app/tauri.conf.json:18` — default 490×844 window.
- `apps/react-ui/src/index.css:1` — Tailwind v4 (`@import 'tailwindcss'`), default breakpoints.
- `apps/react-ui/src/pages/tasks/components/{TasksHeader,QuickAddBar,TaskSearchBar,BulkSelectToolbar,TaskRow}.tsx` — edit sites.
