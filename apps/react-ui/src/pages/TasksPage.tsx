import { useState, useMemo } from 'react'
import { Plus, Trash2, CheckCircle2, Circle, Crosshair, RotateCcw, Pencil, Search } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore } from '@/store/taskStore'
import { useTimerStore } from '@/store/timerStore'
import { TaskFormModal } from '@/components/TaskFormModal'
import { TaskStatus } from '@/types'
import type { Page, Task } from '@/types'

type StatusFilter = 'all' | TaskStatus

function StatBadge({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className={`flex flex-col items-center px-4 py-2 rounded-xl border ${color}`}>
      <span className="text-2xl font-bold tabular-nums">{value}</span>
      <span className="text-[10px] font-semibold uppercase tracking-wider mt-0.5">{label}</span>
    </div>
  )
}

function TaskRow({
  task,
  onComplete,
  onReset,
  onDelete,
  onSetActive,
  onEdit,
}: {
  task: Task
  onComplete: () => void
  onReset: () => void
  onDelete: () => void
  onSetActive: () => void
  onEdit: () => void
}) {
  const isCompleted = task.status === TaskStatus.Completed
  const isActive = task.status === TaskStatus.Active
  const progressPct =
    task.max_sessions > 0
      ? Math.round((task.current_sessions / task.max_sessions) * 100)
      : 0

  return (
    <li
      className={[
        'flex flex-col gap-2 px-4 py-3.5 rounded-xl border transition-colors',
        isActive
          ? 'border-indigo-400/60 bg-indigo-50/60 dark:bg-indigo-950/20'
          : 'border-border bg-card',
        isCompleted ? 'opacity-55' : '',
      ].join(' ')}
    >
      <div className="flex items-center gap-3">
        {/* Complete toggle */}
        <button
          onClick={isCompleted ? onReset : onComplete}
          className="shrink-0 text-muted-foreground hover:text-foreground transition-colors"
          title={isCompleted ? 'Reopen' : 'Complete'}
        >
          {isCompleted ? (
            <CheckCircle2 size={20} className="text-indigo-500" />
          ) : (
            <Circle size={20} />
          )}
        </button>

        {/* Title + meta */}
        <div className="flex-1 min-w-0">
          <span
            className={[
              'block text-sm font-medium truncate',
              isCompleted ? 'line-through text-muted-foreground' : '',
            ].join(' ')}
          >
            {task.name}
          </span>
          {task.description && (
            <p className="text-xs text-muted-foreground truncate mt-0.5">{task.description}</p>
          )}
          {task.tags.length > 0 && (
            <div className="flex gap-1 mt-1 flex-wrap">
              {task.tags.map((tag) => (
                <span
                  key={tag}
                  className="text-[10px] px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Session count */}
        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
          {task.current_sessions}/{task.max_sessions}
        </span>

        {/* Focus */}
        {!isCompleted && (
          <button
            onClick={onSetActive}
            title="Focus on this task"
            className={[
              'shrink-0 p-1 rounded transition-colors',
              isActive
                ? 'text-indigo-500'
                : 'text-muted-foreground hover:text-foreground',
            ].join(' ')}
          >
            <Crosshair size={15} />
          </button>
        )}

        {/* Reset progress */}
        {task.current_sessions > 0 && !isCompleted && (
          <button
            onClick={onReset}
            title="Reset progress"
            className="shrink-0 p-1 text-muted-foreground hover:text-foreground transition-colors"
          >
            <RotateCcw size={15} />
          </button>
        )}

        {/* Edit */}
        <button
          onClick={onEdit}
          title="Edit task"
          className="shrink-0 p-1 text-muted-foreground hover:text-foreground transition-colors"
        >
          <Pencil size={15} />
        </button>

        {/* Delete */}
        {!task.default && (
          <button
            onClick={onDelete}
            title="Delete"
            className="shrink-0 p-1 text-muted-foreground hover:text-destructive transition-colors"
          >
            <Trash2 size={15} />
          </button>
        )}
      </div>

      {/* Progress bar */}
      {task.max_sessions > 1 && (
        <div className="w-full bg-muted rounded-full h-1.5 overflow-hidden ml-8">
          <div
            className="h-full bg-indigo-500 rounded-full transition-all duration-300"
            style={{ width: `${progressPct}%` }}
          />
        </div>
      )}
    </li>
  )
}

interface TasksPageProps {
  onNavigate: (page: Page) => void
}

export function TasksPage({ onNavigate }: TasksPageProps) {
  const { tasks, isLoading, createTask, completeTask, resetTask, deleteTask, setActiveTask } =
    useTaskStore()

  const [title, setTitle] = useState('')
  const [sessions, setSessions] = useState(4)
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [editTask, setEditTask] = useState<Task | undefined>(undefined)
  const [showModal, setShowModal] = useState(false)

  const handleQuickAdd = async () => {
    const name = title.trim()
    if (!name) return
    await createTask({ name, max_sessions: sessions, tags: [] })
    setTitle('')
    setSessions(4)
    toast.success('Task added')
  }

  const filtered = useMemo(() => {
    let result = tasks
    if (statusFilter !== 'all') {
      result = result.filter((t) => t.status === statusFilter)
    }
    if (search.trim()) {
      const q = search.toLowerCase()
      result = result.filter(
        (t) =>
          t.name.toLowerCase().includes(q) ||
          (t.description ?? '').toLowerCase().includes(q) ||
          t.tags.some((tag) => tag.toLowerCase().includes(q)),
      )
    }
    return result
  }, [tasks, statusFilter, search])

  const incomplete = filtered.filter((t) => t.status !== TaskStatus.Completed)
  const completed = filtered.filter((t) => t.status === TaskStatus.Completed)

  const total = tasks.length
  const activeCount = tasks.filter((t) => t.status === TaskStatus.Active).length
  const completedCount = tasks.filter((t) => t.status === TaskStatus.Completed).length

  const openCreate = () => {
    setEditTask(undefined)
    setShowModal(true)
  }

  const openEdit = (task: Task) => {
    setEditTask(task)
    setShowModal(true)
  }

  return (
    <div className="max-w-2xl mx-auto w-full">
      {/* Modal */}
      {showModal && (
        <TaskFormModal
          task={editTask}
          onClose={() => setShowModal(false)}
        />
      )}

      <div className="flex items-start justify-between mb-5">
        <h1 className="text-2xl font-bold">Tasks</h1>

        {/* Stats */}
        <div className="flex gap-2">
          <StatBadge
            label="Total"
            value={total}
            color="border-border bg-card text-foreground"
          />
          <StatBadge
            label="Active"
            value={activeCount}
            color="border-indigo-400/40 bg-indigo-50/50 dark:bg-indigo-950/20 text-indigo-600 dark:text-indigo-400"
          />
          <StatBadge
            label="Done"
            value={completedCount}
            color="border-emerald-400/40 bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400"
          />
        </div>
      </div>

      {/* Quick-add form */}
      <div className="flex gap-2 mb-4">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleQuickAdd()}
          placeholder="Quick add task…"
          className="flex-1 min-w-0 px-4 py-2.5 text-sm rounded-xl border border-input bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
        />
        <input
          type="number"
          min={1}
          max={20}
          value={sessions}
          onChange={(e) => setSessions(Number(e.target.value))}
          title="Sessions"
          className="w-16 px-3 py-2.5 text-sm rounded-xl border border-input bg-background text-foreground text-center focus:outline-none focus:ring-2 focus:ring-ring"
        />
        <button
          onClick={handleQuickAdd}
          disabled={!title.trim()}
          className="flex items-center gap-2 px-4 py-2.5 text-sm rounded-xl bg-primary text-primary-foreground hover:opacity-90 active:scale-95 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <Plus size={16} />
          Add
        </button>
        <button
          onClick={openCreate}
          title="Create task with full details"
          className="flex items-center gap-2 px-4 py-2.5 text-sm rounded-xl border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
        >
          <Pencil size={15} />
          Detail
        </button>
      </div>

      {/* Search + filter */}
      <div className="flex gap-2 mb-6">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search tasks…"
            className="w-full pl-9 pr-4 py-2 text-sm rounded-xl border border-input bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
          className="px-3 py-2 text-sm rounded-xl border border-input bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
        >
          <option value="all">All</option>
          <option value="Active">Active</option>
          <option value="Queued">Queued</option>
          <option value="Paused">Paused</option>
          <option value="Completed">Completed</option>
        </select>
      </div>

      {isLoading && (
        <p className="text-sm text-muted-foreground text-center py-8">Loading…</p>
      )}

      {!isLoading && tasks.length === 0 && (
        <p className="text-sm text-muted-foreground text-center py-12">
          No tasks yet — add one above.
        </p>
      )}

      {!isLoading && tasks.length > 0 && filtered.length === 0 && (
        <p className="text-sm text-muted-foreground text-center py-12">
          No tasks match your search or filter.
        </p>
      )}

      {/* Active / queued tasks */}
      {incomplete.length > 0 && (
        <ul className="flex flex-col gap-2 mb-6">
          {incomplete.map((task) => (
            <TaskRow
              key={task.id}
              task={task}
              onEdit={() => openEdit(task)}
              onComplete={async () => {
                await completeTask(task.id)
                toast.success('Task completed!')
              }}
              onReset={async () => {
                await resetTask(task.id)
                toast.info('Task reopened')
              }}
              onDelete={async () => {
                if (!window.confirm('Delete "' + task.name + '"? This cannot be undone.')) return
                await deleteTask(task.id)
                toast.info('Task deleted')
              }}
              onSetActive={async () => {
                try {
                  await setActiveTask(task.id)
                  await useTimerStore.getState().fetchTimer()
                  toast.info('Focusing on "' + task.name + '"')
                  onNavigate('timer')
                } catch {
                  toast.error('Failed to focus on task')
                }
              }}
            />
          ))}
        </ul>
      )}

      {/* Completed section */}
      {completed.length > 0 && (
        <details className="group">
          <summary className="text-xs font-semibold uppercase tracking-wider text-muted-foreground cursor-pointer mb-3 list-none flex items-center gap-1 select-none">
            <RotateCcw size={11} className="group-open:rotate-180 transition-transform" />
            Completed ({completed.length})
          </summary>
          <ul className="flex flex-col gap-2">
            {completed.map((task) => (
              <TaskRow
                key={task.id}
                task={task}
                onEdit={() => openEdit(task)}
                onComplete={async () => {
                  await completeTask(task.id)
                }}
                onReset={async () => {
                  await resetTask(task.id)
                  toast.info('Task reopened')
                }}
                onDelete={async () => {
                  if (!window.confirm('Delete "' + task.name + '"? This cannot be undone.')) return
                  await deleteTask(task.id)
                  toast.info('Task deleted')
                }}
                onSetActive={async () => {
                  await setActiveTask(task.id)
                }}
              />
            ))}
          </ul>
        </details>
      )}
    </div>
  )
}
