<template>
  <div class="dashboard-container">
    <el-container>
      <!-- Header -->
      <el-header class="dashboard-header">
        <div class="header-left">
          <h1>AI Lottery Prediction</h1>
        </div>
        <div class="header-right">
          <span class="welcome-text">Welcome, {{ authStore.user?.username }}!</span>
          <el-button type="primary" @click="handleLogout">Logout</el-button>
        </div>
      </el-header>

      <!-- Main Content -->
      <el-main>
        <el-row :gutter="20">
          <!-- Quick Stats -->
          <el-col :span="24">
            <el-card class="stats-card">
              <template #header>
                <h3>Quick Overview</h3>
              </template>
              <el-row :gutter="20">
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.totalDraws }}</div>
                    <div class="stat-label">Total Draws</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.hotNumbersCount }}</div>
                    <div class="stat-label">Hot Numbers</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.coldNumbersCount }}</div>
                    <div class="stat-label">Cold Numbers</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.avgJackpot }}</div>
                    <div class="stat-label">Avg Jackpot</div>
                  </div>
                </el-col>
              </el-row>
            </el-card>
          </el-col>

          <!-- Navigation Cards -->
          <template v-for="card in navigationCards" :key="card.id">
            <el-col :span="8">
              <el-card class="nav-card" @click="card.action">
                <div class="nav-content">
                  <div class="nav-icon" :style="{ color: card.color }" v-html="card.icon"></div>
                  <h3>{{ card.title }}</h3>
                  <p>{{ card.description }}</p>
                </div>
              </el-card>
            </el-col>
          </template>

          <!-- Recent Draws -->
          <el-col :span="24">
            <el-card class="recent-draws-card">
              <template #header>
                <div class="card-header">
                  <h3>Recent Draws</h3>
                  <el-button type="text" @click="goToHistory">View All</el-button>
                </div>
              </template>
              <div v-loading="loading" class="recent-draws">
                <template v-if="recentDraws.length === 0">
                  <div class="empty-draws">
                    <p>No recent draws available</p>
                  </div>
                </template>
                <template v-else>
                  <div
                    v-for="draw in recentDraws"
                    :key="draw.id"
                    class="draw-item"
                  >
                    <div class="draw-date">
                      {{ formatDate(draw.draw_date) }}
                    </div>
                    <div class="draw-numbers">
                      <template v-if="draw.winning_numbers && draw.winning_numbers.length > 0">
                        <el-tag
                          v-for="(number, index) in draw.winning_numbers"
                          :key="'num-' + index"
                          class="number-tag"
                        >
                          {{ number }}
                        </el-tag>
                      </template>
                      <el-tag
                        v-if="draw.bonus_number"
                        type="warning"
                        class="bonus-tag"
                      >
                        {{ draw.bonus_number }}
                      </el-tag>
                    </div>
                    <div class="draw-type">
                      {{ formatLotteryType(draw.lottery_type) }}
                    </div>
                  </div>
                </template>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { lotteryApi } from '@/api/tauri'
import { formatDate } from '@/utils/formatters'
import type { LotteryDraw } from '@/types'

const router = useRouter()
const authStore = useAuthStore()

const loading = ref(false)
const recentDraws = ref<LotteryDraw[]>([])

const stats = reactive({
  totalDraws: 0,
  hotNumbersCount: 0,
  coldNumbersCount: 0,
  avgJackpot: '$0'
})

interface NavigationCard {
  id: string
  title: string
  description: string
  color: string
  icon: string
  action: () => void
}

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

const loadRecentDraws = async () => {
  try {
    console.log('🔵 [Dashboard] Loading recent draws...')
    loading.value = true
    const draws = await lotteryApi.getLotteryHistory(undefined, 10, 0)
    console.log('🔵 [Dashboard] Loaded draws:', draws.length)
    recentDraws.value = draws
    stats.totalDraws = draws.length

    // Calculate average jackpot
    const jackpots = draws.filter(d => d.jackpot_amount).map(d => d.jackpot_amount!)
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

const formatLotteryType = (type: string) => {
  return type.charAt(0).toUpperCase() + type.slice(1).replace(/([A-Z])/g, ' $1')
}

onMounted(() => {
  console.log('🟢 [Dashboard] Component mounted, loading recent draws...')
  loadRecentDraws()
})
</script>

<style scoped>
.dashboard-container {
  min-height: 100vh;
  background-color: #f5f5f5;
}

.dashboard-header {
  background: white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
}

.header-left h1 {
  margin: 0;
  color: #303133;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 20px;
}

.welcome-text {
  color: #606266;
  font-size: 16px;
}

.stats-card {
  margin-bottom: 20px;
}

.stat-item {
  text-align: center;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #409EFF;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 14px;
  color: #606266;
}

.nav-card {
  cursor: pointer;
  transition: all 0.3s ease;
  margin-bottom: 20px;
}

.nav-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.nav-content {
  text-align: center;
  padding: 20px;
}

.nav-icon {
  width: 48px;
  height: 48px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-content h3 {
  margin: 15px 0 10px 0;
  color: #303133;
}

.nav-content p {
  margin: 0;
  color: #606266;
  font-size: 14px;
}

.recent-draws-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
}

.recent-draws {
  max-height: 400px;
  overflow-y: auto;
}

.empty-draws {
  text-align: center;
  padding: 40px 20px;
  color: #909399;
}

.empty-draws p {
  margin: 0;
  font-size: 14px;
}

.draw-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 15px 0;
  border-bottom: 1px solid #EBEEF5;
}

.draw-item:last-child {
  border-bottom: none;
}

.draw-date {
  font-weight: 500;
  color: #606266;
  min-width: 100px;
}

.draw-numbers {
  display: flex;
  gap: 8px;
  flex: 1;
  justify-content: center;
}

.number-tag, .bonus-tag {
  font-weight: bold;
}

.bonus-tag {
  border-style: dashed;
}

.draw-type {
  font-size: 12px;
  color: #909399;
  min-width: 100px;
  text-align: right;
}

.el-main {
  padding: 20px;
}
</style>