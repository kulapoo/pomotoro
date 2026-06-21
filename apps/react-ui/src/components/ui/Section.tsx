import type { ReactNode } from 'react'

interface SectionProps {
  title: string
  children: ReactNode
}

export function Section({ title, children }: SectionProps) {
  return (
    <section>
      <h2 className="text-muted-foreground mb-3 text-sm font-semibold tracking-wider uppercase">
        {title}
      </h2>
      <div className="border-border bg-card space-y-4 rounded-xl border p-5">
        {children}
      </div>
    </section>
  )
}
