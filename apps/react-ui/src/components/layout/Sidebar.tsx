import { Timer, ListChecks, Settings } from 'lucide-react'
import { ToroIcon } from '@/components/ui/ToroIcon'
import type { Page } from '@/app/types'

interface SidebarProps {
  currentPage: Page
  onNavigate: (page: Page) => void
}

const NAV_ITEMS = [
  { id: 'timer' as Page, Icon: Timer, label: 'Timer' },
  { id: 'tasks' as Page, Icon: ListChecks, label: 'Tasks' },
  { id: 'settings' as Page, Icon: Settings, label: 'Settings' },
] as const

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="bg-card/50 border-border flex w-16 shrink-0 flex-col border-r backdrop-blur-md">
      {/* Brand mark — the anchor */}
      <div className="flex justify-center pt-6">
        <div className="bg-primary text-primary-foreground ring-primary/25 shadow-primary/30 flex h-10 w-10 items-center justify-center rounded-2xl shadow-lg ring-1">
          <ToroIcon size={22} />
        </div>
      </div>

      {/* Primary nav */}
      <nav className="flex flex-1 flex-col justify-center gap-1 px-2">
        {NAV_ITEMS.map(({ id, Icon, label }) => {
          const active = currentPage === id
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              title={label}
              aria-label={label}
              aria-current={active ? 'page' : undefined}
              className={[
                'group focus-visible:ring-ring relative flex h-11 items-center justify-center rounded-xl transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2',
                active
                  ? 'bg-primary/10 text-primary'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
              ].join(' ')}
            >
              {active && (
                <span className="bg-primary absolute top-1/2 left-0 h-5 w-1 -translate-y-1/2 rounded-r-full" />
              )}
              <Icon
                size={22}
                strokeWidth={active ? 2.5 : 2}
                className="transition-transform duration-200 group-hover:scale-105"
              />
            </button>
          )
        })}
      </nav>

      {/* Quiet brand footer */}
      <div className="flex justify-center pb-5">
        <span className="text-muted-foreground/40 text-[10px] font-semibold tracking-[0.2em]">
          TORO
        </span>
      </div>
    </aside>
  )
}
