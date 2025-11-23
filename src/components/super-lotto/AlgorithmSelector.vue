<template>
  <div class="algorithm-selector">
    <div class="selector-container">
      <label for="algorithm-select">预测算法:</label>
      <select
        id="algorithm-select"
        v-model="selectedAlgorithm"
        @change="handleAlgorithmChange"
        :disabled="loading"
        class="algorithm-select"
      >
        <optgroup label="基础算法">
          <option value="WEIGHTED_FREQUENCY">加权频率分析</option>
          <option value="PATTERN_BASED">模式识别分析</option>
        </optgroup>
        <optgroup label="高级算法">
          <option value="MARKOV_CHAIN">马尔可夫链预测</option>
          <option value="ENSEMBLE">集成学习方法</option>
        </optgroup>
        <optgroup label="专门算法">
          <option value="HOT_NUMBERS">热号优先策略</option>
          <option value="COLD_NUMBERS">冷号回补策略</option>
          <option value="POSITION_ANALYSIS">位置模式分析</option>
        </optgroup>
      </select>

      <div class="algorithm-info" v-if="selectedAlgorithm && showInfo">
        <div class="info-header">
          <h4>{{ getAlgorithmTitle(selectedAlgorithm) }}</h4>
          <button
            @click="toggleInfo"
            class="info-toggle"
            :class="{ active: showDetails }"
          >
            {{ showDetails ? '收起' : '详情' }}
          </button>
        </div>

        <div v-if="showDetails" class="info-content">
          <div class="info-description">
            <p>{{ getAlgorithmDescription(selectedAlgorithm) }}</p>
          </div>

          <div class="info-specs">
            <div class="spec-item">
              <span class="spec-label">准确率:</span>
              <span class="spec-value">{{ getAccuracyRating(selectedAlgorithm) }}</span>
            </div>
            <div class="spec-item">
              <span class="spec-label">计算复杂度:</span>
              <span class="spec-value">{{ getComplexityRating(selectedAlgorithm) }}</span>
            </div>
            <div class="spec-item">
              <span class="spec-label">数据需求:</span>
              <span class="spec-value">{{ getDataRequirement(selectedAlgorithm) }}</span>
            </div>
            <div class="spec-item">
              <span class="spec-label">推荐场景:</span>
              <span class="spec-value">{{ getUseCase(selectedAlgorithm) }}</span>
            </div>
          </div>

          <div class="algorithm-parameters" v-if="parameters.length > 0">
            <h5>算法参数</h5>
            <div class="parameters-grid">
              <div
                v-for="param in parameters"
                :key="param.key"
                class="parameter-item"
              >
                <label :for="`param-${param.key}`">{{ param.label }}:</label>
                <input
                  :id="`param-${param.key}`"
                  v-model="param.value"
                  :type="param.type"
                  :min="param.min"
                  :max="param.max"
                  :step="param.step"
                  class="parameter-input"
                  @change="handleParameterChange"
                />
                <span class="parameter-unit">{{ param.unit }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="algorithm-comparison" v-if="showComparison">
        <button
          @click="toggleComparison"
          class="comparison-toggle"
        >
          {{ showComparisonDetails ? '隐藏对比' : '算法对比' }}
        </button>

        <div v-if="showComparisonDetails" class="comparison-content">
          <table class="comparison-table">
            <thead>
              <tr>
                <th>算法</th>
                <th>准确率</th>
                <th>速度</th>
                <th>数据需求</th>
                <th>推荐度</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="algorithm in availableAlgorithms"
                :key="algorithm.value"
                :class="{ selected: algorithm.value === selectedAlgorithm }"
                @click="selectAlgorithm(algorithm.value)"
              >
                <td>{{ algorithm.title }}</td>
                <td>{{ algorithm.accuracy }}</td>
                <td>{{ algorithm.speed }}</td>
                <td>{{ algorithm.dataNeed }}</td>
                <td>
                  <div class="rating">
                    {{ getStarRating(algorithm.rating) }}
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

// Define types locally to avoid circular dependencies
type PredictionAlgorithm = 'WEIGHTED_FREQUENCY' | 'PATTERN_BASED' | 'MARKOV_CHAIN' | 'ENSEMBLE' | 'HOT_NUMBERS' | 'COLD_NUMBERS' | 'POSITION_ANALYSIS'

interface Props {
  modelValue: PredictionAlgorithm
  loading?: boolean
  showInfo?: boolean
  showComparison?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  showInfo: true,
  showComparison: false
})

const emit = defineEmits<{
  'update:modelValue': [value: PredictionAlgorithm]
  'algorithm-change': [algorithm: PredictionAlgorithm]
  'parameter-change': [parameters: Record<string, any>]
}>()

// State
const selectedAlgorithm = ref(props.modelValue)
const showDetails = ref(false)
const showComparisonDetails = ref(false)
const algorithmParams = ref<Record<string, any>>({})

// Algorithm definitions
const availableAlgorithms = [
  {
    value: 'WEIGHTED_FREQUENCY',
    title: '加权频率分析',
    description: '基于历史频率数据，结合时间衰减因子进行加权分析，优先选择近期出现频率高的号码。',
    accuracy: '中等',
    speed: '快',
    dataNeed: '100+期',
    rating: 4
  },
  {
    value: 'PATTERN_BASED',
    title: '模式识别分析',
    description: '分析号码间的出现模式，包括奇偶分布、和值范围、连续号码等统计规律。',
    accuracy: '中等偏高',
    speed: '中等',
    dataNeed: '200+期',
    rating: 4
  },
  {
    value: 'MARKOV_CHAIN',
    title: '马尔可夫链预测',
    description: '使用马尔可夫链模型分析号码序列的转移概率，预测下一期可能的号码组合。',
    accuracy: '偏高',
    speed: '慢',
    dataNeed: '500+期',
    rating: 5
  },
  {
    value: 'ENSEMBLE',
    title: '集成学习方法',
    description: '综合多种算法的预测结果，通过加权投票机制生成最终预测，提高预测准确性。',
    accuracy: '高',
    speed: '很慢',
    dataNeed: '300+期',
    rating: 5
  },
  {
    value: 'HOT_NUMBERS',
    title: '热号优先策略',
    description: '优先选择近期最热门的号码，基于热度评分和出现趋势进行组合生成。',
    accuracy: '中等',
    speed: '快',
    dataNeed: '50+期',
    rating: 3
  },
  {
    value: 'COLD_NUMBERS',
    title: '冷号回补策略',
    description: '选择长期未出现的冷号，基于概率回归理论预测即将出现的号码。',
    accuracy: '中等偏低',
    speed: '快',
    dataNeed: '100+期',
    rating: 3
  },
  {
    value: 'POSITION_ANALYSIS',
    title: '位置模式分析',
    description: '分析每个位置上号码的出现规律，考虑号码在不同位置的概率分布。',
    accuracy: '中等偏高',
    speed: '中等',
    dataNeed: '200+期',
    rating: 4
  }
]

// Computed properties
const currentAlgorithm = computed(() => {
  return availableAlgorithms.find(alg => alg.value === selectedAlgorithm.value)
})

const parameters = computed(() => {
  switch (selectedAlgorithm.value) {
    case 'WEIGHTED_FREQUENCY':
      return [
        { key: 'timeDecay', label: '时间衰减', type: 'number', value: 0.9, min: 0.1, max: 1, step: 0.1, unit: '' },
        { key: 'minFrequency', label: '最小频率', type: 'number', value: 5, min: 1, max: 20, step: 1, unit: '次' }
      ]
    case 'ENSEMBLE':
      return [
        { key: 'hotWeight', label: '热号权重', type: 'number', value: 0.4, min: 0, max: 1, step: 0.1, unit: '' },
        { key: 'patternWeight', label: '模式权重', type: 'number', value: 0.3, min: 0, max: 1, step: 0.1, unit: '' },
        { key: 'coldWeight', label: '冷号权重', type: 'number', value: 0.3, min: 0, max: 1, step: 0.1, unit: '' }
      ]
    case 'MARKOV_CHAIN':
      return [
        { key: 'order', label: '马尔可夫阶数', type: 'number', value: 2, min: 1, max: 5, step: 1, unit: '' }
      ]
    default:
      return []
  }
})

// Methods
const getAlgorithmTitle = (algorithm: string): string => {
  const alg = availableAlgorithms.find(a => a.value === algorithm)
  return alg?.title || algorithm
}

const getAlgorithmDescription = (algorithm: string): string => {
  const alg = availableAlgorithms.find(a => a.value === algorithm)
  return alg?.description || '暂无描述'
}

const getAccuracyRating = (algorithm: string): string => {
  const alg = availableAlgorithms.find(a => a.value === algorithm)
  return alg?.accuracy || '未知'
}

const getComplexityRating = (algorithm: string): string => {
  const alg = availableAlgorithms.find(a => a.value === algorithm)
  return alg?.speed || '未知'
}

const getDataRequirement = (algorithm: string): string => {
  const alg = availableAlgorithms.find(a => a.value === algorithm)
  return alg?.dataNeed || '未知'
}

const getUseCase = (algorithm: string): string => {
  const useCases: Record<string, string> = {
    'WEIGHTED_FREQUENCY': '短期预测，关注近期趋势',
    'PATTERN_BASED': '综合分析，考虑多种模式',
    'MARKOV_CHAIN': '长期趋势预测',
    'ENSEMBLE': '高精度需求，重要预测',
    'HOT_NUMBERS': '跟随趋势，追热号策略',
    'COLD_NUMBERS': '逆向思维，博冷号策略',
    'POSITION_ANALYSIS': '位置优化，精细预测'
  }
  return useCases[algorithm] || '通用场景'
}

const getStarRating = (rating: number): string => {
  return '★'.repeat(rating) + '☆'.repeat(5 - rating)
}

const handleAlgorithmChange = () => {
  emit('update:modelValue', selectedAlgorithm.value)
  emit('algorithm-change', selectedAlgorithm.value)

  // Reset parameters for new algorithm
  algorithmParams.value = {}
  parameters.value.forEach(param => {
    algorithmParams.value[param.key] = param.value
  })
}

const handleParameterChange = () => {
  const params: Record<string, any> = {}
  parameters.value.forEach(param => {
    params[param.key] = param.value
  })
  algorithmParams.value = params
  emit('parameter-change', params)
}

const selectAlgorithm = (algorithm: string) => {
  selectedAlgorithm.value = algorithm as PredictionAlgorithm
  handleAlgorithmChange()
}

const toggleInfo = () => {
  showDetails.value = !showDetails.value
}

const toggleComparison = () => {
  showComparisonDetails.value = !showComparisonDetails.value
}

// Watch for prop changes
watch(() => props.modelValue, (newValue) => {
  selectedAlgorithm.value = newValue
})
</script>

<style scoped>
.algorithm-selector {
  width: 100%;
}

.selector-container {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.selector-container label {
  display: block;
  color: #2c3e50;
  font-weight: 500;
  margin-bottom: 10px;
  font-size: 0.9rem;
}

.algorithm-select {
  width: 100%;
  padding: 10px 15px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  font-size: 0.9rem;
  cursor: pointer;
  transition: border-color 0.3s;
}

.algorithm-select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.algorithm-select:disabled {
  background-color: #f8f9fa;
  color: #6c757d;
  cursor: not-allowed;
}

.algorithm-info {
  margin-top: 20px;
  background: #f8f9fa;
  border-radius: 6px;
  overflow: hidden;
}

.info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: #ecf0f1;
  cursor: pointer;
}

.info-header h4 {
  margin: 0;
  color: #2c3e50;
  font-size: 1rem;
}

.info-toggle {
  background: none;
  border: 1px solid #bdc3c7;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.3s;
}

.info-toggle:hover {
  border-color: #3498db;
  color: #3498db;
}

.info-toggle.active {
  background: #3498db;
  color: white;
  border-color: #3498db;
}

.info-content {
  padding: 20px;
}

.info-description {
  margin-bottom: 20px;
}

.info-description p {
  color: #2c3e50;
  line-height: 1.6;
  margin: 0;
}

.info-specs {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
  margin-bottom: 20px;
}

.spec-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: white;
  border-radius: 4px;
  border-left: 3px solid #3498db;
}

.spec-label {
  color: #7f8c8d;
  font-size: 0.8rem;
}

.spec-value {
  color: #2c3e50;
  font-weight: 600;
  font-size: 0.8rem;
}

.algorithm-parameters {
  border-top: 1px solid #ecf0f1;
  padding-top: 20px;
}

.algorithm-parameters h5 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 0.9rem;
}

.parameters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.parameter-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  background: white;
  border-radius: 4px;
  border: 1px solid #ecf0f1;
}

.parameter-item label {
  margin: 0;
  font-size: 0.8rem;
  color: #2c3e50;
  min-width: 80px;
}

.parameter-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.8rem;
  min-width: 80px;
}

.parameter-unit {
  font-size: 0.8rem;
  color: #7f8c8d;
  min-width: 30px;
}

.algorithm-comparison {
  margin-top: 20px;
  border-top: 1px solid #ecf0f1;
  padding-top: 20px;
}

.comparison-toggle {
  background: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  padding: 8px 16px;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background-color 0.3s;
}

.comparison-toggle:hover {
  background-color: #2980b9;
}

.comparison-content {
  margin-top: 15px;
  overflow-x: auto;
}

.comparison-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
}

.comparison-table th,
.comparison-table td {
  padding: 10px;
  text-align: left;
  border-bottom: 1px solid #ecf0f1;
}

.comparison-table th {
  background: #f8f9fa;
  font-weight: 600;
  color: #2c3e50;
}

.comparison-table tbody tr {
  cursor: pointer;
  transition: background-color 0.3s;
}

.comparison-table tbody tr:hover {
  background-color: #f8f9fa;
}

.comparison-table tbody tr.selected {
  background-color: #e3f2fd;
  border-left: 4px solid #3498db;
}

.rating {
  color: #f39c12;
  font-size: 0.9rem;
}

/* Responsive design */
@media (max-width: 768px) {
  .selector-container {
    padding: 15px;
  }

  .info-specs {
    grid-template-columns: 1fr;
  }

  .parameters-grid {
    grid-template-columns: 1fr;
  }

  .parameter-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 5px;
  }

  .parameter-item label {
    min-width: auto;
  }

  .parameter-input {
    width: 100%;
  }

  .comparison-table {
    font-size: 0.7rem;
  }

  .comparison-table th,
  .comparison-table td {
    padding: 8px 6px;
  }
}

@media (max-width: 480px) {
  .info-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .spec-item {
    flex-direction: column;
    gap: 5px;
  }

  .comparison-content {
    overflow-x: scroll;
  }

  .comparison-table {
    min-width: 600px;
  }
}
</style>