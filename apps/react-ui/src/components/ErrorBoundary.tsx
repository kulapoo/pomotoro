import { Component } from 'react'
import type { ErrorInfo, ReactNode } from 'react'

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
    console.error('[ErrorBoundary]', error, info.componentStack)
  }

  render() {
    const { error } = this.state
    if (error) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
          <p className="text-sm font-medium text-destructive">{error.message}</p>
          <button
            onClick={() => {
              this.setState({ error: null })
              window.location.reload()
            }}
            className="text-xs px-3 py-1.5 rounded-lg border border-border hover:bg-accent transition-colors"
          >
            Reload
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
