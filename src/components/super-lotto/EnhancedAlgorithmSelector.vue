<!-- Enhanced Algorithm Selector Component -->
<!-- Demonstrates improved component architecture with proper typing, error handling, and reusability -->

<template>
  <div class="enhanced-algorithm-selector" :class="{ 'is-disabled': disabled }">
    <div class="selector-container">
      <!-- Header -->
      <div class="selector-header">
        <label for="algorithm-select" class="selector-label">
          {{ label }}
          <span v-if="required" class="required-indicator">*</span>
        </label>
        <button
          v-if="showInfo"
          @click="toggleInfo"
          class="info-toggle"
          :class="{ 'is-active': showDetails }"
          type="button"
          :aria-expanded="showDetails"
          :aria-controls="`${id}-info`"
        >
          <span class="icon">ℹ️</span>
          <span class="text">{{ showDetails ? '隐藏详情' : '算法详情' }}</span>
        </button>
      </div>

      <!-- Algorithm Select -->
      <div class="selector-control">
        <select
          :id="id"
          v-model="selectedAlgorithmId"
          @change="handleAlgorithmChange"
          :disabled="disabled || loading"
          class="algorithm-select"
          :class="{
            'has-error': hasError,
            'is-loading': loading
          }"
          :aria-describedby="ariaDescribedBy"
        >
          <option value="" disabled>{{ placeholder }}</option>
          <optgroup
            v-for="category in algorithmCategories"
            :key="category.key"
            :label="category.label"
          >
            <option
              v-for="algorithm in category.algorithms"
              :key="algorithm.id"
              :value="algorithm.id"
              :disabled="!algorithm.enabled"
            >
              {{ algorithm.name }} {{ !algorithm.enabled ? '(禁用)' : '' }}
            </option>
          </optgroup>
        </select>

        <!-- Loading Indicator -->
        <div v-if="loading" class="loading-indicator" aria-hidden="true">
          <div class="spinner"></div>
        </div>

        <!-- Error Indicator -->
        <div v-if="hasError" class="error-indicator" aria-hidden="true">
          <span class="icon">⚠️</span>
        </div>
      </div>

      <!-- Error Message -->
      <div v-if="hasError" :id="`${id}-error`" class="error-message" role="alert">
        {{ errorMessage }}
      </div>

      <!-- Help Text -->
      <div v-if="helpText" :id="`${id}-help`" class="help-text">
        {{ helpText }}
      </div>

      <!-- Algorithm Details -->
      <div
        v-if="selectedAlgorithm && showDetails"
        :id="`${id}-info`"
        class="algorithm-details"
        role="region"
        :aria-label="`${selectedAlgorithm.name} 算法详情`"
      >
        <div class="details-header">
          <h4 class="algorithm-name">{{ selectedAlgorithm.name }}</h4>
          <div class="algorithm-meta">
            <span class="algorithm-category">{{ getCategoryLabel(selectedAlgorithm.category) }}</span>
            <span class="algorithm-complexity">{{ getComplexityLabel(selectedAlgorithm.complexity) }}</span>
          </div>
        </div>

        <div class="details-content">
          <div class="description-section">
            <h5>算法描述</h5>
            <p class="description">{{ selectedAlgorithm.description }}</p>
          </div>

          <div class="specs-section">
            <h5>算法规格</h5>
            <div class="specs-grid">
              <div class="spec-item">
                <span class="spec-label">准确率:</span>
                <div class="spec-value">
                  <span class="accuracy-score">{{ formatAccuracy(selectedAlgorithm.performance_stats.average_accuracy) }}</span>
                  <div class="accuracy-bar">
                    <div
                      class="accuracy-fill"
                      :style="{ width: `${selectedAlgorithm.performance_stats.average_accuracy}%` }"
                    ></div>
                  </div>
                </div>
              </div>

              <div class="spec-item">
                <span class="spec-label">预测次数:</span>
                <span class="spec-value">{{ selectedAlgorithm.performance_stats.total_predictions }}</span>
              </div>

              <div class="spec-item">
                <span class="spec-label">可靠性评分:</span>
                <div class="reliability-rating">
                  <span
                    v-for="i in 5"
                    :key="i"
                    class="reliability-star"
                    :class="{ 'is-active': i <= Math.round(selectedAlgorithm.performance_stats.reliability_score / 20) }"
                  >
                    ★
                  </span>
                </div>
              </div>

              <div class="spec-item">
                <span class="spec-label">数据需求:</span>
                <span class="spec-value">{{ formatDataRequirement(selectedAlgorithm.data_requirements) }}</span>
              </div>
            </div>
          </div>

          <!-- Parameters Section -->
          <div v-if="algorithmParameters.length > 0" class="parameters-section">
            <h5>算法参数</h5>
            <div class="parameters-grid">
              <div
                v-for="parameter in algorithmParameters"
                :key="parameter.key"
                class="parameter-item"
                :class="{ 'has-error': getParameterError(parameter.key) }"
              >
                <label :for="`${id}-param-${parameter.key}`" class="parameter-label">
                  {{ parameter.label }}
                  <span v-if="parameter.required" class="required-indicator">*</span>
                </label>

                <div class="parameter-control">
                  <input
                    v-if="parameter.type === 'number'"
                    :id="`${id}-param-${parameter.key}`"
                    v-model.number="parameterValues[parameter.key]"
                    type="number"
                    :min="parameter.min"
                    :max="parameter.max"
                    :step="parameter.step"
                    class="parameter-input"
                    @change="handleParameterChange(parameter)"
                    :disabled="disabled"
                  />

                  <input
                    v-else-if="parameter.type === 'boolean'"
                    :id="`${id}-param-${parameter.key}`"
                    v-model.boolean="parameterValues[parameter.key]"
                    type="checkbox"
                    class="parameter-checkbox"
                    @change="handleParameterChange(parameter)"
                    :disabled="disabled"
                  />

                  <input
                    v-else
                    :id="`${id}-param-${parameter.key}`"
                    v-model="parameterValues[parameter.key]"
                    type="text"
                    class="parameter-input"
                    @change="handleParameterChange(parameter)"
                    :disabled="disabled"
                  />

                  <span v-if="parameter.type === 'number'" class="parameter-unit">
                    {{ getParameterUnit(parameter.key) }}
                  </span>
                </div>

                <div v-if="parameter.description" class="parameter-help">
                  {{ parameter.description }}
                </div>

                <div v-if="getParameterError(parameter.key)" class="parameter-error">
                  {{ getParameterError(parameter.key) }}
                </div>
              </div>
            </div>
          </div>

          <!-- Performance Stats -->
          <div v-if="selectedAlgorithm.performance_stats.total_predictions > 0" class="performance-section">
            <h5>性能统计</h5>
            <div class="performance-grid">
              <div class="performance-item">
                <span class="performance-label">最佳准确率:</span>
                <span class="performance-value">{{ formatAccuracy(selectedAlgorithm.performance_stats.best_accuracy) }}</span>
              </div>

              <div class="performance-item">
                <span class="performance-label">最近表现:</span>
                <span class="performance-value">{{ formatAccuracy(selectedAlgorithm.performance_stats.recent_performance) }}</span>
              </div>

              <div class="performance-item">
                <span class="performance-label">验证预测:</span>
                <span class="performance-value">{{ getValidatedPredictionsCount(selectedAlgorithm.id) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  watch,
  onMounted,
  type PropType,
  type ComponentObject,
  type InjectionKey
} from 'vue'

import type {
  AlgorithmId,
  AlgorithmConfig,
  AlgorithmParameter,
  PredictionParameters,
  AlgorithmCategory,
  ComplexityLevel
} from '@/types/superLotto'

import { useSuperLottoStore } from '@/stores/superLotto'
import { useErrorHandler } from '@/utils/errorHandler'

// =============================================================================
// Props Definition
// =============================================================================

interface Props {
  modelValue?: AlgorithmId | ''
  label?: string
  placeholder?: string
  helpText?: string
  required?: boolean
  disabled?: boolean
  loading?: boolean
  showInfo?: boolean
  enableParameters?: boolean
  showPerformance?: boolean
  categories?: AlgorithmCategory[]
  error?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  label: '选择预测算法',
  placeholder: '请选择算法',
  required: false,
  disabled: false,
  loading: false,
  showInfo: true,
  enableParameters: true,
  showPerformance: true,
  categories: () => ['statistical', 'machine_learning', 'hybrid', 'ensemble'] as AlgorithmCategory[],
  error: ''
})

// =============================================================================
// Emits Definition
// =============================================================================

interface Emits {
  'update:modelValue': [value: AlgorithmId | '']
  'algorithm-change': [algorithm: AlgorithmConfig]
  'parameter-change': [parameters: PredictionParameters]
  'error': [error: string]
  'validation-change': [isValid: boolean]
}

const emit = defineEmits<Emits>()

// =============================================================================
// Dependencies
// =============================================================================

const store = useSuperLottoStore()
const { handleError } = useErrorHandler()

// =============================================================================
// Component State
// =============================================================================

const id = ref(`algorithm-selector-${crypto.randomUUID().slice(0, 8)}`)
const selectedAlgorithmId = ref<AlgorithmId | ''>(props.modelValue)
const showDetails = ref(false)
const parameterValues = ref<Record<string, any>>({})
const parameterErrors = ref<Record<string, string>>({})

// =============================================================================
// Computed Properties
// =============================================================================

const selectedAlgorithm = computed<AlgorithmConfig | null>(() => {
  if (!selectedAlgorithmId.value) return null
  return store.algorithms.find(alg => alg.id === selectedAlgorithmId.value) || null
})

const availableAlgorithms = computed<AlgorithmConfig[]>(() => {
  return store.algorithms.filter(algorithm =>
    props.categories.includes(algorithm.category) && algorithm.enabled
  )
})

const algorithmCategories = computed(() => {
  const categories = new Map<AlgorithmCategory, {
    key: AlgorithmCategory
    label: string
    algorithms: AlgorithmConfig[]
  }>()

  props.categories.forEach(category => {
    const algorithms = availableAlgorithms.value.filter(alg => alg.category === category)
    if (algorithms.length > 0) {
      categories.set(category, {
        key: category,
        label: getCategoryLabel(category),
        algorithms
      })
    }
  })

  return Array.from(categories.values())
})

const algorithmParameters = computed<AlgorithmParameter[]>(() => {
  if (!selectedAlgorithm.value || !props.enableParameters) return []
  return selectedAlgorithm.value.parameters
})

const hasError = computed(() => !!props.error || Object.keys(parameterErrors.value).length > 0)
const errorMessage = computed(() => {
  if (props.error) return props.error
  const errors = Object.values(parameterErrors.value)
  return errors.length > 0 ? errors[0] : ''
})

const ariaDescribedBy = computed(() => {
  const descriptors = []
  if (props.helpText) descriptors.push(`${id.value}-help`)
  if (hasError.value) descriptors.push(`${id.value}-error`)
  if (showDetails.value) descriptors.push(`${id.value}-info`)
  return descriptors.join(' ')
})

const isValid = computed(() => {
  return !hasError.value && !!selectedAlgorithmId.value
})

// =============================================================================
// Methods
// =============================================================================

const getCategoryLabel = (category: AlgorithmCategory): string => {
  const labels: Record<AlgorithmCategory, string> = {
    'statistical': '统计算法',
    'machine_learning': '机器学习',
    'hybrid': '混合算法',
    'ensemble': '集成算法'
  }
  return labels[category] || category
}

const getComplexityLabel = (complexity: ComplexityLevel): string => {
  const labels: Record<ComplexityLevel, string> = {
    'low': '低复杂度',
    'medium': '中等复杂度',
    'high': '高复杂度',
    'very_high': '极高复杂度'
  }
  return labels[complexity] || complexity
}

const formatAccuracy = (accuracy: number): string => {
  return `${accuracy.toFixed(1)}%`
}

const formatDataRequirement = (requirement: any): string => {
  if (!requirement) return '未知'
  const { min_draws, recommended_draws } = requirement
  return `${min_draws}-${recommended_draws}期数据`
}

const getParameterUnit = (key: string): string => {
  const units: Record<string, string> = {
    'draw_count': '期',
    'weight_factor': '',
    'time_decay_factor': '',
    'pattern_weight': '',
    'hot_weight': '',
    'cold_weight': ''
  }
  return units[key] || ''
}

const getParameterError = (key: string): string => {
  return parameterErrors.value[key] || ''
}

const getValidatedPredictionsCount = (algorithmId: AlgorithmId): number => {
  return store.validatedPredictions.filter(p => p.algorithm_id === algorithmId).length
}

const toggleInfo = () => {
  showDetails.value = !showDetails.value
}

const handleAlgorithmChange = () => {
  emit('update:modelValue', selectedAlgorithmId.value)

  if (selectedAlgorithm.value) {
    emit('algorithm-change', selectedAlgorithm.value)

    // Initialize parameter values
    initializeParameterValues()

    // Clear previous parameter errors
    parameterErrors.value = {}
  }

  // Emit validation change
  emit('validation-change', isValid.value)
}

const initializeParameterValues = () => {
  if (!selectedAlgorithm.value) return

  const values: Record<string, any> = {}
  selectedAlgorithm.value.parameters.forEach(param => {
    if (param.type === 'boolean') {
      values[param.key] = param.default_value === true
    } else if (param.type === 'number') {
      values[param.key] = Number(param.default_value) || 0
    } else {
      values[param.key] = param.default_value || ''
    }
  })

  parameterValues.value = values
}

const handleParameterChange = (parameter: AlgorithmParameter) => {
  const value = parameterValues.value[parameter.key]

  // Validate parameter
  validateParameter(parameter, value)

  // Emit parameter change event
  if (selectedAlgorithm.value) {
    const parameters: PredictionParameters = {
      ...selectedAlgorithm.value.default_parameters,
      ...parameterValues.value
    }
    emit('parameter-change', parameters)
  }

  // Emit validation change
  emit('validation-change', isValid.value)
}

const validateParameter = (parameter: AlgorithmParameter, value: any): void => {
  const errors: Record<string, string> = {}

  // Required validation
  if (parameter.required && (value === undefined || value === null || value === '')) {
    errors[parameter.key] = `${parameter.label} 是必填项`
  }

  // Type-specific validation
  if (value !== undefined && value !== null && value !== '') {
    if (parameter.type === 'number') {
      const numValue = Number(value)

      if (isNaN(numValue)) {
        errors[parameter.key] = `${parameter.label} 必须是数字`
      } else {
        if (parameter.min !== undefined && numValue < parameter.min) {
          errors[parameter.key] = `${parameter.label} 不能小于 ${parameter.min}`
        }

        if (parameter.max !== undefined && numValue > parameter.max) {
          errors[parameter.key] = `${parameter.label} 不能大于 ${parameter.max}`
        }
      }
    }
  }

  // Update errors
  if (errors[parameter.key]) {
    parameterErrors.value[parameter.key] = errors[parameter.key]
  } else {
    delete parameterErrors.value[parameter.key]
  }
}

const validate = (): boolean => {
  if (!selectedAlgorithmId.value) return false

  // Validate all parameters
  algorithmParameters.value.forEach(parameter => {
    handleParameterChange(parameter)
  })

  return isValid.value
}

const reset = () => {
  selectedAlgorithmId.value = ''
  parameterValues.value = {}
  parameterErrors.value = {}
  showDetails.value = false
}

// =============================================================================
// Watchers
// =============================================================================

watch(
  () => props.modelValue,
  (newValue) => {
    selectedAlgorithmId.value = newValue
  },
  { immediate: true }
)

watch(
  () => props.error,
  (newError) => {
    if (newError && !Object.keys(parameterErrors.value).length) {
      // Clear parameter errors if prop error is set
      parameterErrors.value = {}
    }
  }
)

// =============================================================================
// Lifecycle
// =============================================================================

onMounted(async () => {
  try {
    // Load algorithms if not already loaded
    if (store.algorithms.length === 0) {
      // This would typically be handled by the store initialization
      // For now, we'll use mock data if needed
      console.log('AlgorithmSelector mounted')
    }
  } catch (error) {
    const errorInfo = handleError(error as Error)
    emit('error', errorInfo.message)
  }
})

// =============================================================================
// Expose Component Interface
// =============================================================================

defineExpose({
  validate,
  reset,
  isValid,
  selectedAlgorithm,
  parameterValues
})
</script>

<style scoped>
.enhanced-algorithm-selector {
  width: 100%;
  max-width: 600px;
}

.enhanced-algorithm-selector.is-disabled {
  opacity: 0.6;
  pointer-events: none;
}

.selector-container {
  background: white;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.selector-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.selector-label {
  display: flex;
  align-items: center;
  font-weight: 500;
  color: #2c3e50;
  font-size: 0.9rem;
}

.required-indicator {
  color: #e74c3c;
  margin-left: 4px;
}

.info-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 0.8rem;
  color: #666;
  transition: all 0.2s ease;
}

.info-toggle:hover {
  border-color: #3498db;
  color: #3498db;
}

.info-toggle.is-active {
  background: #3498db;
  color: white;
  border-color: #3498db;
}

.selector-control {
  position: relative;
}

.algorithm-select {
  width: 100%;
  padding: 10px 12px;
  border: 2px solid #ddd;
  border-radius: 6px;
  background: white;
  font-size: 0.9rem;
  cursor: pointer;
  transition: border-color 0.2s ease;
}

.algorithm-select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.algorithm-select.has-error {
  border-color: #e74c3c;
}

.algorithm-select.is-loading {
  padding-right: 36px;
}

.loading-indicator,
.error-indicator {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-indicator {
  color: #e74c3c;
}

.error-message {
  color: #e74c3c;
  font-size: 0.8rem;
  margin-top: 4px;
}

.help-text {
  color: #666;
  font-size: 0.8rem;
  margin-top: 4px;
}

.algorithm-details {
  margin-top: 20px;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #ecf0f1;
}

.details-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.algorithm-name {
  margin: 0 0 8px 0;
  color: #2c3e50;
  font-size: 1.2rem;
}

.algorithm-meta {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.algorithm-category,
.algorithm-complexity {
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 0.7rem;
  font-weight: 500;
}

.algorithm-category {
  background: #e3f2fd;
  color: #1976d2;
}

.algorithm-complexity {
  background: #f3e5f5;
  color: #7b1fa2;
}

.details-content > div {
  margin-bottom: 20px;
}

.details-content > div:last-child {
  margin-bottom: 0;
}

.details-content h5 {
  margin: 0 0 12px 0;
  color: #2c3e50;
  font-size: 1rem;
  font-weight: 600;
}

.description {
  line-height: 1.6;
  color: #555;
}

.specs-grid,
.parameters-grid,
.performance-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
}

.spec-item,
.parameter-item,
.performance-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.spec-label,
.parameter-label,
.performance-label {
  font-size: 0.8rem;
  color: #666;
  font-weight: 500;
}

.spec-value {
  display: flex;
  align-items: center;
  gap: 8px;
}

.accuracy-score {
  font-weight: 600;
  color: #27ae60;
  min-width: 50px;
}

.accuracy-bar {
  flex: 1;
  height: 6px;
  background: #ecf0f1;
  border-radius: 3px;
  overflow: hidden;
  min-width: 60px;
}

.accuracy-fill {
  height: 100%;
  background: linear-gradient(to right, #27ae60, #2ecc71);
  transition: width 0.3s ease;
}

.reliability-rating {
  display: flex;
  gap: 2px;
}

.reliability-star {
  color: #ddd;
  font-size: 14px;
}

.reliability-star.is-active {
  color: #f39c12;
}

.parameter-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.parameter-input,
.parameter-checkbox {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
}

.parameter-input:focus,
.parameter-checkbox:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.parameter-item.has-error .parameter-input,
.parameter-item.has-error .parameter-checkbox {
  border-color: #e74c3c;
}

.parameter-unit {
  font-size: 0.8rem;
  color: #666;
  min-width: 30px;
}

.parameter-help {
  font-size: 0.7rem;
  color: #888;
  line-height: 1.4;
}

.parameter-error {
  font-size: 0.8rem;
  color: #e74c3c;
}

.performance-value {
  font-weight: 600;
  color: #2c3e50;
}

/* Responsive Design */
@media (max-width: 768px) {
  .selector-container {
    padding: 12px;
  }

  .specs-grid,
  .parameters-grid,
  .performance-grid {
    grid-template-columns: 1fr;
  }

  .details-header {
    flex-direction: column;
    gap: 12px;
  }

  .algorithm-meta {
    justify-content: flex-start;
  }
}

@media (max-width: 480px) {
  .info-toggle .text {
    display: none;
  }

  .algorithm-select {
    font-size: 0.8rem;
  }
}
</style>