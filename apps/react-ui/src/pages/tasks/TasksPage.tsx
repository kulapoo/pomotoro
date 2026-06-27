import { useTaskStore } from '@/pages/tasks/useTasks'
import { useTaskActions } from '@/pages/tasks/hooks/useTaskActions'
import { useTaskFilters } from '@/pages/tasks/hooks/useTaskFilters'
import { useTaskSelection } from '@/pages/tasks/hooks/useTaskSelection'
import { TaskFormModal } from '@/pages/tasks/components/TaskFormModal'
import { TasksHeader } from '@/pages/tasks/components/TasksHeader'
import { QuickAddBar } from '@/pages/tasks/components/QuickAddBar'
import { TaskSearchBar } from '@/pages/tasks/components/TaskSearchBar'
import { BulkSelectToolbar } from '@/pages/tasks/components/BulkSelectToolbar'
import { TaskListState } from '@/pages/tasks/components/TaskListState'
import { IncompleteTaskList } from '@/pages/tasks/components/IncompleteTaskList'
import { CompletedTaskList } from '@/pages/tasks/components/CompletedTaskList'
import { LoadingOverlay } from '@/components/LoadingOverlay'
import { useConfirm } from '@/components/ConfirmProvider'
import { useSettingsStore } from '@/pages/settings/useSettings'
import { useTimerStore, isTimerRunning } from '@/pages/timer/useTimer'
import { DEFAULT_DURATIONS } from '@/lib/duration'
import type { Page } from '@/app/types'

interface TasksPageProps {
  onNavigate: (page: Page) => void
}

export function TasksPage({ onNavigate }: TasksPageProps) {
  const tasks = useTaskStore((s) => s.tasks)
  const isLoading = useTaskStore((s) => s.isLoading)
  const error = useTaskStore((s) => s.error)
  const isBusy = useTaskStore((s) => s.isBusy)
  const activeTaskId = useTimerStore((s) => s.timer?.task_id)
  const timerRunning = useTimerStore((s) => (s.timer ? isTimerRunning(s.timer) : false))
  const { confirm } = useConfirm()

  const defaultSessions =
    useSettingsStore((s) => s.config?.timer.sessions_until_long_break) ??
    DEFAULT_DURATIONS.sessionsUntilLongBreak

  const filters = useTaskFilters(tasks, activeTaskId)
  const actions = useTaskActions(onNavigate)
  const selection = useTaskSelection({
    visibleIds: filters.visibleIds,
    completedIds: filters.completedIds,
    search: filters.search,
    statusFilter: filters.statusFilter,
  })

  const handleResetSelected = async () => {
    const ids = selection.selectedArray
    if (ids.length === 0) return
    const confirmed = await confirm({
      title: 'Reset Tasks',
      message: `Reset ${ids.length} selected task${ids.length === 1 ? '' : 's'}? This zeroes their progress.`,
      variant: 'danger',
      confirmLabel: 'Reset',
    })
    if (!confirmed) return
    const count = await actions.resetMany(ids)
    if (count > 0) selection.clearSelection()
  }

  const handleResetAll = async () => {
    if (tasks.length === 0) return
    const confirmed = await confirm({
      title: 'Reset All Tasks',
      message: `Reset ALL ${tasks.length} task${tasks.length === 1 ? '' : 's'}? This zeroes progress on every task.`,
      variant: 'danger',
      confirmLabel: 'Reset All',
    })
    if (!confirmed) return
    selection.clearSelection()
    await actions.resetMany(tasks.map((t) => t.id))
  }

  const navigateToTimer = () => onNavigate('timer')

  return (
    <div className="mx-auto w-full max-w-2xl">
      <LoadingOverlay open={isBusy} />
      {actions.showModal && (
        <TaskFormModal task={actions.editTask} onClose={actions.closeModal} />
      )}

      <TasksHeader
        total={filters.total}
        activeCount={filters.activeCount}
        completedCount={filters.completedCount}
        hasTasks={tasks.length > 0}
        isBusy={isBusy}
        onResetAll={handleResetAll}
      />

      <QuickAddBar defaultSessions={defaultSessions} onOpenCreate={actions.openCreate} />

      <TaskSearchBar
        search={filters.search}
        onSearchChange={filters.setSearch}
        statusFilter={filters.statusFilter}
        onStatusChange={filters.setStatusFilter}
      />

      {selection.size > 0 && (
        <BulkSelectToolbar
          selectedCount={selection.size}
          isAllVisibleSelected={selection.isAllVisibleSelected}
          isBusy={isBusy}
          onSelectAllVisible={selection.toggleSelectAllVisible}
          onClear={selection.clearSelection}
          onResetSelected={handleResetSelected}
        />
      )}

      <TaskListState
        error={error}
        isLoading={isLoading}
        hasTasks={tasks.length > 0}
        hasResults={filters.filtered.length > 0}
        onRetry={() => void useTaskStore.getState().loadTasks()}
        onOpenCreate={actions.openCreate}
      />

      <IncompleteTaskList
        tasks={filters.incomplete}
        handlers={actions.handlers}
        onNavigateToTimer={navigateToTimer}
        selectedIds={selection.selectedIds}
        onToggleSelect={selection.toggleSelect}
        timerRunning={timerRunning}
        activeTaskId={activeTaskId}
      />

      <CompletedTaskList
        tasks={filters.completed}
        handlers={actions.handlers}
        onNavigateToTimer={navigateToTimer}
        selectedIds={selection.selectedIds}
        isAllCompletedSelected={selection.isAllCompletedSelected}
        onToggleSelect={selection.toggleSelect}
        onToggleSelectAllCompleted={selection.toggleSelectAllCompleted}
        timerRunning={timerRunning}
        activeTaskId={activeTaskId}
      />
    </div>
  )
}
