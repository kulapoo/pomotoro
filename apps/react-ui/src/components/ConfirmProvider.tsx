import { createContext, useCallback, useContext, useState, type ReactNode } from 'react'
import { ConfirmationDialog, type ConfirmationDialogOptions } from '@/components/ui/ConfirmationDialog'

interface ConfirmContextValue {
  confirm: (opts: ConfirmationDialogOptions) => Promise<boolean>
}

const ConfirmContext = createContext<ConfirmContextValue | null>(null)

// eslint-disable-next-line react-refresh/only-export-components
export function useConfirm(): ConfirmContextValue {
  const ctx = useContext(ConfirmContext)
  if (!ctx) throw new Error('useConfirm must be used within a ConfirmProvider')
  return ctx
}

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false)
  const [opts, setOpts] = useState<ConfirmationDialogOptions>({ message: '' })
  const [resolveRef, setResolveRef] = useState<((value: boolean) => void) | null>(null)

  const confirm = useCallback(
    (dialogOpts: ConfirmationDialogOptions): Promise<boolean> =>
      new Promise((resolve) => {
        setResolveRef(() => resolve)
        setOpts(dialogOpts)
        setOpen(true)
      }),
    [],
  )

  const handleOpenChange = useCallback(
    (nextOpen: boolean) => {
      setOpen(nextOpen)
      if (!nextOpen && resolveRef) {
        resolveRef(false)
        setResolveRef(null)
      }
    },
    [resolveRef],
  )

  const handleConfirm = useCallback(() => {
    setOpen(false)
    if (resolveRef) {
      resolveRef(true)
      setResolveRef(null)
    }
  }, [resolveRef])

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      <ConfirmationDialog
        open={open}
        onOpenChange={handleOpenChange}
        onConfirm={handleConfirm}
        {...opts}
      />
    </ConfirmContext.Provider>
  )
}
