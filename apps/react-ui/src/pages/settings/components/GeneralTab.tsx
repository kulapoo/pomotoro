import { Row } from '@/components/ui/Row'
import { Section } from '@/components/ui/Section'
import { Toggle } from '@/components/ui/Toggle'
import { SelectInput } from '@/components/ui/SelectInput'
import type { Config, Theme, TaskCyclingBehavior } from '@/pages/settings/useSettings'

interface GeneralTabProps {
  config: Config
  patch: (config: Config) => void
}

const CYCLING_OPTIONS: { value: TaskCyclingBehavior; label: string }[] = [
  { value: 'Manual', label: 'Manual' },
  { value: 'AutoAdvance', label: 'Auto Advance' },
]

const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: 'System', label: 'System' },
  { value: 'Light', label: 'Light' },
  { value: 'Dark', label: 'Dark' },
]

export function GeneralTab({ config, patch }: GeneralTabProps) {
  const { general, appearance } = config

  return (
    <div className="space-y-8">
      <Section title="Automation">
        <Row
          label="Auto-start breaks"
          hint="Automatically start break after work session"
        >
          <Toggle
            checked={general.auto_start_breaks}
            onChange={(v) =>
              patch({ ...config, general: { ...general, auto_start_breaks: v } })
            }
          />
        </Row>
        <Row
          label="Auto-start work after break"
          hint="Automatically start work after break"
        >
          <Toggle
            checked={general.auto_start_work_after_break}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, auto_start_work_after_break: v },
              })
            }
          />
        </Row>
        <Row label="Task cycling" hint="How tasks advance after session completion">
          <SelectInput
            value={general.task_cycling_behavior}
            options={CYCLING_OPTIONS}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, task_cycling_behavior: v },
              })
            }
          />
        </Row>
      </Section>

      <Section title="Appearance">
        <Row label="Theme">
          <SelectInput
            value={appearance.theme}
            options={THEME_OPTIONS}
            onChange={(v) =>
              patch({ ...config, appearance: { ...appearance, theme: v } })
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
              patch({ ...config, appearance: { ...appearance, always_on_top: v } })
            }
          />
        </Row>
        <Row label="Compact mode">
          <Toggle
            checked={appearance.compact_mode}
            onChange={(v) =>
              patch({ ...config, appearance: { ...appearance, compact_mode: v } })
            }
          />
        </Row>
        <Row label="Animate progress">
          <Toggle
            checked={appearance.animate_progress}
            onChange={(v) =>
              patch({ ...config, appearance: { ...appearance, animate_progress: v } })
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
