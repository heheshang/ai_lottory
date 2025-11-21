<template>
  <div class="analysis-controls">
    <div class="controls-container">
      <div class="control-group">
        <label for="period-select">分析周期:</label>
        <select
          id="period-select"
          v-model="selectedPeriod"
          @change="handlePeriodChange"
          :disabled="loading"
          class="control-select"
        >
          <option value="30">最近30天</option>
          <option value="60">最近60天</option>
          <option value="90">最近90天</option>
          <option value="180">最近180天</option>
          <option value="365">最近一年</option>
          <option value="all">全部数据</option>
        </select>
      </div>

      <div class="control-group" v-if="showZoneSelect">
        <label for="zone-select">号码区域:</label>
        <select
          id="zone-select"
          v-model="selectedZone"
          @change="handleZoneChange"
          :disabled="loading"
          class="control-select"
        >
          <option value="FRONT">前区 (1-35)</option>
          <option value="BACK">后区 (1-12)</option>
          <option value="ALL">全部区域</option>
        </select>
      </div>

      <div class="control-group" v-if="showLimitSelect">
        <label for="limit-select">显示数量:</label>
        <select
          id="limit-select"
          v-model="selectedLimit"
          @change="handleLimitChange"
          :disabled="loading"
          class="control-select"
        >
          <option value="10">前10名</option>
          <option value="15">前15名</option>
          <option value="20">前20名</option>
          <option value="30">前30名</option>
        </select>
      </div>

      <div class="control-group" v-if="showAlgorithmSelect">
        <label for="algorithm-select">分析算法:</label>
        <select
          id="algorithm-select"
          v-model="selectedAlgorithm"
          @change="handleAlgorithmChange"
          :disabled="loading"
          class="control-select"
        >
          <option value="FREQUENCY">频率分析</option>
          <option value="WEIGHTED">加权频率</option>
          <option value="PATTERN">模式分析</option>
          <option value="ENSEMBLE">集成方法</option>
        </select>
      </div>

      <div class="control-actions">
        <button
          @click="runAnalysis"
          :disabled="loading || !canRunAnalysis"
          class="btn btn-primary"
        >
          {{ loading ? '分析中...' : '开始分析' }}
        </button>
        <button
          @click="resetControls"
          :disabled="loading"
          class="btn btn-secondary"
        >
          重置
        </button>
        <button
          v-if="showExportButton"
          @click="exportResults"
          :disabled="loading || !hasResults"
          class="btn btn-export"
        >
          导出结果
        </button>
      </div>
    </div>

    <div class="status-info" v-if="loading || statusMessage">
      <div v-if="loading" class="loading-status">
        <div class="loading-spinner"></div>
        <span>{{ loadingMessage }}</span>
      </div>
      <div v-else-if="statusMessage" :class="['status-message', statusType]">
        {{ statusMessage }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Props {
  loading?: boolean
  showZoneSelect?: boolean
  showLimitSelect?: boolean
  showAlgorithmSelect?: boolean
  showExportButton?: boolean
  hasResults?: boolean
  initialPeriod?: number
  initialZone?: string
  initialLimit?: number
  initialAlgorithm?: string
  loadingMessage?: string
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  showZoneSelect: true,
  showLimitSelect: false,
  showAlgorithmSelect: false,
  showExportButton: false,
  hasResults: false,
  initialPeriod: 90,
  initialZone: 'FRONT',
  initialLimit: 15,
  initialAlgorithm: 'FREQUENCY',
  loadingMessage: '分析中...'
})

const emit = defineEmits<{
  'period-change': [period: number | string]
  'zone-change': [zone: string]
  'limit-change': [limit: number]
  'algorithm-change': [algorithm: string]
  'run-analysis': [params: AnalysisParams]
  'reset': []
  'export-results': []
}>()

interface AnalysisParams {
  period: number | string
  zone: string
  limit?: number
  algorithm?: string
}

// State
const selectedPeriod = ref<number | string>(props.initialPeriod)
const selectedZone = ref(props.initialZone)
const selectedLimit = ref(props.initialLimit)
const selectedAlgorithm = ref(props.initialAlgorithm)
const statusMessage = ref('')
const statusType = ref<'info' | 'success' | 'error'>('info')

// Computed properties
const canRunAnalysis = computed(() => {
  return selectedPeriod.value && selectedZone.value
})

// Methods
const handlePeriodChange = () => {
  emit('period-change', selectedPeriod.value)
}

const handleZoneChange = () => {
  emit('zone-change', selectedZone.value)
}

const handleLimitChange = () => {
  emit('limit-change', selectedLimit.value)
}

const handleAlgorithmChange = () => {
  emit('algorithm-change', selectedAlgorithm.value)
}

const runAnalysis = () => {
  if (!canRunAnalysis.value) return

  const params: AnalysisParams = {
    period: selectedPeriod.value,
    zone: selectedZone.value,
    limit: props.showLimitSelect ? selectedLimit.value : undefined,
    algorithm: props.showAlgorithmSelect ? selectedAlgorithm.value : undefined
  }

  emit('run-analysis', params)
  showStatus('正在分析数据...', 'info')
}

const resetControls = () => {
  selectedPeriod.value = props.initialPeriod
  selectedZone.value = props.initialZone
  selectedLimit.value = props.initialLimit
  selectedAlgorithm.value = props.initialAlgorithm

  emit('reset')
  showStatus('已重置为默认设置', 'info')
}

const exportResults = () => {
  emit('export-results')
  showStatus('正在导出分析结果...', 'info')
}

const showStatus = (message: string, type: 'info' | 'success' | 'error' = 'info') => {
  statusMessage.value = message
  statusType.value = type

  // Auto-hide success messages after 3 seconds
  if (type === 'success') {
    setTimeout(() => {
      statusMessage.value = ''
    }, 3000)
  }
}

// Watch for prop changes
watch(() => props.initialPeriod, (newPeriod) => {
  selectedPeriod.value = newPeriod
})

watch(() => props.initialZone, (newZone) => {
  selectedZone.value = newZone
})

watch(() => props.initialLimit, (newLimit) => {
  selectedLimit.value = newLimit
})

watch(() => props.initialAlgorithm, (newAlgorithm) => {
  selectedAlgorithm.value = newAlgorithm
})

// Expose status method for parent components
defineExpose({
  showStatus,
  clearStatus: () => { statusMessage.value = '' }
})
</script>

<style scoped>
.analysis-controls {
  width: 100%;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.controls-container {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-group label {
  color: #2c3e50;
  font-weight: 500;
  font-size: 0.9rem;
  white-space: nowrap;
}

.control-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 0.9rem;
  min-width: 120px;
  cursor: pointer;
  transition: border-color 0.3s;
}

.control-select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.control-select:disabled {
  background-color: #f8f9fa;
  color: #6c757d;
  cursor: not-allowed;
}

.control-actions {
  display: flex;
  gap: 10px;
  margin-left: auto;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  font-weight: 500;
  transition: all 0.3s;
  white-space: nowrap;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2980b9;
  transform: translateY(-1px);
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #5a6268;
  transform: translateY(-1px);
}

.btn-export {
  background-color: #27ae60;
  color: white;
}

.btn-export:hover:not(:disabled) {
  background-color: #229954;
  transform: translateY(-1px);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.status-info {
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #ecf0f1;
}

.loading-status {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #3498db;
  font-size: 0.9rem;
}

.loading-spinner {
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

.status-message {
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
}

.status-message.info {
  background-color: #e3f2fd;
  color: #1565c0;
  border: 1px solid #bbdefb;
}

.status-message.success {
  background-color: #e8f5e8;
  color: #27ae60;
  border: 1px solid #c3e6c3;
}

.status-message.error {
  background-color: #fdeaea;
  color: #e74c3c;
  border: 1px solid #f5c6cb;
}

/* Responsive design */
@media (max-width: 1024px) {
  .controls-container {
    gap: 15px;
  }

  .control-actions {
    margin-left: 0;
    width: 100%;
    justify-content: center;
  }
}

@media (max-width: 768px) {
  .controls-container {
    flex-direction: column;
    align-items: stretch;
    gap: 15px;
  }

  .control-group {
    justify-content: space-between;
    width: 100%;
  }

  .control-select {
    flex: 1;
    min-width: auto;
  }

  .control-actions {
    flex-direction: column;
    width: 100%;
  }

  .btn {
    width: 100%;
    text-align: center;
  }
}

@media (max-width: 480px) {
  .analysis-controls {
    padding: 15px;
  }

  .control-group {
    flex-direction: column;
    align-items: flex-start;
    gap: 5px;
  }

  .control-select {
    width: 100%;
  }
}
</style>