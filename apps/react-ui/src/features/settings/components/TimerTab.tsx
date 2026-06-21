import { useState } from 'react'
import { Row } from '@/components/ui/Row'
import { Section } from '@/components/ui/Section'
import { NumberInput } from '@/components/ui/NumberInput'
import { Toggle } from '@/components/ui/Toggle'
import { toSeconds, fromSeconds } from '@/lib/duration'
import type { Config } from '@/features/settings/types'

interface TimerTabProps {
  config: Config
  patch: (config: Config) => void
}

export function TimerTab({ config, patch }: TimerTabProps) {
  const [unit, setUnit] = useState<'seconds' | 'minutes'>('minutes')
  const { timer } = config

  const workVal = fromSeconds(timer.work_duration, unit)
  const shortVal = fromSeconds(timer.short_break_duration, unit)
  const longVal = fromSeconds(timer.long_break_duration, unit)

  return (
    <div className="space-y-8">
      <Section title="Timer">
        <Row label="Use seconds instead of minutes">
          <Toggle
            checked={unit === 'seconds'}
            onChange={(v) => setUnit(v ? 'seconds' : 'minutes')}
          />
        </Row>
        <Row
          label={`Focus duration (${unit})`}
          hint={unit === 'seconds' ? '5–10800 seconds' : '1–180 minutes'}
        >
          <NumberInput
            value={workVal}
            min={unit === 'seconds' ? 5 : 1}
            max={unit === 'seconds' ? 10800 : 180}
            onChange={(v) =>
              patch({ ...config, timer: { ...timer, work_duration: toSeconds(v, unit) } })
            }
          />
        </Row>
        <Row
          label={`Short break (${unit})`}
          hint={unit === 'seconds' ? '5–3600 seconds' : '1–60 minutes'}
        >
          <NumberInput
            value={shortVal}
            min={unit === 'seconds' ? 5 : 1}
            max={unit === 'seconds' ? 3600 : 60}
            onChange={(v) =>
              patch({
                ...config,
                timer: { ...timer, short_break_duration: toSeconds(v, unit) },
              })
            }
          />
        </Row>
        <Row
          label={`Long break (${unit})`}
          hint={unit === 'seconds' ? '5–7200 seconds' : '1–120 minutes'}
        >
          <NumberInput
            value={longVal}
            min={unit === 'seconds' ? 5 : 1}
            max={unit === 'seconds' ? 7200 : 120}
            onChange={(v) =>
              patch({
                ...config,
                timer: { ...timer, long_break_duration: toSeconds(v, unit) },
              })
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
