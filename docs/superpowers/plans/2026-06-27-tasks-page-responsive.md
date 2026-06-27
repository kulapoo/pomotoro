# Tasks Page Responsive Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `TasksPage` reflow gracefully at narrow window widths (default window is 490×844, content area ~378px) using mobile-first Tailwind utilities, with no JS or API changes.

**Architecture:** Mobile-first Tailwind class edits only. Each affected component's **base** classes produce the stacked layout; `sm:`/`md:` variants relax back to the current wide layout. No new components, props, hooks, or files. `TaskRow` is intentionally untouched (already wraps correctly).

**Tech Stack:** React 19, Tailwind CSS v4 (`@import 'tailwindcss'`, default breakpoints `sm`=640px / `md`=768px), TypeScript, ESLint, Prettier with `prettier-plugin-tailwindcss` (auto-sorts classes).

## Global Constraints

- **Mobile-first ordering:** base classes = stacked layout; `sm:`/`md:` = wider layout. Never use `max-sm:` for stacking (the default 490px window is below `sm`, so stacked must be the base).
- **No unit tests.** The `pomotoro-react-ui` package has no test framework (no vitest/jest, zero `*.test.*` files) and these are pure className/CSS changes. Per the design spec, verification per task = `format` + `lint` + `typecheck` + manual visual check. Do NOT add a test framework — YAGNI.
- **Scope:** only the 4 components listed below. Do NOT edit `TasksPage.tsx`, `App.tsx`, `Sidebar.tsx`, `TaskRow.tsx`, or `TaskFormModal.tsx`.
- **Keep `max-w-2xl`** on the page container (unchanged — not in scope).
- **Run format after every edit:** `prettier-plugin-tailwindcss` auto-sorts Tailwind classes; class order in this plan is illustrative and will be normalized by `format`.
- **All commands run from repo root** (`/home/jpt/src/oss/pomotoro`) using `pnpm --filter pomotoro-react-ui <script>`.
- **Commit after each task** with conventional-commit style matching repo history (e.g. `feat(tasks): ...`, `style(tasks): ...`).

## File Structure

No files created or deleted. Four existing files modified, each independently:

- `apps/react-ui/src/pages/tasks/components/TasksHeader.tsx` — stats wrap below title on narrow widths.
- `apps/react-ui/src/pages/tasks/components/QuickAddBar.tsx` — title input on its own row; sessions/Add/Detail wrap to a second row.
- `apps/react-ui/src/pages/tasks/components/TaskSearchBar.tsx` — defensive `flex-wrap` overflow guard.
- `apps/react-ui/src/pages/tasks/components/BulkSelectToolbar.tsx` — defensive `flex-wrap` overflow guard.

Not modified: `TaskRow.tsx` (already `flex flex-wrap` with `basis-40` title + `shrink-0` actions; verified to fit ~378px).

---

## Task 1: TasksHeader — stats wrap below title on narrow widths

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/components/TasksHeader.tsx:22`

**Interfaces:**
- Consumes: none (leaf presentational component).
- Produces: none (no API change; same props).

On the default narrow window the title + "Reset all" button + three `StatBadge`s crowd each other. Stack the stats below the title block on narrow widths; relax to the current side-by-side header at `sm:`.

- [ ] **Step 1: Edit the header container to be mobile-first**

In `apps/react-ui/src/pages/tasks/components/TasksHeader.tsx`, change the container `<div>` on line 22 from:

```tsx
<div className="mb-5 flex items-start justify-between">
```

to:

```tsx
<div className="mb-5 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
```

No other edits in this file. The stats `<div className="flex gap-2">` (lines 38–54) stays unchanged — when the parent is `flex-col` it simply renders on its own row beneath the title block.

- [ ] **Step 2: Format, lint, and typecheck**

Run from repo root:

```bash
pnpm --filter pomotoro-react-ui format
pnpm --filter pomotoro-react-ui lint
pnpm --filter pomotoro-react-ui typecheck
```

Expected: `format` may reorder classes (fine). `lint` and `typecheck` both pass with no errors.

- [ ] **Step 3: Visual check at three widths**

Run `pnpm --filter pomotoro-react-ui dev` and open the dev URL (default http://localhost:5173). Navigate to the Tasks page. Verify the header at three browser window widths:

- **~390px** (below `sm`, default-ish): "Tasks" title + "Reset all" button on the top row; the three stat badges wrap to a second row beneath. Nothing overflows.
- **≥ 640px** (`sm`): title block on the left, stat badges on the right — the current desktop layout.
- **≥ 900px** (`md`+): unchanged desktop layout, `max-w-2xl` centered.

Expected: smooth wrap/unwrap at the `sm` boundary, no horizontal overflow at any width.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/pages/tasks/components/TasksHeader.tsx
git commit -m "style(tasks): stack header stats below title on narrow widths"
```

---

## Task 2: QuickAddBar — title input over actions

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/components/QuickAddBar.tsx:29-63`

**Interfaces:**
- Consumes: none.
- Produces: none (same props, same handlers).

This is the most crowded element at narrow widths: title input + sessions number + "Add" + "Detail" in one row. Restructure so the title input is full-width on its own row, and the three trailing controls (sessions + Add + Detail) wrap to a second row. Relax back to the current single row at `sm:`.

> **Critical detail:** the title input currently has always-on `flex-1`. In a `flex-col` container, `flex-1` would stretch the input **vertically** (main axis). The input must be `sm:flex-1` instead — full width in column mode comes from the default `align-items: stretch`, and `flex-1` is only needed in the `sm:` row to grow horizontally.

- [ ] **Step 1: Restructure the container and wrap the action controls**

In `apps/react-ui/src/pages/tasks/components/QuickAddBar.tsx`, replace the entire container `<div>` block (lines 29–64) — from the opening `<div className="mb-4 flex gap-2">` through its closing `</div>` — with:

```tsx
<div className="mb-4 flex flex-col gap-2 sm:flex-row sm:gap-2">
  <input
    type="text"
    value={title}
    onChange={(e) => setTitle(e.target.value)}
    onKeyDown={(e) => e.key === 'Enter' && handleQuickAdd()}
    placeholder="Quick add task…"
    className="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring min-w-0 rounded-xl border px-4 py-2.5 text-sm focus:ring-2 focus:outline-none sm:flex-1"
  />
  <div className="flex gap-2">
    <input
      type="number"
      min={1}
      max={20}
      value={sessions}
      onChange={(e) => setSessions(Number(e.target.value))}
      title="Sessions"
      className="border-input bg-background text-foreground focus:ring-ring w-16 rounded-xl border px-3 py-2.5 text-center text-sm focus:ring-2 focus:outline-none"
    />
    <button
      onClick={handleQuickAdd}
      disabled={!title.trim() || atLimit}
      title={atLimit ? `Task limit reached (${MAX_TASKS})` : undefined}
      className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
    >
      <Plus size={16} />
      Add
    </button>
    <button
      onClick={onOpenCreate}
      title="Create task with full details"
      className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex items-center gap-2 rounded-xl border px-4 py-2.5 text-sm transition-colors"
    >
      <Pencil size={15} />
      Detail
    </button>
  </div>
</div>
```

The only functional changes versus the original:
1. Container: `flex gap-2` → `flex flex-col gap-2 sm:flex-row sm:gap-2`.
2. Title input: `min-w-0 flex-1` → `min-w-0 … sm:flex-1` (prevents vertical stretch in column mode).
3. A new wrapping `<div className="flex gap-2">` around the sessions input + Add + Detail buttons (these three elements are otherwise byte-identical to the original).

- [ ] **Step 2: Format, lint, and typecheck**

```bash
pnpm --filter pomotoro-react-ui format
pnpm --filter pomotoro-react-ui lint
pnpm --filter pomotoro-react-ui typecheck
```

Expected: all pass. (`format` will sort classes; the `sm:flex-1` placement is preserved.)

- [ ] **Step 3: Visual + behavior check at three widths**

Run `pnpm --filter pomotoro-react-ui dev`, open the Tasks page, and verify:

- **~390px:** title input is full-width on row 1; sessions number + "Add" + "Detail" sit left-to-right on row 2. Nothing overflows. The title input is a single-line height (NOT stretched tall — confirms `sm:flex-1` is correct).
- **≥ 640px:** all four controls back on one row, title input growing to fill — identical to the pre-change layout.
- **Behavior:** type a title and press Enter → task creates (toast "Task added"). Click "Detail" → opens the create modal. Number input still bounds 1–20.

Expected: layout reflows at `sm` boundary; all quick-add behavior unchanged.

- [ ] **Step 4: Commit**

```bash
git add apps/react-ui/src/pages/tasks/components/QuickAddBar.tsx
git commit -m "style(tasks): wrap QuickAddBar actions below title on narrow widths"
```

---

## Task 3: Defensive overflow guards (TaskSearchBar + BulkSelectToolbar)

**Files:**
- Modify: `apps/react-ui/src/pages/tasks/components/TaskSearchBar.tsx:18`
- Modify: `apps/react-ui/src/pages/tasks/components/BulkSelectToolbar.tsx:21`

**Interfaces:**
- Consumes: none.
- Produces: none.

Both bars already fit at ~378px but are tight and have no overflow safety net. Add `flex-wrap` so controls can drop to a second row instead of overflowing if localized text or future content grows. No structural or behavior change.

- [ ] **Step 1: Add flex-wrap to TaskSearchBar**

In `apps/react-ui/src/pages/tasks/components/TaskSearchBar.tsx`, change line 18 from:

```tsx
<div className="mb-6 flex gap-2">
```

to:

```tsx
<div className="mb-6 flex flex-wrap gap-2">
```

- [ ] **Step 2: Add flex-wrap to BulkSelectToolbar**

In `apps/react-ui/src/pages/tasks/components/BulkSelectToolbar.tsx`, change the container `<div>` on line 21 from:

```tsx
<div className="border-border bg-card mb-4 flex items-center justify-between gap-3 rounded-xl border px-4 py-2.5">
```

to:

```tsx
<div className="border-border bg-card mb-4 flex flex-wrap items-center justify-between gap-3 gap-y-2 rounded-xl border px-4 py-2.5">
```

(`gap-y-2` tightens the vertical gap when the action group wraps; harmless when it doesn't.)

- [ ] **Step 3: Format, lint, and typecheck**

```bash
pnpm --filter pomotoro-react-ui format
pnpm --filter pomotoro-react-ui lint
pnpm --filter pomotoro-react-ui typecheck
```

Expected: all pass.

- [ ] **Step 4: Visual check at narrow width**

Run `pnpm --filter pomotoro-react-ui dev`, open the Tasks page:

- **~390px:** search input + status `<select>` sit on one row (they fit); if a future longer locale string appeared, the select would now wrap instead of overflowing. Select one or more tasks to surface the `BulkSelectToolbar` and confirm "N selected / Select all" and "Cancel / Reset selected" render without overflow.

Expected: no visual regression; bars render as before at normal widths, with wrap as a safety net.

- [ ] **Step 5: Commit**

```bash
git add apps/react-ui/src/pages/tasks/components/TaskSearchBar.tsx apps/react-ui/src/pages/tasks/components/BulkSelectToolbar.tsx
git commit -m "style(tasks): add flex-wrap overflow guards to search and bulk-select bars"
```

---

## Final Verification

After all three tasks:

- [ ] **Run full checks from repo root:**

```bash
pnpm --filter pomotoro-react-ui format:check
pnpm --filter pomotoro-react-ui lint
pnpm --filter pomotoro-react-ui typecheck
pnpm --filter pomotoro-react-ui build
```

Expected: all pass; `build` (`tsc -b && vite build`) succeeds.

- [ ] **Full visual sweep at 390px / 640px / 900px:** confirm the entire Tasks page (header, quick-add, search, bulk toolbar, incomplete + completed lists, task rows) reflows with no horizontal overflow at any width, and that the desktop (≥900px) layout is visually identical to before the change.

- [ ] **Confirm no out-of-scope files were touched:**

```bash
git diff --stat main
```

Expected: only the four component files in `apps/react-ui/src/pages/tasks/components/` (and the plan/spec docs). `TasksPage.tsx`, `App.tsx`, `Sidebar.tsx`, `TaskRow.tsx`, and `TaskFormModal.tsx` must NOT appear.
