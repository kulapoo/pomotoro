import { useEffect } from 'react'

interface ScreenBlockerProps {
  message: string
  onDismiss: () => void
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed',
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  width: '100vw',
  height: '100vh',
  zIndex: 9999,
  background: 'rgba(0, 0, 0, 0.95)',
  backdropFilter: 'blur(10px)',
  WebkitBackdropFilter: 'blur(10px)',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '2rem',
  textAlign: 'center',
  color: '#ffffff',
  fontFamily: 'inherit',
}

const messageStyle: React.CSSProperties = {
  fontSize: '3rem',
  fontWeight: 700,
  lineHeight: 1.15,
  marginBottom: '1.5rem',
  maxWidth: '60ch',
}

const subtitleStyle: React.CSSProperties = {
  fontSize: '1.25rem',
  fontWeight: 500,
  opacity: 0.85,
  marginBottom: '0.75rem',
  maxWidth: '50ch',
}

const hintStyle: React.CSSProperties = {
  fontSize: '0.95rem',
  opacity: 0.65,
  marginBottom: '2.5rem',
  maxWidth: '50ch',
}

const buttonStyle: React.CSSProperties = {
  padding: '0.75rem 1.75rem',
  fontSize: '0.95rem',
  fontWeight: 600,
  color: '#ffffff',
  background: 'rgba(255, 255, 255, 0.12)',
  border: '1px solid rgba(255, 255, 255, 0.25)',
  borderRadius: '0.75rem',
  cursor: 'pointer',
  transition: 'background 0.15s ease',
}

export function ScreenBlocker({ message, onDismiss }: ScreenBlockerProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onDismiss()
      }
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [onDismiss])

  return (
    <div style={overlayStyle} role="dialog" aria-modal="true" aria-label="Screen blocker">
      <div style={messageStyle}>{message}</div>
      <div style={subtitleStyle}>
        You&apos;re in a focused work session. Stay concentrated!
      </div>
      <div style={hintStyle}>
        Press ESC or click below if you need to temporarily disable blocking.
      </div>
      <button
        type="button"
        style={buttonStyle}
        onClick={onDismiss}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)'
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = 'rgba(255, 255, 255, 0.12)'
        }}
      >
        Temporarily Disable Blocking
      </button>
    </div>
  )
}
