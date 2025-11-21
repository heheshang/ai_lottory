<template>
  <div class="consecutive-patterns-chart">
    <div class="chart-container">
      <div class="chart-header">
        <h3>连续号码模式分析</h3>
        <div class="chart-controls">
          <label>数据源:</label>
          <select v-model="dataSource" @change="updateChart">
            <option value="front">前区</option>
            <option value="back">后区</option>
            <option value="both">综合</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="chart-loading">
        <div class="loading-spinner"></div>
        <p>分析连续模式中...</p>
      </div>

      <div v-else-if="patterns.length === 0" class="chart-empty">
        <div class="empty-icon">📈</div>
        <p>暂无连续模式数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Summary Statistics -->
        <div class="summary-cards">
          <div class="summary-card">
            <h4>连续号出现率</h4>
            <div class="stat-value">{{ (consecutiveRate * 100).toFixed(1) }}%</div>
            <div class="stat-desc">{{ consecutiveCount }} / {{ totalDraws }} 期</div>
          </div>
          <div class="summary-card">
            <h4>最常见长度</h4>
            <div class="stat-value">{{ mostCommonLength }}</div>
            <div class="stat-desc">{{ getLengthFrequency(mostCommonLength) }} 次</div>
          </div>
          <div class="summary-card">
            <h4>最大连续长度</h4>
            <div class="stat-value">{{ maxLength }}</div>
            <div class="stat-desc">历史记录</div>
          </div>
        </div>

        <!-- Length Distribution Chart -->
        <div class="length-distribution">
          <h4>连续号长度分布</h4>
          <div class="distribution-chart">
            <div
              v-for="(item, index) in lengthDistribution"
              :key="item.length"
              class="distribution-bar"
            >
              <div class="bar-label">{{ item.length }}个连续</div>
              <div class="bar-container">
                <div
                  class="bar-fill"
                  :style="{ width: getBarWidth(item.count) + '%' }"
                ></div>
              </div>
              <div class="bar-value">{{ item.count }}次</div>
            </div>
          </div>
        </div>

        <!-- Pattern Examples -->
        <div class="pattern-examples">
          <h4>典型连续模式</h4>
          <div class="examples-grid">
            <div
              v-for="(example, index) in patternExamples"
              :key="`example-${index}`"
              class="example-card"
            >
              <div class="example-header">
                <span class="example-title">{{ example.title }}</span>
                <span class="example-frequency">{{ example.frequency }}次</span>
              </div>
              <div class="example-numbers">
                <span
                  v-for="number in example.numbers"
                  :key="number"
                  class="number"
                  :class="{ consecutive: example.consecutive.includes(number) }"
                >
                  {{ number }}
                </span>
              </div>
              <div class="example-description">
                {{ example.description }}
              </div>
            </div>
          </div>
        </div>

        <!-- Trend Analysis -->
        <div class="trend-analysis">
          <h4>趋势分析</h4>
          <div class="trend-grid">
            <div class="trend-item">
              <div class="trend-label">近期连续号出现频率</div>
              <div class="trend-value" :class="recentTrend.class">
                {{ recentTrend.text }}
              </div>
            </div>
            <div class="trend-item">
              <div class="trend-label">最热门连续号对</div>
              <div class="trend-value">
                {{ hottestPair }}
              </div>
            </div>
            <div class="trend-item">
              <div class="trend-label">预测下期连续概率</div>
              <div class="trend-value prediction">
                {{ (nextDrawProbability * 100).toFixed(1) }}%
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

interface ConsecutivePattern {
  length: number
  count: number
  percentage: number
}

interface PatternExample {
  title: string
  numbers: number[]
  consecutive: number[]
  frequency: number
  description: string
}

const props = defineProps<{
  patterns: ConsecutivePattern[]
  loading?: boolean
  totalDraws?: number
}>()

// State
const dataSource = ref('front')

// Mock data for demonstration
const mockPatterns: ConsecutivePattern[] = [
  { length: 2, count: 45, percentage: 15.2 },
  { length: 3, count: 12, percentage: 4.1 },
  { length: 4, count: 3, percentage: 1.0 },
  { length: 5, count: 1, percentage: 0.3 }
]

const mockExamples: PatternExample[] = [
  {
    title: '经典连号',
    numbers: [12, 13, 14, 25, 26],
    consecutive: [12, 13, 14, 25, 26],
    frequency: 8,
    description: '2+3连续组合'
  },
  {
    title: '起始连号',
    numbers: [1, 2, 3, 18, 25],
    consecutive: [1, 2, 3],
    frequency: 12,
    description: '小号起始连号'
  },
  {
    title: '分散连号',
    numbers: [8, 9, 15, 16, 30],
    consecutive: [8, 9, 15, 16],
    frequency: 6,
    description: '两组不相关连号'
  }
]

// Computed properties
const patterns = computed(() => props.patternes?.length > 0 ? props.patternes : mockPatterns)

const totalDraws = computed(() => props.totalDraws || 295)

const consecutiveCount = computed(() => {
  return patterns.value.reduce((sum, pattern) => sum + pattern.count, 0)
})

const consecutiveRate = computed(() => {
  return consecutiveCount.value / totalDraws.value
})

const lengthDistribution = computed(() => {
  return patterns.value.filter(p => p.length >= 2).sort((a, b) => a.length - b.length)
})

const mostCommonLength = computed(() => {
  const sorted = [...patterns.value].sort((a, b) => b.count - a.count)
  return sorted[0]?.length || 0
})

const maxLength = computed(() => {
  return Math.max(...patterns.value.map(p => p.length), 0)
})

const patternExamples = computed(() => mockExamples)

const recentTrend = computed(() => {
  // Mock trend analysis
  const recentRate = 0.18 // 18% recent occurrence rate
  const overallRate = consecutiveRate.value

  if (recentRate > overallRate * 1.2) {
    return { text: '上升趋势 ↗', class: 'up' }
  } else if (recentRate < overallRate * 0.8) {
    return { text: '下降趋势 ↘', class: 'down' }
  } else {
    return { text: '稳定保持 →', class: 'stable' }
  }
})

const hottestPair = computed(() => {
  // Mock hottest consecutive pair
  return '12-13'
})

const nextDrawProbability = computed(() => {
  // Mock prediction based on patterns
  const baseProbability = consecutiveRate.value
  const recentAdjustment = recentTrend.value.class === 'up' ? 0.05 :
                          recentTrend.value.class === 'down' ? -0.05 : 0

  return Math.max(0, Math.min(1, baseProbability + recentAdjustment))
})

// Methods
const getLengthFrequency = (length: number): number => {
  const pattern = patterns.value.find(p => p.length === length)
  return pattern?.count || 0
}

const getBarWidth = (count: number): number => {
  const maxCount = Math.max(...patterns.value.map(p => p.count), 1)
  return (count / maxCount) * 100
}

const updateChart = () => {
  // Update chart based on data source selection
  console.log('Updating chart for data source:', dataSource.value)
}

// Lifecycle
onMounted(() => {
  updateChart()
})
</script>

<style scoped>
.consecutive-patterns-chart {
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
  border-top: 4px solid #e74c3c;
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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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

.stat-value {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.stat-desc {
  font-size: 0.8rem;
  opacity: 0.8;
}

.length-distribution,
.pattern-examples,
.trend-analysis {
  margin-bottom: 30px;
}

.length-distribution h4,
.pattern-examples h4,
.trend-analysis h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.distribution-chart {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
}

.distribution-bar {
  display: grid;
  grid-template-columns: 100px 1fr 60px;
  align-items: center;
  gap: 15px;
  margin-bottom: 15px;
}

.bar-label {
  font-size: 0.9rem;
  color: #2c3e50;
  font-weight: 500;
}

.bar-container {
  height: 20px;
  background: #ecf0f1;
  border-radius: 10px;
  overflow: hidden;
  position: relative;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(to right, #3498db, #2980b9);
  border-radius: 10px;
  transition: width 0.5s ease;
}

.bar-value {
  font-size: 0.8rem;
  color: #2c3e50;
  font-weight: 600;
  text-align: right;
}

.examples-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
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
  margin-bottom: 10px;
}

.example-title {
  font-weight: 600;
  color: #2c3e50;
}

.example-frequency {
  background: #3498db;
  color: white;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.7rem;
}

.example-numbers {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
  justify-content: center;
}

.number {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  font-weight: 600;
  background: #ecf0f1;
  color: #2c3e50;
}

.number.consecutive {
  background: #3498db;
  color: white;
}

.example-description {
  font-size: 0.8rem;
  color: #7f8c8d;
  text-align: center;
}

.trend-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.trend-item {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  text-align: center;
  border: 1px solid #ecf0f1;
}

.trend-label {
  font-size: 0.8rem;
  color: #7f8c8d;
  margin-bottom: 8px;
}

.trend-value {
  font-size: 1.2rem;
  font-weight: bold;
  color: #2c3e50;
}

.trend-value.up {
  color: #27ae60;
}

.trend-value.down {
  color: #e74c3c;
}

.trend-value.stable {
  color: #f39c12;
}

.trend-value.prediction {
  background: linear-gradient(135deg, #e74c3c, #f39c12);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
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

  .distribution-bar {
    grid-template-columns: 80px 1fr 50px;
    gap: 10px;
  }

  .examples-grid {
    grid-template-columns: 1fr;
  }

  .trend-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .example-numbers {
    gap: 5px;
  }

  .number {
    width: 25px;
    height: 25px;
    font-size: 0.7rem;
  }

  .stat-value {
    font-size: 1.5rem;
  }
}
</style>