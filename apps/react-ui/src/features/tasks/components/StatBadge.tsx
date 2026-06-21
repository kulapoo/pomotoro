interface StatBadgeProps {
  label: string
  value: number
  color: string
}

export function StatBadge({ label, value, color }: StatBadgeProps) {
  return (
    <div className={`flex flex-col items-center rounded-xl border px-4 py-2 ${color}`}>
      <span className="text-2xl font-bold tabular-nums">{value}</span>
      <span className="mt-0.5 text-[10px] font-semibold tracking-wider uppercase">
        {label}
      </span>
    </div>
  )
}
