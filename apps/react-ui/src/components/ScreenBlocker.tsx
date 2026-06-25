import { useEffect, useRef } from 'react'
import { useScreenBlockerStore } from '@/app/useScreenBlocker'

/**
 * Focus-enforcement overlay shown when a work or break phase expires while
 * screen blocking is enabled. Covers the whole viewport and dismisses on ESC
 * or the button, which restores the native window via `deactivate_screen_block`.
 */
export function ScreenBlocker() {
  const { isBlocking, message, dismiss } = useScreenBlockerStore()
  const dismissBtnRef = useRef<HTMLButtonElement>(null)

  // ESC dismisses; focus the button when shown so keyboard users can act.
  useEffect(() => {
    if (!isBlocking) return
    dismissBtnRef.current?.focus()

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        void dismiss()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [isBlocking, dismiss])

  if (!isBlocking) return null

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Screen blocker"
      className="fixed inset-0 z-[9999] flex flex-col items-center justify-center gap-6 bg-black/95 p-8 text-center"
    >
      <p className="max-w-md text-balance text-xl font-medium text-white">
        {message}
      </p>
      <button
        ref={dismissBtnRef}
        type="button"
        onClick={() => void dismiss()}
        className="rounded-lg bg-indigo-500 px-6 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-indigo-600 focus:ring-ring focus:ring-2 focus:outline-none"
      >
        Dismiss
      </button>
    </div>
  )
}
