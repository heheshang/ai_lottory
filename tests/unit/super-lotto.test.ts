import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSuperLottoStore } from '@/stores/superLotto'
import * as api from '@/api/superLotto'

// Mock API
vi.mock('@/api/superLotto', () => ({
  getSuperLottoDraws: vi.fn(),
  analyzeHotNumbers: vi.fn(),
  analyzeColdNumbers: vi.fn(),
  generatePrediction: vi.fn()
}))

describe('Super Lotto Store', () => {
  let store: ReturnType<typeof useSuperLottoStore>

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useSuperLottoStore()
    vi.clearAllMocks()
  })

  it('should initialize with correct default values', () => {
    expect(store.draws).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.error).toBe(null)
    expect(store.predictions).toEqual([])
  })

  it('should handle loading state correctly', async () => {
    const mockDraws = [
      {
        id: 1,
        draw_date: '2024-01-01',
        front_zone: [1, 2, 3, 4, 5],
        back_zone: [6, 7],
        jackpot_amount: 1000000
      }
    ]

    vi.mocked(api.getSuperLottoDraws).mockResolvedValue(mockDraws)

    const promise = store.fetchDraws()
    expect(store.loading).toBe(true)

    await promise
    expect(store.loading).toBe(false)
    expect(store.draws).toEqual(mockDraws)
  })

  it('should handle API errors correctly', async () => {
    const error = new Error('Network error')
    vi.mocked(api.getSuperLottoDraws).mockRejectedValue(error)

    await store.fetchDraws()

    expect(store.loading).toBe(false)
    expect(store.error).toBeTruthy()
    expect(store.draws).toEqual([])
  })

  it('should generate predictions successfully', async () => {
    const mockPrediction = {
      id: 1,
      algorithm: 'WEIGHTED_FREQUENCY',
      front_numbers: [1, 2, 3, 4, 5],
      back_numbers: [6, 7],
      confidence_score: 0.75
    }

    vi.mocked(api.generatePrediction).mockResolvedValue(mockPrediction)

    await store.generatePrediction('WEIGHTED_FREQUENCY', 90)

    expect(store.predictions).toContainEqual(mockPrediction)
  })

  it('should cache draw data correctly', async () => {
    const mockDraws = [{ id: 1, front_zone: [1, 2, 3, 4, 5], back_zone: [6, 7] }]
    vi.mocked(api.getSuperLottoDraws).mockResolvedValue(mockDraws)

    // First call should hit API
    await store.fetchDraws()
    expect(api.getSuperLottoDraws).toHaveBeenCalledTimes(1)

    // Second call should use cache
    await store.fetchDraws()
    expect(api.getSuperLottoDraws).toHaveBeenCalledTimes(1)
  })

  it('should clear cache when refreshing', async () => {
    const mockDraws = [{ id: 1, front_zone: [1, 2, 3, 4, 5], back_zone: [6, 7] }]
    vi.mocked(api.getSuperLottoDraws).mockResolvedValue(mockDraws)

    // Load initial data
    await store.fetchDraws()
    expect(api.getSuperLottoDraws).toHaveBeenCalledTimes(1)

    // Refresh should bypass cache
    await store.refreshDraws()
    expect(api.getSuperLottoDraws).toHaveBeenCalledTimes(2)
  })

  it('should validate prediction parameters', () => {
    expect(store.validatePredictionParams()).toBe(false)

    store.selectedAlgorithm = 'WEIGHTED_FREQUENCY'
    store.analysisPeriod = 90
    expect(store.validatePredictionParams()).toBe(true)
  })

  it('should handle hot numbers analysis', async () => {
    const mockHotNumbers = [
      { number: 1, zone: 'FRONT', hot_score: 0.8, frequency: 0.15 },
      { number: 2, zone: 'FRONT', hot_score: 0.7, frequency: 0.12 }
    ]

    vi.mocked(api.analyzeHotNumbers).mockResolvedValue(mockHotNumbers)

    await store.analyzeHotNumbers(30)

    expect(store.hotNumbers).toEqual(mockHotNumbers)
  })
})