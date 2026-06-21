import { toast } from 'sonner'
import { logger } from '@/lib/logger'
import { BackendError } from '@/lib/errors'

/**
 * Single entry point for surfacing errors to the user.
 * Logs (console + Rust log file) and toasts a human-readable message.
 * Use this instead of ad-hoc `toast.error(String(e))` / `console.error`.
 */
export function toastError(err: unknown, fallback = 'Something went wrong'): void {
  logger.error(err)
  const message =
    err instanceof BackendError
      ? err.message
      : err instanceof Error
        ? err.message
        : fallback
  toast.error(message)
}
