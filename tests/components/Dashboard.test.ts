/**
 * Dashboard Component Tests
 */

import { describe, it, expect, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import Dashboard from '../../src/components/Dashboard.vue'
import { useAuthStore } from '../../src/stores/auth'
import { useLotteryDataStore } from '../../src/stores/lottery-data'
import { usePredictionsStore } from '../../src/stores/predictions'
import { generateMockDraws, generateMockPredictions, setupTestEnvironment } from '../setup'

describe('Dashboard Component', () => {
  let wrapper: any
  let authStore: ReturnType<typeof useAuthStore>
  let dataStore: ReturnType<typeof useLotteryDataStore>
  let predictionsStore: ReturnType<typeof usePredictionsStore>

  beforeEach(() => {
    setupTestEnvironment()

    const pinia = createPinia()
    setActivePinia(pinia)

    authStore = useAuthStore()
    dataStore = useLotteryDataStore()
    predictionsStore = usePredictionsStore()

    // Setup test data
    authStore.$patch({
      user: {
        id: 1,
        username: 'testuser',
        email: 'test@example.com',
        created_at: new Date().toISOString(),
        last_login: new Date().toISOString()
      },
      isAuthenticated: true
    })

    dataStore.$patch({
      draws: generateMockDraws(50)
    })

    predictionsStore.$patch({
      predictions: generateMockPredictions(10)
    })

    wrapper = mount(Dashboard, {
      global: {
        plugins: [pinia]
      }
    })
  })

  describe('Component Rendering', () => {
    it('should render dashboard correctly', () => {
      expect(wrapper.find('.dashboard').exists()).toBe(true)
      expect(wrapper.find('.dashboard-header').exists()).toBe(true)
      expect(wrapper.find('.dashboard-content').exists()).toBe(true)
      expect(wrapper.find('.dashboard-stats').exists()).toBe(true)
    })

    it('should display user information', () => {
      expect(wrapper.text()).toContain('Welcome, testuser!')
      expect(wrapper.text()).toContain('test@example.com')
    })

    it('should display statistics', () => {
      expect(wrapper.find('.stats-cards').exists()).toBe(true)
      expect(wrapper.text()).toContain('50 Draws')
      expect(wrapper.text()).toContain('10 Predictions')
    })

    it('should display recent draws', () => {
      expect(wrapper.find('.recent-draws').exists()).toBe(true)
      expect(wrapper.find('.draw-card').exists()).toBe(true)
    })

    it('should display recent predictions', () => {
      expect(wrapper.find('.recent-predictions').exists()).toBe(true)
      expect(wrapper.find('.prediction-card').exists()).toBe(true)
    })
  })

  describe('User Authentication State', () => {
    it('should show welcome message when authenticated', () => {
      authStore.$patch({
        isAuthenticated: true,
        user: {
          id: 1,
          username: 'testuser',
          email: 'test@example.com'
        }
      })

      expect(wrapper.find('.welcome-message').exists()).toBe(true)
      expect(wrapper.text()).toContain('Welcome, testuser')
    })

    it('should show login prompt when not authenticated', () => {
      authStore.$patch({
        isAuthenticated: false,
        user: null
      })

      expect(wrapper.find('.login-prompt').exists()).toBe(true)
      expect(wrapper.text()).toContain('Please log in')
    })

    it('should handle user logout', async () => {
      // Mock the logout method
      const logoutSpy = vi.spyOn(authStore, 'logout').mockResolvedValue(true)

      await wrapper.vm.logout()

      expect(logoutSpy).toHaveBeenCalled()
    })
  })

  describe('Data Loading States', () => {
    it('should show loading state while fetching data', async () => {
      dataStore.$patch({
        loading: {
          draws: true
        }
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.loading-spinner').exists()).toBe(true)
      expect(wrapper.text()).toContain('Loading draws...')
    })

    it('should show error state when data fetch fails', () => {
      dataStore.$patch({
        error: 'Failed to fetch draws'
      })

      expect(wrapper.find('.error-message').exists()).toBe(true)
      expect(wrapper.text()).toContain('Failed to fetch draws')
    })

    it('should show empty state when no data available', () => {
      dataStore.$patch({
        draws: []
      })

      expect(wrapper.find('.empty-state').exists()).toBe(true)
      expect(wrapper.text()).toContain('No draws available')
    })
  })

  describe('Draw Display', () => {
    it('should display correct draw information', () => {
      const draws = wrapper.findAll('.draw-card')
      expect(draws.length).toBeGreaterThan(0)

      const firstDraw = draws[0]
      expect(firstDraw.find('.draw-numbers').exists()).toBe(true)
      expect(firstDraw.find('.draw-date').exists()).toBe(true)
      expect(firstDraw.find('.jackpot-amount').exists()).toBe(true)
    })

    it('should format draw numbers correctly', () => {
      const firstDraw = wrapper.find('.draw-card')
      const numbers = firstDraw.findAll('.number')

      expect(numbers.length).toBe(5) // Main numbers
      expect(numbers[0].text()).toMatch(/\d+/) // Should contain a number
    })

    it('should handle large jackpot amounts', () => {
      dataStore.$patch({
        draws: [{
          id: 1,
          draw_date: new Date().toISOString(),
          winning_numbers: [1, 2, 3, 4, 5],
          bonus_number: 1,
          jackpot_amount: 10000000,
          created_at: new Date().toISOString()
        }]
      })

      expect(wrapper.text()).toContain('$10,000,000')
    })
  })

  describe('Prediction Display', () => {
    it('should display prediction information', () => {
      const predictions = wrapper.findAll('.prediction-card')
      expect(predictions.length).toBeGreaterThan(0)

      const firstPrediction = predictions[0]
      expect(firstPrediction.find('.algorithm-name').exists()).toBe(true)
      expect(firstPrediction.find('.confidence-score').exists()).toBe(true)
      expect(firstPrediction.find('.prediction-numbers').exists()).toBe(true)
    })

    it('should show confidence score with correct formatting', () => {
      const firstPrediction = wrapper.find('.prediction-card')
      const confidenceElement = firstPrediction.find('.confidence-score')

      expect(confidenceElement.text()).toMatch(/\d+%/) // Should contain a percentage
    })

    it('should color-code confidence scores', () => {
      const predictions = wrapper.findAll('.prediction-card')

      // Find predictions with different confidence scores
      predictions.forEach(prediction => {
        const confidenceText = prediction.find('.confidence-score').text()
        const confidenceValue = parseFloat(confidenceText.replace('%', ''))

        if (confidenceValue >= 80) {
          expect(prediction.find('.high-confidence').exists()).toBe(true)
        } else if (confidenceValue >= 60) {
          expect(prediction.find('.medium-confidence').exists()).toBe(true)
        } else {
          expect(prediction.find('.low-confidence').exists()).toBe(true)
        }
      })
    })
  })

  describe('Interactive Elements', () => {
    it('should handle draw selection', async () => {
      const draws = wrapper.findAll('.draw-card')
      const firstDraw = draws[0]

      await firstDraw.trigger('click')

      expect(wrapper.vm.selectedDraws).toContain(1)
      expect(firstDraw.classes()).toContain('selected')
    })

    it('should handle multiple draw selection', async () => {
      const draws = wrapper.findAll('.draw-card')
      const firstDraw = draws[0]
      const secondDraw = draws[1]

      await firstDraw.trigger('click')
      await secondDraw.trigger('click')

      expect(wrapper.vm.selectedDraws).toHaveLength(2)
      expect(firstDraw.classes()).toContain('selected')
      expect(secondDraw.classes()).toContain('selected')
    })

    it('should handle prediction generation', async () => {
      const generateSpy = vi.spyOn(wrapper.vm, 'generatePrediction').mockResolvedValue({})

      await wrapper.vm.$nextTick()

      const generateButton = wrapper.find('.generate-prediction-btn')
      await generateButton.trigger('click')

      expect(generateSpy).toHaveBeenCalled()
    })

    it('should handle refresh data', async () => {
      const refreshSpy = vi.spyOn(dataStore, 'fetchDraws').mockResolvedValue([])

      const refreshButton = wrapper.find('.refresh-btn')
      await refreshButton.trigger('click')

      expect(refreshSpy).toHaveBeenCalled()
    })
  })

  describe('Responsive Behavior', () => {
    it('should adapt layout for mobile screens', async () => {
      // Mock mobile viewport
      window.innerWidth = 768
      window.innerHeight = 1024

      // Trigger resize
      window.dispatchEvent(new Event('resize'))

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.dashboard.mobile-layout').exists()).toBe(true)
    })

    it('should adapt layout for tablet screens', async () => {
      // Mock tablet viewport
      window.innerWidth = 1024
      window.innerHeight = 768

      // Trigger resize
      window.dispatchEvent(new Event('resize'))

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.dashboard.tablet-layout').exists()).toBe(true)
    })

    it('should adapt layout for desktop screens', async () => {
      // Mock desktop viewport
      window.innerWidth = 1920
      window.innerHeight = 1080

      // Trigger resize
      window.dispatchEvent(new Event('resize'))

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.dashboard.desktop-layout').exists()).toBe(true)
    })
  })

  describe('Error Handling', () => {
    it('should handle fetch errors gracefully', async () => {
      const error = new Error('Network error')
      vi.spyOn(dataStore, 'fetchDraws').mockRejectedValue(error)

      await wrapper.vm.$nextTick()

      expect(wrapper.find('.error-state').exists()).toBe(true)
      expect(wrapper.text()).toContain('Failed to load data')
    })

    it('should provide retry functionality on errors', async () => {
      const error = new Error('Network error')
      const fetchSpy = vi.spyOn(dataStore, 'fetchDraws')
        .mockRejectedValueOnce(error)
        .mockResolvedValueOnce(generateMockDraws(10))

      // First attempt (fails)
      await wrapper.vm.fetchData()

      expect(wrapper.find('.error-state').exists()).toBe(true)

      // Retry attempt (succeeds)
      await wrapper.vm.retryFetch()

      expect(fetchSpy).toHaveBeenCalledTimes(2)
      expect(wrapper.find('.error-state').exists()).toBe(false)
    })

    it('should handle invalid data gracefully', () => {
      // Mock invalid draw data
      dataStore.$patch({
        draws: [{
          id: 'invalid',
          draw_date: 'invalid-date',
          winning_numbers: [],
          bonus_number: null,
          created_at: new Date().toISOString()
        }]
      })

      expect(wrapper.find('.data-error').exists()).toBe(true)
      expect(wrapper.text()).toContain('Invalid draw data')
    })
  })

  describe('Performance', () => {
    it('should render large datasets efficiently', async () => {
      // Mock large dataset
      dataStore.$patch({
        draws: Array.from({ length: 1000 }, (_, i) => ({
          id: i + 1,
          draw_date: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString(),
          winning_numbers: [1, 2, 3, 4, 5],
          bonus_number: 1,
          created_at: new Date().toISOString()
        }))
      })

      const startTime = performance.now()
      await wrapper.vm.$nextTick()
      const renderTime = performance.now() - startTime

      // Should render within reasonable time
      expect(renderTime).toBeLessThan(100)
      expect(wrapper.findAll('.draw-card').length).toBeLessThanOrEqual(50) // Should be paginated
    })

    it('should implement virtual scrolling for large lists', () => {
      dataStore.$patch({
        draws: Array.from({ length: 10000 }, (_, i) => ({
          id: i + 1,
          draw_date: new Date().toISOString(),
          winning_numbers: [1, 2, 3, 4, 5],
          bonus_number: 1,
          created_at: new Date().toISOString()
        }))
      })

      expect(wrapper.vm.useVirtualScroll).toBe(true)
      expect(wrapper.find('.virtual-scroll-container').exists()).toBe(true)
    })
  })

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      expect(wrapper.find('.dashboard').attributes('aria-label')).toBe('Lottery Dashboard')
      expect(wrapper.find('.recent-draws').attributes('aria-label')).toBe('Recent Lottery Draws')
      expect(wrapper.find('.recent-predictions').attributes('aria-label')).toBe('Recent Predictions')
    })

    it('should be keyboard navigable', async () => {
      const drawCards = wrapper.findAll('.draw-card')
      const firstCard = drawCards[0]

      await firstCard.trigger('keydown', { key: 'Enter' })
      expect(firstCard.classes()).toContain('selected')

      await firstCard.trigger('keydown', { key: ' ' })
      expect(firstCard.classes()).toContain('selected')
    })

    it('should support screen readers', () => {
      const drawCards = wrapper.findAll('.draw-card')

      drawCards.forEach(card => {
        expect(card.attributes('role')).toBe('button')
        expect(card.attributes('tabindex')).toBe('0')
      })
    })
  })

  describe('Data Updates', () => {
    it('should react to store updates', async () => {
      const initialDrawCount = wrapper.findAll('.draw-card').length

      // Add new draw
      dataStore.$patch({
        draws: [...dataStore.draws, {
          id: 999,
          draw_date: new Date().toISOString(),
          winning_numbers: [10, 20, 30, 40, 50],
          bonus_number: 5,
          created_at: new Date().toISOString()
        }]
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.findAll('.draw-card').length).toBeGreaterThan(initialDrawCount)
    })

    it('should handle real-time predictions', async () => {
      const initialPredictionCount = wrapper.findAll('.prediction-card').length

      // Add new prediction
      predictionsStore.$patch({
        predictions: [...predictionsStore.predictions, {
          id: 'prediction_999',
          algorithm: 'new_algorithm',
          front_numbers: [11, 21, 31, 41, 51],
          back_numbers: [6],
          confidence_score: 0.95,
          reasoning: 'Real-time prediction',
          created_at: new Date().toISOString()
        }]
      })

      await wrapper.vm.$nextTick()

      expect(wrapper.findAll('.prediction-card').length).toBeGreaterThan(initialPredictionCount)
    })
  })
})