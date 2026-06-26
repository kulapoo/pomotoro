import { RotateCcw } from 'lucide-react'

interface BulkSelectToolbarProps {
  selectedCount: number
  isAllVisibleSelected: boolean
  isBusy: boolean
  onSelectAllVisible: () => void
  onClear: () => void
  onResetSelected: () => void
}

export function BulkSelectToolbar({
  selectedCount,
  isAllVisibleSelected,
  isBusy,
  onSelectAllVisible,
  onClear,
  onResetSelected,
}: BulkSelectToolbarProps) {
  return (
    <div className="border-border bg-card mb-4 flex items-center justify-between gap-3 rounded-xl border px-4 py-2.5">
      <div className="flex items-center gap-3 text-sm">
        <span className="font-medium">{selectedCount} selected</span>
        <button
          onClick={onSelectAllVisible}
          className="text-muted-foreground hover:text-foreground text-xs underline transition-colors"
        >
          {isAllVisibleSelected ? 'Clear' : 'Select all'}
        </button>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={onClear}
          className="border-border text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
        >
          Cancel
        </button>
        <button
          onClick={onResetSelected}
          disabled={isBusy}
          className="bg-primary text-primary-foreground flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs transition-all hover:opacity-90 active:scale-95 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <RotateCcw size={12} />
          Reset selected
        </button>
      </div>
    </div>
  )
}
