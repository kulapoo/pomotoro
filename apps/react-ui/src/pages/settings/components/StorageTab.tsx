import { Section } from '@/components/ui/Section'
import { toast } from 'sonner'
import { useSettingsStore } from '@/pages/settings/useSettings'
import { useConfirm } from '@/components/ConfirmProvider'

export function StorageTab() {
  const openDataDirectory = useSettingsStore((s) => s.openDataDirectory)
  const clearAllData = useSettingsStore((s) => s.clearAllData)
  const { confirm } = useConfirm()

  const openDataDir = async () => {
    await openDataDirectory()
  }

  const handleClearAll = async () => {
    const confirmed = await confirm({
      title: 'Clear All Data',
      message: 'Delete all tasks, settings, and history? This cannot be undone.',
      variant: 'danger',
      confirmLabel: 'Clear',
    })
    if (!confirmed) return
    const ok = await clearAllData()
    if (ok) toast.success('All data cleared')
  }

  return (
    <div className="space-y-8">
      <Section title="Data">
        <div className="flex items-center justify-between gap-4 py-0.5">
          <div>
            <p className="text-sm font-medium">Open data directory</p>
            <p className="text-muted-foreground mt-0.5 text-xs">
              Browse the folder where all app data is stored
            </p>
          </div>
          <button
            onClick={openDataDir}
            className="border-border text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg border px-4 py-2 text-sm transition-colors"
          >
            Open
          </button>
        </div>
      </Section>

      <section>
        <h2 className="text-destructive mb-3 text-sm font-semibold tracking-wider uppercase">
          Danger Zone
        </h2>
        <div className="border-destructive/25 bg-card space-y-4 rounded-xl border p-5">
          <div className="flex items-center justify-between gap-4">
            <div>
              <p className="text-sm font-medium">Clear all data</p>
              <p className="text-muted-foreground mt-0.5 text-xs">
                Delete all tasks, settings, and history
              </p>
            </div>
            <button
              onClick={handleClearAll}
              className="border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground rounded-lg border px-4 py-2 text-sm transition-colors"
            >
              Clear
            </button>
          </div>
        </div>
      </section>
    </div>
  )
}
