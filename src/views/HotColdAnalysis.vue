<template>
  <div class="hot-cold-analysis">
    <div class="page-header">
      <h1>热冷号分析</h1>
      <p class="page-description">分析大乐透号码的出现频率和趋势</p>
    </div>

    <div class="controls-section">
      <AnalysisControls
        @period-change="handlePeriodChange"
        @zone-change="handleZoneChange"
        :loading="loading"
      />
    </div>

    <div class="data-section">
      <div v-if="loading" class="loading-container">
        <div class="loading-spinner"></div>
        <p>分析中...</p>
      </div>

      <div v-else-if="error" class="error-container">
        <div class="error-message">
          <i class="icon-error"></i>
          <p>{{ error }}</p>
          <button @click="clearError" class="btn btn-primary">重试</button>
        </div>
      </div>

      <div v-else-if="!hasAnalysisData" class="empty-state">
        <div class="empty-icon">📊</div>
        <h3>暂无分析数据</h3>
        <p>请先导入历史数据以进行热冷号分析</p>
        <router-link to="/super-lotto/history" class="btn btn-primary">
          导入数据
        </router-link>
      </div>

      <div v-else class="data-container">
        <div class="analysis-tabs">
          <button
            :class="['tab-btn', { active: activeTab === 'hot' }]"
            @click="activeTab = 'hot'"
          >
            热号分析
          </button>
          <button
            :class="['tab-btn', { active: activeTab === 'cold' }]"
            @click="activeTab = 'cold'"
          >
            冷号分析
          </button>
          <button
            :class="['tab-btn', { active: activeTab === 'comparison' }]"
            @click="activeTab = 'comparison'"
          >
            对比分析
          </button>
        </div>

        <div class="tab-content">
          <!-- Hot Numbers Tab -->
          <div v-if="activeTab === 'hot'" class="analysis-panel">
            <div class="panel-header">
              <h3>热号排名</h3>
              <div class="stats-info">
                <span>分析周期: {{ currentPeriod }} 天</span>
                <span>统计期数: {{ totalDraws }}</span>
              </div>
            </div>
            <HotNumbersChart :numbers="hotNumbers" />
            <div class="number-details">
              <h4>热号详细数据</h4>
              <div class="number-grid">
                <div
                  v-for="number in hotNumbers"
                  :key="`${number.number}-${number.zone}`"
                  class="number-card hot"
                >
                  <div class="number-value">{{ number.number }}</div>
                  <div class="number-stats">
                    <span>出现次数: {{ number.frequency }}</span>
                    <span>热值: {{ number.hot_score.toFixed(2) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Cold Numbers Tab -->
          <div v-if="activeTab === 'cold'" class="analysis-panel">
            <div class="panel-header">
              <h3>冷号排名</h3>
              <div class="stats-info">
                <span>分析周期: {{ currentPeriod }} 天</span>
                <span>统计期数: {{ totalDraws }}</span>
              </div>
            </div>
            <ColdNumbersChart :numbers="coldNumbers" />
            <div class="number-details">
              <h4>冷号详细数据</h4>
              <div class="number-grid">
                <div
                  v-for="number in coldNumbers"
                  :key="`${number.number}-${number.zone}`"
                  class="number-card cold"
                >
                  <div class="number-value">{{ number.number }}</div>
                  <div class="number-stats">
                    <span>出现次数: {{ number.frequency }}</span>
                    <span>冷值: {{ number.cold_score.toFixed(2) }}</span>
                    <span v-if="number.last_seen">
                      最后出现: {{ formatDate(number.last_seen) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Comparison Tab -->
          <div v-if="activeTab === 'comparison'" class="analysis-panel">
            <div class="panel-header">
              <h3>热冷号对比</h3>
            </div>
            <div class="comparison-content">
              <div class="comparison-section">
                <h4>热号 vs 冷号统计</h4>
                <div class="comparison-stats">
                  <div class="stat-card">
                    <h5>热号数量</h5>
                    <p class="stat-value hot">{{ hotNumbers.length }}</p>
                  </div>
                  <div class="stat-card">
                    <h5>冷号数量</h5>
                    <p class="stat-value cold">{{ coldNumbers.length }}</p>
                  </div>
                  <div class="stat-card">
                    <h5>平均热值</h5>
                    <p class="stat-value">{{ averageHotScore.toFixed(2) }}</p>
                  </div>
                  <div class="stat-card">
                    <h5>平均冷值</h5>
                    <p class="stat-value">{{ averageColdScore.toFixed(2) }}</p>
                  </div>
                </div>
              </div>

              <div class="comparison-section">
                <h4>号码分布</h4>
                <div class="distribution-chart">
                  <!-- 这里可以添加分布图表 -->
                  <p>号码分布分析图表</p>
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
import { useSuperLottoStore } from '@/stores/superLotto'
import type { NumberFrequency, AnalysisParams } from '@/stores/superLotto'
import HotNumbersChart from '@/components/super-lotto/HotNumbersChart.vue'
import ColdNumbersChart from '@/components/super-lotto/ColdNumbersChart.vue'
import AnalysisControls from '@/components/super-lotto/AnalysisControls.vue'

const superLottoStore = useSuperLottoStore()

// Reactive state
const activeTab = ref('hot')
const currentPeriod = ref(30)
const currentZone = ref('FRONT')

// Computed properties
const loading = computed(() => superLottoStore.loading)
const error = computed(() => superLottoStore.error)
const hotNumbers = computed(() => superLottoStore.hotNumbers)
const coldNumbers = computed(() => superLottoStore.coldNumbers)
const totalDraws = computed(() => superLottoStore.totalDraws)
const hasAnalysisData = computed(() => superLottoStore.hasAnalysisData)

const averageHotScore = computed(() => {
  if (hotNumbers.value.length === 0) return 0
  const sum = hotNumbers.value.reduce((acc, num) => acc + num.hot_score, 0)
  return sum / hotNumbers.value.length
})

const averageColdScore = computed(() => {
  if (coldNumbers.value.length === 0) return 0
  const sum = coldNumbers.value.reduce((acc, num) => acc + num.cold_score, 0)
  return sum / coldNumbers.value.length
})

// Methods
const analyzeHotNumbers = async () => {
  try {
    await superLottoStore.analyzeHotNumbers({
      days: currentPeriod.value,
      zone: currentZone.value,
      limit: 20
    })
  } catch (err) {
    console.error('Failed to analyze hot numbers:', err)
  }
}

const analyzeColdNumbers = async () => {
  try {
    await superLottoStore.analyzeColdNumbers({
      days: currentPeriod.value,
      zone: currentZone.value,
      limit: 20
    })
  } catch (err) {
    console.error('Failed to analyze cold numbers:', err)
  }
}

const handlePeriodChange = async (period: number) => {
  currentPeriod.value = period
  await runAnalysis()
}

const handleZoneChange = async (zone: string) => {
  currentZone.value = zone
  await runAnalysis()
}

const runAnalysis = async () => {
  await Promise.all([analyzeHotNumbers(), analyzeColdNumbers()])
}

const clearError = () => {
  superLottoStore.clearError()
}

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateString
  }
}

// Lifecycle
onMounted(async () => {
  await runAnalysis()
})

// Watch for tab changes to ensure data is loaded
watch(activeTab, async () => {
  if (!hasAnalysisData.value) {
    await runAnalysis()
  }
})
</script>

<style scoped>
.hot-cold-analysis {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.page-header {
  text-align: center;
  margin-bottom: 30px;
}

.page-header h1 {
  color: #2c3e50;
  font-size: 2.5rem;
  margin-bottom: 10px;
}

.page-description {
  color: #7f8c8d;
  font-size: 1.1rem;
}

.controls-section {
  margin-bottom: 30px;
}

.data-section {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  overflow: hidden;
}

.loading-container,
.error-container,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
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

.error-message {
  color: #e74c3c;
  text-align: center;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 20px;
}

.analysis-tabs {
  display: flex;
  border-bottom: 1px solid #ecf0f1;
}

.tab-btn {
  padding: 15px 25px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1rem;
  color: #7f8c8d;
  border-bottom: 3px solid transparent;
  transition: all 0.3s;
}

.tab-btn:hover {
  color: #3498db;
}

.tab-btn.active {
  color: #3498db;
  border-bottom-color: #3498db;
}

.tab-content {
  padding: 20px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #ecf0f1;
}

.panel-header h3 {
  color: #2c3e50;
  margin: 0;
}

.stats-info {
  color: #7f8c8d;
  font-size: 0.9rem;
}

.stats-info span {
  margin-left: 15px;
}

.number-details {
  margin-top: 30px;
}

.number-details h4 {
  color: #2c3e50;
  margin-bottom: 20px;
}

.number-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 15px;
}

.number-card {
  padding: 15px;
  border-radius: 8px;
  text-align: center;
  transition: transform 0.2s;
}

.number-card:hover {
  transform: translateY(-2px);
}

.number-card.hot {
  background: linear-gradient(135deg, #ff6b6b, #ff8e8e);
  color: white;
}

.number-card.cold {
  background: linear-gradient(135deg, #4ecdc4, #6dd5d1);
  color: white;
}

.number-value {
  font-size: 2rem;
  font-weight: bold;
  margin-bottom: 10px;
}

.number-stats {
  font-size: 0.8rem;
  opacity: 0.9;
}

.number-stats span {
  display: block;
  margin-bottom: 2px;
}

.comparison-content {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.comparison-section h4 {
  color: #2c3e50;
  margin-bottom: 15px;
}

.comparison-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 15px;
}

.stat-card {
  padding: 20px;
  border-radius: 8px;
  text-align: center;
  background: #f8f9fa;
  border: 1px solid #ecf0f1;
}

.stat-card h5 {
  color: #7f8c8d;
  margin-bottom: 10px;
  font-size: 0.9rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: bold;
  margin: 0;
}

.stat-value.hot {
  color: #e74c3c;
}

.stat-value.cold {
  color: #3498db;
}

.distribution-chart {
  background: #f8f9fa;
  border: 1px solid #ecf0f1;
  border-radius: 8px;
  padding: 40px;
  text-align: center;
  color: #7f8c8d;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.3s;
  text-decoration: none;
  display: inline-block;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover {
  background-color: #2980b9;
}

/* Responsive design */
@media (max-width: 768px) {
  .panel-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .stats-info span {
    margin: 0;
    display: block;
  }

  .analysis-tabs {
    overflow-x: auto;
  }

  .number-grid {
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  }

  .comparison-stats {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>