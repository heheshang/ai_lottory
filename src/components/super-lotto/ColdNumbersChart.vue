<template>
  <div class="cold-numbers-chart">
    <div class="chart-container">
      <div class="chart-header">
        <h3>冷号分析图表</h3>
        <div class="chart-controls">
          <label>显示数量:</label>
          <select v-model="displayCount" @change="updateChart">
            <option value="10">前10名</option>
            <option value="15">前15名</option>
            <option value="20">前20名</option>
          </select>
        </div>
      </div>

      <div v-if="loading" class="chart-loading">
        <div class="loading-spinner"></div>
        <p>分析中...</p>
      </div>

      <div v-else-if="chartData.length === 0" class="chart-empty">
        <div class="empty-icon">📊</div>
        <p>暂无冷号数据</p>
      </div>

      <div v-else class="chart-content">
        <!-- Reverse Bar Chart (Colder = Lower) -->
        <div class="bar-chart">
          <div class="chart-bars">
            <div
              v-for="item in chartData"
              :key="item.number"
              class="bar-item cold"
              :style="{ height: getBarHeight(item.frequency) + '%' }"
            >
              <div class="bar cold-bar">
                <span class="bar-label">{{ item.number }}</span>
                <span class="bar-value">{{ item.frequency }}</span>
              </div>
              <div class="bar-info">
                <span class="cold-score">冷值: {{ item.cold_score.toFixed(2) }}</span>
                <span v-if="item.last_seen" class="last-seen">
                  遗漏: {{ getDaysSinceLastSeen(item.last_seen) }}天
                </span>
              </div>
            </div>
          </div>
          <div class="chart-axis">
            <div class="axis-labels">
              <span v-for="i in 5" :key="i" class="axis-label">
                {{ (maxFrequency * i / 5).toFixed(0) }}
              </span>
            </div>
          </div>
        </div>

        <!-- Statistics Summary -->
        <div class="chart-summary">
          <div class="summary-stats">
            <div class="stat-item">
              <h4>最冷号码</h4>
              <div class="coldest-number">
                {{ chartData[0]?.number || '-' }}
                <span class="frequency">({{ chartData[0]?.frequency || 0 }}次)</span>
              </div>
            </div>
            <div class="stat-item">
              <h4>平均遗漏</h4>
              <div class="average-gap">
                {{ averageGap.toFixed(0) }}天
              </div>
            </div>
            <div class="stat-item">
              <h4>最大遗漏</h4>
              <div class="max-gap">
                {{ maxGap }}天
              </div>
            </div>
          </div>
        </div>

        <!-- Cold Trend Indicator -->
        <div class="trend-indicator">
          <h4>冷号趋势</h4>
          <div class="trend-items">
            <div
              v-for="(item, index) in chartData.slice(0, 5)"
              :key="`trend-${item.number}`"
              class="trend-item"
            >
              <span class="trend-number">{{ item.number }}</span>
              <div class="trend-bar">
                <div
                  class="trend-fill"
                  :style="{ width: getTrendWidth(item.cold_score) + '%' }"
                ></div>
              </div>
              <span class="trend-score">{{ item.cold_score.toFixed(1) }}</span>
            </div>
          </div>
        </div>

        <!-- Zone Toggle -->
        <div class="zone-toggle" v-if="showZoneToggle">
          <button
            :class="['zone-btn', { active: selectedZone === 'FRONT' }]"
            @click="$emit('zone-change', 'FRONT')"
          >
            前区
          </button>
          <button
            :class="['zone-btn', { active: selectedZone === 'BACK' }]"
            @click="$emit('zone-change', 'BACK')"
          >
            后区
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { NumberFrequency } from '@/types'

interface Props {
  numbers: NumberFrequency[]
  loading?: boolean
  selectedZone?: string
  showZoneToggle?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  selectedZone: 'FRONT',
  showZoneToggle: true
})

const emit = defineEmits<{
  'zone-change': [zone: string]
  'number-select': [number: NumberFrequency]
}>()

// State
const displayCount = ref(15)

// Computed properties
const chartData = computed(() => {
  return props.numbers
    .slice(0, displayCount.value)
    .sort((a, b) => a.frequency - b.frequency) // Sort ascending for cold numbers
})

const maxFrequency = computed(() => {
  return Math.max(...chartData.value.map(n => n.frequency), 1)
})

const minFrequency = computed(() => {
  return Math.min(...chartData.value.map(n => n.frequency), 0)
})

const averageGap = computed(() => {
  if (chartData.value.length === 0) return 0
  const gaps = chartData.value
    .filter(n => n.last_seen)
    .map(n => getDaysSinceLastSeen(n.last_seen!))
  return gaps.length > 0 ? gaps.reduce((a, b) => a + b, 0) / gaps.length : 0
})

const maxGap = computed(() => {
  if (chartData.value.length === 0) return 0
  const gaps = chartData.value
    .filter(n => n.last_seen)
    .map(n => getDaysSinceLastSeen(n.last_seen!))
  return gaps.length > 0 ? Math.max(...gaps) : 0
})

// Methods
const getBarHeight = (frequency: number): number => {
  return (frequency / maxFrequency.value) * 100
}

const getTrendWidth = (coldScore: number): number => {
  return Math.min(coldScore * 20, 100) // Scale cold score to percentage
}

const getDaysSinceLastSeen = (lastSeen: string): number => {
  try {
    const lastDate = new Date(lastSeen)
    const now = new Date()
    const diffTime = Math.abs(now.getTime() - lastDate.getTime())
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24))
  } catch {
    return 0
  }
}

const updateChart = () => {
  // Trigger chart update when display count changes
}

const selectNumber = (number: NumberFrequency) => {
  emit('number-select', number)
}

// Lifecycle
onMounted(() => {
  updateChart()
})

watch(() => props.numbers, () => {
  updateChart()
})
</script>

<style scoped>
.cold-numbers-chart {
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

.chart-content {
  position: relative;
}

.bar-chart {
  margin-bottom: 30px;
}

.chart-bars {
  display: flex;
  align-items: flex-end;
  justify-content: space-around;
  height: 200px;
  padding: 0 10px;
  margin-bottom: 10px;
  background: linear-gradient(to top, #f8f9fa 0%, transparent 100%);
  border-radius: 4px;
}

.bar-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 40px;
  max-width: 60px;
  transition: all 0.3s;
  cursor: pointer;
}

.bar-item:hover {
  transform: translateY(-5px);
}

.bar {
  flex: 1;
  border-radius: 4px 4px 0 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  padding: 8px 4px;
  min-height: 40px;
  color: white;
  width: 100%;
}

.cold-bar {
  background: linear-gradient(to top, #3498db, #2980b9);
}

.bar-label {
  font-weight: bold;
  font-size: 0.9rem;
  margin-bottom: 2px;
}

.bar-value {
  font-size: 0.8rem;
  opacity: 0.9;
}

.bar-info {
  margin-top: 8px;
  font-size: 0.7rem;
  color: #7f8c8d;
  text-align: center;
  line-height: 1.2;
}

.cold-score {
  font-weight: 600;
  color: #3498db;
}

.last-seen {
  display: block;
  margin-top: 2px;
}

.chart-axis {
  height: 30px;
  border-left: 2px solid #bdc3c7;
  border-bottom: 2px solid #bdc3c7;
  position: relative;
  margin-left: 20px;
}

.axis-labels {
  display: flex;
  justify-content: space-between;
  height: 100%;
  padding: 0 10px;
}

.axis-label {
  font-size: 0.7rem;
  color: #7f8c8d;
  align-self: flex-end;
}

.chart-summary {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  margin-bottom: 20px;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 20px;
}

.stat-item {
  text-align: center;
}

.stat-item h4 {
  color: #7f8c8d;
  margin-bottom: 8px;
  font-size: 0.9rem;
  font-weight: 500;
}

.coldest-number {
  font-size: 1.5rem;
  font-weight: bold;
  color: #3498db;
}

.frequency {
  font-size: 0.8rem;
  color: #7f8c8d;
  font-weight: normal;
}

.average-gap,
.max-gap {
  font-size: 1.2rem;
  font-weight: bold;
  color: #2c3e50;
}

.trend-indicator {
  background: #ecf0f1;
  border-radius: 6px;
  padding: 15px;
  margin-bottom: 20px;
}

.trend-indicator h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.trend-items {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.trend-item {
  display: flex;
  align-items: center;
  gap: 15px;
}

.trend-number {
  width: 30px;
  height: 30px;
  background: #3498db;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 0.9rem;
  flex-shrink: 0;
}

.trend-bar {
  flex: 1;
  height: 8px;
  background: #bdc3c7;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.trend-fill {
  height: 100%;
  background: linear-gradient(to right, #3498db, #5dade2);
  border-radius: 4px;
  transition: width 0.5s ease;
}

.trend-score {
  font-size: 0.8rem;
  font-weight: 600;
  color: #3498db;
  min-width: 30px;
  text-align: right;
}

.zone-toggle {
  display: flex;
  justify-content: center;
  gap: 10px;
}

.zone-btn {
  padding: 8px 20px;
  border: 2px solid #ecf0f1;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.3s;
}

.zone-btn:hover {
  border-color: #3498db;
  color: #3498db;
}

.zone-btn.active {
  background: #3498db;
  color: white;
  border-color: #3498db;
}

/* Responsive design */
@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .chart-bars {
    height: 150px;
    padding: 0 5px;
  }

  .bar-item {
    min-width: 30px;
    max-width: 40px;
  }

  .bar {
    padding: 6px 2px;
    min-height: 30px;
  }

  .bar-label {
    font-size: 0.8rem;
  }

  .bar-value {
    font-size: 0.7rem;
  }

  .summary-stats {
    grid-template-columns: 1fr;
  }

  .trend-item {
    gap: 10px;
  }

  .trend-number {
    width: 25px;
    height: 25px;
    font-size: 0.8rem;
  }

  .zone-toggle {
    flex-direction: column;
  }
}

@media (max-width: 480px) {
  .chart-bars {
    height: 120px;
  }

  .bar-item {
    min-width: 25px;
  }

  .bar {
    padding: 4px 1px;
    min-height: 25px;
  }

  .bar-label {
    font-size: 0.7rem;
  }

  .bar-info {
    font-size: 0.6rem;
  }

  .trend-score {
    font-size: 0.7rem;
  }
}
</style>