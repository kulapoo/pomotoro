import { PartyPopper } from 'lucide-react'
import type { Page } from '@/app/types'

interface AllDonePromptProps {
  onNavigate: (page: Page) => void
}

export function AllDonePrompt({ onNavigate }: AllDonePromptProps) {
  return (
    <div className="flex min-h-full flex-col items-center justify-center gap-4 px-4 py-16 text-center">
      <div className="bg-muted/60 mb-2 flex h-14 w-14 items-center justify-center rounded-2xl">
        <PartyPopper size={26} className="text-muted-foreground" />
      </div>
      <h3 className="text-base font-semibold">All done!</h3>
      <p className="text-muted-foreground mb-3 max-w-xs text-sm">
        Every task is complete. Take a well-earned break.
      </p>
      <button
        onClick={() => onNavigate('tasks')}
        className="bg-primary text-primary-foreground flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm transition-all hover:opacity-90 active:scale-95"
      >
        <PartyPopper size={15} />
        Create a task
      </button>
    </div>
  )
}
