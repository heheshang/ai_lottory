import { vi } from 'vitest'
import { config } from '@vue/test-utils'

// Mock Tauri API
global.__TAURI__ = {
  invoke: vi.fn(() => Promise.resolve({})),
  listen: vi.fn(() => Promise.resolve(() => {}))
}

// Mock Element Plus components
config.global.mocks = {
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn()
  },
  ElMessageBox: {
    confirm: vi.fn(() => Promise.resolve('confirm')),
    alert: vi.fn(() => Promise.resolve()),
    prompt: vi.fn(() => Promise.resolve('test'))
  },
  ElNotification: {
    success: vi.fn(),
    error: vi.fn()
  }
}

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))