import { ListTodo } from 'lucide-react'
import { useTaskStore } from '@/pages/tasks/useTasks'
import { useTimerSession } from '@/pages/timer/useTimerSession'
import type { Page } from '@/app/types'

interface EmptyTaskPromptProps {
  onNavigate: (page: Page) => void
}

export function EmptyTaskPrompt({ onNavigate }: EmptyTaskPromptProps) {
  const activeTask = useTaskStore((s) => s.activeTask)
  const { running, paused } = useTimerSession()

  if (activeTask || running || paused) return null

  return (
    <div className="mt-2 flex flex-col items-center gap-3">
      <div className="bg-muted/60 flex h-12 w-12 items-center justify-center rounded-2xl">
        <ListTodo size={22} className="text-muted-foreground" />
      </div>
      <p className="text-muted-foreground max-w-xs text-center text-sm">
        No task selected. Pick one to start focusing.
      </p>
      <button
        onClick={() => onNavigate('tasks')}
        className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2 text-sm transition-all hover:opacity-90 active:scale-95"
      >
        <ListTodo size={15} />
        Choose a task
      </button>
    </div>
  )
}
