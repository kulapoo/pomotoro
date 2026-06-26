import { useEffect, useRef } from 'react'
import { X, AlertTriangle } from 'lucide-react'

export interface ConfirmationDialogOptions {
  title?: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  variant?: 'default' | 'danger'
}

export interface ConfirmationDialogProps extends ConfirmationDialogOptions {
  open: boolean
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function ConfirmationDialog({
  open,
  onOpenChange,
  onConfirm,
  title,
  message,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  variant = 'default',
}: ConfirmationDialogProps) {
  const confirmRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (!open) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onOpenChange(false)
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [open, onOpenChange])

  useEffect(() => {
    if (open) {
      confirmRef.current?.focus()
    }
  }, [open])

  if (!open) return null

  const isDanger = variant === 'danger'

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={() => onOpenChange(false)}
      />

      <div className="bg-card border-border relative w-full max-w-md overflow-hidden rounded-2xl border shadow-2xl">
        <div className="border-border flex items-start gap-3 border-b px-6 py-4">
          {isDanger && (
            <div className="bg-destructive/10 text-destructive mt-0.5 rounded-full p-1.5">
              <AlertTriangle size={18} />
            </div>
          )}
          <div className="flex-1">
            {title && <h2 className="text-lg font-semibold">{title}</h2>}
            <p className="text-muted-foreground mt-1 text-sm">{message}</p>
          </div>
          <button
            onClick={() => onOpenChange(false)}
            className="text-muted-foreground hover:text-foreground hover:bg-accent -mt-1 -mr-1 rounded-lg p-1.5 transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex gap-3 px-6 py-4">
          <button
            onClick={() => onOpenChange(false)}
            className="border-border text-muted-foreground hover:text-foreground hover:bg-accent flex-1 rounded-xl border py-2.5 text-sm transition-colors"
          >
            {cancelLabel}
          </button>
          <button
            ref={confirmRef}
            onClick={onConfirm}
            className={[
              'flex-1 rounded-xl py-2.5 text-sm transition-all active:scale-95',
              isDanger
                ? 'bg-destructive text-destructive-foreground hover:opacity-90'
                : 'bg-primary text-primary-foreground hover:opacity-90',
            ].join(' ')}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
