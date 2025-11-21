<template>
  <div class="odd-even-distribution-chart">
    <div class="chart-container">
      <div class="chart-header">
        <h3>奇偶分布分析</h3>
        <div class="chart-controls">
          <label>分析区域:</label>
          <select v-model="selectedZone" @change="updateChart">
            <option value="front">前区</option>
            <option value="back">后区</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="chart-loading">
        <div class="loading-spinner"></div>
        <p>分析奇偶分布中...</p>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <div class="empty-icon">📊</div>
        <p>暂无奇偶分布数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Distribution Overview -->
        <div class="distribution-overview">
          <div class="overview-cards">
            <div class="overview-card">
              <h4>最常见比例</h4>
              <div class="most-common">
                {{ mostCommonRatio.ratio }}
              </div>
              <div class="frequency">出现 {{ mostCommonRatio.count }} 次</div>
            </div>
            <div class="overview-card">
              <h4>奇数频率</h4>
              <div class="odd-percentage">
                {{ oddPercentage.toFixed(1) }}%
              </div>
              <div class="detail">平均 {{ averageOddCount }} 个/期</div>
            </div>
            <div class="overview-card">
              <h4>偶数频率</h4>
              <div class="even-percentage">
                {{ evenPercentage.toFixed(1) }}%
              </div>
              <div class="detail">平均 {{ averageEvenCount }} 个/期</div>
            </div>
          </div>
        </div>

        <!-- Bar Chart -->
        <div class="ratio-distribution">
          <h4>奇偶比例分布</h4>
          <div class="chart-bars">
            <div
              v-for="(item, index) in ratioData"
              :key="item.ratio"
              class="bar-item"
            >
              <div class="bar-container">
                <div
                  class="bar-fill"
                  :style="{ width: getBarWidth(item.count) + '%' }"
                ></div>
              </div>
              <div class="bar-info">
                <div class="ratio-label">{{ item.ratio }}</div>
                <div class="ratio-count">{{ item.count }}次</div>
                <div class="ratio-percentage">{{ (item.percentage * 100).toFixed(1) }}%</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Trend Analysis -->
        <div class="trend-analysis">
          <h4>趋势分析</h4>
          <div class="trend-cards">
            <div class="trend-card">
              <h5>近期奇偶偏好</h5>
              <div class="trend-value" :class="trendDirection.class">
                {{ trendDirection.text }}
              </div>
              <div class="trend-detail">最近 {{ recentDraws }} 期</div>
            </div>
            <div class="trend-card">
              <h5>平衡度</h5>
              <div class="balance-indicator">
                <div class="balance-bar">
                  <div
                    class="balance-fill"
                    :style="{ width: (balanceScore * 100) + '%' }"
                  ></div>
                </div>
                <span class="balance-score">{{ (balanceScore * 100).toFixed(0) }}%</span>
              </div>
              <div class="trend-detail">奇偶平衡指数</div>
            </div>
          </div>
        </div>

        <!-- Pattern Examples -->
        <div class="pattern-examples">
          <h4>典型模式示例</h4>
          <div class="examples-grid">
            <div
              v-for="(example, index) in patternExamples"
              :key="`example-${index}`"
              class="example-card"
            >
              <div class="example-header">
                <span class="example-title">{{ example.title }}</span>
                <span class="example-frequency">{{ example.frequency }}%</span>
              </div>
              <div class="example-numbers">
                <div class="number-group">
                  <span class="group-label">奇数:</span>
                  <div class="numbers odd">
                    <span
                      v-for="number in example.odds"
                      :key="number"
                      class="number odd"
                    >
                      {{ number }}
                    </span>
                  </div>
                </div>
                <div class="number-group">
                  <span class="group-label">偶数:</span>
                  <div class="numbers even">
                    <span
                      v-for="number in example.evens"
                      :key="number"
                      class="number even"
                    >
                      {{ number }}
                    </span>
                  </div>
                </div>
              </div>
              <div class="example-description">
                {{ example.description }}
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

interface RatioData {
  ratio: string
  count: number
  percentage: number
}

interface PatternExample {
  title: string
  odds: number[]
  evens: number[]
  frequency: number
  description: string
}

interface Props {
  distribution?: RatioData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// State
const selectedZone = ref('front')

// Mock data for demonstration
const mockRatioData: RatioData[] = [
  { ratio: '3奇2偶', count: 98, percentage: 0.332 },
  { ratio: '2奇3偶', count: 87, percentage: 0.295 },
  { ratio: '4奇1偶', count: 52, percentage: 0.176 },
  { ratio: '1奇4偶', count: 35, percentage: 0.119 },
  { ratio: '5奇0偶', count: 15, percentage: 0.051 },
  { ratio: '0奇5偶', count: 8, percentage: 0.027 }
]

const mockPatternExamples: PatternExample[] = [
  {
    title: '均衡分布',
    odds: [12, 23, 35],
    evens: [8, 16],
    frequency: 33.2,
    description: '3奇2偶，最常见的分布模式'
  },
  {
    title: '奇数主导',
    odds: [7, 15, 21, 33],
    evens: [12],
    frequency: 17.6,
    description: '4奇1偶，奇数偏好明显'
  },
  {
    title: '偶数主导',
    odds: [17],
    evens: [4, 12, 20, 28],
    frequency: 11.9,
    description: '1奇4偶，偶数占主导'
  }
]

// Computed properties
const ratioData = computed(() => {
  return props.distribution?.length > 0 ? props.distribution : mockRatioData
})

const hasData = computed(() => {
  return ratioData.value.length > 0
})

const totalDraws = computed(() => {
  return ratioData.value.reduce((sum, item) => sum + item.count, 0)
})

const mostCommonRatio = computed(() => {
  const sorted = [...ratioData.value].sort((a, b) => b.count - a.count)
  return sorted[0] || { ratio: '-', count: 0 }
})

const oddCount = computed(() => {
  // Calculate total odd numbers across all draws
  const oddRatios = ratioData.value.filter(item => {
    const match = item.ratio.match(/(\d+)奇/)
    return match ? parseInt(match[1]) : 0
  })

  return oddRatios.reduce((sum, item) => {
    const oddMatch = item.ratio.match(/(\d+)奇/)
    const oddCount = oddMatch ? parseInt(oddMatch[1]) : 0
    return sum + (oddCount * item.count)
  }, 0)
})

const evenCount = computed(() => {
  // Calculate total even numbers across all draws
  return totalDraws.value * 5 - oddCount.value
})

const oddPercentage = computed(() => {
  return (oddCount.value / (totalDraws.value * 5)) * 100
})

const evenPercentage = computed(() => {
  return (evenCount.value / (totalDraws.value * 5)) * 100
})

const averageOddCount = computed(() => {
  return (oddCount.value / totalDraws.value).toFixed(1)
})

const averageEvenCount = computed(() => {
  return (evenCount.value / totalDraws.value).toFixed(1)
})

const patternExamples = computed(() => mockPatternExamples)

const recentDraws = computed(() => 30)

const trendDirection = computed(() => {
  // Mock trend analysis
  const recentOddPercentage = 52.3
  const overallOddPercentage = oddPercentage.value

  if (recentOddPercentage > overallOddPercentage + 5) {
    return { text: '偏奇数 ↗', class: 'odd-trend' }
  } else if (recentOddPercentage < overallOddPercentage - 5) {
    return { text: '偏偶数 ↘', class: 'even-trend' }
  } else {
    return { text: '保持平衡 →', class: 'balanced' }
  }
})

const balanceScore = computed(() => {
  // Calculate balance score (0.5 = perfectly balanced)
  const targetRatio = 0.5
  const currentRatio = oddPercentage.value / 100
  const deviation = Math.abs(currentRatio - targetRatio)
  return Math.max(0, 1 - (deviation * 2)) // Convert deviation to balance score
})

// Methods
const getBarWidth = (count: number): number => {
  const maxCount = Math.max(...ratioData.value.map(item => item.count), 1)
  return (count / maxCount) * 100
}

const updateChart = () => {
  // Update chart based on zone selection
  console.log('Updating chart for zone:', selectedZone.value)
}

// Lifecycle
onMounted(() => {
  updateChart()
})

watch(() => selectedZone, () => {
  updateChart()
})
</script>

<style scoped>
.odd-even-distribution-chart {
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

.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.overview-card {
  background: linear-gradient(135deg, #3498db, #2980b9);
  color: white;
  padding: 20px;
  border-radius: 8px;
  text-align: center;
}

.overview-card h4 {
  margin-bottom: 10px;
  font-size: 0.9rem;
  opacity: 0.9;
}

.most-common {
  font-size: 1.8rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.odd-percentage,
.even-percentage {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.frequency,
.detail {
  font-size: 0.8rem;
  opacity: 0.8;
}

.ratio-distribution,
.trend-analysis,
.pattern-examples {
  margin-bottom: 30px;
}

.ratio-distribution h4,
.trend-analysis h4,
.pattern-examples h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.chart-bars {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
}

.bar-item {
  display: grid;
  grid-template-columns: 1fr auto;
  align-items: center;
  gap: 15px;
  margin-bottom: 15px;
}

.bar-container {
  height: 25px;
  background: #ecf0f1;
  border-radius: 12px;
  overflow: hidden;
  position: relative;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(to right, #27ae60, #2ecc71);
  border-radius: 12px;
  transition: width 0.5s ease;
}

.bar-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  min-width: 100px;
}

.ratio-label {
  font-weight: 600;
  color: #2c3e50;
  font-size: 0.9rem;
}

.ratio-count {
  color: #7f8c8d;
  font-size: 0.8rem;
}

.ratio-percentage {
  font-weight: 500;
  color: #27ae60;
  font-size: 0.8rem;
}

.trend-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
}

.trend-card {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
  text-align: center;
  border: 1px solid #ecf0f1;
}

.trend-card h5 {
  color: #2c3e50;
  margin-bottom: 10px;
  font-size: 0.9rem;
}

.trend-value {
  font-size: 1.2rem;
  font-weight: bold;
  margin-bottom: 8px;
}

.trend-value.odd-trend {
  color: #e67e22;
}

.trend-value.even-trend {
  color: #3498db;
}

.trend-value.balanced {
  color: #27ae60;
}

.trend-detail {
  font-size: 0.7rem;
  color: #7f8c8d;
}

.balance-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.balance-bar {
  width: 100px;
  height: 8px;
  background: #ecf0f1;
  border-radius: 4px;
  overflow: hidden;
}

.balance-fill {
  height: 100%;
  background: linear-gradient(to right, #e74c3c, #f39c12, #27ae60);
  transition: width 0.5s ease;
}

.balance-score {
  font-weight: 600;
  color: #2c3e50;
  min-width: 35px;
}

.examples-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 15px;
}

.example-card {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  border: 1px solid #ecf0f1;
}

.example-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.example-title {
  font-weight: 600;
  color: #2c3e50;
}

.example-frequency {
  background: #27ae60;
  color: white;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.7rem;
}

.example-numbers {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 10px;
}

.number-group {
  display: flex;
  align-items: center;
  gap: 10px;
}

.group-label {
  font-size: 0.8rem;
  color: #7f8c8d;
  min-width: 40px;
}

.numbers {
  display: flex;
  gap: 5px;
}

.number {
  width: 25px;
  height: 25px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7rem;
  font-weight: 600;
}

.number.odd {
  background: #f39c12;
  color: white;
}

.number.even {
  background: #3498db;
  color: white;
}

.example-description {
  font-size: 0.8rem;
  color: #7f8c8d;
  text-align: center;
}

/* Responsive design */
@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .overview-cards {
    grid-template-columns: 1fr;
  }

  .trend-cards {
    grid-template-columns: 1fr;
  }

  .examples-grid {
    grid-template-columns: 1fr;
  }

  .bar-item {
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .bar-info {
    align-items: flex-start;
    flex-direction: row;
    gap: 15px;
  }
}

@media (max-width: 480px) {
  .most-common {
    font-size: 1.5rem;
  }

  .odd-percentage,
  .even-percentage {
    font-size: 1.5rem;
  }

  .number-group {
    flex-direction: column;
    align-items: flex-start;
    gap: 5px;
  }

  .balance-indicator {
    flex-direction: column;
  }
}
</style>