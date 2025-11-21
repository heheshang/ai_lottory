import { invoke } from '@tauri-apps/api/core'
import type {
  User,
  UserLogin,
  UserRegistration,
  LotteryDraw,
  NewLotteryDraw,
  LotterySearchQuery,
  AnalysisRequest,
  NumberStatistics,
  HotNumbersResponse,
  ColdNumbersResponse
} from '@/types'

// Helper to check if running in Tauri environment
const isTauri = () => {
  return typeof window !== 'undefined' && window.__TAURI__ !== undefined
}

// Mock data for browser development
const mockUsers: User[] = [
  {
    id: 1,
    username: 'admin',
    email: 'admin@example.com',
    created_at: new Date().toISOString(),
    last_login: new Date().toISOString()
  }
]

// Authentication API
export const authApi = {
  async login(credentials: UserLogin): Promise<User> {
    console.log('🔵 [Tauri API] Login attempt for:', credentials.username)

    if (isTauri()) {
      console.log('🔵 [Tauri API] Running in Tauri environment, calling Rust command')
      const result = await invoke('login', { login: credentials })
      console.log('🔵 [Tauri API] Login command result:', result)
      return result
    } else {
      console.log('🔵 [Tauri API] Running in browser, using mock data')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 500))

      // Mock login logic
      const user = mockUsers.find(u => u.username === credentials.username)
      if (user && credentials.password === '123456') {
        console.log('🔵 [Tauri API] Mock login successful')
        return {
          ...user,
          last_login: new Date().toISOString()
        }
      } else {
        console.error('🔴 [Tauri API] Mock login failed: Invalid credentials')
        throw new Error('Invalid username or password')
      }
    }
  },

  async register(userData: UserRegistration): Promise<User> {
    console.log('🔵 [Tauri API] Registration attempt for:', userData.username)

    if (isTauri()) {
      return await invoke('register', { registration: userData })
    } else {
      console.log('🔵 [Tauri API] Running in browser, using mock registration')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 500))

      // Check if user already exists
      if (mockUsers.find(u => u.username === userData.username)) {
        throw new Error('Username already exists')
      }

      // Create new user
      const newUser: User = {
        id: mockUsers.length + 1,
        username: userData.username,
        email: userData.email,
        created_at: new Date().toISOString(),
        last_login: new Date().toISOString()
      }
      mockUsers.push(newUser)
      console.log('🔵 [Tauri API] Mock registration successful:', newUser)
      return newUser
    }
  },

  async logout(): Promise<void> {
    if (isTauri()) {
      return await invoke('logout')
    } else {
      console.log('🔵 [Tauri API] Running in browser, mock logout')
      // Mock logout - just a no-op in browser
    }
  },

  async getCurrentUser(userId: number): Promise<User | null> {
    if (isTauri()) {
      return await invoke('get_current_user', { userId })
    } else {
      console.log('🔵 [Tauri API] Running in browser, returning mock user')
      return mockUsers.find(u => u.id === userId) || null
    }
  }
}

// Mock lottery data for browser development
const mockLotteryDraws: LotteryDraw[] = [
  {
    id: 1,
    draw_date: new Date('2024-01-15').toISOString(),
    winning_numbers: [5, 12, 23, 34, 45, 56],
    bonus_number: 7,
    jackpot_amount: 1000000,
    lottery_type: 'powerball'
  },
  {
    id: 2,
    draw_date: new Date('2024-01-08').toISOString(),
    winning_numbers: [3, 11, 22, 33, 44, 55],
    bonus_number: 9,
    jackpot_amount: 800000,
    lottery_type: 'powerball'
  },
  {
    id: 3,
    draw_date: new Date('2024-01-01').toISOString(),
    winning_numbers: [7, 14, 21, 28, 35, 42],
    bonus_number: 14,
    jackpot_amount: 1200000,
    lottery_type: 'megamillions'
  }
]

// Lottery API
export const lotteryApi = {
  async getLotteryHistory(
    lotteryType?: string,
    limit?: number,
    offset?: number
  ): Promise<LotteryDraw[]> {
    if (isTauri()) {
      return await invoke('get_lottery_history', {
        lotteryType,
        limit,
        offset
      })
    } else {
      console.log('🔵 [Tauri API] Running in browser, returning mock lottery data')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 300))

      let filteredDraws = mockLotteryDraws

      // Apply filters
      if (lotteryType) {
        filteredDraws = filteredDraws.filter(draw => draw.lottery_type === lotteryType)
      }

      // Apply pagination
      if (offset) {
        filteredDraws = filteredDraws.slice(offset)
      }

      if (limit) {
        filteredDraws = filteredDraws.slice(0, limit)
      }

      return filteredDraws
    }
  },

  async addLotteryDraw(draw: NewLotteryDraw): Promise<LotteryDraw> {
    if (isTauri()) {
      return await invoke('add_lottery_draw', { draw })
    } else {
      console.log('🔵 [Tauri API] Running in browser, mock add lottery draw')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 300))

      const newDraw: LotteryDraw = {
        id: mockLotteryDraws.length + 1,
        ...draw
      }
      mockLotteryDraws.push(newDraw)
      return newDraw
    }
  },

  async searchLotteryDraws(query: LotterySearchQuery): Promise<LotteryDraw[]> {
    if (isTauri()) {
      return await invoke('search_lottery_draws', { query })
    } else {
      console.log('🔵 [Tauri API] Running in browser, mock search lottery draws')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 300))

      let filteredDraws = mockLotteryDraws

      if (query.lottery_type) {
        filteredDraws = filteredDraws.filter(draw => draw.lottery_type === query.lottery_type)
      }

      if (query.number_filter && query.number_filter.length > 0) {
        filteredDraws = filteredDraws.filter(draw =>
          query.number_filter!.some(num => draw.winning_numbers.includes(num))
        )
      }

      return filteredDraws
    }
  }
}

// Analysis API
export const analysisApi = {
  async getHotNumbers(request: AnalysisRequest): Promise<HotNumbersResponse> {
    if (isTauri()) {
      return await invoke('get_hot_numbers', { request })
    } else {
      console.log('🔵 [Tauri API] Running in browser, returning mock hot numbers')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 400))

      // Mock hot numbers analysis
      return {
        numbers: [
          { number: 23, frequency: 15, last_drawn: '2024-01-15T00:00:00Z', hot_score: 9.5, cold_score: 2.1 },
          { number: 34, frequency: 12, last_drawn: '2024-01-08T00:00:00Z', hot_score: 8.7, cold_score: 3.2 },
          { number: 45, frequency: 11, last_drawn: '2024-01-15T00:00:00Z', hot_score: 8.3, cold_score: 3.8 }
        ],
        analysis_period: 'Last 30 days',
        total_draws_analyzed: 100
      }
    }
  },

  async getColdNumbers(request: AnalysisRequest): Promise<ColdNumbersResponse> {
    if (isTauri()) {
      return await invoke('get_cold_numbers', { request })
    } else {
      console.log('🔵 [Tauri API] Running in browser, returning mock cold numbers')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 400))

      // Mock cold numbers analysis
      return {
        numbers: [
          { number: 2, frequency: 3, last_drawn: '2023-12-01T00:00:00Z', hot_score: 1.2, cold_score: 8.9 },
          { number: 7, frequency: 4, last_drawn: '2023-11-15T00:00:00Z', hot_score: 2.1, cold_score: 7.6 },
          { number: 19, frequency: 5, last_drawn: '2023-12-10T00:00:00Z', hot_score: 3.0, cold_score: 6.8 }
        ],
        analysis_period: 'Last 30 days',
        total_draws_analyzed: 100
      }
    }
  },

  async getNumberStatistics(
    number: number,
    lotteryType: string
  ): Promise<NumberStatistics> {
    if (isTauri()) {
      return await invoke('get_number_statistics', {
        number,
        lotteryType
      })
    } else {
      console.log('🔵 [Tauri API] Running in browser, returning mock number statistics')
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 300))

      // Mock number statistics
      return {
        number,
        total_draws: 100,
        frequency: 0.15,
        average_gap: 6.7,
        current_gap: 4,
        longest_gap: 15,
        shortest_gap: 1
      }
    }
  }
}