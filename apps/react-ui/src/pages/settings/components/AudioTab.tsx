import { Row } from '@/components/ui/Row'
import { Section } from '@/components/ui/Section'
import { Toggle } from '@/components/ui/Toggle'
import { SelectInput } from '@/components/ui/SelectInput'
import { useSettingsStore } from '@/pages/settings/useSettings'
import type { Config } from '@/pages/settings/useSettings'

interface AudioTabProps {
  config: Config
  patch: (config: Config) => void
}

const SOUND_OPTIONS = [
  { value: '', label: 'None' },
  { value: 'bell.wav', label: 'Bell' },
  { value: 'chime.wav', label: 'Chime' },
  { value: 'gong.wav', label: 'Gong' },
] as const

const BG_SOUND_OPTIONS = [
  { value: '', label: 'None' },
  { value: 'rain.wav', label: 'Rain' },
  { value: 'forest.wav', label: 'Forest' },
  { value: 'ocean.wav', label: 'Ocean' },
  { value: 'whitenoise.wav', label: 'White Noise' },
] as const

export function AudioTab({ config, patch }: AudioTabProps) {
  const { audio } = config
  const testAudioPreview = useSettingsStore((s) => s.testAudioPreview)

  const testAudio = async (category: 'work' | 'break' | 'background') => {
    const assetId =
      category === 'break'
        ? (audio.break_notification_sound ?? '')
        : category === 'background'
          ? (audio.background_sound ?? '')
          : (audio.work_notification_sound ?? '')
    await testAudioPreview(assetId, audio.volume)
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
            <span className="text-muted-foreground w-10 text-right text-sm tabular-nums">
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
              options={[...SOUND_OPTIONS]}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, work_notification_sound: v || null },
                })
              }
            />
            <TestButton onClick={() => testAudio('work')} />
          </div>
        </Row>

        <Row label="Break notification sound" hint="Sound when break session ends">
          <div className="flex items-center gap-2">
            <SelectInput
              value={audio.break_notification_sound ?? ''}
              options={[...SOUND_OPTIONS]}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, break_notification_sound: v || null },
                })
              }
            />
            <TestButton onClick={() => testAudio('break')} />
          </div>
        </Row>

        <Row label="Background sound" hint="Ambient sound during work sessions">
          <div className="flex items-center gap-2">
            <SelectInput
              value={audio.background_sound ?? ''}
              options={[...BG_SOUND_OPTIONS]}
              onChange={(v) =>
                patch({
                  ...config,
                  audio: { ...audio, background_sound: v || null },
                })
              }
            />
            <TestButton onClick={() => testAudio('background')} />
          </div>
        </Row>
      </Section>
    </div>
  )
}

function TestButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="border-border text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
    >
      Test
    </button>
  )
}
