<template>
  <div class="prediction-dashboard">
    <div class="page-header">
      <h1>预测仪表板</h1>
      <p class="page-description">基于统计分析生成大乐透号码预测</p>
    </div>

    <div class="controls-section">
      <div class="prediction-controls">
        <AlgorithmSelector
          v-model="selectedAlgorithm as any"
          @algorithm-change="handleAlgorithmChange"
        />
        <div class="period-selector">
          <label>分析周期:</label>
          <select v-model="analysisPeriod">
            <option v-for="period in ANALYSIS_PERIODS" :key="period.value" :value="period.value">
              {{ period.label }}
            </option>
          </select>
        </div>
        <button
          @click="generateNewPrediction"
          :disabled="loading || isGenerating"
          class="btn btn-primary"
        >
          {{ (loading || isGenerating) ? '生成中...' : '生成预测' }}
        </button>
      </div>
    </div>

    <div class="content-section">
      <LoadingSpinner v-if="loading" message="分析历史数据并生成预测..." />

      <BaseCard v-else-if="error" variant="danger">
        <div class="error-message">
          <p>{{ error }}</p>
          <button @click="clearError" class="btn btn-primary">重试</button>
        </div>
      </BaseCard>

      <div v-else class="prediction-content">
        <!-- Latest Prediction -->
        <BaseCard v-if="latestPrediction" variant="primary" class="latest-prediction">
          <template #header>
            <div>
              <h3 class="card-title">最新预测</h3>
              <div class="prediction-meta">
                <span>算法: {{ getAlgorithmName(latestPrediction.algorithm) }}</span>
                <span>置信度: {{ formatPercentage(latestPrediction.confidence_score) }}</span>
                <span>生成时间: {{ formatDate(latestPrediction.created_at) }}</span>
              </div>
            </div>
          </template>
          <PredictionDisplay
            :prediction="latestPrediction"
            :show-details="showDetails"
            @toggle-details="showDetails = !showDetails"
          />
        </BaseCard>

        <!-- No Predictions Yet -->
        <EmptyState
          v-else
          icon="🔮"
          title="暂无预测结果"
          description="请先生成预测以查看分析结果"
          action-text="生成第一个预测"
          @action="generateNewPrediction"
        />

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
              v-for="prediction in predictions.slice(1)"
              :key="prediction.id"
              class="history-item"
            >
              <div class="history-meta">
                <span>{{ getAlgorithmName(prediction.algorithm) }}</span>
                <span>{{ formatDate(prediction.created_at) }}</span>
                <span>置信度: {{ (prediction.confidence_score * 100).toFixed(1) }}%</span>
              </div>
              <PredictionDisplay
                :prediction="prediction"
                :compact="true"
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
              <h4>{{ getAlgorithmName(stat.algorithm) }}</h4>
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
import PredictionDisplay from '@/components/super-lotto/PredictionDisplay.vue'
import AlgorithmSelector from '@/components/super-lotto/AlgorithmSelector.vue'
import BaseCard from '@/components/common/BaseCard.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'

import { usePrediction } from '@/composables/usePrediction'
import { useAlgorithm } from '@/composables/useAlgorithm'
import { formatDate, formatPercentage, formatConfidence } from '@/utils/formatters'
import { ANALYSIS_PERIODS } from '@/constants/lottery'
import type { AlgorithmId, PredictionResult } from '@/types/superLotto'

// Use composables
const {
  selectedAlgorithm,
  analysisPeriod,
  isGenerating,
  predictions,
  loading,
  error,
  latestPrediction,
  validatedPredictions,
  averageAccuracy,
  bestPrediction,
  generatePrediction,
  loadPredictions,
  clearError
} = usePrediction({ autoLoad: true, defaultPeriod: 90 })

const { getAlgorithmName } = useAlgorithm()

// Local state
const showDetails = ref(false)
const showHistory = ref(false)

// Computed properties
const maxAccuracy = computed(() => {
  if (validatedPredictions.value.length === 0) return 0
  return Math.max(...validatedPredictions.value.map((p: PredictionResult) => p.confidence_score))
})

const minAccuracy = computed(() => {
  if (validatedPredictions.value.length === 0) return 0
  return Math.min(...validatedPredictions.value.map((p: PredictionResult) => p.confidence_score))
})

const algorithmStats = computed(() => {
  const stats = new Map()
  predictions.value.forEach((p: PredictionResult) => {
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
    stat.avgAccuracy = stat.accuracies.reduce((a: number, b: number) => a + b, 0) / stat.accuracies.length
    stat.maxAccuracy = Math.max(...stat.accuracies)
    stat.minAccuracy = Math.min(...stat.accuracies)
  })
  return Array.from(stats.values())
})

// Methods
const generateNewPrediction = async () => {
  const customParams = getAlgorithmParameters(selectedAlgorithm.value)
  await generatePrediction({ custom_parameters: customParams })
}

const handleAlgorithmChange = (algorithm: AlgorithmId) => {
  selectedAlgorithm.value = algorithm
}

const getAlgorithmParameters = (algorithm: string) => {
  switch (algorithm) {
    case 'ENSEMBLE':
      return { hot_weight: 0.4, cold_weight: 0.3, pattern_weight: 0.3 }
    case 'WEIGHTED_FREQUENCY':
      return { time_decay_factor: 0.9 }
    default:
      return {}
  }
}
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