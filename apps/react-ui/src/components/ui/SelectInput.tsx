interface SelectInputProps<T extends string> {
  value: T
  options: { value: T; label: string }[]
  onChange: (v: T) => void
}

export function SelectInput<T extends string>({
  value,
  options,
  onChange,
}: SelectInputProps<T>) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value as T)}
      className="border-input bg-background text-foreground focus:ring-ring rounded-lg border px-3 py-1.5 text-sm focus:ring-2 focus:outline-none"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  )
}
