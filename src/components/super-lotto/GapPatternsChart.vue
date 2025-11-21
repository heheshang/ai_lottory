<template>
  <div class="gap-patterns-chart">
    <div class="chart-container">
      <div class="chart-header">
        <h3>间隔模式分析</h3>
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
        <p>分析间隔模式中...</p>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <div class="empty-icon">📊</div>
        <p>暂无间隔模式数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Summary Statistics -->
        <div class="summary-cards">
          <div class="summary-card">
            <h4>最常见间隔</h4>
            <div class="most-common-gap">
              {{ mostCommonGap }}期
            </div>
            <div class="frequency">出现 {{ mostCommonGapCount }} 次</div>
          </div>
          <div class="summary-card">
            <h4>平均间隔</h4>
            <div class="average-gap">
              {{ averageGap.toFixed(1) }}期
            </div>
            <div class="detail">基于{{ totalDraws }}期数据</div>
          </div>
          <div class="summary-card">
            <h4>最大间隔</h4>
            <div class="max-gap">
              {{ maxGap }}期
            </div>
            <div class="detail">历史记录</div>
          </div>
        </div>

        <!-- Gap Distribution Chart -->
        <div class="gap-distribution">
          <h4>间隔分布图</h4>
          <div class="distribution-chart">
            <div
              v-for="(item, index) in gapDistribution"
              :key="item.gap"
              class="distribution-bar"
            >
              <div class="gap-label">{{ item.gap }}期</div>
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

        <!-- Gap Analysis Matrix -->
        <div class="gap-matrix">
          <h4>间隔分析矩阵</h4>
          <div class="matrix-grid">
            <div class="matrix-header">
              <div class="cell header">号码</div>
              <div class="cell header">当前间隔</div>
              <div class="cell header">平均间隔</div>
              <div class="cell header">热度</div>
            </div>
            <div
              v-for="(number, index) in numberGaps"
              :key="number.number"
              class="matrix-row"
            >
              <div class="cell number-cell">{{ number.number }}</div>
              <div class="cell current-gap">{{ number.currentGap }}</div>
              <div class="cell avg-gap">{{ number.avgGap.toFixed(1) }}</div>
              <div class="cell" :class="getHeatClass(number.heatScore)">
                {{ getHeatLabel(number.heatScore) }}
              </div>
            </div>
          </div>
        </div>

        <!-- Pattern Examples -->
        <div class="pattern-examples">
          <h4>典型间隔模式</h4>
          <div class="examples-grid">
            <div
              v-for="(example, index) in patternExamples"
              :key="`example-${index}`"
              class="example-card"
            >
              <div class="example-header">
                <span class="example-title">{{ example.title }}</span>
                <span class="example-badge" :class="example.rarity">
                  {{ example.rarity }}
                </span>
              </div>
              <div class="example-numbers">
                <div class="sequence-display">
                  <span
                    v-for="(num, idx) in example.sequence"
                    :key="idx"
                    class="sequence-number"
                    :class="{ gap: example.gaps.includes(idx) }"
                  >
                    {{ num }}
                  </span>
                </div>
                <div class="gap-indicators">
                  <span
                    v-for="(gap, idx) in example.gaps"
                    :key="`gap-${idx}`"
                    class="gap-indicator"
                  >
                    {{ gap }}期
                  </span>
                </div>
              </div>
              <div class="example-description">
                {{ example.description }}
              </div>
            </div>
          </div>
        </div>

        <!-- Prediction Insights -->
        <div class="prediction-insights">
          <h4>预测洞察</h4>
          <div class="insights-grid">
            <div class="insight-card">
              <div class="insight-icon">🔥</div>
              <div class="insight-content">
                <h5>热门间隔</h5>
                <p>间隔{{ hotGaps.join(', ') }}期的号码近期可能开出</p>
              </div>
            </div>
            <div class="insight-card">
              <div class="insight-icon">❄️</div>
              <div class="insight-content">
                <h5>冷门间隔</h5>
                <p>间隔{{ coldGaps.join(', ') }}期的号码已超过平均周期</p>
              </div>
            </div>
            <div class="insight-card">
              <div class="insight-icon">📈</div>
              <div class="insight-content">
                <h5>趋势预测</h5>
                <p>下期可能出现{{ predictedGaps.join(', ') }}期间隔的号码</p>
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

interface GapData {
  gap: number
  count: number
  percentage: number
}

interface NumberGap {
  number: number
  currentGap: number
  avgGap: number
  heatScore: number
}

interface PatternExample {
  title: string
  sequence: number[]
  gaps: number[]
  rarity: 'common' | 'unusual' | 'rare'
  description: string
}

interface Props {
  patterns?: GapData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// State
const selectedZone = ref('front')

// Mock data for demonstration
const mockGapData: GapData[] = [
  { gap: 1, count: 45, percentage: 0.152 },
  { gap: 2, count: 38, percentage: 0.129 },
  { gap: 3, count: 42, percentage: 0.142 },
  { gap: 4, count: 28, percentage: 0.095 },
  { gap: 5, count: 25, percentage: 0.085 },
  { gap: 6, count: 31, percentage: 0.105 },
  { gap: 7, count: 18, percentage: 0.061 },
  { gap: 8, count: 15, percentage: 0.051 },
  { gap: 9, count: 22, percentage: 0.075 },
  { gap: 10, count: 12, percentage: 0.041 }
]

const mockNumberGaps: NumberGap[] = [
  { number: 1, currentGap: 3, avgGap: 4.2, heatScore: 0.75 },
  { number: 2, currentGap: 7, avgGap: 5.1, heatScore: 0.30 },
  { number: 3, currentGap: 2, avgGap: 3.8, heatScore: 0.85 },
  { number: 4, currentGap: 1, avgGap: 4.5, heatScore: 0.95 },
  { number: 5, currentGap: 8, avgGap: 6.2, heatScore: 0.25 },
  { number: 6, currentGap: 0, avgGap: 3.5, heatScore: 1.00 }, // Appeared in last draw
  { number: 7, currentGap: 5, avgGap: 4.8, heatScore: 0.60 },
  { number: 8, currentGap: 12, avgGap: 7.1, heatScore: 0.10 }
]

const mockPatternExamples: PatternExample[] = [
  {
    title: '连续出现',
    sequence: [12, 13, 14],
    gaps: [1, 1],
    rarity: 'common',
    description: '短期间隔连续出现模式'
  },
  {
    title: '等距分布',
    sequence: [5, 10, 15],
    gaps: [5, 5],
    rarity: 'unusual',
    description: '固定间隔的等距模式'
  },
  {
    title: '长期遗漏',
    sequence: [3, 18, 32],
    gaps: [15, 14],
    rarity: 'rare',
    description: '超长间隔的回补模式'
  }
]

// Computed properties
const gapDistribution = computed(() => {
  return props.patterns?.length > 0 ? props.patterns : mockGapData
})

const hasData = computed(() => {
  return gapDistribution.value.length > 0
})

const totalDraws = computed(() => {
  return 295 // Mock total draws
})

const mostCommonGap = computed(() => {
  const sorted = [...gapDistribution.value].sort((a, b) => b.count - a.count)
  return sorted[0]?.gap || 0
})

const mostCommonGapCount = computed(() => {
  const sorted = [...gapDistribution.value].sort((a, b) => b.count - a.count)
  return sorted[0]?.count || 0
})

const averageGap = computed(() => {
  const totalGap = gapDistribution.value.reduce((sum, item) => sum + (item.gap * item.count), 0)
  const totalOccurrences = gapDistribution.value.reduce((sum, item) => sum + item.count, 0)
  return totalOccurrences > 0 ? totalGap / totalOccurrences : 0
})

const maxGap = computed(() => {
  return Math.max(...gapDistribution.value.map(item => item.gap), 0)
})

const numberGaps = computed(() => mockNumberGaps)

const patternExamples = computed(() => mockPatternExamples)

const hotGaps = computed(() => {
  // Mock hot gaps (gaps that are due to appear)
  return [3, 5, 7]
})

const coldGaps = computed(() => {
  // Mock cold gaps (gaps that have exceeded average)
  return [9, 10, 12]
})

const predictedGaps = computed(() => {
  // Mock predicted gaps for next draw
  return [2, 4, 6]
})

// Methods
const getBarWidth = (count: number): number => {
  const maxCount = Math.max(...gapDistribution.value.map(item => item.count), 1)
  return (count / maxCount) * 100
}

const getHeatClass = (score: number): string => {
  if (score >= 0.8) return 'hot'
  if (score >= 0.5) return 'warm'
  if (score >= 0.3) return 'cool'
  return 'cold'
}

const getHeatLabel = (score: number): string => {
  if (score >= 0.8) return '热门'
  if (score >= 0.5) return '温号'
  if (score >= 0.3) return '冷号'
  return '极冷'
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
.gap-patterns-chart {
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
  background: linear-gradient(135deg, #e74c3c, #c0392b);
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

.most-common-gap,
.average-gap,
.max-gap {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 5px;
}

.frequency,
.detail {
  font-size: 0.8rem;
  opacity: 0.8;
}

.gap-distribution,
.gap-matrix,
.pattern-examples,
.prediction-insights {
  margin-bottom: 30px;
}

.gap-distribution h4,
.gap-matrix h4,
.pattern-examples h4,
.prediction-insights h4 {
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
  grid-template-columns: 60px 1fr 50px;
  align-items: center;
  gap: 15px;
  margin-bottom: 12px;
}

.gap-label {
  font-size: 0.9rem;
  color: #2c3e50;
  font-weight: 500;
  text-align: right;
}

.bar-container {
  height: 20px;
  background: #ecf0f1;
  border-radius: 10px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(to right, #e74c3c, #f39c12);
  border-radius: 10px;
  transition: width 0.5s ease;
}

.bar-value {
  font-size: 0.8rem;
  color: #2c3e50;
  font-weight: 600;
  text-align: left;
}

.matrix-grid {
  background: #f8f9fa;
  border-radius: 6px;
  overflow: hidden;
}

.matrix-header {
  display: grid;
  grid-template-columns: 60px 1fr 1fr 1fr;
  background: #ecf0f1;
  font-weight: 600;
  color: #2c3e50;
}

.matrix-row {
  display: grid;
  grid-template-columns: 60px 1fr 1fr 1fr;
  border-bottom: 1px solid #ecf0f1;
}

.matrix-row:last-child {
  border-bottom: none;
}

.cell {
  padding: 12px 15px;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cell.header {
  background: #ecf0f1;
  font-weight: 600;
}

.number-cell {
  font-weight: 600;
  color: #2c3e50;
}

.cell.hot {
  background: #fdeaea;
  color: #e74c3c;
  font-weight: 600;
}

.cell.warm {
  background: #fef9e7;
  color: #f39c12;
}

.cell.cool {
  background: #eafaf1;
  color: #27ae60;
}

.cell.cold {
  background: #ebf5fb;
  color: #3498db;
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
  margin-bottom: 15px;
}

.example-title {
  font-weight: 600;
  color: #2c3e50;
}

.example-badge {
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 0.7rem;
  font-weight: 500;
}

.example-badge.common {
  background: #e8f5e8;
  color: #27ae60;
}

.example-badge.unusual {
  background: #fef9e7;
  color: #f39c12;
}

.example-badge.rare {
  background: #fdeaea;
  color: #e74c3c;
}

.example-numbers {
  margin-bottom: 10px;
}

.sequence-display {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  justify-content: center;
}

.sequence-number {
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

.sequence-number.gap {
  background: #e74c3c;
  color: white;
}

.gap-indicators {
  display: flex;
  gap: 5px;
  justify-content: center;
}

.gap-indicator {
  font-size: 0.7rem;
  color: #7f8c8d;
  padding: 2px 6px;
  background: #ecf0f1;
  border-radius: 3px;
}

.example-description {
  font-size: 0.8rem;
  color: #7f8c8d;
  text-align: center;
}

.insights-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.insight-card {
  display: flex;
  align-items: flex-start;
  gap: 15px;
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  border: 1px solid #ecf0f1;
}

.insight-icon {
  font-size: 1.5rem;
  flex-shrink: 0;
}

.insight-content h5 {
  color: #2c3e50;
  margin-bottom: 5px;
  font-size: 0.9rem;
}

.insight-content p {
  color: #7f8c8d;
  font-size: 0.8rem;
  line-height: 1.4;
  margin: 0;
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
    grid-template-columns: 50px 1fr 40px;
    gap: 10px;
  }

  .matrix-header,
  .matrix-row {
    grid-template-columns: 50px 1fr 1fr 1fr;
  }

  .cell {
    padding: 10px 8px;
    font-size: 0.8rem;
  }

  .examples-grid {
    grid-template-columns: 1fr;
  }

  .insights-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 480px) {
  .most-common-gap,
  .average-gap,
  .max-gap {
    font-size: 1.5rem;
  }

  .distribution-bar {
    grid-template-columns: 40px 1fr 35px;
  }

  .gap-label {
    font-size: 0.8rem;
  }

  .bar-value {
    font-size: 0.7rem;
  }

  .matrix-header,
  .matrix-row {
    grid-template-columns: 40px repeat(3, 1fr);
  }

  .cell {
    padding: 8px 4px;
    font-size: 0.7rem;
  }

  .insight-card {
    flex-direction: column;
    text-align: center;
  }
}
</style>