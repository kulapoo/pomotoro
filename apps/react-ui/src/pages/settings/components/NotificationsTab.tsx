import { Row } from '@/components/ui/Row'
import { Section } from '@/components/ui/Section'
import { NumberInput } from '@/components/ui/NumberInput'
import { Toggle } from '@/components/ui/Toggle'
import { SelectInput } from '@/components/ui/SelectInput'
import type { Config, NotificationPosition } from '@/pages/settings/useSettings'

interface NotificationsTabProps {
  config: Config
  patch: (config: Config) => void
}

const POSITION_OPTIONS: { value: NotificationPosition; label: string }[] = [
  { value: 'TopRight', label: 'Top Right' },
  { value: 'TopLeft', label: 'Top Left' },
  { value: 'BottomRight', label: 'Bottom Right' },
  { value: 'BottomLeft', label: 'Bottom Left' },
  { value: 'Center', label: 'Center' },
]

export function NotificationsTab({ config, patch }: NotificationsTabProps) {
  const { notification } = config

  return (
    <div className="space-y-8">
      <Section title="Notifications">
        <Row
          label="Desktop notifications"
          hint="Show system notifications for timer events"
        >
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
        <Row
          label="Phase transition alerts"
          hint="Notify when switching between work and break"
        >
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
            options={POSITION_OPTIONS}
            onChange={(v) =>
              patch({
                ...config,
                notification: { ...notification, notification_position: v },
              })
            }
          />
        </Row>
      </Section>
    </div>
  )
}
