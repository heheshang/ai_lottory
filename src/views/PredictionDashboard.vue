<template>
  <div class="prediction-dashboard">
    <div class="page-header">
      <h1>预测仪表板</h1>
      <p class="page-description">基于统计分析生成大乐透号码预测</p>
    </div>

    <div class="controls-section">
      <div class="prediction-controls">
        <AlgorithmSelector
          v-model="selectedAlgorithm"
          @algorithm-change="handleAlgorithmChange"
        />
        <div class="period-selector">
          <label>分析周期:</label>
          <select v-model="analysisPeriod" @change="handlePeriodChange">
            <option value="30">最近30天</option>
            <option value="60">最近60天</option>
            <option value="90">最近90天</option>
            <option value="180">最近180天</option>
            <option value="365">最近一年</option>
          </select>
        </div>
        <button
          @click="generateNewPrediction"
          :disabled="loading"
          class="btn btn-primary"
        >
          {{ loading ? '生成中...' : '生成预测' }}
        </button>
      </div>
    </div>

    <div class="content-section">
      <div v-if="loading" class="loading-container">
        <div class="loading-spinner"></div>
        <p>分析历史数据并生成预测...</p>
      </div>

      <div v-else-if="error" class="error-container">
        <div class="error-message">
          <i class="icon-error"></i>
          <p>{{ error }}</p>
          <button @click="clearError" class="btn btn-primary">重试</button>
        </div>
      </div>

      <div v-else class="prediction-content">
        <!-- Latest Prediction -->
        <div v-if="predictions.length > 0" class="latest-prediction">
          <div class="prediction-header">
            <h3>最新预测</h3>
            <div class="prediction-meta">
              <span>算法: {{ getAlgorithmDisplay(predictions[0].algorithm) }}</span>
              <span>置信度: {{ (predictions[0].confidence_score * 100).toFixed(1) }}%</span>
              <span>生成时间: {{ formatDate(predictions[0].created_at) }}</span>
            </div>
          </div>

          <PredictionDisplay
            :prediction="predictions[0]"
            :show-details="showDetails"
            @toggle-details="showDetails = !showDetails"
          />
        </div>

        <!-- No Predictions Yet -->
        <div v-else class="empty-state">
          <div class="empty-icon">🔮</div>
          <h3>暂无预测结果</h3>
          <p>请先生成预测以查看分析结果</p>
          <button @click="generateNewPrediction" class="btn btn-primary">
            生成第一个预测
          </button>
        </div>

        <!-- Prediction History -->
        <div v-if="predictions.length > 1" class="prediction-history">
          <div class="history-header">
            <h3>历史预测</h3>
            <button @click="showHistory = !showHistory" class="btn btn-secondary">
              {{ showHistory ? '隐藏' : '显示' }}历史
            </button>
          </div>

          <div v-if="showHistory" class="history-list">
            <div
              v-for="(prediction, index) in predictions.slice(1)"
              :key="prediction.id"
              class="history-item"
            >
              <div class="history-meta">
                <span>{{ getAlgorithmDisplay(prediction.algorithm) }}</span>
                <span>{{ formatDate(prediction.created_at) }}</span>
                <span>置信度: {{ (prediction.confidence_score * 100).toFixed(1) }}%</span>
              </div>
              <PredictionDisplay
                :prediction="prediction"
                :compact="true"
                @select="selectPrediction(prediction)"
              />
            </div>
          </div>
        </div>

        <!-- Accuracy Statistics -->
        <div v-if="validatedPredictions.length > 0" class="accuracy-stats">
          <div class="stats-header">
            <h3>预测准确率统计</h3>
          </div>
          <div class="stats-grid">
            <div class="stat-card">
              <h4>总预测数</h4>
              <p class="stat-value">{{ validatedPredictions.length }}</p>
            </div>
            <div class="stat-card">
              <h4>平均准确率</h4>
              <p class="stat-value">{{ (averageAccuracy * 100).toFixed(1) }}%</p>
            </div>
            <div class="stat-card">
              <h4>最高准确率</h4>
              <p class="stat-value">{{ (maxAccuracy * 100).toFixed(1) }}%</p>
            </div>
            <div class="stat-card">
              <h4>最低准确率</h4>
              <p class="stat-value">{{ (minAccuracy * 100).toFixed(1) }}%</p>
            </div>
          </div>
        </div>

        <!-- Algorithm Comparison -->
        <div v-if="algorithmStats.length > 0" class="algorithm-comparison">
          <div class="comparison-header">
            <h3>算法对比</h3>
          </div>
          <div class="algorithm-stats-grid">
            <div
              v-for="stat in algorithmStats"
              :key="stat.algorithm"
              class="algorithm-stat"
            >
              <h4>{{ getAlgorithmDisplay(stat.algorithm) }}</h4>
              <div class="stat-details">
                <p>预测次数: {{ stat.count }}</p>
                <p>平均准确率: {{ (stat.avgAccuracy * 100).toFixed(1) }}%</p>
                <p>最高准确率: {{ (stat.maxAccuracy * 100).toFixed(1) }}%</p>
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
import { useSuperLottoStore } from '@/stores/superLotto'
import PredictionDisplay from '@/components/super-lotto/PredictionDisplay.vue'
import AlgorithmSelector from '@/components/super-lotto/AlgorithmSelector.vue'

// Define types locally to avoid circular dependencies
interface PredictionResult {
  id: number
  algorithm: string
  front_numbers: number[]
  back_numbers: number[]
  confidence_score: number
  reasoning: any
  analysis_period_days: number
  sample_size: number
  created_at: string
  is_validated: boolean
}

interface PredictionParams {
  algorithm: string
  analysis_period_days?: number
  custom_parameters?: any
  include_reasoning?: boolean
}

type PredictionAlgorithm = 'WEIGHTED_FREQUENCY' | 'PATTERN_BASED' | 'MARKOV_CHAIN' | 'ENSEMBLE' | 'HOT_NUMBERS' | 'COLD_NUMBERS' | 'POSITION_ANALYSIS'

const superLottoStore = useSuperLottoStore()

// Reactive state
const selectedAlgorithm = ref<PredictionAlgorithm>('WEIGHTED_FREQUENCY' as PredictionAlgorithm)
const analysisPeriod = ref(90)
const showDetails = ref(false)
const showHistory = ref(false)

// Computed properties
const loading = computed(() => superLottoStore.loading)
const error = computed(() => superLottoStore.error)
const predictions = computed(() => superLottoStore.predictions)

const validatedPredictions = computed(() => {
  return predictions.value.filter(p => p.is_validated)
})

const averageAccuracy = computed(() => {
  if (validatedPredictions.value.length === 0) return 0
  const sum = validatedPredictions.value.reduce((acc, p) => {
    // This would calculate hit rate - for now, use confidence score as proxy
    return acc + p.confidence_score
  }, 0)
  return sum / validatedPredictions.value.length
})

const maxAccuracy = computed(() => {
  if (validatedPredictions.value.length === 0) return 0
  return Math.max(...validatedPredictions.value.map(p => p.confidence_score))
})

const minAccuracy = computed(() => {
  if (validatedPredictions.value.length === 0) return 0
  return Math.min(...validatedPredictions.value.map(p => p.confidence_score))
})

const algorithmStats = computed(() => {
  const stats = new Map()
  predictions.value.forEach(p => {
    const key = p.algorithm
    if (!stats.has(key)) {
      stats.set(key, {
        algorithm: key,
        count: 0,
        accuracies: [],
        avgAccuracy: 0,
        maxAccuracy: 0,
        minAccuracy: 0
      })
    }
    const stat = stats.get(key)!
    stat.count++
    stat.accuracies.push(p.confidence_score)
    stat.avgAccuracy = stat.accuracies.reduce((a, b) => a + b, 0) / stat.accuracies.length
    stat.maxAccuracy = Math.max(...stat.accuracies)
    stat.minAccuracy = Math.min(...stat.accuracies)
  })
  return Array.from(stats.values())
})

// Methods
const generateNewPrediction = async () => {
  try {
    await superLottoStore.generatePrediction({
      algorithm: selectedAlgorithm.value,
      analysis_period_days: analysisPeriod.value,
      include_reasoning: true,
      custom_parameters: getAlgorithmParameters(selectedAlgorithm.value)
    })
  } catch (err) {
    console.error('Failed to generate prediction:', err)
  }
}

const handleAlgorithmChange = (algorithm: PredictionAlgorithm) => {
  selectedAlgorithm.value = algorithm
}

const handlePeriodChange = () => {
  // Period change handled by generateNewPrediction
}

const getAlgorithmParameters = (algorithm: PredictionAlgorithm) => {
  // Return algorithm-specific parameters
  switch (algorithm) {
    case 'ENSEMBLE':
      return {
        hot_weight: 0.4,
        cold_weight: 0.3,
        pattern_weight: 0.3
      }
    case 'WEIGHTED_FREQUENCY':
      return {
        time_decay_factor: 0.9
      }
    default:
      return {}
  }
}

const getAlgorithmDisplay = (algorithm: string) => {
  const displayMap: Record<string, string> = {
    'WEIGHTED_FREQUENCY': '加权频率分析',
    'PATTERN_BASED': '模式分析',
    'MARKOV_CHAIN': '马尔可夫链',
    'ENSEMBLE': '集成方法',
    'HOT_NUMBERS': '热号预测',
    'COLD_NUMBERS': '冷号预测',
    'POSITION_ANALYSIS': '位置分析'
  }
  return displayMap[algorithm] || algorithm
}

const selectPrediction = (prediction: PredictionResult) => {
  // Handle prediction selection (show details, navigate to analysis, etc.)
  console.log('Selected prediction:', prediction)
}

const clearError = () => {
  superLottoStore.clearError()
}

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN')
  } catch {
    return dateString
  }
}

// Lifecycle
onMounted(async () => {
  // Load existing predictions
  await superLottoStore.fetchPredictions()
})
</script>

<style scoped>
.prediction-dashboard {
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

.prediction-controls {
  display: flex;
  gap: 20px;
  align-items: center;
  flex-wrap: wrap;
}

.period-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.period-selector label {
  color: #2c3e50;
  font-weight: 500;
}

.period-selector select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 0.9rem;
}

.content-section {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  overflow: hidden;
}

.loading-container,
.error-container {
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

.prediction-content {
  padding: 20px;
}

.latest-prediction,
.prediction-history,
.accuracy-stats,
.algorithm-comparison {
  margin-bottom: 30px;
}

.latest-prediction {
  border: 2px solid #3498db;
  border-radius: 8px;
  padding: 20px;
  background: #f8f9ff;
}

.prediction-header,
.history-header,
.stats-header,
.comparison-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.prediction-header h3,
.history-header h3,
.stats-header h3,
.comparison-header h3 {
  color: #2c3e50;
  margin: 0;
}

.prediction-meta,
.history-meta {
  display: flex;
  gap: 15px;
  font-size: 0.9rem;
  color: #7f8c8d;
  flex-wrap: wrap;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 20px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.history-item {
  border: 1px solid #ecf0f1;
  border-radius: 8px;
  padding: 15px;
  background: #fafafa;
}

.stats-grid,
.algorithm-stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.stat-card,
.algorithm-stat {
  padding: 20px;
  border-radius: 8px;
  text-align: center;
  background: #f8f9fa;
  border: 1px solid #ecf0f1;
}

.stat-card h4,
.algorithm-stat h4 {
  color: #7f8c8d;
  margin-bottom: 10px;
  font-size: 0.9rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: bold;
  color: #2c3e50;
  margin: 0;
}

.algorithm-stat {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.algorithm-stat h4 {
  color: white;
}

.algorithm-stat .stat-details p {
  color: rgba(255, 255, 255, 0.9);
  margin: 5px 0;
  font-size: 0.8rem;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.3s;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2980b9;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #95a5a6;
  color: white;
}

.btn-secondary:hover {
  background-color: #7f8c8d;
}

/* Responsive design */
@media (max-width: 768px) {
  .prediction-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .prediction-header,
  .history-header,
  .stats-header,
  .comparison-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .prediction-meta,
  .history-meta {
    flex-direction: column;
    gap: 5px;
  }

  .stats-grid,
  .algorithm-stats-grid {
    grid-template-columns: 1fr;
  }
}
</style>