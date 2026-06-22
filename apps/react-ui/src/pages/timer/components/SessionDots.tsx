interface SessionDotsProps {
  dotTotal: number
  dotFilled: number
}

export function SessionDots({ dotTotal, dotFilled }: SessionDotsProps) {
  const dots = Array.from({ length: dotTotal }, (_, i) => i)

  return (
    <div className="flex items-center gap-2">
      {dots.map((i) => (
        <div
          key={i}
          className={[
            'h-2.5 w-2.5 rounded-full transition-all duration-300',
            i < dotFilled ? 'bg-indigo-500' : 'bg-muted-foreground/25',
          ].join(' ')}
        />
      ))}
    </div>
  )
}
