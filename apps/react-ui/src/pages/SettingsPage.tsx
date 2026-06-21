import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { useSettingsStore } from '@/store/settingsStore'
import type { Config, NotificationPosition, TaskCyclingBehavior } from '@/types'

type Tab = 'timer' | 'notifications' | 'audio' | 'general' | 'storage'

// ---- Shared UI primitives ----

function Row({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-0.5">
      <div className="flex-1 min-w-0">
        <span className="text-sm font-medium">{label}</span>
        {hint && <p className="text-xs text-muted-foreground mt-0.5">{hint}</p>}
      </div>
      {children}
    </div>
  )
}

function NumberInput({
  value,
  min,
  max,
  onChange,
  className,
}: {
  value: number
  min: number
  max: number
  onChange: (v: number) => void
  className?: string
}) {
  return (
    <input
      type="number"
      min={min}
      max={max}
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
      className={`w-20 px-3 py-1.5 text-sm rounded-lg border border-input bg-background text-foreground text-center focus:outline-none focus:ring-2 focus:ring-ring ${className ?? ''}`}
    />
  )
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={[
        'relative shrink-0 w-10 h-6 rounded-full transition-colors',
        checked ? 'bg-indigo-500' : 'bg-muted',
      ].join(' ')}
    >
      <span
        className={[
          'absolute top-1 w-4 h-4 rounded-full bg-white shadow transition-transform',
          checked ? 'translate-x-5' : 'translate-x-1',
        ].join(' ')}
      />
    </button>
  )
}

function SelectInput({
  value,
  options,
  onChange,
}: {
  value: string
  options: { value: string; label: string }[]
  onChange: (v: string) => void
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="px-3 py-1.5 text-sm rounded-lg border border-input bg-background text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-3">
        {title}
      </h2>
      <div className="rounded-xl border border-border bg-card p-5 space-y-4">{children}</div>
    </section>
  )
}

// ---- Tab panels ----

function TimerTab({ config, patch }: { config: Config; patch: (c: Config) => void }) {
  const [useSeconds, setUseSeconds] = useState(false)
  const { timer } = config

  const workVal = useSeconds ? timer.work_duration : Math.round(timer.work_duration / 60)
  const shortVal = useSeconds
    ? timer.short_break_duration
    : Math.round(timer.short_break_duration / 60)
  const longVal = useSeconds
    ? timer.long_break_duration
    : Math.round(timer.long_break_duration / 60)

  const toSecs = (v: number) => (useSeconds ? v : v * 60)

  return (
    <div className="space-y-8">
      <Section title="Timer">
        <Row label="Use seconds instead of minutes">
          <Toggle checked={useSeconds} onChange={setUseSeconds} />
        </Row>
        <Row
          label={`Focus duration (${useSeconds ? 'seconds' : 'minutes'})`}
          hint={useSeconds ? '5–10800 seconds' : '1–180 minutes'}
        >
          <NumberInput
            value={workVal}
            min={useSeconds ? 5 : 1}
            max={useSeconds ? 10800 : 180}
            onChange={(v) =>
              patch({ ...config, timer: { ...timer, work_duration: toSecs(v) } })
            }
          />
        </Row>
        <Row
          label={`Short break (${useSeconds ? 'seconds' : 'minutes'})`}
          hint={useSeconds ? '5–3600 seconds' : '1–60 minutes'}
        >
          <NumberInput
            value={shortVal}
            min={useSeconds ? 5 : 1}
            max={useSeconds ? 3600 : 60}
            onChange={(v) =>
              patch({ ...config, timer: { ...timer, short_break_duration: toSecs(v) } })
            }
          />
        </Row>
        <Row
          label={`Long break (${useSeconds ? 'seconds' : 'minutes'})`}
          hint={useSeconds ? '5–7200 seconds' : '1–120 minutes'}
        >
          <NumberInput
            value={longVal}
            min={useSeconds ? 5 : 1}
            max={useSeconds ? 7200 : 120}
            onChange={(v) =>
              patch({ ...config, timer: { ...timer, long_break_duration: toSecs(v) } })
            }
          />
        </Row>
        <Row label="Sessions until long break" hint="2–10 sessions">
          <NumberInput
            value={timer.sessions_until_long_break}
            min={2}
            max={10}
            onChange={(v) =>
              patch({
                ...config,
                timer: { ...timer, sessions_until_long_break: v },
              })
            }
          />
        </Row>
      </Section>
    </div>
  )
}

function NotificationsTab({ config, patch }: { config: Config; patch: (c: Config) => void }) {
  const { notification } = config
  const positionOptions: { value: NotificationPosition; label: string }[] = [
    { value: 'TopRight', label: 'Top Right' },
    { value: 'TopLeft', label: 'Top Left' },
    { value: 'BottomRight', label: 'Bottom Right' },
    { value: 'BottomLeft', label: 'Bottom Left' },
    { value: 'Center', label: 'Center' },
  ]

  return (
    <div className="space-y-8">
      <Section title="Notifications">
        <Row label="Desktop notifications" hint="Show system notifications for timer events">
          <Toggle
            checked={notification.enable_desktop_notifications}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, enable_desktop_notifications: v },
              })
            }
          />
        </Row>
        <Row label="Sound notifications" hint="Play sounds for timer events">
          <Toggle
            checked={notification.enable_sound_notifications}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, enable_sound_notifications: v },
              })
            }
          />
        </Row>
        <Row label="Phase transition alerts" hint="Notify when switching between work and break">
          <Toggle
            checked={notification.show_phase_transition_notifications}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, show_phase_transition_notifications: v },
              })
            }
          />
        </Row>
        <Row label="Task completion alerts" hint="Notify when tasks are completed">
          <Toggle
            checked={notification.show_task_completion_notifications}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, show_task_completion_notifications: v },
              })
            }
          />
        </Row>
        <Row label="Auto-dismiss delay (seconds)" hint="Time before notifications close">
          <NumberInput
            value={notification.auto_dismiss_delay_seconds}
            min={1}
            max={300}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, auto_dismiss_delay_seconds: v },
              })
            }
          />
        </Row>
        <Row label="Position" hint="Where notifications appear on screen">
          <SelectInput
            value={notification.notification_position}
            options={positionOptions}
            onChange={(v) =>
              patch({
                ...config,
                notification: {
                  ...notification,
                  notification_position: v as NotificationPosition,
                },
              })
            }
          />
        </Row>
      </Section>
    </div>
  )
}

function AudioTab({ config, patch }: { config: Config; patch: (c: Config) => void }) {
  const { audio } = config

  const soundOptions = [
    { value: '', label: 'None' },
    { value: 'bell.wav', label: 'Bell' },
    { value: 'chime.wav', label: 'Chime' },
    { value: 'gong.wav', label: 'Gong' },
  ]
  const bgSoundOptions = [
    { value: '', label: 'None' },
    { value: 'rain.wav', label: 'Rain' },
    { value: 'forest.wav', label: 'Forest' },
    { value: 'ocean.wav', label: 'Ocean' },
    { value: 'whitenoise.wav', label: 'White Noise' },
  ]

  const testAudio = (type: string) => {
    invoke('test_audio_preview', { sound_type: type }).catch(() => {})
  }

  return (
    <div className="space-y-8">
      <Section title="Audio">
        <Row label="Enable audio" hint="Master audio toggle">
          <Toggle
            checked={!audio.muted}
            onChange={(v) => patch({ ...config, audio: { ...audio, muted: !v } })}
          />
        </Row>

        <Row label="Volume" hint="Master volume for all sounds">
          <div className="flex items-center gap-3">
            <input
              type="range"
              min={0}
              max={100}
              value={Math.round(audio.volume * 100)}
              onChange={(e) =>
                patch({
                  ...config,
                  audio: { ...audio, volume: Number(e.target.value) / 100 },
                })
              }
              className="w-32 accent-indigo-500"
            />
            <span className="text-sm tabular-nums w-10 text-right text-muted-foreground">
              {Math.round(audio.volume * 100)}%
            </span>
          </div>
        </Row>

        <Row label="Background audio" hint="Play ambient sounds during work sessions">
          <Toggle
            checked={audio.enable_background_audio}
            onChange={(v) =>
              patch({ ...config, audio: { ...audio, enable_background_audio: v } })
            }
          />
        </Row>
      </Section>

      <Section title="Sound Selection">
        <Row label="Work notification sound" hint="Sound when work session ends">
          <div className="flex items-center gap-2">
            <SelectInput
              value={audio.work_notification_sound ?? ''}
              options={soundOptions}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, work_notification_sound: v || null },
                })
              }
            />
            <button
              onClick={() => testAudio('work')}
              className="px-3 py-1.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            >
              Test
            </button>
          </div>
        </Row>

        <Row label="Break notification sound" hint="Sound when break session ends">
          <div className="flex items-center gap-2">
            <SelectInput
              value={audio.break_notification_sound ?? ''}
              options={soundOptions}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, break_notification_sound: v || null },
                })
              }
            />
            <button
              onClick={() => testAudio('break')}
              className="px-3 py-1.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            >
              Test
            </button>
          </div>
        </Row>

        <Row label="Background sound" hint="Ambient sound during work sessions">
          <div className="flex items-center gap-2">
            <SelectInput
              value={audio.background_sound ?? ''}
              options={bgSoundOptions}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, background_sound: v || null },
                })
              }
            />
            <button
              onClick={() => testAudio('background')}
              className="px-3 py-1.5 text-xs rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
            >
              Test
            </button>
          </div>
        </Row>
      </Section>
    </div>
  )
}

function GeneralTab({ config, patch }: { config: Config; patch: (c: Config) => void }) {
  const { general, appearance } = config

  const cyclingOptions: { value: TaskCyclingBehavior; label: string }[] = [
    { value: 'Manual', label: 'Manual' },
    { value: 'AutoAdvance', label: 'Auto Advance' },
  ]

  return (
    <div className="space-y-8">
      <Section title="Automation">
        <Row label="Auto-start breaks" hint="Automatically start break after work session">
          <Toggle
            checked={general.auto_start_breaks}
            onChange={(v) =>
              patch({ ...config, general: { ...general, auto_start_breaks: v } })
            }
          />
        </Row>
        <Row label="Auto-start work after break" hint="Automatically start work after break">
          <Toggle
            checked={general.auto_start_work_after_break}
            onChange={(v) =>
              patch({ ...config, general: { ...general, auto_start_work_after_break: v } })
            }
          />
        </Row>
        <Row label="Task cycling" hint="How tasks advance after session completion">
          <SelectInput
            value={general.task_cycling_behavior}
            options={cyclingOptions}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, task_cycling_behavior: v as TaskCyclingBehavior },
              })
            }
          />
        </Row>
      </Section>

      <Section title="Appearance">
        <Row label="Theme">
          <SelectInput
            value={appearance.theme}
            options={[
              { value: 'System', label: 'System' },
              { value: 'Light', label: 'Light' },
              { value: 'Dark', label: 'Dark' },
            ]}
            onChange={(v) =>
              patch({
                ...config,
                appearance: { ...appearance, theme: v as 'Light' | 'Dark' | 'System' },
              })
            }
          />
        </Row>
        <Row label="Show seconds in timer">
          <Toggle
            checked={appearance.show_seconds_in_display}
            onChange={(v) =>
              patch({
                ...config,
                appearance: { ...appearance, show_seconds_in_display: v },
              })
            }
          />
        </Row>
        <Row label="Always on top">
          <Toggle
            checked={appearance.always_on_top}
            onChange={(v) =>
              patch({
                ...config,
                appearance: { ...appearance, always_on_top: v },
              })
            }
          />
        </Row>
        <Row label="Compact mode">
          <Toggle
            checked={appearance.compact_mode}
            onChange={(v) =>
              patch({
                ...config,
                appearance: { ...appearance, compact_mode: v },
              })
            }
          />
        </Row>
        <Row label="Animate progress">
          <Toggle
            checked={appearance.animate_progress}
            onChange={(v) =>
              patch({
                ...config,
                appearance: { ...appearance, animate_progress: v },
              })
            }
          />
        </Row>
      </Section>

      <Section title="Window">
        <Row label="Minimize to system tray" hint="Hide to tray when minimized">
          <Toggle
            checked={general.minimize_to_tray}
            onChange={(v) =>
              patch({ ...config, general: { ...general, minimize_to_tray: v } })
            }
          />
        </Row>
        <Row label="Start minimized" hint="Launch application minimized">
          <Toggle
            checked={general.start_minimized}
            onChange={(v) =>
              patch({ ...config, general: { ...general, start_minimized: v } })
            }
          />
        </Row>
      </Section>
    </div>
  )
}

function StorageTab() {
  const openDataDir = () => {
    invoke('open_data_directory').catch(() => toast.error('Could not open directory'))
  }

  const clearAllData = async () => {
    if (!window.confirm('Delete all tasks, settings, and history? This cannot be undone.')) return
    try {
      await invoke('clear_all_data')
      toast.success('All data cleared')
    } catch {
      toast.error('Failed to clear data')
    }
  }

  return (
    <div className="space-y-8">
      <Section title="Data">
        <div className="flex items-center justify-between gap-4 py-0.5">
          <div>
            <p className="text-sm font-medium">Open data directory</p>
            <p className="text-xs text-muted-foreground mt-0.5">
              Browse the folder where all app data is stored
            </p>
          </div>
          <button
            onClick={openDataDir}
            className="px-4 py-2 text-sm rounded-lg border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          >
            Open
          </button>
        </div>
      </Section>

      <section>
        <h2 className="text-sm font-semibold uppercase tracking-wider text-destructive mb-3">
          Danger Zone
        </h2>
        <div className="rounded-xl border border-destructive/25 bg-card p-5 space-y-4">
          <div className="flex items-center justify-between gap-4">
            <div>
              <p className="text-sm font-medium">Clear all data</p>
              <p className="text-xs text-muted-foreground mt-0.5">
                Delete all tasks, settings, and history
              </p>
            </div>
            <button
              onClick={clearAllData}
              className="px-4 py-2 text-sm rounded-lg border border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground transition-colors"
            >
              Clear
            </button>
          </div>
        </div>
      </section>
    </div>
  )
}

// ---- Main page ----

const TABS: { id: Tab; label: string }[] = [
  { id: 'timer', label: 'Timer' },
  { id: 'notifications', label: 'Notifications' },
  { id: 'audio', label: 'Audio' },
  { id: 'general', label: 'General' },
  { id: 'storage', label: 'Storage' },
]

export function SettingsPage() {
  const { config, saveConfig, resetToDefaults } = useSettingsStore()
  const [activeTab, setActiveTab] = useState<Tab>('timer')

  if (!config) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading settings…
      </div>
    )
  }

  const patch = async (updated: Config) => {
    await saveConfig(updated)
    toast.success('Saved')
  }

  return (
    <div className="max-w-2xl mx-auto w-full">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Settings</h1>
        <button
          onClick={async () => {
            await resetToDefaults()
            toast.success('Settings reset to defaults')
          }}
          className="px-4 py-2 text-sm rounded-lg border border-destructive/50 text-destructive hover:bg-destructive hover:text-destructive-foreground transition-colors"
        >
          Reset to Defaults
        </button>
      </div>

      {/* Tab bar */}
      <div className="flex gap-1 mb-6 overflow-x-auto pb-1">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={[
              'px-4 py-2 text-sm font-medium rounded-lg whitespace-nowrap transition-colors',
              activeTab === tab.id
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-accent',
            ].join(' ')}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === 'timer' && <TimerTab config={config} patch={patch} />}
      {activeTab === 'notifications' && <NotificationsTab config={config} patch={patch} />}
      {activeTab === 'audio' && <AudioTab config={config} patch={patch} />}
      {activeTab === 'general' && <GeneralTab config={config} patch={patch} />}
      {activeTab === 'storage' && <StorageTab />}
    </div>
  )
}
