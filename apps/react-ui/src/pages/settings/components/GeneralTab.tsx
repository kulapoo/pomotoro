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
        <Row label="Show countdown in tray" hint="Always display remaining time beside the tray icon">
          <Toggle
            checked={general.show_countdown_in_tray}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, show_countdown_in_tray: v },
              })
            }
          />
        </Row>
      </Section>

      <Section title="Screen Blocking">
        <Row
          label="Block screen after work"
          hint="Show a blocking overlay when a work session expires"
        >
          <Toggle
            checked={general.block_screen_after_work}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, block_screen_after_work: v },
              })
            }
          />
        </Row>
        {general.block_screen_after_work && (
          <Row label="Work message" hint="Shown on the overlay after work">
            <input
              type="text"
              value={general.block_screen_after_work_message}
              onChange={(e) =>
                patch({
                  ...config,
                  general: {
                    ...general,
                    block_screen_after_work_message: e.target.value,
                  },
                })
              }
              className="border-input bg-background text-foreground focus:ring-ring w-64 rounded-lg border px-3 py-1.5 text-sm focus:ring-2 focus:outline-none"
            />
          </Row>
        )}
        <Row
          label="Block screen after break"
          hint="Show a blocking overlay when a break expires"
        >
          <Toggle
            checked={general.block_screen_after_break}
            onChange={(v) =>
              patch({
                ...config,
                general: { ...general, block_screen_after_break: v },
              })
            }
          />
        </Row>
        {general.block_screen_after_break && (
          <Row label="Break message" hint="Shown on the overlay after break">
            <input
              type="text"
              value={general.block_screen_after_break_message}
              onChange={(e) =>
                patch({
                  ...config,
                  general: {
                    ...general,
                    block_screen_after_break_message: e.target.value,
                  },
                })
              }
              className="border-input bg-background text-foreground focus:ring-ring w-64 rounded-lg border px-3 py-1.5 text-sm focus:ring-2 focus:outline-none"
            />
          </Row>
        )}
      </Section>
    </div>
  )
}
