import {
  info as tauriInfo,
  warn as tauriWarn,
  error as tauriError,
} from '@tauri-apps/plugin-log'

/**
 * True when running inside a Tauri webview (IPC available).
 * In Vite-only mode (`just serve-react`) the plugin-log calls are skipped.
 */
const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const isDev = import.meta.env.DEV

function format(value: unknown): string {
  if (value instanceof Error) return value.stack ?? `${value.name}: ${value.message}`
  if (typeof value === 'string') return value
  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

function compose(msg: unknown, rest: unknown[]): string {
  return rest.length ? `${format(msg)} ${rest.map(format).join(' ')}` : format(msg)
}

function forward(level: 'info' | 'warn' | 'error', line: string): void {
  if (!inTauri) return
  const fn = level === 'info' ? tauriInfo : level === 'warn' ? tauriWarn : tauriError
  void fn(line).catch(() => {})
}

/**
 * Structured logger. `debug` is dev-only (console). `info`/`warn`/`error`
 * hit the browser console AND — when in Tauri — the Rust log pipeline
 * (stdout + webview console + ~/.local/share/<bundle>/logs/pomotoro.log),
 * so a single log file contains both layers.
 */
export const logger = {
  debug(msg: unknown, ...rest: unknown[]): void {
    if (!isDev) return
    console.debug(compose(msg, rest))
  },
  info(msg: unknown, ...rest: unknown[]): void {
    const line = compose(msg, rest)
    console.info(line)
    forward('info', line)
  },
  warn(msg: unknown, ...rest: unknown[]): void {
    const line = compose(msg, rest)
    console.warn(line)
    forward('warn', line)
  },
  error(msg: unknown, ...rest: unknown[]): void {
    const line = compose(msg, rest)
    console.error(line)
    forward('error', line)
  },
}
