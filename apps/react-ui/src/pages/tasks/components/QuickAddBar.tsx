import { useState } from 'react'
import { Plus, Pencil } from 'lucide-react'
import { toast } from 'sonner'
import { useTaskStore, MAX_TASKS } from '@/pages/tasks/useTasks'

interface QuickAddBarProps {
  defaultSessions: number
  onOpenCreate: () => void
}

export function QuickAddBar({ defaultSessions, onOpenCreate }: QuickAddBarProps) {
  const createTask = useTaskStore((s) => s.createTask)
  const atLimit = useTaskStore((s) => s.tasks.length >= MAX_TASKS)
  const [title, setTitle] = useState('')
  const [sessions, setSessions] = useState(defaultSessions)

  const handleQuickAdd = async () => {
    const name = title.trim()
    if (!name) return
    const ok = await createTask({ name, max_sessions: sessions, tags: [] })
    if (ok) {
      setTitle('')
      setSessions(defaultSessions)
      toast.success('Task added')
    }
  }

  return (
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
  )
}
