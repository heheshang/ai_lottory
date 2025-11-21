<template>
  <div class="sum-range-analysis">
    <div class="chart-container">
      <div class="chart-header">
        <h3>和值范围分析</h3>
        <div class="chart-controls">
          <label>显示模式:</label>
          <select v-model="displayMode" @change="updateChart">
            <option value="histogram">直方图</option>
            <option value="trend">趋势图</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="chart-loading">
        <div class="loading-spinner"></div>
        <p>分析和值范围中...</p>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <div class="empty-icon">📊</div>
        <p>暂无和值范围数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Summary Statistics -->
        <div class="summary-cards">
          <div class="summary-card">
            <h4>平均和值</h4>
            <div class="average-sum">{{ averageSum.toFixed(0) }}</div>
            <div class="range">{{ minSum }} - {{ maxSum }}</div>
          </div>
          <div class="summary-card">
            <h4>最常见范围</h4>
            <div class="common-range">{{ mostCommonRange }}</div>
            <div class="frequency">出现 {{ mostCommonRangeCount }} 次</div>
          </div>
          <div class="summary-card">
            <h4>和值分布</h4>
            <div class="distribution-type">{{ distributionType }}</div>
            <div class="std-dev">标准差: {{ standardDeviation.toFixed(1) }}</div>
          </div>
        </div>

        <!-- Histogram View -->
        <div v-if="displayMode === 'histogram'" class="histogram-view">
          <h4>和值分布直方图</h4>
          <div class="histogram-chart">
            <div class="chart-container-histogram">
              <div
                v-for="(range, index) in sumRanges"
                :key="range.label"
                class="histogram-bar"
              >
                <div class="bar-label">{{ range.label }}</div>
                <div class="bar-container">
                  <div
                    class="bar-fill"
                    :style="{ width: getBarWidth(range.count) + '%' }"
                  ></div>
                </div>
                <div class="bar-value">{{ range.count }}</div>
              </div>
            </div>
            <div class="range-indicators">
              <div class="indicator hot">热门</div>
              <div class="indicator cold">冷门</div>
              <div class="indicator normal">正常</div>
            </div>
          </div>
        </div>

        <!-- Trend View -->
        <div v-else class="trend-view">
          <h4>和值趋势图</h4>
          <div class="trend-chart">
            <div class="trend-line">
              <svg width="100%" height="200" viewBox="0 0 800 200">
                <polyline
                  :points="trendLinePoints"
                  fill="none"
                  stroke="#3498db"
                  stroke-width="2"
                />
                <circle
                  v-for="(point, index) in trendPoints"
                  :key="index"
                  :cx="point.x"
                  :cy="point.y"
                  r="3"
                  fill="#3498db"
                />
              </svg>
            </div>
            <div class="trend-stats">
              <div class="trend-stat">
                <span class="label">最近趋势:</span>
                <span class="value" :class="recentTrend.class">
                  {{ recentTrend.text }}
                </span>
              </div>
              <div class="trend-stat">
                <span class="label">波动性:</span>
                <span class="value">{{ volatility.toFixed(1) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Range Analysis -->
        <div class="range-analysis">
          <h4>区间分析</h4>
          <div class="ranges-grid">
            <div
              v-for="(range, index) in rangeAnalysis"
              :key="range.name"
              class="range-card"
              :class="{ active: range.isActive }"
            >
              <div class="range-header">
                <span class="range-name">{{ range.name }}</span>
                <span class="range-percentage">{{ (range.percentage * 100).toFixed(1) }}%</span>
              </div>
              <div class="range-details">
                <div class="range-count">出现 {{ range.count }} 次</div>
                <div class="range-interval">{{ range.min }} - {{ range.max }}</div>
              </div>
              <div class="range-status" :class="range.status">
                {{ getStatusText(range.status) }}
              </div>
            </div>
          </div>
        </div>

        <!-- Prediction -->
        <div class="prediction-section">
          <h4>下期预测</h4>
          <div class="prediction-cards">
            <div class="prediction-card recommended">
              <div class="prediction-header">
                <span class="prediction-icon">🎯</span>
                <h5>推荐范围</h5>
              </div>
              <div class="prediction-content">
                <div class="predicted-range">{{ recommendedRange }}</div>
                <div class="confidence">置信度: {{ recommendedConfidence }}%</div>
              </div>
            </div>
            <div class="prediction-card alternative">
              <div class="prediction-header">
                <span class="prediction-icon">📈</span>
                <h5>备选方案</h5>
              </div>
              <div class="prediction-content">
                <div class="predicted-range">{{ alternativeRange }}</div>
                <div class="confidence">置信度: {{ alternativeConfidence }}%</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'

interface SumRange {
  label: string
  min: number
  max: number
  count: number
  percentage: number
}

interface TrendPoint {
  x: number
  y: number
  value: number
}

interface RangeAnalysis {
  name: string
  min: number
  max: number
  count: number
  percentage: number
  status: 'hot' | 'normal' | 'cold'
  isActive: boolean
}

interface Props {
  ranges?: SumRange[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// State
const displayMode = ref('histogram')

// Mock data for demonstration
const mockSumRanges: SumRange[] = [
  { label: '50-70', min: 50, max: 70, count: 8, percentage: 0.027 },
  { label: '70-90', min: 70, max: 90, count: 23, percentage: 0.078 },
  { label: '90-110', min: 90, max: 110, count: 45, percentage: 0.153 },
  { label: '110-130', min: 110, max: 130, count: 68, percentage: 0.231 },
  { label: '130-150', min: 130, max: 150, count: 72, percentage: 0.244 },
  { label: '150-170', min: 150, max: 170, count: 48, percentage: 0.163 },
  { label: '170-190', min: 170, max: 190, count: 25, percentage: 0.085 },
  { label: '190-210', min: 190, max: 210, count: 6, percentage: 0.020 }
]

const mockRangeAnalysis: RangeAnalysis[] = [
  { name: '小和值', min: 50, max: 100, count: 35, percentage: 0.119, status: 'cold', isActive: false },
  { name: '中和值', min: 100, max: 140, count: 125, percentage: 0.424, status: 'hot', isActive: true },
  { name: '大和值', min: 140, max: 200, count: 65, percentage: 0.220, status: 'normal', isActive: false },
  { name: '超大和值', min: 200, max: 250, count: 8, percentage: 0.027, status: 'cold', isActive: false }
]

// Computed properties
const sumRanges = computed(() => {
  return props.ranges?.length > 0 ? props.ranges : mockSumRanges
})

const hasData = computed(() => {
  return sumRanges.value.length > 0
})

const totalDraws = computed(() => {
  return sumRanges.value.reduce((sum, range) => sum + range.count, 0)
})

const averageSum = computed(() => {
  const weightedSum = sumRanges.value.reduce((sum, range) => {
    const rangeMid = (range.min + range.max) / 2
    return sum + (rangeMid * range.count)
  }, 0)
  return totalDraws.value > 0 ? weightedSum / totalDraws.value : 0
})

const minSum = computed(() => {
  return Math.min(...sumRanges.value.map(range => range.min))
})

const maxSum = computed(() => {
  return Math.max(...sumRanges.value.map(range => range.max))
})

const mostCommonRange = computed(() => {
  const sorted = [...sumRanges.value].sort((a, b) => b.count - a.count)
  return sorted[0]?.label || '未知'
})

const mostCommonRangeCount = computed(() => {
  const sorted = [...sumRanges.value].sort((a, b) => b.count - a.count)
  return sorted[0]?.count || 0
})

const standardDeviation = computed(() => {
  // Mock standard deviation calculation
  return 28.5
})

const distributionType = computed(() => {
  // Mock distribution type analysis
  if (standardDeviation.value < 20) return '集中分布'
  if (standardDeviation.value < 30) return '正态分布'
  return '分散分布'
})

const rangeAnalysis = computed(() => mockRangeAnalysis)

const trendPoints = computed((): TrendPoint[] => {
  // Mock trend points for visualization
  const data = [118, 125, 142, 98, 135, 156, 102, 128, 145, 115, 132, 98]
  return data.map((value, index) => ({
    x: (index / (data.length - 1)) * 750 + 25,
    y: 200 - ((value - 50) / 200) * 180,
    value
  }))
})

const trendLinePoints = computed(() => {
  return trendPoints.value.map(point => `${point.x},${point.y}`).join(' ')
})

const recentTrend = computed(() => {
  const recentValues = trendPoints.value.slice(-5).map(p => p.value)
  const averageRecent = recentValues.reduce((a, b) => a + b, 0) / recentValues.length
  const overallAverage = averageSum.value

  if (averageRecent > overallAverage + 10) {
    return { text: '上升趋势 ↗', class: 'up' }
  } else if (averageRecent < overallAverage - 10) {
    return { text: '下降趋势 ↘', class: 'down' }
  } else {
    return { text: '稳定保持 →', class: 'stable' }
  }
})

const volatility = computed(() => {
  const values = trendPoints.value.map(p => p.value)
  const avg = values.reduce((a, b) => a + b, 0) / values.length
  const variance = values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) / values.length
  return Math.sqrt(variance)
})

const recommendedRange = computed(() => {
  // Mock recommendation based on analysis
  return '115-145'
})

const recommendedConfidence = computed(() => {
  return 78
})

const alternativeRange = computed(() => {
  // Mock alternative recommendation
  return '125-155'
})

const alternativeConfidence = computed(() => {
  return 65
})

// Methods
const getBarWidth = (count: number): number => {
  const maxCount = Math.max(...sumRanges.value.map(range => range.count), 1)
  return (count / maxCount) * 100
}

const getStatusText = (status: string): string => {
  const statusMap: Record<string, string> = {
    hot: '热门',
    normal: '正常',
    cold: '冷门'
  }
  return statusMap[status] || status
}

const updateChart = () => {
  // Update chart based on display mode
  console.log('Updating chart with mode:', displayMode.value)
}

// Lifecycle
onMounted(() => {
  updateChart()
})

watch(() => displayMode, () => {
  updateChart()
})
</script>

<style scoped>
.sum-range-analysis {
  width: 100%;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.chart-container {
  height: 100%;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #ecf0f1;
}

.chart-header h3 {
  color: #2c3e50;
  margin: 0;
  font-size: 1.2rem;
}

.chart-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.chart-controls label {
  color: #7f8c8d;
  font-size: 0.9rem;
  font-weight: 500;
}

.chart-controls select {
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 0.9rem;
}

.chart-loading,
.chart-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  color: #7f8c8d;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 15px;
}

.summary-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.summary-card {
  background: linear-gradient(135deg, #9b59b6, #8e44ad);
  color: white;
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.summary-card h4 {
  margin-bottom: 10px;
  font-size: 0.9rem;
  opacity: 0.9;
}

.average-sum {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.common-range {
  font-size: 1.5rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.distribution-type {
  font-size: 1.3rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.range,
.frequency,
.std-dev,
.detail {
  font-size: 0.8rem;
  opacity: 0.8;
}

.histogram-view,
.trend-view,
.range-analysis,
.prediction-section {
  margin-bottom: 30px;
}

.histogram-view h4,
.trend-view h4,
.range-analysis h4,
.prediction-section h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.histogram-chart {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
}

.chart-container-histogram {
  margin-bottom: 15px;
}

.histogram-bar {
  display: grid;
  grid-template-columns: 80px 1fr 50px;
  align-items: center;
  gap: 15px;
  margin-bottom: 12px;
}

.bar-label {
  font-size: 0.9rem;
  color: #2c3e50;
  font-weight: 500;
  text-align: right;
}

.bar-container {
  height: 25px;
  background: #ecf0f1;
  border-radius: 12px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(to right, #9b59b6, #8e44ad);
  border-radius: 12px;
  transition: width 0.5s ease;
}

.bar-value {
  font-size: 0.8rem;
  color: #2c3e50;
  font-weight: 600;
  text-align: left;
}

.range-indicators {
  display: flex;
  justify-content: center;
  gap: 20px;
  font-size: 0.8rem;
}

.indicator {
  display: flex;
  align-items: center;
  gap: 5px;
}

.indicator.hot::before {
  content: '●';
  color: #e74c3c;
}

.indicator.cold::before {
  content: '●';
  color: #3498db;
}

.indicator.normal::before {
  content: '●';
  color: #27ae60;
}

.trend-chart {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
}

.trend-line {
  margin-bottom: 15px;
}

.trend-stats {
  display: flex;
  justify-content: space-around;
}

.trend-stat {
  text-align: center;
}

.trend-stat .label {
  display: block;
  font-size: 0.8rem;
  color: #7f8c8d;
  margin-bottom: 5px;
}

.trend-stat .value {
  font-size: 1rem;
  font-weight: 600;
  color: #2c3e50;
}

.trend-stat .value.up {
  color: #27ae60;
}

.trend-stat .value.down {
  color: #e74c3c;
}

.trend-stat .value.stable {
  color: #f39c12;
}

.ranges-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.range-card {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  border: 2px solid #ecf0f1;
  transition: all 0.3s;
}

.range-card.active {
  border-color: #9b59b6;
  background: #faf5ff;
}

.range-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.range-name {
  font-weight: 600;
  color: #2c3e50;
}

.range-percentage {
  background: #9b59b6;
  color: white;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.7rem;
}

.range-details {
  margin-bottom: 10px;
}

.range-count {
  font-size: 0.8rem;
  color: #7f8c8d;
  margin-bottom: 3px;
}

.range-interval {
  font-size: 0.8rem;
  color: #2c3e50;
  font-weight: 500;
}

.range-status {
  font-size: 0.7rem;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 3px;
  text-align: center;
}

.range-status.hot {
  background: #fdeaea;
  color: #e74c3c;
}

.range-status.normal {
  background: #e8f5e8;
  color: #27ae60;
}

.range-status.cold {
  background: #eafaf1;
  color: #3498db;
}

.prediction-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.prediction-card {
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.prediction-card.recommended {
  background: linear-gradient(135deg, #27ae60, #229954);
  color: white;
}

.prediction-card.alternative {
  background: linear-gradient(135deg, #3498db, #2980b9);
  color: white;
}

.prediction-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-bottom: 15px;
}

.prediction-icon {
  font-size: 1.5rem;
}

.prediction-header h5 {
  margin: 0;
  font-size: 1rem;
}

.predicted-range {
  font-size: 1.8rem;
  font-weight: bold;
  margin-bottom: 10px;
}

.confidence {
  font-size: 0.9rem;
  opacity: 0.9;
}

/* Responsive design */
@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .summary-cards {
    grid-template-columns: 1fr;
  }

  .histogram-bar {
    grid-template-columns: 70px 1fr 40px;
    gap: 10px;
  }

  .ranges-grid {
    grid-template-columns: 1fr;
  }

  .prediction-cards {
    grid-template-columns: 1fr;
  }

  .trend-stats {
    flex-direction: column;
    gap: 10px;
  }
}

@media (max-width: 480px) {
  .average-sum {
    font-size: 1.5rem;
  }

  .common-range {
    font-size: 1.3rem;
  }

  .distribution-type {
    font-size: 1.1rem;
  }

  .range-indicators {
    flex-direction: column;
    gap: 10px;
  }

  .predicted-range {
    font-size: 1.5rem;
  }
}
</style>