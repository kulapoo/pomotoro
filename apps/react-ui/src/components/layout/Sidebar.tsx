import { useState } from 'react'
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
  const [collapsed, setCollapsed] = useState<boolean>(false)

  return (
    <aside
      className={[
        'border-border flex shrink-0 flex-col items-center gap-2 border-r bg-white/60 py-6 backdrop-blur-sm transition-[width] duration-150 dark:bg-gray-900/60',
        collapsed ? 'w-12' : 'w-16 md:w-20',
      ].join(' ')}
    >
      {/* Logo mark */}
      <div className="bg-primary text-primary-foreground mb-2 flex h-9 w-9 items-center justify-center rounded-xl">
        <ToroIcon size={22} />
      </div>

      {/* Collapse toggle */}
      <button
        onClick={() => setCollapsed((c) => !c)}
        title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        className="text-muted-foreground hover:text-foreground hover:bg-accent mb-4 rounded-lg p-1.5 transition-colors"
      >
        {collapsed ? '→' : '←'}
      </button>

      <nav className="flex w-full flex-1 flex-col items-center gap-1 px-2">
        {NAV_ITEMS.map(({ id, Icon, label }) => {
          const active = currentPage === id
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              title={label}
              className={[
                'flex w-full flex-col items-center gap-1 rounded-xl py-2.5 transition-colors duration-150',
                active
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
              ].join(' ')}
            >
              <Icon size={20} strokeWidth={active ? 2.5 : 2} />
              <span
                className={
                  collapsed ? 'hidden' : 'hidden text-[10px] font-medium md:block'
                }
              >
                {label}
              </span>
            </button>
          )
        })}
      </nav>
    </aside>
  )
}
