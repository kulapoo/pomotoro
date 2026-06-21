export interface BackendErrorInit {
  command: string
  args?: unknown
  cause?: unknown
}

/**
 * Error thrown by {@link invokeCmd} when a Tauri backend command fails.
 * Preserves the command name, arguments and original cause so callers and
 * the logger/toast pipeline can surface actionable diagnostics instead of a
 * lossy `String(e)`.
 */
export class BackendError extends Error {
  readonly command: string
  readonly args: unknown
  readonly cause: unknown

  constructor({ command, args, cause }: BackendErrorInit) {
    super(`[${command}] ${describeCause(cause)}`)
    this.name = 'BackendError'
    this.command = command
    this.args = args
    this.cause = cause
  }
}

function describeCause(cause: unknown): string {
  if (typeof cause === 'string') return cause
  if (cause instanceof Error) return cause.message
  if (cause && typeof cause === 'object' && 'message' in cause) {
    return String((cause as { message: unknown }).message)
  }
  try {
    return JSON.stringify(cause)
  } catch {
    return String(cause)
  }
}
