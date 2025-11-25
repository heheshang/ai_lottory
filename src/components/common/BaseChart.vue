<template>
  <div class="base-chart" :class="{ 'base-chart--loading': loading, 'base-chart--error': error }">
    <!-- Chart header -->
    <div v-if="title || showControls" class="chart-header">
      <div v-if="title" class="chart-title">
        <h3>{{ title }}</h3>
      </div>
      <div v-if="showControls" class="chart-controls">
        <slot name="controls"></slot>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="chart-loading">
      <div class="loading-spinner"></div>
      <p>{{ loadingText }}</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="chart-error">
      <div class="error-icon">⚠️</div>
      <p class="error-message">{{ error }}</p>
      <button @click="$emit('retry')" class="retry-button" v-if="showRetry">
        重试
      </button>
    </div>

    <!-- Empty state -->
    <div v-else-if="isEmpty" class="chart-empty">
      <div class="empty-icon">{{ emptyIcon || '📊' }}</div>
      <p>{{ emptyText || '暂无数据' }}</p>
    </div>

    <!-- Chart content -->
    <div v-else class="chart-content">
      <slot></slot>
    </div>

    <!-- Chart footer -->
    <div v-if="$slots.footer" class="chart-footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { usePerformanceStore } from '@/stores/performance'

interface Props {
  title?: string
  loading?: boolean
  error?: string | null
  loadingText?: string
  emptyText?: string
  emptyIcon?: string
  showControls?: boolean
  showRetry?: boolean
  isEmpty?: boolean
  showFooter?: boolean
  chartType?: string
}

const props = withDefaults(defineProps<Props>(), {
  loadingText: '加载中...',
  showControls: false,
  showRetry: true,
  isEmpty: false,
  showFooter: false,
  chartType: 'default'
})

const emit = defineEmits<{
  retry: []
  ready: [chartType: string]
  rendered: []
  error: [error: string]
}>()

// Performance tracking
const performanceStore = usePerformanceStore()
const renderStartTime = ref(0)
const renderCount = ref(0)

// Computed properties
const hasContent = computed(() => {
  return !props.loading && !props.error && !props.isEmpty
})

// Track chart rendering performance
const trackRender = () => {
  renderCount.value++

  if (renderCount.value === 1) {
    renderStartTime.value = performance.now()
  }

  if (renderCount.value > 0) {
    const renderTime = performance.now() - renderStartTime.value
    performanceStore.recordChartRender(props.chartType || 'unknown', renderTime)
  }
}

// Watch for content changes
watch(() => hasContent.value, (newVal) => {
  if (newVal) {
    trackRender()
    emit('rendered')
  }
}, { immediate: true })

onMounted(() => {
  emit('ready', props.chartType)
})
</script>

<style scoped>
.base-chart {
  width: 100%;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  transition: all 0.3s ease;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #ecf0f1;
}

.chart-title h3 {
  color: #2c3e50;
  margin: 0;
  font-size: 1.1rem;
  font-weight: 500;
}

.chart-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.chart-loading,
.chart-error,
.chart-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  padding: 32px 20px;
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
  margin-bottom: 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.error-message,
.empty-text,
.loading-text {
  font-size: 14px;
  margin: 0 0 8px 0;
}

.retry-button {
  padding: 8px 16px;
  background: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

.retry-button:hover {
  background: #2980b9;
  transform: translateY(-1px);
}

.chart-content {
  padding: 20px;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.chart-footer {
  padding: 12px 20px;
  border-top: 1px solid #ecf0f1;
  background: #fafbfc;
}

/* State-specific styles */
.base-chart--loading {
  opacity: 0.7;
}

.base-chart--error {
  border-color: #e74c3c;
}

/* Responsive design */
@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    gap: 12px;
    text-align: center;
  }

  .chart-title h3 {
    font-size: 1rem;
  }

  .chart-controls {
    justify-content: center;
    flex-wrap: wrap;
  }

  .chart-loading,
  .chart-error,
  .chart-empty {
    min-height: 150px;
    padding: 24px 16px;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border-width: 3px;
  }

  .error-icon,
  .empty-icon {
    font-size: 36px;
  }

  .chart-content {
    padding: 16px;
  }
}
</style>