import { useState, useMemo } from 'react'
import { Plus, RotateCcw, Pencil, Search, ListTodo } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore, useTasksEventBus, TaskStatus } from '@/pages/tasks/useTasks'
import { useTimerStore, isTimerRunning } from '@/pages/timer/useTimer'
import { StatBadge } from '@/pages/tasks/components/StatBadge'
import { TaskRow } from '@/pages/tasks/components/TaskRow'
import { TaskFormModal } from '@/pages/tasks/components/TaskFormModal'
import type { Task } from '@/pages/tasks/useTasks'
import type { Page } from '@/app/types'

type StatusFilter = 'all' | TaskStatus

interface TasksPageProps {
  onNavigate: (page: Page) => void
}

export function TasksPage({ onNavigate }: TasksPageProps) {
  useTasksEventBus()

  const tasks = useTaskStore((s) => s.tasks)
  const isLoading = useTaskStore((s) => s.isLoading)
  const error = useTaskStore((s) => s.error)
  const createTask = useTaskStore((s) => s.createTask)
  const completeTask = useTaskStore((s) => s.completeTask)
  const resetTask = useTaskStore((s) => s.resetTask)
  const deleteTask = useTaskStore((s) => s.deleteTask)
  const setActiveTask = useTaskStore((s) => s.setActiveTask)
  const timerRunning = useTimerStore((s) => (s.timer ? isTimerRunning(s.timer) : false))

  const [title, setTitle] = useState('')
  const [sessions, setSessions] = useState(4)
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all')
  const [editTask, setEditTask] = useState<Task | undefined>(undefined)
  const [showModal, setShowModal] = useState(false)

  const handleQuickAdd = async () => {
    const name = title.trim()
    if (!name) return
    const ok = await createTask({ name, max_sessions: sessions, tags: [] })
    if (ok) {
      setTitle('')
      setSessions(4)
      toast.success('Task added')
    }
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

  const handleSetActive = async (task: Task) => {
    const ok = await setActiveTask(task.id)
    if (ok) {
      toast.info('Focusing on "' + task.name + '"')
      onNavigate('timer')
    }
  }

  return (
    <div className="mx-auto w-full max-w-2xl">
      {showModal && <TaskFormModal task={editTask} onClose={() => setShowModal(false)} />}

      <div className="mb-5 flex items-start justify-between">
        <h1 className="text-2xl font-bold">Tasks</h1>

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

      <div className="mb-4 flex gap-2">
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleQuickAdd()}
          placeholder="Quick add task…"
          className="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring min-w-0 flex-1 rounded-xl border px-4 py-2.5 text-sm focus:ring-2 focus:outline-none"
        />
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
          disabled={!title.trim()}
          className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Plus size={16} />
          Add
        </button>
        <button
          onClick={openCreate}
          title="Create task with full details"
          className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex items-center gap-2 rounded-xl border px-4 py-2.5 text-sm transition-colors"
        >
          <Pencil size={15} />
          Detail
        </button>
      </div>

      <div className="mb-6 flex gap-2">
        <div className="relative flex-1">
          <Search
            size={14}
            className="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2"
          />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search tasks…"
            className="border-input bg-background text-foreground placeholder:text-muted-foreground focus:ring-ring w-full rounded-xl border py-2 pr-4 pl-9 text-sm focus:ring-2 focus:outline-none"
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
          className="border-input bg-background text-foreground focus:ring-ring rounded-xl border px-3 py-2 text-sm focus:ring-2 focus:outline-none"
        >
          <option value="all">All</option>
          <option value="Active">Active</option>
          <option value="Queued">Queued</option>
          <option value="Paused">Paused</option>
          <option value="Completed">Completed</option>
        </select>
      </div>

      {error && !isLoading && (
        <div className="flex flex-col items-center gap-2 py-8 text-center">
          <span className="text-destructive text-sm">{error.message}</span>
          <button
            onClick={() => void useTaskStore.getState().loadTasks()}
            className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
          >
            Retry
          </button>
        </div>
      )}

      {isLoading && (
        <p className="text-muted-foreground py-8 text-center text-sm">Loading…</p>
      )}

      {!isLoading && !error && tasks.length === 0 && (
        <div className="flex flex-col items-center justify-center px-4 py-16 text-center">
          <div className="bg-muted/60 mb-4 flex h-14 w-14 items-center justify-center rounded-2xl">
            <ListTodo size={26} className="text-muted-foreground" />
          </div>
          <h3 className="mb-1 text-base font-semibold">No tasks yet</h3>
          <p className="text-muted-foreground mb-5 max-w-xs text-sm">
            Create your first task to start focusing. You can edit or delete it any time.
          </p>
          <button
            onClick={openCreate}
            className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm transition-all hover:opacity-90 active:scale-95"
          >
            <Plus size={16} />
            Create task
          </button>
        </div>
      )}

      {!isLoading && !error && tasks.length > 0 && filtered.length === 0 && (
        <p className="text-muted-foreground py-12 text-center text-sm">
          No tasks match your search or filter.
        </p>
      )}

      {incomplete.length > 0 && (
        <ul className="mb-6 flex flex-col gap-2">
          {incomplete.map((task) => (
            <TaskRow
              key={task.id}
              task={task}
              onEdit={() => openEdit(task)}
              onComplete={async () => {
                const ok = await completeTask(task.id)
                if (ok) {
                  toast.success('Task completed!')
                  void useTimerStore.getState().fetchTimer()
                }
              }}
              onReset={async () => {
                const ok = await resetTask(task.id)
                if (ok) toast.info('Task reopened')
              }}
              onDelete={async () => {
                if (!window.confirm('Delete "' + task.name + '"? This cannot be undone.'))
                  return
                const ok = await deleteTask(task.id)
                if (ok) toast.info('Task deleted')
              }}
              onSetActive={() => handleSetActive(task)}
              timerRunning={timerRunning}
              onNavigateToTimer={() => onNavigate('timer')}
            />
          ))}
        </ul>
      )}

      {completed.length > 0 && (
        <details className="group">
          <summary className="text-muted-foreground mb-3 flex cursor-pointer list-none items-center gap-1 text-xs font-semibold tracking-wider uppercase select-none">
            <RotateCcw size={11} className="transition-transform group-open:rotate-180" />
            Completed ({completed.length})
          </summary>
          <ul className="flex flex-col gap-2">
            {completed.map((task) => (
              <TaskRow
                key={task.id}
                task={task}
                onEdit={() => openEdit(task)}
                onComplete={() => void completeTask(task.id)}
                onReset={async () => {
                  const ok = await resetTask(task.id)
                  if (ok) toast.info('Task reopened')
                }}
                onDelete={async () => {
                  if (
                    !window.confirm('Delete "' + task.name + '"? This cannot be undone.')
                  )
                    return
                  const ok = await deleteTask(task.id)
                  if (ok) toast.info('Task deleted')
                }}
                onSetActive={() => void setActiveTask(task.id)}
                timerRunning={timerRunning}
                onNavigateToTimer={() => onNavigate('timer')}
              />
            ))}
          </ul>
        </details>
      )}
    </div>
  )
}
