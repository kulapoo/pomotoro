import { useState } from 'react'
import { toast } from 'sonner'
import { useSettingsStore } from '@/pages/settings/useSettings'
import { TimerTab } from '@/pages/settings/components/TimerTab'
import { NotificationsTab } from '@/pages/settings/components/NotificationsTab'
import { AudioTab } from '@/pages/settings/components/AudioTab'
import { GeneralTab } from '@/pages/settings/components/GeneralTab'
import { StorageTab } from '@/pages/settings/components/StorageTab'
import type { Config } from '@/pages/settings/useSettings'

type Tab = 'timer' | 'notifications' | 'audio' | 'general' | 'storage'

const TABS: { id: Tab; label: string }[] = [
  { id: 'timer', label: 'Timer' },
  { id: 'notifications', label: 'Notifications' },
  { id: 'audio', label: 'Audio' },
  { id: 'general', label: 'General' },
  { id: 'storage', label: 'Storage' },
]

export function SettingsPage() {
  const config = useSettingsStore((s) => s.config)
  const error = useSettingsStore((s) => s.error)
  const saveConfig = useSettingsStore((s) => s.saveConfig)
  const resetToDefaults = useSettingsStore((s) => s.resetToDefaults)
  const [activeTab, setActiveTab] = useState<Tab>('timer')

  if (!config) {
    return (
      <div className="text-muted-foreground flex h-full flex-col items-center justify-center gap-3">
        {error ? (
          <>
            <span className="text-destructive text-sm">{error.message}</span>
            <button
              onClick={() => void useSettingsStore.getState().loadConfig()}
              className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
            >
              Retry
            </button>
          </>
        ) : (
          <span>Loading settings…</span>
        )}
      </div>
    )
  }

  const patch = async (updated: Config) => {
    const ok = await saveConfig(updated)
    if (ok) toast.success('Saved')
  }

  const handleReset = async () => {
    const ok = await resetToDefaults()
    if (ok) toast.success('Settings reset to defaults')
  }

  return (
    <div className="mx-auto w-full max-w-2xl">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-bold">Settings</h1>
        <button
          onClick={handleReset}
          className="border-destructive/50 text-destructive hover:bg-destructive hover:text-destructive-foreground rounded-lg border px-4 py-2 text-sm transition-colors"
        >
          Reset to Defaults
        </button>
      </div>

      <div className="mb-6 flex gap-1 overflow-x-auto pb-1">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={[
              'rounded-lg px-4 py-2 text-sm font-medium whitespace-nowrap transition-colors',
              activeTab === tab.id
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-accent',
            ].join(' ')}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === 'timer' && <TimerTab config={config} patch={patch} />}
      {activeTab === 'notifications' && (
        <NotificationsTab config={config} patch={patch} />
      )}
      {activeTab === 'audio' && <AudioTab config={config} patch={patch} />}
      {activeTab === 'general' && <GeneralTab config={config} patch={patch} />}
      {activeTab === 'storage' && <StorageTab />}
    </div>
  )
}
