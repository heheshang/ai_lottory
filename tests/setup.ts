/**
 * Test Setup and Configuration
 *
 * Provides testing utilities, mock setup, and test helpers for the
 * refactored frontend codebase. Includes configuration for Jest,
 * Vue Test Utils, and custom test utilities.
 */

import { config } from '@vue/test-utils'
import { vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

// Global test configuration
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock Tauri APIs
global.__TAURI__ = {
  invoke: vi.fn(),
  listen: vi.fn(),
  emit: vi.fn(),
  transformCallback: vi.fn()
}

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}

  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value.toString()
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key]
    }),
    clear: vi.fn(() => {
      store = {}
    }),
    get length() {
      return Object.keys(store).length
    },
    key: vi.fn((index: number) => {
      const keys = Object.keys(store)
      return keys[index] || null
    })
  }
})()

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock
})

// Mock performance API
Object.defineProperty(window, 'performance', {
  value: {
    now: vi.fn(() => Date.now()),
    getEntriesByType: vi.fn(() => []),
    observe: vi.fn(),
    memory: {
      usedJSHeapSize: 10000000,
      totalJSHeapSize: 50000000
    }
  }
})

// Mock fetch
global.fetch = vi.fn()

// Mock matchMedia
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

// Vue Test Utils configuration
config.global.plugins = []
config.global.stubs = {
  // Global component stubs
  'router-link': {
    template: '<a><slot /></a>',
    props: ['to']
  },
  'router-view': {
    template: '<div><slot /></div>'
  },
  // UI component stubs
  'el-button': {
    template: '<button><slot /></button>',
    props: ['type', 'size', 'disabled', 'loading']
  },
  'el-input': {
    template: '<input />',
    props: ['modelValue', 'placeholder', 'disabled']
  },
  'el-table': {
    template: '<table><slot /></table>',
    props: ['data', 'stripe']
  },
  'el-table-column': {
    template: '<td><slot /></td>',
    props: ['prop', 'label', 'width']
  },
  'el-pagination': {
    template: '<div>Pagination</div>',
    props: ['total', 'pageSize', 'currentPage']
  },
  'el-loading': {
    template: '<div>Loading...</div>',
    props: ['visible', 'text']
  },
  'el-notification': {
    template: '<div>Notification</div>',
    props: ['title', 'message', 'type']
  }
}

// Create and set active Pinia for tests
beforeEach(() => {
  const pinia = createPinia()
  setActivePinia(pinia)
})

// Clear all mocks after each test
afterEach(() => {
  vi.clearAllMocks()
  localStorageMock.clear()
})

// Export test utilities
export const mockTauriInvoke = (returnValue: any) => {
  vi.mocked(global.__TAURI__.invoke).mockResolvedValue(returnValue)
}

export const mockFetch = (returnValue: any, options: { status?: number; ok?: boolean } = {}) => {
  vi.mocked(global.fetch).mockResolvedValue({
    ok: options.ok !== false,
    status: options.status || 200,
    json: () => Promise.resolve(returnValue),
    text: () => Promise.resolve(JSON.stringify(returnValue))
  })
}

export const createMockUser = () => ({
  id: 1,
  username: 'testuser',
  email: 'test@example.com',
  created_at: new Date().toISOString(),
  last_login: new Date().toISOString()
})

export const createMockLotteryDraw = () => ({
  id: 1,
  draw_date: new Date().toISOString(),
  winning_numbers: [1, 2, 3, 4, 5],
  bonus_number: 1,
  jackpot_amount: 1000000,
  created_at: new Date().toISOString()
})

export const createMockPrediction = (algorithm: string = 'weighted_frequency') => ({
  id: 'prediction_1',
  algorithm,
  front_numbers: [1, 2, 3, 4, 5],
  back_numbers: [1],
  confidence_score: 0.75,
  reasoning: 'Test prediction reasoning',
  analysis_period_days: 90,
  sample_size: 1000,
  is_accurate: false,
  created_at: new Date().toISOString()
})

export const advanceTime = (ms: number) => {
  vi.setSystemTime(new Date(Date.now() + ms))
}

export const waitFor = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

// Test data generators
export const generateMockDraws = (count: number) =>
  Array.from({ length: count }, (_, i) => ({
    ...createMockLotteryDraw(),
    id: i + 1,
    draw_date: new Date(Date.now() - (i * 24 * 60 * 60 * 1000)).toISOString(),
    winning_numbers: Array.from({ length: 5 }, (_, j) => (j + 1) % 35 + 1)
  }))

export const generateMockPredictions = (count: number, algorithms: string[] = ['weighted_frequency', 'pattern_based', 'markov_chain']) =>
  Array.from({ length: count }, (_, i) => ({
    ...createMockPrediction(algorithms[i % algorithms.length]),
    id: `prediction_${i + 1}`,
    confidence_score: 0.5 + (Math.random() * 0.5)
  }))

// Performance testing utilities
export const measurePerformance = (fn: () => void | Promise<void>) => {
  const start = performance.now()
  return Promise.resolve(fn()).then(() => {
    const end = performance.now()
    return end - start
  })
}

// Error testing utilities
export const createMockError = (message: string, code: string = 'TEST_ERROR') => ({
  code,
  message,
  timestamp: new Date()
})

// Store testing utilities
export const createMockStoreState = (overrides: Record<string, any> = {}) => ({
  loading: false,
  error: null,
  data: [],
  ...overrides
})

// Component testing utilities
export const mountComponentWithMocks = async (
  component: any,
  props: Record<string, any> = {},
  mocks: Record<string, any> = {}
) => {
  // Apply mocks
  Object.entries(mocks).forEach(([key, value]) => {
    vi.mocked(key).mockImplementation(value)
  })

  // Mount component
  return mount(component, {
    props,
    global: {
      plugins: [...config.global.plugins],
      stubs: config.global.stubs
    }
  })
}

// API testing utilities
export const createMockAPIResponse = <T>(data: T, status = 200) => ({
  data,
  status,
  statusText: 'OK',
  headers: {},
  cached: false,
  requestId: `test_${Date.now()}`,
  duration: 100
})

export const createMockAPIError = (message: string, code = 'API_ERROR') => ({
  code,
  message,
  details: null,
  retryable: true,
  endpoint: '/test',
  requestId: `test_${Date.now()}`,
  timestamp: new Date()
})

// Test environment utilities
export const setupTestEnvironment = () => {
  // Reset DOM
  document.body.innerHTML = ''

  // Reset mocks
  vi.clearAllMocks()

  // Reset localStorage
  localStorageMock.clear()

  // Reset Tauri mock
  global.__TAURI__.invoke.mockClear()

  // Reset fetch mock
  global.fetch.mockClear()
}

export const cleanupTestEnvironment = () => {
  setupTestEnvironment()
}