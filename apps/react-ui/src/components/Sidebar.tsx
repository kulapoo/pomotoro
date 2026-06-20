import { useState } from 'react'
import { Timer, ListChecks, Settings } from 'lucide-react'
import type { Page } from '@/types'

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
        'flex flex-col items-center gap-2 py-6 bg-white/60 dark:bg-gray-900/60 backdrop-blur-sm border-r border-border shrink-0 transition-[width] duration-150',
        collapsed ? 'w-12' : 'w-16 md:w-20',
      ].join(' ')}
    >
      {/* Logo mark */}
      <div className="mb-2 w-9 h-9 rounded-xl bg-primary text-primary-foreground flex items-center justify-center font-bold text-sm select-none">
        P
      </div>

      {/* Collapse toggle */}
      <button
        onClick={() => setCollapsed((c) => !c)}
        title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        className="mb-4 p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
      >
        {collapsed ? '→' : '←'}
      </button>

      <nav className="flex flex-col items-center gap-1 flex-1 w-full px-2">
        {NAV_ITEMS.map(({ id, Icon, label }) => {
          const active = currentPage === id
          return (
            <button
              key={id}
              onClick={() => onNavigate(id)}
              title={label}
              className={[
                'flex flex-col items-center gap-1 py-2.5 w-full rounded-xl transition-colors duration-150',
                active
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
              ].join(' ')}
            >
              <Icon size={20} strokeWidth={active ? 2.5 : 2} />
              <span
                className={
                  collapsed ? 'hidden' : 'hidden md:block text-[10px] font-medium'
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
