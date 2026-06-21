interface NumberInputProps {
  value: number
  min: number
  max: number
  onChange: (v: number) => void
  className?: string
}

export function NumberInput({ value, min, max, onChange, className }: NumberInputProps) {
  return (
    <input
      type="number"
      min={min}
      max={max}
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
      className={`border-input bg-background text-foreground focus:ring-ring w-20 rounded-lg border px-3 py-1.5 text-center text-sm focus:ring-2 focus:outline-none ${className ?? ''}`}
    />
  )
}
