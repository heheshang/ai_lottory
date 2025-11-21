<template>
  <div class="hot-numbers-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <h2>Hot Numbers Analysis</h2>
          <el-button type="primary" @click="goToDashboard">Back to Dashboard</el-button>
        </div>
      </template>

      <div class="analysis-controls">
        <el-row :gutter="20">
          <el-col :span="6">
            <el-select v-model="selectedLotteryType" placeholder="Select Lottery Type">
              <el-option label="Powerball" value="powerball" />
              <el-option label="Mega Millions" value="megamillions" />
              <el-option label="Lotto" value="lotto" />
            </el-select>
          </el-col>
          <el-col :span="6">
            <el-select v-model="analysisPeriod" placeholder="Analysis Period">
              <el-option label="Last 30 days" :value="30" />
              <el-option label="Last 60 days" :value="60" />
              <el-option label="Last 90 days" :value="90" />
              <el-option label="Last 100 draws" :value="100" />
              <el-option label="Last 200 draws" :value="200" />
            </el-select>
          </el-col>
          <el-col :span="6">
            <el-button type="primary" @click="analyzeHotNumbers">Analyze</el-button>
          </el-col>
        </el-row>
      </div>

      <div v-loading="loading" class="analysis-results">
        <div v-if="hotNumbersData" class="results-section">
          <el-alert
            :title="`Analysis Complete: ${hotNumbersData.analysis_period} - ${hotNumbersData.total_draws_analyzed} draws analyzed`"
            type="success"
            :closable="false"
            class="analysis-summary"
          />

          <div class="hot-numbers-grid">
            <el-row :gutter="20">
              <el-col
                v-for="item in hotNumbersData.numbers.slice(0, 12)"
                :key="item.number"
                :span="6"
              >
                <div class="hot-number-card">
                  <div class="number-display">{{ item.number }}</div>
                  <div class="frequency">{{ item.frequency }} times</div>
                  <div class="hot-score">
                    <el-progress
                      :percentage="item.hot_score * 100"
                      :color="getHotScoreColor(item.hot_score)"
                      :show-text="false"
                      :stroke-width="8"
                    />
                    <span class="score-text">{{ (item.hot_score * 100).toFixed(1) }}%</span>
                  </div>
                  <div v-if="item.last_drawn" class="last-drawn">
                    Last: {{ formatDate(item.last_drawn) }}
                  </div>
                </div>
              </el-col>
            </el-row>
          </div>

          <div class="chart-section">
            <h3>Hot Numbers Distribution</h3>
            <div ref="chartContainer" class="chart-container"></div>
          </div>
        </div>

        <el-empty v-else description="Select lottery type and period to analyze hot numbers" />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { analysisApi } from '@/api/tauri'
import type { HotNumbersResponse } from '@/types'

const router = useRouter()

const loading = ref(false)
const selectedLotteryType = ref('powerball')
const analysisPeriod = ref(30)
const hotNumbersData = ref<HotNumbersResponse | null>(null)
const chartContainer = ref<HTMLElement>()

const analyzeHotNumbers = async () => {
  if (!selectedLotteryType.value) {
    ElMessage.warning('Please select a lottery type')
    return
  }

  try {
    loading.value = true

    const request = {
      lottery_type: selectedLotteryType.value,
      days: analysisPeriod.value <= 90 ? analysisPeriod.value : undefined,
      draw_count: analysisPeriod.value > 90 ? analysisPeriod.value : undefined
    }

    const data = await analysisApi.getHotNumbers(request)
    hotNumbersData.value = data

    // Draw chart after data is loaded
    await nextTick()
    drawChart()
  } catch (error) {
    console.error('Failed to analyze hot numbers:', error)
    ElMessage.error('Failed to analyze hot numbers')
  } finally {
    loading.value = false
  }
}

const getHotScoreColor = (score: number) => {
  if (score >= 0.8) return '#f56c6c'
  if (score >= 0.6) return '#e6a23c'
  if (score >= 0.4) return '#409eff'
  return '#67c23a'
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString()
}

const goToDashboard = () => {
  router.push('/dashboard')
}

const drawChart = () => {
  if (!chartContainer.value || !hotNumbersData.value) return

  // Simple chart visualization using divs (in real app, you'd use ECharts or similar)
  const container = chartContainer.value
  container.innerHTML = ''

  const chartDiv = document.createElement('div')
  chartDiv.className = 'simple-chart'

  const data = hotNumbersData.value.numbers.slice(0, 10)
  const maxFreq = Math.max(...data.map(item => item.frequency))

  data.forEach(item => {
    const barContainer = document.createElement('div')
    barContainer.className = 'chart-bar-container'

    const bar = document.createElement('div')
    bar.className = 'chart-bar'
    bar.style.height = `${(item.frequency / maxFreq) * 100}%`
    bar.style.backgroundColor = getHotScoreColor(item.hot_score)

    const label = document.createElement('div')
    label.className = 'chart-label'
    label.textContent = `${item.number} (${item.frequency})`

    barContainer.appendChild(bar)
    barContainer.appendChild(label)
    chartDiv.appendChild(barContainer)
  })

  container.appendChild(chartDiv)
}

onMounted(() => {
  // Auto-analyze on mount with default values
  analyzeHotNumbers()
})
</script>

<style scoped>
.hot-numbers-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
}

.analysis-controls {
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 4px;
}

.analysis-results {
  margin-top: 20px;
}

.analysis-summary {
  margin-bottom: 20px;
}

.hot-numbers-grid {
  margin-bottom: 30px;
}

.hot-number-card {
  background: white;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  padding: 20px;
  text-align: center;
  transition: all 0.3s ease;
  cursor: pointer;
}

.hot-number-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.number-display {
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 8px;
}

.frequency {
  font-size: 14px;
  color: #606266;
  margin-bottom: 12px;
}

.hot-score {
  margin-bottom: 12px;
}

.score-text {
  font-size: 12px;
  color: #909399;
  margin-left: 8px;
}

.last-drawn {
  font-size: 12px;
  color: #909399;
}

.chart-section {
  margin-top: 30px;
}

.chart-section h3 {
  margin-bottom: 20px;
  color: #303133;
}

.chart-container {
  height: 300px;
  background: #fafafa;
  border-radius: 4px;
  padding: 20px;
}

.simple-chart {
  display: flex;
  align-items: end;
  justify-content: space-around;
  height: 100%;
  padding: 20px 0;
}

.chart-bar-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  max-width: 60px;
  margin: 0 5px;
}

.chart-bar {
  width: 100%;
  background: linear-gradient(to top, #409eff, #66b1ff);
  border-radius: 4px 4px 0 0;
  transition: height 0.5s ease;
}

.chart-label {
  margin-top: 8px;
  font-size: 12px;
  text-align: center;
  color: #606266;
}
</style>