import { useState, useEffect } from 'react'
import { X } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore } from '@/store/taskStore'
import type { Task } from '@/types'

interface TaskFormModalProps {
  task?: Task
  onClose: () => void
}

export function TaskFormModal({ task, onClose }: TaskFormModalProps) {
  const { createTask, updateTask } = useTaskStore()
  const isEdit = !!task

  const [name, setName] = useState(task?.name ?? '')
  const [description, setDescription] = useState(task?.description ?? '')
  const [maxSessions, setMaxSessions] = useState(task?.max_sessions ?? 4)
  const [tagsInput, setTagsInput] = useState(task?.tags.join(', ') ?? '')
  const [useCustomTimer, setUseCustomTimer] = useState(false)
  const [useSeconds, setUseSeconds] = useState(false)
  const [workDuration, setWorkDuration] = useState(25)
  const [shortBreak, setShortBreak] = useState(5)
  const [longBreak, setLongBreak] = useState(15)
  const [sessionsUntilLongBreak, setSessionsUntilLongBreak] = useState(4)
  const [validationError, setValidationError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  useEffect(() => {
    if (task) {
      const tc = task.config.timer
      const hasCustom =
        tc.work_duration !== 1500 ||
        tc.short_break_duration !== 300 ||
        tc.long_break_duration !== 900 ||
        tc.sessions_until_long_break !== 4
      setUseCustomTimer(hasCustom)
      if (hasCustom) {
        const hasSecondsPrecision =
          tc.work_duration % 60 !== 0 ||
          tc.short_break_duration % 60 !== 0 ||
          tc.long_break_duration % 60 !== 0
        setUseSeconds(hasSecondsPrecision)
        if (hasSecondsPrecision) {
          setWorkDuration(tc.work_duration)
          setShortBreak(tc.short_break_duration)
          setLongBreak(tc.long_break_duration)
        } else {
          setWorkDuration(Math.round(tc.work_duration / 60))
          setShortBreak(Math.round(tc.short_break_duration / 60))
          setLongBreak(Math.round(tc.long_break_duration / 60))
        }
        setSessionsUntilLongBreak(tc.sessions_until_long_break)
      }
    }
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

    const toSecs = (v: number) => (useSeconds ? v : v * 60)

    try {
      if (isEdit && task) {
        await updateTask({
          id: task.id,
          name: name.trim(),
          description: description.trim() || undefined,
          max_sessions: maxSessions,
          tags,
          ...(useCustomTimer
            ? {
                work_duration: toSecs(workDuration),
                short_break_duration: toSecs(shortBreak),
                long_break_duration: toSecs(longBreak),
                sessions_until_long_break: sessionsUntilLongBreak,
              }
            : {}),
        })
        toast.success('Task updated')
      } else {
        await createTask({
          name: name.trim(),
          description: description.trim() || undefined,
          max_sessions: maxSessions,
          tags,
          ...(useCustomTimer
            ? {
                work_duration: toSecs(workDuration),
                short_break_duration: toSecs(shortBreak),
                long_break_duration: toSecs(longBreak),
                sessions_until_long_break: sessionsUntilLongBreak,
              }
            : {}),
        })
        toast.success('Task created')
      }
      onClose()
    } catch {
      toast.error('Failed to save task')
    } finally {
      setIsSubmitting(false)
    }
  }

  const inputCls =
    'w-full px-4 py-2.5 text-sm rounded-xl border border-input bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50'

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-lg bg-card border border-border rounded-2xl shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 className="text-lg font-semibold">
            {isEdit ? 'Edit Task' : 'New Task'}
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-5 space-y-4 max-h-[70vh] overflow-y-auto">
          {validationError && (
            <div className="px-4 py-3 rounded-xl bg-destructive/10 border border-destructive/30 text-destructive text-sm">
              {validationError}
            </div>
          )}

          {/* Name */}
          <div>
            <label className="block text-sm font-medium mb-1.5">
              Task Name <span className="text-destructive">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
              placeholder="Enter task name…"
              className={inputCls}
              disabled={isSubmitting}
              autoFocus
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Description</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description…"
              rows={2}
              className={inputCls}
              disabled={isSubmitting}
            />
          </div>

          {/* Max sessions */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Max Sessions</label>
            <input
              type="number"
              value={maxSessions}
              min={1}
              max={100}
              onChange={(e) => setMaxSessions(Number(e.target.value))}
              className={inputCls}
              disabled={isSubmitting}
            />
            <p className="text-xs text-muted-foreground mt-1">
              Number of pomodoro sessions for this task
            </p>
          </div>

          {/* Tags */}
          <div>
            <label className="block text-sm font-medium mb-1.5">Tags</label>
            <input
              type="text"
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder="work, personal, urgent…"
              className={inputCls}
              disabled={isSubmitting}
            />
            <p className="text-xs text-muted-foreground mt-1">Comma-separated</p>
          </div>

          {/* Custom timer toggle */}
          <label className="flex items-center gap-2.5 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={useCustomTimer}
              onChange={(e) => setUseCustomTimer(e.target.checked)}
              disabled={isSubmitting}
              className="w-4 h-4 rounded accent-indigo-500"
            />
            <span className="text-sm font-medium">Custom timer settings for this task</span>
          </label>

          {/* Custom timer fields */}
          {useCustomTimer && (
            <div className="rounded-xl bg-muted/40 border border-border p-4 space-y-4">
              <label className="flex items-center gap-2.5 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={useSeconds}
                  onChange={(e) => setUseSeconds(e.target.checked)}
                  disabled={isSubmitting}
                  className="w-4 h-4 rounded accent-indigo-500"
                />
                <span className="text-sm">Use seconds instead of minutes</span>
              </label>

              <div>
                <label className="block text-sm font-medium mb-1.5">
                  Work Duration ({useSeconds ? 'seconds' : 'minutes'})
                </label>
                <input
                  type="number"
                  value={workDuration}
                  min={useSeconds ? 5 : 1}
                  max={useSeconds ? 10800 : 180}
                  onChange={(e) => setWorkDuration(Number(e.target.value))}
                  className={inputCls}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1.5">
                  Short Break ({useSeconds ? 'seconds' : 'minutes'})
                </label>
                <input
                  type="number"
                  value={shortBreak}
                  min={useSeconds ? 5 : 1}
                  max={useSeconds ? 3600 : 60}
                  onChange={(e) => setShortBreak(Number(e.target.value))}
                  className={inputCls}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1.5">
                  Long Break ({useSeconds ? 'seconds' : 'minutes'})
                </label>
                <input
                  type="number"
                  value={longBreak}
                  min={useSeconds ? 5 : 1}
                  max={useSeconds ? 7200 : 120}
                  onChange={(e) => setLongBreak(Number(e.target.value))}
                  className={inputCls}
                  disabled={isSubmitting}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1.5">
                  Sessions Until Long Break
                </label>
                <input
                  type="number"
                  value={sessionsUntilLongBreak}
                  min={2}
                  max={10}
                  onChange={(e) => setSessionsUntilLongBreak(Number(e.target.value))}
                  className={inputCls}
                  disabled={isSubmitting}
                />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex gap-3 px-6 py-4 border-t border-border">
          <button
            onClick={onClose}
            disabled={isSubmitting}
            className="flex-1 py-2.5 text-sm rounded-xl border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-40"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!name.trim() || isSubmitting}
            className="flex-1 py-2.5 text-sm rounded-xl bg-primary text-primary-foreground hover:opacity-90 active:scale-95 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
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
