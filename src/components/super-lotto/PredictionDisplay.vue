<template>
  <div class="prediction-display" :class="{ compact }">
    <div class="prediction-container">
      <div class="prediction-header" v-if="!compact">
        <h3>预测结果</h3>
        <div class="prediction-meta">
          <span class="algorithm">{{ getAlgorithmDisplay(prediction.algorithm) }}</span>
          <span class="confidence">{{ (prediction.confidence_score * 100).toFixed(1) }}%</span>
          <span class="date">{{ formatDate(prediction.created_at) }}</span>
        </div>
      </div>

      <div class="prediction-content">
        <!-- Number Display -->
        <div class="numbers-section">
          <div class="number-group">
            <h4>前区号码</h4>
            <div class="numbers front-zone">
              <div
                v-for="(number, index) in frontNumbers"
                :key="`front-${index}`"
                class="number front"
                :class="{ 'recommended': isRecommended(number, 'front') }"
              >
                {{ number }}
              </div>
            </div>
          </div>

          <div class="number-group">
            <h4>后区号码</h4>
            <div class="numbers back-zone">
              <div
                v-for="(number, index) in backNumbers"
                :key="`back-${index}`"
                class="number back"
                :class="{ 'recommended': isRecommended(number, 'back') }"
              >
                {{ number }}
              </div>
            </div>
          </div>
        </div>

        <!-- Confidence and Details -->
        <div class="prediction-details" v-if="showDetails && !compact">
          <div class="confidence-section">
            <h4>置信度分析</h4>
            <div class="confidence-meter">
              <div class="confidence-bar">
                <div
                  class="confidence-fill"
                  :style="{ width: (prediction.confidence_score * 100) + '%' }"
                ></div>
              </div>
              <span class="confidence-value">{{ (prediction.confidence_score * 100).toFixed(1) }}%</span>
            </div>
            <div class="confidence-factors" v-if="prediction.reasoning">
              <div
                v-for="(factor, index) in confidenceFactors"
                :key="`factor-${index}`"
                class="factor-item"
              >
                <span class="factor-name">{{ factor.name }}</span>
                <span class="factor-value">{{ factor.value }}</span>
              </div>
            </div>
          </div>

          <!-- Reasoning -->
          <div class="reasoning-section" v-if="prediction.reasoning">
            <h4>预测依据</h4>
            <div class="reasoning-content">
              <p>{{ prediction.reasoning }}</p>
            </div>
          </div>

          <!-- Pattern Analysis -->
          <div class="pattern-section" v-if="prediction.pattern_analysis">
            <h4>模式分析</h4>
            <div class="patterns-grid">
              <div class="pattern-item">
                <span class="pattern-label">奇偶比:</span>
                <span class="pattern-value">{{ oddEvenRatio }}</span>
              </div>
              <div class="pattern-item">
                <span class="pattern-label">和值:</span>
                <span class="pattern-value">{{ sumValue }}</span>
              </div>
              <div class="pattern-item">
                <span class="pattern-label">连续号:</span>
                <span class="pattern-value">{{ consecutiveCount }}</span>
              </div>
              <div class="pattern-item">
                <span class="pattern-label">跨度:</span>
                <span class="pattern-value">{{ range }}</span>
              </div>
            </div>
          </div>

          <!-- Alternative Predictions -->
          <div class="alternatives-section" v-if="alternatives.length > 0">
            <h4>备选方案</h4>
            <div class="alternatives-grid">
              <div
                v-for="(alt, index) in alternatives"
                :key="`alt-${index}`"
                class="alternative-item"
                @click="$emit('select-alternative', alt)"
              >
                <div class="alternative-numbers">
                  <span class="alt-front">{{ alt.front_numbers.join('-') }}</span>
                  <span class="alt-back">{{ alt.back_numbers.join('-') }}</span>
                </div>
                <span class="alt-confidence">{{ (alt.confidence_score * 100).toFixed(1) }}%</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="prediction-actions">
          <button
            v-if="!compact"
            @click="toggleDetails"
            class="btn btn-outline"
          >
            {{ showDetails ? '隐藏详情' : '显示详情' }}
          </button>
          <button
            @click="copyToClipboard"
            class="btn btn-secondary"
            title="复制号码"
          >
            📋
          </button>
          <button
            v-if="!compact"
            @click="$emit('save-prediction', prediction)"
            class="btn btn-primary"
          >
            保存预测
          </button>
          <button
            v-if="compact"
            @click="$emit('select', prediction)"
            class="btn btn-primary"
          >
            选择
          </button>
        </div>
      </div>
    </div>

    <!-- Compact View Extra Info -->
    <div v-if="compact" class="compact-info">
      <span class="compact-algorithm">{{ getAlgorithmDisplay(prediction.algorithm) }}</span>
      <span class="compact-confidence">{{ (prediction.confidence_score * 100).toFixed(1) }}%</span>
      <span class="compact-date">{{ formatDate(prediction.created_at) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { PredictionResult } from '@/types/superLotto'

interface Props {
  prediction: PredictionResult
  compact?: boolean
  showDetails?: boolean
  alternatives?: PredictionResult[]
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
  showDetails: false,
  alternatives: () => []
})

const emit = defineEmits<{
  'toggle-details': []
  'select-alternative': [alternative: PredictionResult]
  'save-prediction': [prediction: PredictionResult]
  'select': [prediction: PredictionResult]
}>()

// State
const showDetails = ref(props.showDetails)

// Computed properties
const frontNumbers = computed(() => {
  return props.prediction.front_numbers || []
})

const backNumbers = computed(() => {
  return props.prediction.back_numbers || []
})

const oddEvenRatio = computed(() => {
  const frontOdds = frontNumbers.value.filter(n => n % 2 === 1).length
  const frontEvens = 5 - frontOdds
  return `${frontOdds}奇${frontEvens}偶`
})

const sumValue = computed(() => {
  return frontNumbers.value.reduce((sum, n) => sum + n, 0)
})

const consecutiveCount = computed(() => {
  const sorted = [...frontNumbers.value].sort((a, b) => a - b)
  let count = 0
  for (let i = 0; i < sorted.length - 1; i++) {
    if (sorted[i + 1] - sorted[i] === 1) count++
  }
  return count || '无'
})

const range = computed(() => {
  const sorted = [...frontNumbers.value].sort((a, b) => a - b)
  return sorted[sorted.length - 1] - sorted[0]
})

const confidenceFactors = computed(() => {
  if (!props.prediction.reasoning) return []

  // Extract confidence factors from reasoning (mock implementation)
  return [
    { name: '频率分析', value: '85%' },
    { name: '模式匹配', value: '78%' },
    { name: '近期趋势', value: '72%' }
  ]
})

// Methods
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

const formatDate = (dateString: string): string => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateString
  }
}

const isRecommended = (number: number, zone: 'front' | 'back'): boolean => {
  // Mock implementation - could be enhanced with actual recommendation logic
  const hotNumbers = zone === 'front'
    ? [1, 7, 12, 18, 23, 28, 33] // Mock hot front numbers
    : [1, 6, 11] // Mock hot back numbers

  return hotNumbers.includes(number)
}

const toggleDetails = () => {
  showDetails.value = !showDetails.value
  emit('toggle-details')
}

const copyToClipboard = async (): Promise<void> => {
  const text = `前区: ${frontNumbers.value.join(' ')} 后区: ${backNumbers.value.join(' ')}`

  try {
    await navigator.clipboard.writeText(text)
    // Show success message (could be enhanced with a toast)
    console.log('号码已复制到剪贴板')
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    console.log('号码已复制到剪贴板')
  }
}
</script>

<style scoped>
.prediction-display {
  background: white;
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.3s;
}

.prediction-display.compact {
  border: 1px solid #ecf0f1;
  margin-bottom: 10px;
}

.prediction-container {
  padding: 20px;
}

.prediction-display.compact .prediction-container {
  padding: 15px;
}

.prediction-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #ecf0f1;
}

.prediction-header h3 {
  color: #2c3e50;
  margin: 0;
  font-size: 1.3rem;
}

.prediction-meta {
  display: flex;
  gap: 15px;
  font-size: 0.8rem;
  color: #7f8c8d;
}

.algorithm {
  background: #e3f2fd;
  color: #1565c0;
  padding: 2px 6px;
  border-radius: 3px;
}

.confidence {
  background: #e8f5e8;
  color: #27ae60;
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 600;
}

.numbers-section {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
  justify-content: center;
}

.number-group {
  text-align: center;
}

.number-group h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
  font-weight: 500;
}

.numbers {
  display: flex;
  gap: 10px;
  justify-content: center;
}

.number {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 1rem;
  transition: all 0.3s;
  position: relative;
}

.number.front {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  border: 2px solid #5a6fd8;
}

.number.back {
  background: linear-gradient(135deg, #f093fb, #f5576c);
  color: white;
  border: 2px solid #e84393;
}

.number.recommended {
  box-shadow: 0 0 15px rgba(255, 215, 0, 0.6);
  transform: scale(1.1);
}

.number.recommended::after {
  content: '★';
  position: absolute;
  top: -8px;
  right: -5px;
  color: #f39c12;
  font-size: 12px;
}

.prediction-details {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 20px;
  margin-bottom: 20px;
}

.prediction-details h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  font-size: 1rem;
}

.confidence-meter {
  display: flex;
  align-items: center;
  gap: 15px;
  margin-bottom: 20px;
}

.confidence-bar {
  flex: 1;
  height: 8px;
  background: #ecf0f1;
  border-radius: 4px;
  overflow: hidden;
}

.confidence-fill {
  height: 100%;
  background: linear-gradient(to right, #27ae60, #2ecc71);
  transition: width 0.5s ease;
}

.confidence-value {
  font-weight: bold;
  color: #27ae60;
  min-width: 50px;
  text-align: right;
}

.confidence-factors {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 10px;
}

.factor-item {
  display: flex;
  justify-content: space-between;
  padding: 8px;
  background: white;
  border-radius: 4px;
  font-size: 0.8rem;
}

.factor-name {
  color: #7f8c8d;
}

.factor-value {
  font-weight: 600;
  color: #2c3e50;
}

.reasoning-content {
  background: white;
  padding: 15px;
  border-radius: 4px;
  border-left: 4px solid #3498db;
  color: #2c3e50;
  line-height: 1.6;
}

.patterns-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 10px;
}

.pattern-item {
  display: flex;
  justify-content: space-between;
  padding: 10px;
  background: white;
  border-radius: 4px;
  text-align: center;
}

.pattern-label {
  color: #7f8c8d;
  font-size: 0.8rem;
}

.pattern-value {
  font-weight: 600;
  color: #2c3e50;
}

.alternatives-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
}

.alternative-item {
  background: white;
  border: 1px solid #ecf0f1;
  border-radius: 6px;
  padding: 15px;
  cursor: pointer;
  transition: all 0.3s;
  text-align: center;
}

.alternative-item:hover {
  border-color: #3498db;
  transform: translateY(-2px);
}

.alternative-numbers {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 10px;
}

.alt-front {
  color: #2c3e50;
  font-weight: 600;
}

.alt-back {
  color: #e74c3c;
  font-size: 0.9rem;
}

.alt-confidence {
  background: #e3f2fd;
  color: #1565c0;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 0.8rem;
}

.prediction-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.3s;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover {
  background-color: #2980b9;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background-color: #5a6268;
}

.btn-outline {
  background-color: transparent;
  color: #3498db;
  border: 1px solid #3498db;
}

.btn-outline:hover {
  background-color: #3498db;
  color: white;
}

.compact-info {
  background: #f8f9fa;
  padding: 10px 15px;
  border-top: 1px solid #ecf0f1;
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  color: #7f8c8d;
}

.compact-algorithm {
  background: #e3f2fd;
  color: #1565c0;
  padding: 1px 4px;
  border-radius: 2px;
}

.compact-confidence {
  background: #e8f5e8;
  color: #27ae60;
  padding: 1px 4px;
  border-radius: 2px;
  font-weight: 600;
}

/* Responsive design */
@media (max-width: 768px) {
  .prediction-container {
    padding: 15px;
  }

  .numbers-section {
    flex-direction: column;
    gap: 20px;
  }

  .numbers {
    gap: 8px;
  }

  .number {
    width: 35px;
    height: 35px;
    font-size: 0.9rem;
  }

  .prediction-meta {
    flex-direction: column;
    gap: 5px;
    text-align: right;
  }

  .confidence-factors {
    grid-template-columns: 1fr;
  }

  .patterns-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .alternatives-grid {
    grid-template-columns: 1fr;
  }

  .prediction-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
    text-align: center;
  }
}

@media (max-width: 480px) {
  .number {
    width: 30px;
    height: 30px;
    font-size: 0.8rem;
  }

  .pattern-item {
    padding: 8px;
  }

  .alternative-item {
    padding: 10px;
  }
}
</style>
