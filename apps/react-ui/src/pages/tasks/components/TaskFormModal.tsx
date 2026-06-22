import { useState, useEffect } from 'react'
import { X } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { toSeconds, DEFAULT_DURATIONS } from '@/lib/duration'
import type { Task } from '@/pages/tasks/useTasks'
import type { DurationUnit } from '@/lib/duration'

interface TaskFormModalProps {
  task?: Task
  onClose: () => void
}

const INPUT_CLS =
  'w-full px-4 py-2.5 text-sm rounded-xl border border-input bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50'

export function TaskFormModal({ task, onClose }: TaskFormModalProps) {
  const createTask = useTaskStore((s) => s.createTask)
  const updateTask = useTaskStore((s) => s.updateTask)
  const isEdit = !!task

  const [name, setName] = useState(task?.name ?? '')
  const [description, setDescription] = useState(task?.description ?? '')
  const [maxSessions, setMaxSessions] = useState<number>(
    task?.max_sessions ?? DEFAULT_DURATIONS.sessionsUntilLongBreak,
  )
  const [tagsInput, setTagsInput] = useState(task?.tags.join(', ') ?? '')
  const [useCustomTimer, setUseCustomTimer] = useState(false)
  const [unit, setUnit] = useState<DurationUnit>('minutes')
  const [workDuration, setWorkDuration] = useState(25)
  const [shortBreak, setShortBreak] = useState(5)
  const [longBreak, setLongBreak] = useState(15)
  const [sessionsUntilLongBreak, setSessionsUntilLongBreak] = useState<number>(
    DEFAULT_DURATIONS.sessionsUntilLongBreak,
  )
  const [validationError, setValidationError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  useEffect(() => {
    if (!task) return
    const tc = task.config.timer
    const hasCustom =
      tc.work_duration !== DEFAULT_DURATIONS.work ||
      tc.short_break_duration !== DEFAULT_DURATIONS.shortBreak ||
      tc.long_break_duration !== DEFAULT_DURATIONS.longBreak ||
      tc.sessions_until_long_break !== DEFAULT_DURATIONS.sessionsUntilLongBreak
    setUseCustomTimer(hasCustom)
    if (!hasCustom) return
    const hasSecondsPrecision =
      tc.work_duration % 60 !== 0 ||
      tc.short_break_duration % 60 !== 0 ||
      tc.long_break_duration % 60 !== 0
    const u: DurationUnit = hasSecondsPrecision ? 'seconds' : 'minutes'
    setUnit(u)
    const conv = (s: number) => (u === 'seconds' ? s : Math.round(s / 60))
    setWorkDuration(conv(tc.work_duration))
    setShortBreak(conv(tc.short_break_duration))
    setLongBreak(conv(tc.long_break_duration))
    setSessionsUntilLongBreak(tc.sessions_until_long_break)
  }, [task])

  const validate = (): string | null => {
    if (!name.trim()) return 'Task name is required'
    if (name.trim().length > 100) return 'Task name must be under 100 characters'
    if (maxSessions < 1 || maxSessions > 100) return 'Sessions must be between 1 and 100'
    return null
  }

  const handleSubmit = async () => {
    const err = validate()
    if (err) {
      setValidationError(err)
      return
    }
    setIsSubmitting(true)
    setValidationError(null)

    const tags = tagsInput
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)

    const custom = useCustomTimer
      ? {
          work_duration: toSeconds(workDuration, unit),
          short_break_duration: toSeconds(shortBreak, unit),
          long_break_duration: toSeconds(longBreak, unit),
          sessions_until_long_break: sessionsUntilLongBreak,
        }
      : {}

    try {
      if (isEdit && task) {
        const ok = await updateTask({
          id: task.id,
          name: name.trim(),
          description: description.trim() || undefined,
          max_sessions: maxSessions,
          tags,
          ...custom,
        })
        if (ok) toast.success('Task updated')
      } else {
        const ok = await createTask({
          name: name.trim(),
          description: description.trim() || undefined,
          max_sessions: maxSessions,
          tags,
          ...custom,
        })
        if (ok) toast.success('Task created')
      }
      onClose()
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" onClick={onClose} />

      {/* Modal */}
      <div className="bg-card border-border relative w-full max-w-lg overflow-hidden rounded-2xl border shadow-2xl">
        {/* Header */}
        <div className="border-border flex items-center justify-between border-b px-6 py-4">
          <h2 className="text-lg font-semibold">{isEdit ? 'Edit Task' : 'New Task'}</h2>
          <button
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg p-1.5 transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="max-h-[70vh] space-y-4 overflow-y-auto px-6 py-5">
          {validationError && (
            <div className="bg-destructive/10 border-destructive/30 text-destructive rounded-xl border px-4 py-3 text-sm">
              {validationError}
            </div>
          )}

          {/* Name */}
          <div>
            <label className="mb-1.5 block text-sm font-medium">
              Task Name <span className="text-destructive">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
              placeholder="Enter task name…"
              className={INPUT_CLS}
              disabled={isSubmitting}
              autoFocus
            />
          </div>

          {/* Description */}
          <div>
            <label className="mb-1.5 block text-sm font-medium">Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description…"
              rows={2}
              className={INPUT_CLS}
              disabled={isSubmitting}
            />
          </div>

          {/* Max sessions */}
          <div>
            <label className="mb-1.5 block text-sm font-medium">Max Sessions</label>
            <input
              type="number"
              value={maxSessions}
              min={1}
              max={100}
              onChange={(e) => setMaxSessions(Number(e.target.value))}
              className={INPUT_CLS}
              disabled={isSubmitting}
            />
            <p className="text-muted-foreground mt-1 text-xs">
              Number of pomodoro sessions for this task
            </p>
          </div>

          {/* Tags */}
          <div>
            <label className="mb-1.5 block text-sm font-medium">Tags</label>
            <input
              type="text"
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder="work, personal, urgent…"
              className={INPUT_CLS}
              disabled={isSubmitting}
            />
            <p className="text-muted-foreground mt-1 text-xs">Comma-separated</p>
          </div>

          {/* Custom timer toggle */}
          <label className="flex cursor-pointer items-center gap-2.5 select-none">
            <input
              type="checkbox"
              checked={useCustomTimer}
              onChange={(e) => setUseCustomTimer(e.target.checked)}
              disabled={isSubmitting}
              className="h-4 w-4 rounded accent-indigo-500"
            />
            <span className="text-sm font-medium">
              Custom timer settings for this task
            </span>
          </label>

          {/* Custom timer fields */}
          {useCustomTimer && (
            <div className="bg-muted/40 border-border space-y-4 rounded-xl border p-4">
              <label className="flex cursor-pointer items-center gap-2.5 select-none">
                <input
                  type="checkbox"
                  checked={unit === 'seconds'}
                  onChange={(e) => setUnit(e.target.checked ? 'seconds' : 'minutes')}
                  disabled={isSubmitting}
                  className="h-4 w-4 rounded accent-indigo-500"
                />
                <span className="text-sm">Use seconds instead of minutes</span>
              </label>

              <div>
                <label className="mb-1.5 block text-sm font-medium">
                  Work Duration ({unit})
                </label>
                <input
                  type="number"
                  value={workDuration}
                  min={unit === 'seconds' ? 5 : 1}
                  max={unit === 'seconds' ? 10800 : 180}
                  onChange={(e) => setWorkDuration(Number(e.target.value))}
                  className={INPUT_CLS}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="mb-1.5 block text-sm font-medium">
                  Short Break ({unit})
                </label>
                <input
                  type="number"
                  value={shortBreak}
                  min={unit === 'seconds' ? 5 : 1}
                  max={unit === 'seconds' ? 3600 : 60}
                  onChange={(e) => setShortBreak(Number(e.target.value))}
                  className={INPUT_CLS}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="mb-1.5 block text-sm font-medium">
                  Long Break ({unit})
                </label>
                <input
                  type="number"
                  value={longBreak}
                  min={unit === 'seconds' ? 5 : 1}
                  max={unit === 'seconds' ? 7200 : 120}
                  onChange={(e) => setLongBreak(Number(e.target.value))}
                  className={INPUT_CLS}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="mb-1.5 block text-sm font-medium">
                  Sessions Until Long Break
                </label>
                <input
                  type="number"
                  value={sessionsUntilLongBreak}
                  min={2}
                  max={10}
                  onChange={(e) => setSessionsUntilLongBreak(Number(e.target.value))}
                  className={INPUT_CLS}
                  disabled={isSubmitting}
                />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-border flex gap-3 border-t px-6 py-4">
          <button
            onClick={onClose}
            disabled={isSubmitting}
            className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex-1 rounded-xl border py-2.5 text-sm transition-colors disabled:opacity-40"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!name.trim() || isSubmitting}
            className="bg-primary text-primary-foreground flex-1 rounded-xl py-2.5 text-sm transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
          >
            {isSubmitting
              ? isEdit
                ? 'Updating…'
                : 'Creating…'
              : isEdit
                ? 'Update Task'
                : 'Create Task'}
          </button>
        </div>
      </div>
    </div>
  )
}
