// Tauri window type declarations
declare global {
  interface Window {
    __TAURI__: {
      invoke: <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>
      listen: <T = unknown>(event: string, handler: (event: { payload: T }) => void) => Promise<() => void>
      emit: (event: string, payload?: unknown) => Promise<void>
    }
  }
}

export {}
