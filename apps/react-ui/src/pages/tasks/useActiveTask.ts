import { useEffect, useState } from 'react'
import { invokeCmd, onEvent, events } from '@/lib/tauri'
import type { Task } from '@/pages/tasks/useTasks'

export const useActiveTask = (): Task | undefined => {
  const [activeTask, setActiveTask] = useState<Task | undefined>(undefined)

  useEffect(() => {
    let cancelled = false

    const fetch = async () => {
      try {
        const task = await invokeCmd('get_active_task')
        if (!cancelled) setActiveTask(task ?? undefined)
      } catch {
        if (!cancelled) setActiveTask(undefined)
      }
    }

    fetch()

    const unlisten = onEvent(events.taskActiveChanged, fetch)

    return () => {
      cancelled = true
      void unlisten.then((fn) => fn())
    }
  }, [])

  return activeTask
}
