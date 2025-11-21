<template>
  <div class="position-patterns-chart">
    <div class="chart-container">
      <div class="chart-header">
        <h3>位置模式分析</h3>
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
        <p>分析位置模式中...</p>
      </div>

      <div v-else-if="!hasData" class="chart-empty">
        <div class="empty-icon">📊</div>
        <p>暂无位置模式数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Position Heatmap -->
        <div class="position-heatmap">
          <h4>位置热度图</h4>
          <div class="heatmap-grid">
            <div class="position-header">
              <div class="cell label">位置\\号码</div>
              <div
                v-for="num in numberRange"
                :key="num"
                class="cell number-header"
              >
                {{ num }}
              </div>
            </div>
            <div
              v-for="pos in positionRange"
              :key="pos"
              class="position-row"
            >
              <div class="cell position-label">{{ getPositionLabel(pos) }}</div>
              <div
                v-for="num in numberRange"
                :key="`${pos}-${num}`"
                class="cell heatmap-cell"
                :class="getHeatClass(getFrequency(pos, num))"
                :style="{ opacity: getOpacity(getFrequency(pos, num)) }"
              >
                {{ getFrequency(pos, num) }}
              </div>
            </div>
          </div>
          <div class="heatmap-legend">
            <div class="legend-item">
              <div class="legend-color hot"></div>
              <span>高频 (10+次)</span>
            </div>
            <div class="legend-item">
              <div class="legend-color warm"></div>
              <span>中频 (5-9次)</span>
            </div>
            <div class="legend-item">
              <div class="legend-color cool"></div>
              <span>低频 (1-4次)</span>
            </div>
            <div class="legend-item">
              <div class="legend-color cold"></div>
              <span>零频 (0次)</span>
            </div>
          </div>
        </div>

        <!-- Position Statistics -->
        <div class="position-stats">
          <h4>位置统计</h4>
          <div class="stats-grid">
            <div
              v-for="(stat, index) in positionStats"
              :key="index"
              class="stat-card"
            >
              <div class="stat-header">
                <span class="stat-position">{{ stat.position }}</span>
                <span class="stat-range">{{ stat.range }}</span>
              </div>
              <div class="stat-details">
                <div class="most-common">
                  <span class="label">最常见:</span>
                  <span class="value">{{ stat.mostCommon }}</span>
                </div>
                <div class="frequency">
                  <span class="label">频率:</span>
                  <span class="value">{{ stat.frequency }}%</span>
                </div>
                <div class="avg-value">
                  <span class="label">平均值:</span>
                  <span class="value">{{ stat.average.toFixed(1) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Pattern Examples -->
        <div class="pattern-examples">
          <h4>典型位置模式</h4>
          <div class="examples-grid">
            <div
              v-for="(example, index) in patternExamples"
              :key="`example-${index}`"
              class="example-card"
            >
              <div class="example-header">
                <span class="example-title">{{ example.title }}</span>
                <span class="example-badge" :class="example.type">
                  {{ example.type }}
                </span>
              </div>
              <div class="example-pattern">
                <div class="pattern-display">
                  <div
                    v-for="(pos, idx) in example.pattern"
                    :key="idx"
                    class="pattern-slot"
                  >
                    <div class="slot-label">{{ getPositionLabel(idx + 1) }}</div>
                    <div class="slot-value">{{ pos }}</div>
                  </div>
                </div>
                <div class="pattern-features">
                  <div
                    v-for="(feature, idx) in example.features"
                    :key="`feature-${idx}`"
                    class="feature-tag"
                  >
                    {{ feature }}
                  </div>
                </div>
              </div>
              <div class="example-description">
                {{ example.description }}
              </div>
            </div>
          </div>
        </div>

        <!-- Position Trends -->
        <div class="position-trends">
          <h4>位置趋势分析</h4>
          <div class="trends-container">
            <div class="trend-chart">
              <div
                v-for="(trend, index) in positionTrends"
                :key="index"
                class="trend-item"
              >
                <div class="trend-label">{{ getPositionLabel(index + 1) }}</div>
                <div class="trend-bar">
                  <div
                    class="trend-fill"
                    :style="{ width: getTrendWidth(trend.currentValue) + '%' }"
                  ></div>
                </div>
                <div class="trend-value">{{ trend.currentValue.toFixed(1) }}</div>
                <div class="trend-change" :class="trend.changeClass">
                  {{ trend.changeText }}
                </div>
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

interface PositionStat {
  position: string
  range: string
  mostCommon: number
  frequency: number
  average: number
}

interface PatternExample {
  title: string
  pattern: number[]
  features: string[]
  type: 'ascending' | 'descending' | 'random' | 'clustered'
  description: string
}

interface PositionTrend {
  currentValue: number
  changeClass: 'up' | 'down' | 'stable'
  changeText: string
}

interface Props {
  patterns?: any
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// State
const selectedZone = ref('front')

// Mock data for demonstration
const mockPositionStats: PositionStat[] = [
  {
    position: '第一位',
    range: '1-35',
    mostCommon: 12,
    frequency: 8.5,
    average: 17.8
  },
  {
    position: '第二位',
    range: '1-35',
    mostCommon: 18,
    frequency: 7.2,
    average: 18.2
  },
  {
    position: '第三位',
    range: '1-35',
    mostCommon: 23,
    frequency: 6.8,
    average: 18.5
  },
  {
    position: '第四位',
    range: '1-35',
    mostCommon: 28,
    frequency: 7.5,
    average: 19.1
  },
  {
    position: '第五位',
    range: '1-35',
    mostCommon: 33,
    frequency: 9.1,
    average: 20.3
  }
]

const mockPatternExamples: PatternExample[] = [
  {
    title: '递增模式',
    pattern: [5, 12, 19, 26, 33],
    features: ['等差递增', '均匀分布', '跨度大'],
    type: 'ascending',
    description: '号码呈递增趋势，间隔相对均匀'
  },
  {
    title: '聚簇模式',
    pattern: [11, 12, 13, 25, 26],
    features: ['连续号码', '聚簇分布', '小跨度'],
    type: 'clustered',
    description: '号码聚集成簇，连续出现'
  },
  {
    title: '随机分布',
    pattern: [3, 18, 25, 7, 31],
    features: ['无规律', '大跨度', '散乱分布'],
    type: 'random',
    description: '号码分布无明显规律，完全随机'
  }
]

const mockPositionTrends: PositionTrend[] = [
  { currentValue: 17.8, changeClass: 'up', changeText: '+2.1' },
  { currentValue: 18.2, changeClass: 'stable', changeText: '±0.0' },
  { currentValue: 18.5, changeClass: 'down', changeText: '-1.3' },
  { currentValue: 19.1, changeClass: 'up', changeText: '+3.2' },
  { currentValue: 20.3, changeClass: 'up', changeText: '+1.8' }
]

// Computed properties
const hasData = computed(() => {
  return true // Always has mock data
})

const positionRange = computed(() => {
  return selectedZone.value === 'front' ? [1, 2, 3, 4, 5] : [1, 2]
})

const numberRange = computed(() => {
  return selectedZone.value === 'front' ? Array.from({ length: 35 }, (_, i) => i + 1) : Array.from({ length: 12 }, (_, i) => i + 1)
})

const positionStats = computed(() => {
  return selectedZone.value === 'front' ? mockPositionStats : mockPositionStats.slice(0, 2)
})

const patternExamples = computed(() => mockPatternExamples)

const positionTrends = computed(() => {
  return selectedZone.value === 'front' ? mockPositionTrends : mockPositionTrends.slice(0, 2)
})

// Mock frequency matrix for heatmap
const mockFrequencyMatrix: Record<string, Record<number, number>> = {
  '1': { 1: 3, 2: 5, 3: 8, 4: 12, 5: 15, 6: 18, 7: 22, 8: 25, 9: 28, 10: 30, 11: 29, 12: 26, 13: 22, 14: 18, 15: 15, 16: 12, 17: 10, 18: 8, 19: 7, 20: 6, 21: 5, 22: 4, 23: 3, 24: 2, 25: 2, 26: 3, 27: 4, 28: 5, 29: 7, 30: 9, 31: 12, 32: 15, 33: 18, 34: 22, 35: 25 }
  // Additional positions would have different patterns...
}

// Methods
const getPositionLabel = (position: number): string => {
  return `第${position}位`
}

const getFrequency = (position: number, number: number): number => {
  // Mock frequency calculation
  const matrix = mockFrequencyMatrix[position.toString()] || {}
  return matrix[number] || Math.floor(Math.random() * 30)
}

const getHeatClass = (frequency: number): string => {
  if (frequency >= 20) return 'hot'
  if (frequency >= 10) return 'warm'
  if (frequency >= 5) return 'cool'
  return 'cold'
}

const getOpacity = (frequency: number): number => {
  const maxFreq = 30
  return 0.3 + (frequency / maxFreq) * 0.7
}

const getTrendWidth = (value: number): number => {
  const maxValue = 35
  return (value / maxValue) * 100
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
.position-patterns-chart {
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
  border-top: 4px solid #f39c12;
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

.position-heatmap,
.position-stats,
.pattern-examples,
.position-trends {
  margin-bottom: 30px;
}

.position-heatmap h4,
.position-stats h4,
.pattern-examples h4,
.position-trends h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.heatmap-grid {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  overflow-x: auto;
}

.position-header {
  display: grid;
  grid-template-columns: 80px repeat(auto-fit, 40px);
  gap: 5px;
  margin-bottom: 10px;
}

.position-row {
  display: grid;
  grid-template-columns: 80px repeat(auto-fit, 40px);
  gap: 5px;
}

.cell {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  padding: 8px 4px;
  border-radius: 4px;
}

.cell.label {
  background: #ecf0f1;
  font-weight: 600;
  color: #2c3e50;
}

.cell.number-header {
  background: #3498db;
  color: white;
  font-weight: 600;
}

.cell.position-label {
  background: #95a5a6;
  color: white;
  font-weight: 500;
}

.cell.heatmap-cell {
  font-weight: 500;
  transition: all 0.3s;
  cursor: pointer;
}

.cell.heatmap-cell:hover {
  transform: scale(1.1);
  z-index: 10;
}

.cell.heatmap-cell.hot {
  background: #e74c3c;
  color: white;
}

.cell.heatmap-cell.warm {
  background: #f39c12;
  color: white;
}

.cell.heatmap-cell.cool {
  background: #3498db;
  color: white;
}

.cell.heatmap-cell.cold {
  background: #ecf0f1;
  color: #7f8c8d;
}

.heatmap-legend {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #ecf0f1;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  color: #7f8c8d;
}

.legend-color {
  width: 16px;
  height: 16px;
  border-radius: 3px;
}

.legend-color.hot {
  background: #e74c3c;
}

.legend-color.warm {
  background: #f39c12;
}

.legend-color.cool {
  background: #3498db;
}

.legend-color.cold {
  background: #ecf0f1;
  border: 1px solid #bdc3c7;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.stat-card {
  background: linear-gradient(135deg, #f39c12, #e67e22);
  color: white;
  padding: 15px;
  border-radius: 6px;
}

.stat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.3);
}

.stat-position {
  font-weight: 600;
}

.stat-range {
  font-size: 0.7rem;
  opacity: 0.8;
}

.stat-details {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.most-common,
.frequency,
.avg-value {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
}

.label {
  opacity: 0.8;
}

.value {
  font-weight: 600;
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

.example-badge.ascending {
  background: #e8f5e8;
  color: #27ae60;
}

.example-badge.descending {
  background: #fdeaea;
  color: #e74c3c;
}

.example-badge.random {
  background: #fef9e7;
  color: #f39c12;
}

.example-badge.clustered {
  background: #eafaf1;
  color: #3498db;
}

.example-pattern {
  margin-bottom: 10px;
}

.pattern-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 8px;
}

.pattern-slot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
  padding: 5px 8px;
  border-radius: 3px;
  border: 1px solid #ecf0f1;
}

.slot-label {
  font-size: 0.7rem;
  color: #7f8c8d;
  min-width: 40px;
}

.slot-value {
  font-weight: 600;
  color: #2c3e50;
}

.pattern-features {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.feature-tag {
  background: #3498db;
  color: white;
  padding: 1px 6px;
  border-radius: 2px;
  font-size: 0.6rem;
}

.example-description {
  font-size: 0.8rem;
  color: #7f8c8d;
  text-align: center;
}

.trends-container {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
}

.trend-chart {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.trend-item {
  display: grid;
  grid-template-columns: 80px 1fr 60px 50px;
  align-items: center;
  gap: 15px;
  padding: 10px;
  background: white;
  border-radius: 4px;
  border: 1px solid #ecf0f1;
}

.trend-label {
  font-size: 0.8rem;
  color: #2c3e50;
  font-weight: 500;
}

.trend-bar {
  height: 20px;
  background: #ecf0f1;
  border-radius: 10px;
  overflow: hidden;
}

.trend-fill {
  height: 100%;
  background: linear-gradient(to right, #f39c12, #e67e22);
  border-radius: 10px;
  transition: width 0.5s ease;
}

.trend-value {
  font-weight: 600;
  color: #2c3e50;
  text-align: center;
}

.trend-change {
  font-weight: 600;
  text-align: center;
  font-size: 0.8rem;
}

.trend-change.up {
  color: #27ae60;
}

.trend-change.down {
  color: #e74c3c;
}

.trend-change.stable {
  color: #f39c12;
}

/* Responsive design */
@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .examples-grid {
    grid-template-columns: 1fr;
  }

  .heatmap-legend {
    flex-direction: column;
    gap: 10px;
    align-items: center;
  }

  .position-header,
  .position-row {
    grid-template-columns: 60px repeat(auto-fit, 35px);
    gap: 3px;
  }

  .cell {
    padding: 6px 2px;
    font-size: 0.7rem;
  }

  .trend-item {
    grid-template-columns: 70px 1fr 50px 40px;
    gap: 10px;
  }
}

@media (max-width: 480px) {
  .position-header,
  .position-row {
    grid-template-columns: 50px repeat(auto-fit, 30px);
    gap: 2px;
  }

  .cell {
    padding: 4px 1px;
    font-size: 0.6rem;
  }

  .trend-item {
    grid-template-columns: 60px 1fr 40px 35px;
    gap: 8px;
    padding: 8px;
  }

  .pattern-slot {
    padding: 3px 5px;
  }

  .slot-label {
    min-width: 35px;
    font-size: 0.6rem;
  }

  .feature-tag {
    font-size: 0.5rem;
  }
}
</style>