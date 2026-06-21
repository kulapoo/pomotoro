import { Component } from 'react'
import type { ErrorInfo, ReactNode } from 'react'
import { logger } from '@/lib/logger'

interface Props {
  children: ReactNode
}

interface State {
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error('[ErrorBoundary]', error, info.componentStack ?? '')
  }

  render() {
    const { error } = this.state
    if (error) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
          <p className="text-destructive text-sm font-medium">{error.message}</p>
          <button
            onClick={() => {
              this.setState({ error: null })
              window.location.reload()
            }}
            className="border-border hover:bg-accent rounded-lg border px-3 py-1.5 text-xs transition-colors"
          >
            Reload
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
