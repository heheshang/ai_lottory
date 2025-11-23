<template>
  <div class="one-click-prediction">
    <div class="prediction-header">
      <h2>一键预测全部算法</h2>
      <p class="description">同时使用多种预测算法生成大乐透号码，提供更全面的分析结果</p>
    </div>

    <!-- Prediction Configuration -->
    <div class="configuration-section">
      <h3>预测配置</h3>
      <div class="config-grid">
        <div class="config-item">
          <label>分析周期（天）：</label>
          <el-select v-model="config.analysisPeriodDays" @change="onConfigChange">
            <el-option label="最近30天" :value="30" />
            <el-option label="最近60天" :value="60" />
            <el-option label="最近90天" :value="90" />
            <el-option label="最近180天" :value="180" />
            <el-option label="全部数据" :value="365" />
          </el-select>
        </div>

        <div class="config-item">
          <label>选择算法：</label>
          <div class="algorithm-selection">
            <el-checkbox
              v-model="selectedAlgorithms"
              :label="algorithm"
              v-for="algorithm in availableAlgorithms"
              :key="algorithm.value"
              :value="algorithm.value"
              @change="onAlgorithmChange"
            >
              {{ algorithm.label }}
            </el-checkbox>
          </div>
        </div>

        <div class="config-item">
          <el-checkbox v-model="config.includeReasoning">包含详细推理</el-checkbox>
        </div>
      </div>
    </div>

    <!-- Generate Button -->
    <div class="action-section">
      <el-button
        type="primary"
        size="large"
        :loading="isGenerating"
        :disabled="!canGenerate"
        @click="generateAllPredictions"
        class="generate-button"
      >
        <template v-if="!isGenerating">
          <span class="button-icon">🚀</span>
          一键预测全部
        </template>
        <template v-else>
          <span class="loading-icon">⚙️</span>
          正在生成预测...
        </template>
      </el-button>

      <div class="button-info" v-if="!canGenerate && !isGenerating">
        <el-alert
          title="无法生成预测"
          type="warning"
          :closable="false"
          show-icon
        >
          {{ generateWarning }}
        </el-alert>
      </div>
    </div>

    <!-- Batch Results -->
    <div v-if="batchResult" class="results-section">
      <div class="results-header">
        <h3>批量预测结果</h3>
        <div class="results-summary">
          <el-tag type="success" size="large">
            成功: {{ batchResult.successful_predictions }}
          </el-tag>
          <el-tag type="info" size="large" v-if="batchResult.failed_predictions > 0">
            失败: {{ batchResult.failed_predictions }}
          </el-tag>
          <el-tag type="primary" size="large">
            耗时: {{ batchResult.processing_time_ms }}ms
          </el-tag>
        </div>
      </div>

      <!-- Best Prediction Highlight -->
      <div v-if="bestPrediction" class="best-prediction">
        <h4>🏆 最佳预测</h4>
        <PredictionDisplay
          :prediction="bestPrediction"
          :compact="false"
          :show-details="true"
          @save-prediction="onSavePrediction"
        />
      </div>

      <!-- All Predictions Grid -->
      <div class="predictions-grid">
        <h4>所有算法结果</h4>
        <div class="grid-container">
          <div
            v-for="prediction in batchResult.predictions"
            :key="prediction.id"
            class="prediction-card"
            :class="{ 'best-prediction': isBestPrediction(prediction) }"
          >
            <div class="card-header">
              <span class="algorithm-name">{{ getAlgorithmDisplay(prediction.algorithm) }}</span>
              <el-tag
                :type="getConfidenceType(prediction.confidence_score)"
                size="small"
              >
                {{ (prediction.confidence_score * 100).toFixed(1) }}%
              </el-tag>
            </div>

            <div class="card-numbers">
              <div class="number-row">
                <span class="label">前区:</span>
                <div class="numbers front">
                  <span
                    v-for="(num, index) in prediction.front_numbers"
                    :key="`front-${index}`"
                    class="number"
                  >
                    {{ num }}
                  </span>
                </div>
              </div>
              <div class="number-row">
                <span class="label">后区:</span>
                <div class="numbers back">
                  <span
                    v-for="(num, index) in prediction.back_numbers"
                    :key="`back-${index}`"
                    class="number"
                  >
                    {{ num }}
                  </span>
                </div>
              </div>
            </div>

            <div class="card-actions">
              <el-button
                size="small"
                type="primary"
                @click="selectPrediction(prediction)"
              >
                选择
              </el-button>
              <el-button
                size="small"
                @click="viewDetails(prediction)"
              >
                详情
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- Comparison Section -->
      <div class="comparison-section" v-if="showComparison">
        <el-button
          type="text"
          @click="toggleComparison"
          class="comparison-toggle"
        >
          {{ showComparisonDetails ? '隐藏' : '显示' }}算法对比
          <el-icon class="toggle-icon">
            <component :is="showComparisonDetails ? 'arrow-up' : 'arrow-down'" />
          </el-icon>
        </el-button>

        <div v-show="showComparisonDetails" class="comparison-content">
          <h4>算法对比分析</h4>
          <div class="comparison-table">
            <el-table :data="comparisonData" style="width: 100%">
              <el-table-column prop="algorithm" label="算法" width="120" />
              <el-table-column prop="confidence" label="置信度" width="100">
                <template #default="{ row }">
                  <el-progress
                    :percentage="row.confidence * 100"
                    :color="getProgressColor(row.confidence)"
                    :show-text="true"
                  />
                </template>
              </el-table-column>
              <el-table-column prop="frontNumbers" label="前区号码" width="200" />
              <el-table-column prop="backNumbers" label="后区号码" width="100" />
              <el-table-column prop="rank" label="排名" width="80" />
              <el-table-column prop="uniqueNumbers" label="独特号码" width="100" />
            </el-table>
          </div>
        </div>
      </div>

      <!-- Consensus Analysis -->
      <div class="consensus-section">
        <el-button
          type="text"
          @click="toggleConsensus"
          class="consensus-toggle"
        >
          {{ showConsensus ? '隐藏' : '显示' }}共识分析
          <el-icon class="toggle-icon">
            <component :is="showConsensus ? 'arrow-up' : 'arrow-down'" />
          </el-icon>
        </el-button>

        <div v-show="showConsensus" class="consensus-content">
          <h4>算法共识分析</h4>
          <div class="consensus-numbers">
            <div class="consensus-group">
              <h5>前区共识号码：</h5>
              <div class="consensus-front">
                <span
                  v-for="(num, index) in consensusNumbers.front"
                  :key="`consensus-front-${index}`"
                  class="consensus-number"
                >
                  {{ num }}
                  <span class="vote-count">{{ getVoteCount(num, 'front') }}</span>
                </span>
              </div>
            </div>
            <div class="consensus-group">
              <h5>后区共识号码：</h5>
              <div class="consensus-back">
                <span
                  v-for="(num, index) in consensusNumbers.back"
                  :key="`consensus-back-${index}`"
                  class="consensus-number"
                >
                  {{ num }}
                  <span class="vote-count">{{ getVoteCount(num, 'back') }}</span>
                </span>
              </div>
            </div>
          </div>

          <div class="consensus-strength">
            <p>共识强度：{{ (consensusNumbers.strength * 100).toFixed(1) }}%</p>
            <el-progress
              :percentage="consensusNumbers.strength * 100"
              :color="getConsensusColor(consensusNumbers.strength)"
              :show-text="false"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Details Dialog -->
    <el-dialog
      v-model="detailsVisible"
      title="预测详情"
      width="80%"
      :before-close="handleDetailsClose"
    >
      <PredictionDisplay
        v-if="selectedPrediction"
        :prediction="selectedPrediction"
        :compact="false"
        :show-details="true"
      />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import PredictionDisplay from './PredictionDisplay.vue'
import { generateAllPredictions, getPredictionComparison } from '@/api/superLotto'

// Types
interface PredictionAlgorithm {
  value: string
  label: string
  description: string
}

interface PredictionConfig {
  analysisPeriodDays: number
  includeReasoning: boolean
  algorithms: string[]
}

interface BatchResult {
  id: number
  request_id: string
  predictions: any[]
  generated_at: string
  total_predictions: number
  successful_predictions: number
  failed_predictions: number
  processing_time_ms: number
  analysis_period_days: number
  sample_size: number
}

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

interface ConsensusNumbers {
  front: number[]
  back: number[]
  strength: number
  voting_details: any[]
}

// Props
interface Props {
  initialConfig?: Partial<PredictionConfig>
}

const props = withDefaults(defineProps<Props>(), {
  initialConfig: () => ({})
})

// Reactive data
const isGenerating = ref(false)
const batchResult = ref<BatchResult | null>(null)
const selectedPrediction = ref<PredictionResult | null>(null)
const detailsVisible = ref(false)
const showComparison = ref(false)
const showComparisonDetails = ref(false)
const showConsensus = ref(false)
const consensusNumbers = ref<ConsensusNumbers | null>(null)

const config = ref<PredictionConfig>({
  analysisPeriodDays: 90,
  includeReasoning: true,
  algorithms: ['WEIGHTED_FREQUENCY', 'PATTERN_BASED', 'ENSEMBLE', 'HOT_NUMBERS', 'COLD_NUMBERS'],
  ...props.initialConfig
})

const selectedAlgorithms = ref([...config.value.algorithms])

// Available algorithms
const availableAlgorithms: PredictionAlgorithm[] = [
  { value: 'WEIGHTED_FREQUENCY', label: '加权频率', description: '基于历史频率分析' },
  { value: 'PATTERN_BASED', label: '模式分析', description: '基于数字模式识别' },
  { value: 'MARKOV_CHAIN', label: '马尔可夫链', description: '基于状态转移概率' },
  { value: 'ENSEMBLE', label: '集成方法', description: '多算法综合结果' },
  { value: 'HOT_NUMBERS', label: '热号预测', description: '基于热门号码分析' },
  { value: 'COLD_NUMBERS', label: '冷号预测', description: '基于冷门号码分析' },
  { value: 'POSITION_ANALYSIS', label: '位置分析', description: '基于位置分布' }
]

// Computed properties
const canGenerate = computed(() => {
  return selectedAlgorithms.value.length > 0 && !isGenerating.value
})

const generateWarning = computed(() => {
  if (selectedAlgorithms.value.length === 0) {
    return '请至少选择一种预测算法'
  }
  return ''
})

const bestPrediction = computed(() => {
  if (!batchResult.value || !batchResult.value.predictions.length) {
    return null
  }
  return batchResult.value.predictions.reduce((best, current) =>
    current.confidence_score > best.confidence_score ? current : best
  )
})

const comparisonData = computed(() => {
  if (!batchResult.value) return []

  return batchResult.value.predictions
    .map((prediction, index) => ({
      algorithm: getAlgorithmDisplay(prediction.algorithm),
      confidence: prediction.confidence_score,
      frontNumbers: prediction.front_numbers.join('-'),
      backNumbers: prediction.back_numbers.join('-'),
      rank: index + 1,
      uniqueNumbers: calculateUniqueNumbers(prediction)
    }))
    .sort((a, b) => b.confidence - a.confidence)
})

// Methods
const onConfigChange = () => {
  config.value.algorithms = selectedAlgorithms.value
}

const onAlgorithmChange = () => {
  config.value.algorithms = selectedAlgorithms.value
}

const generateAllPredictions = async () => {
  if (!canGenerate.value) return

  isGenerating.value = true

  try {
    const request = {
      algorithms: selectedAlgorithms.value,
      analysis_period_days: config.value.analysisPeriodDays,
      include_reasoning: config.value.includeReasoning
    }

    const result = await invoke('plugin:super_lotto|generate_all_predictions', request)
    batchResult.value = result

    // Generate comparison data
    await generateComparison()

    ElMessage.success(`成功生成 ${result.successful_predictions} 个预测结果`)

  } catch (error: any) {
    console.error('Failed to generate predictions:', error)
    ElMessage.error('生成预测失败，请重试')
  } finally {
    isGenerating.value = false
  }
}

const generateComparison = async () => {
  if (!batchResult.value) return

  try {
    const comparison = await getPredictionComparison(batchResult.value.request_id)
    // Process comparison data and update consensus numbers
    updateConsensusNumbers(comparison)
  } catch (error: any) {
    console.error('Failed to generate comparison:', error)
  }
}

const updateConsensusNumbers = (comparison: any) => {
  if (comparison?.consensus_numbers) {
    consensusNumbers.value = comparison.consensus_numbers
  }
}

const selectPrediction = (prediction: PredictionResult) => {
  selectedPrediction.value = prediction
  ElMessage.success(`已选择 ${getAlgorithmDisplay(prediction.algorithm)} 的预测结果`)
}

const viewDetails = (prediction: PredictionResult) => {
  selectedPrediction.value = prediction
  detailsVisible.value = true
}

const handleDetailsClose = () => {
  detailsVisible.value = false
  selectedPrediction.value = null
}

const onSavePrediction = (prediction: PredictionResult) => {
  // Handle saving prediction
  ElMessage.success('预测已保存')
}

const toggleComparison = () => {
  showComparison.value = !showComparison.value
  if (showComparison.value) {
    showComparisonDetails.value = true
  }
}

const toggleConsensus = () => {
  showConsensus.value = !showConsensus.value
}

const isBestPrediction = (prediction: PredictionResult): boolean => {
  return bestPrediction.value?.id === prediction.id
}

const getAlgorithmDisplay = (algorithm: string): string => {
  const displayMap: Record<string, string> = {
    'WEIGHTED_FREQUENCY': '加权频率',
    'PATTERN_BASED': '模式分析',
    'MARKOV_CHAIN': '马尔可夫链',
    'ENSEMBLE': '集成方法',
    'HOT_NUMBERS': '热号预测',
    'COLD_NUMBERS': '冷号预测',
    'POSITION_ANALYSIS': '位置分析'
  }
  return displayMap[algorithm] || algorithm
}

const getConfidenceType = (confidence: number): string => {
  if (confidence >= 0.8) return 'success'
  if (confidence >= 0.6) return 'warning'
  return 'danger'
}

const getProgressColor = (confidence: number): string => {
  if (confidence >= 0.8) return '#67c23a'
  if (confidence >= 0.6) return '#e6a23c'
  return '#f56c6c'
}

const getConsensusColor = (strength: number): string => {
  if (strength >= 0.8) return '#67c23a'
  if (strength >= 0.6) return '#e6a23c'
  return '#f56c6c'
}

const calculateUniqueNumbers = (prediction: PredictionResult): number => {
  // Mock implementation - calculate unique numbers across algorithms
  return prediction.front_numbers.length + prediction.back_numbers.length
}

const getVoteCount = (number: number, zone: 'front' | 'back'): number => {
  // Mock implementation - get vote count from consensus data
  if (!consensusNumbers.value?.voting_details) return 0

  const vote = consensusNumbers.value.voting_details.find(
    (v: any) => v.number === number && v.zone === zone.toUpperCase()
  )
  return vote?.votes || 0
}

// Lifecycle
onMounted(() => {
  // Initialize component
})
</script>

<style scoped>
.one-click-prediction {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.prediction-header {
  text-align: center;
  margin-bottom: 40px;
}

.prediction-header h2 {
  color: #2c3e50;
  margin-bottom: 10px;
  font-size: 2rem;
}

.prediction-header .description {
  color: #7f8c8d;
  font-size: 1.1rem;
  margin: 0;
}

.configuration-section {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 30px;
}

.configuration-section h3 {
  color: #2c3e50;
  margin-bottom: 20px;
  font-size: 1.2rem;
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 2fr 1fr;
  gap: 20px;
  align-items: center;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-item label {
  font-weight: 500;
  color: #2c3e50;
}

.algorithm-selection {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 8px;
}

.action-section {
  text-align: center;
  margin-bottom: 40px;
}

.generate-button {
  padding: 15px 40px;
  font-size: 1.1rem;
  font-weight: 600;
  border-radius: 8px;
  transition: all 0.3s;
}

.button-icon, .loading-icon {
  margin-right: 8px;
  font-size: 1.2rem;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.button-info {
  margin-top: 15px;
  max-width: 400px;
  margin-left: auto;
  margin-right: auto;
}

.results-section {
  background: white;
  border-radius: 8px;
  padding: 30px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.results-header h3 {
  color: #2c3e50;
  margin: 0;
}

.results-summary {
  display: flex;
  gap: 10px;
}

.best-prediction {
  background: linear-gradient(135deg, #667eea, #764ba2);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 30px;
  color: white;
}

.best-prediction h4 {
  color: white;
  margin-bottom: 20px;
  text-align: center;
  font-size: 1.3rem;
}

.predictions-grid {
  margin-bottom: 30px;
}

.predictions-grid h4 {
  color: #2c3e50;
  margin-bottom: 20px;
  font-size: 1.2rem;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.prediction-card {
  background: white;
  border: 2px solid #ecf0f1;
  border-radius: 8px;
  padding: 20px;
  transition: all 0.3s;
  position: relative;
}

.prediction-card:hover {
  border-color: #3498db;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.prediction-card.best-prediction {
  border-color: #f39c12;
  background: linear-gradient(135deg, #fff9e6, #fff3cd);
}

.best-prediction::before {
  content: '🏆';
  position: absolute;
  top: -10px;
  right: 10px;
  font-size: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.algorithm-name {
  font-weight: 600;
  color: #2c3e50;
}

.card-numbers {
  margin-bottom: 15px;
}

.number-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.number-row .label {
  font-size: 0.9rem;
  color: #7f8c8d;
  min-width: 50px;
}

.numbers {
  display: flex;
  gap: 6px;
}

.numbers .number {
  min-width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.8rem;
}

.numbers.front .number {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
}

.numbers.back .number {
  background: linear-gradient(135deg, #f093fb, #f5576c);
  color: white;
}

.card-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
}

.comparison-section,
.consensus-section {
  margin-bottom: 30px;
}

.comparison-toggle,
.consensus-toggle {
  margin-bottom: 15px;
  color: #3498db;
  font-weight: 500;
}

.toggle-icon {
  margin-left: 5px;
  transition: transform 0.3s;
}

.comparison-content,
.consensus-content {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 20px;
}

.comparison-content h4,
.consensus-content h4 {
  color: #2c3e50;
  margin-bottom: 20px;
}

.consensus-numbers {
  margin-bottom: 20px;
}

.consensus-group {
  margin-bottom: 25px;
}

.consensus-group h5 {
  color: #2c3e50;
  margin-bottom: 15px;
}

.consensus-front,
.consensus-back {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.consensus-number {
  position: relative;
  padding: 8px 12px;
  background: white;
  border: 2px solid #3498db;
  border-radius: 6px;
  font-weight: 600;
  color: #2c3e50;
}

.consensus-number .vote-count {
  position: absolute;
  top: -8px;
  right: -8px;
  background: #3498db;
  color: white;
  border-radius: 50%;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7rem;
  font-weight: bold;
}

.consensus-strength {
  text-align: center;
}

.consensus-strength p {
  margin-bottom: 10px;
  font-weight: 600;
  color: #2c3e50;
}

/* Responsive design */
@media (max-width: 768px) {
  .one-click-prediction {
    padding: 15px;
  }

  .config-grid {
    grid-template-columns: 1fr;
    gap: 15px;
  }

  .results-header {
    flex-direction: column;
    gap: 15px;
    text-align: center;
  }

  .results-summary {
    flex-wrap: wrap;
    justify-content: center;
  }

  .grid-container {
    grid-template-columns: 1fr;
  }

  .comparison-content,
  .consensus-content {
    padding: 15px;
  }
}

@media (max-width: 480px) {
  .generate-button {
    padding: 12px 30px;
    font-size: 1rem;
  }

  .prediction-card {
    padding: 15px;
  }

  .card-actions {
    flex-direction: column;
  }

  .consensus-front,
  .consensus-back {
    justify-content: center;
  }
}
</style>