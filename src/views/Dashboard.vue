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
          <el-col :span="8">
            <el-card class="nav-card" @click="goToHistory">
              <div class="nav-content">
                <el-icon size="48" color="#409EFF"><Document /></el-icon>
                <h3>Lottery History</h3>
                <p>View past winning numbers and results</p>
              </div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card class="nav-card" @click="goToHotNumbers">
              <div class="nav-content">
                <el-icon size="48" color="#F56C6C"><TrendCharts /></el-icon>
                <h3>Hot Numbers</h3>
                <p>Discover frequently drawn numbers</p>
              </div>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card class="nav-card" @click="goToColdNumbers">
              <div class="nav-content">
                <el-icon size="48" color="#67C23A"><TrendCharts /></el-icon>
                <h3>Cold Numbers</h3>
                <p>Find numbers that are due for drawing</p>
              </div>
            </el-card>
          </el-col>

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
                <div
                  v-for="draw in recentDraws"
                  :key="draw.id"
                  class="draw-item"
                >
                  <div class="draw-date">
                    {{ formatDate(draw.draw_date) }}
                  </div>
                  <div class="draw-numbers">
                    <el-tag
                      v-for="number in draw.winning_numbers"
                      :key="number"
                      class="number-tag"
                    >
                      {{ number }}
                    </el-tag>
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
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Document, TrendCharts } from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { lotteryApi } from '@/api/tauri'
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

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString()
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