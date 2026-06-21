import type { ReactNode } from 'react'

interface RowProps {
  label: string
  hint?: string
  children: ReactNode
}

export function Row({ label, hint, children }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-4 py-0.5">
      <div className="min-w-0 flex-1">
        <span className="text-sm font-medium">{label}</span>
        {hint && <p className="text-muted-foreground mt-0.5 text-xs">{hint}</p>}
      </div>
      {children}
    </div>
  )
}
