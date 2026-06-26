import { Loader2 } from 'lucide-react'

interface LoadingOverlayProps {
  open: boolean
  label?: string
}

export function LoadingOverlay({ open, label = 'Saving…' }: LoadingOverlayProps) {
  if (!open) return null

  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={label}
      className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/40 p-8 backdrop-blur-sm"
    >
      <div className="bg-card border-border text-foreground flex flex-col items-center gap-3 rounded-2xl border px-8 py-6 shadow-2xl">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p className="text-sm font-medium">{label}</p>
      </div>
    </div>
  )
}
