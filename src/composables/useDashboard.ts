/**
 * Dashboard composable - handles dashboard business logic
 */

import { ref, onMounted, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { lotteryApi } from '@/api/tauri'
import { formatDate } from '@/utils/formatters'
import type { LotteryDraw } from '@/types'

export interface NavigationCard {
  id: string
  title: string
  description: string
  color: string
  icon: string
  action: () => void
}

export interface DashboardStats {
  totalDraws: number
  hotNumbersCount: number
  coldNumbersCount: number
  avgJackpot: string
}

export function useDashboard() {
  const router = useRouter()
  const authStore = useAuthStore()

  // State
  const loading = ref(false)
  const recentDraws = ref<LotteryDraw[]>([])

  const stats = reactive<DashboardStats>({
    totalDraws: 0,
    hotNumbersCount: 0,
    coldNumbersCount: 0,
    avgJackpot: '$0'
  })

  // Navigation functions
  const goToHistory = () => {
    router.push('/history')
  }

  const goToHotNumbers = () => {
    router.push('/hot-numbers')
  }

  const goToColdNumbers = () => {
    router.push('/cold-numbers')
  }

  const goToPrediction = () => {
    router.push('/super-lotto/prediction')
  }

  const goToOneClickPrediction = () => {
    router.push('/super-lotto/one-click-prediction')
  }

  // Navigation cards configuration
  const navigationCards = computed<NavigationCard[]>(() => [
    {
      id: 'history',
      title: 'Lottery History',
      description: 'View past winning numbers and results',
      color: '#409EFF',
      icon: '<svg viewBox="0 0 1024 1024" width="48" height="48"><path fill="currentColor" d="M832 384H576V128H192v768h640V384zm-26.496-64L640 154.496V320h165.504zM160 64h480l256 256v608a32 32 0 0 1-32 32H160a32 32 0 0 1-32-32V96a32 32 0 0 1 32-32zm160 448h384v64H320v-64zm0-192h160v64H320v-64zm0 384h384v64H320v-64z"></path></svg>',
      action: goToHistory
    },
    {
      id: 'hot-numbers',
      title: 'Hot Numbers',
      description: 'Discover frequently drawn numbers',
      color: '#F56C6C',
      icon: '<svg viewBox="0 0 1024 1024" width="48" height="48"><path fill="currentColor" d="M128 896V128h768v768H128zm291.712-327.296l128 102.4 180.16-201.792-47.744-42.624-139.84 156.608-128-102.4-180.16 201.792 47.744 42.624 139.84-156.608z"></path></svg>',
      action: goToHotNumbers
    },
    {
      id: 'cold-numbers',
      title: 'Cold Numbers',
      description: 'Find numbers that are due for drawing',
      color: '#67C23A',
      icon: '<svg viewBox="0 0 1024 1024" width="48" height="48"><path fill="currentColor" d="M128 896V128h768v768H128zm291.712-327.296l128 102.4 180.16-201.792-47.744-42.624-139.84 156.608-128-102.4-180.16 201.792 47.744 42.624 139.84-156.608z"></path></svg>',
      action: goToColdNumbers
    },
    {
      id: 'prediction',
      title: 'AI Prediction',
      description: 'Generate lottery number predictions',
      color: '#E6A23C',
      icon: '<svg viewBox="0 0 1024 1024" width="48" height="48"><path fill="currentColor" d="M512 64 128 192v384c0 212.064 114.624 407.424 288 511.488C688.384 983.424 896 788.064 896 576V192L512 64zm0 64l320 106.688V576c0 188.16-101.504 362.048-256 456.832C421.504 938.048 256 764.16 256 576V234.688L512 128z"></path><path fill="currentColor" d="M480 416h64v192h-64zm0-128h64v64h-64z"></path></svg>',
      action: goToPrediction
    },
    {
      id: 'one-click',
      title: 'One-Click Prediction',
      description: 'Generate all algorithm predictions instantly',
      color: '#67C23A',
      icon: '<svg viewBox="0 0 1024 1024" width="48" height="48"><path fill="currentColor" d="M679.872 348.8l-301.76 188.608a127.808 127.808 0 0 1 5.12 52.16l279.936 104.96a128 128 0 1 1-22.464 59.904l-279.872-104.96a128 128 0 1 1-16.64-166.272l301.696-188.608a128 128 0 1 1 33.92 54.272z"></path></svg>',
      action: goToOneClickPrediction
    }
  ])

  // Methods
  const loadRecentDraws = async () => {
    try {
      console.log('🔵 [Dashboard] Loading recent draws...')
      loading.value = true
      const draws = await lotteryApi.getLotteryHistory(undefined, 10, 0)
      console.log('🔵 [Dashboard] Loaded draws:', draws.length)
      recentDraws.value = draws
      stats.totalDraws = draws.length

      // Calculate average jackpot
      const jackpots = draws
        .filter(d => d.jackpot_amount)
        .map(d => d.jackpot_amount!)

      if (jackpots.length > 0) {
        const avg = jackpots.reduce((a, b) => a + b, 0) / jackpots.length
        stats.avgJackpot = `$${avg.toFixed(0)}M`
      }
      console.log('🔵 [Dashboard] Recent draws loaded successfully')
    } catch (error) {
      console.error('🔴 [Dashboard] Failed to load recent draws:', error)
      ElMessage.error('Failed to load recent draws')
    } finally {
      loading.value = false
    }
  }

  const handleLogout = async () => {
    await authStore.logout()
    ElMessage.success('Logged out successfully')
    router.push('/login')
  }

  const formatLotteryType = (type: string) => {
    return type.charAt(0).toUpperCase() + type.slice(1).replace(/([A-Z])/g, ' $1')
  }

  // Lifecycle
  onMounted(() => {
    console.log('🟢 [Dashboard] Component mounted, loading recent draws...')
    loadRecentDraws()
  })

  return {
    // State
    loading,
    recentDraws,
    stats,
    navigationCards,

    // Methods
    handleLogout,
    formatLotteryType,
    formatDate,
    loadRecentDraws
  }
}