import { useState } from 'react'
import { Bug } from 'lucide-react'
import { useTimerStore } from '@/features/timer/model/useTimerStore'
import { useTaskStore } from '@/features/tasks/model/useTaskStore'
import { useSettingsStore } from '@/features/settings/model/useSettingsStore'

/**
 * Dev-only debug overlay showing live store snapshots.
 * Renders only when `import.meta.env.DEV` is true (stripped from prod builds).
 */
export function DevPanel() {
  const [open, setOpen] = useState(false)
  const timer = useTimerStore((s) => s.timer)
  const timerError = useTimerStore((s) => s.error)
  const tasks = useTaskStore((s) => s.tasks)
  const taskError = useTaskStore((s) => s.error)
  const config = useSettingsStore((s) => s.config)
  const settingsError = useSettingsStore((s) => s.error)

  return (
    <div className="fixed right-2 bottom-2 z-100 font-mono text-xs print:hidden">
      <button
        onClick={() => setOpen((v) => !v)}
        className="bg-card border-border text-muted-foreground hover:text-foreground flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 shadow-md transition-colors"
        title="Toggle dev panel"
      >
        <Bug size={13} />
        Dev
      </button>
      {open && (
        <div className="bg-card/95 border-border text-muted-foreground mt-1.5 max-h-[60vh] w-80 space-y-2.5 overflow-y-auto rounded-xl border p-3 shadow-xl backdrop-blur">
          <Snapshot label="timer" data={timer} error={timerError} />
          <Snapshot label="tasks" data={`${tasks.length} item(s)`} error={taskError} />
          {tasks.length > 0 && (
            <pre className="text-muted-foreground/80 text-[10px] leading-tight break-all whitespace-pre-wrap">
              {JSON.stringify(
                tasks.map((t) => ({
                  id: t.id.slice(0, 8),
                  name: t.name,
                  status: t.status,
                  sess: `${t.current_sessions}/${t.max_sessions}`,
                })),
                null,
                2,
              )}
            </pre>
          )}
          <Snapshot label="config" data={config} error={settingsError} />
        </div>
      )}
    </div>
  )
}

interface SnapshotProps {
  label: string
  data: unknown
  error: unknown
}

function Snapshot({ label, data, error }: SnapshotProps) {
  return (
    <div>
      <div className="flex items-center justify-between">
        <span className="text-foreground font-semibold">{label}</span>
        {error ? (
          <span className="text-destructive">error</span>
        ) : data ? (
          <span className="text-emerald-500">ok</span>
        ) : (
          <span className="text-muted-foreground/60">empty</span>
        )}
      </div>
      {data != null && typeof data !== 'string' && (
        <pre className="text-muted-foreground/80 mt-0.5 text-[10px] leading-tight break-all whitespace-pre-wrap">
          {JSON.stringify(data, null, 2)}
        </pre>
      )}
      {typeof data === 'string' && <p className="mt-0.5 text-[10px]">{data}</p>}
      {error != null && (
        <pre className="text-destructive/80 mt-0.5 text-[10px] leading-tight break-all whitespace-pre-wrap">
          {error instanceof Error ? error.message : JSON.stringify(error)}
        </pre>
      )}
    </div>
  )
}
