<template>
  <div class="performance-monitor" v-if="isDevMode && showMonitor">
    <div class="monitor-header" @click="toggleExpanded">
      <span>Performance Monitor</span>
      <el-icon>
        <Expand v-if="!expanded" />
        <Fold v-else />
      </el-icon>
    </div>

    <transition name="slide-down">
      <div v-if="expanded" class="monitor-content">
        <!-- Real-time metrics -->
        <div class="metrics-grid">
          <div class="metric-card">
            <div class="metric-label">FPS</div>
            <div class="metric-value" :class="getFpsClass(currentFps)">
              {{ currentFps.toFixed(1) }}
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-label">Memory (MB)</div>
            <div class="metric-value" :class="getMemoryClass(memoryUsage?.used || 0)">
              {{ (memoryUsage?.used / (1024 * 1024)).toFixed(1) }}
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-label">Render Time (ms)</div>
            <div class="metric-value" :class="getRenderTimeClass(avgRenderTime)">
              {{ avgRenderTime.toFixed(1) }}
            </div>
          </div>

          <div class="metric-card">
            <div class="metric-label">API Response (ms)</div>
            <div class="metric-value" :class="getApiTimeClass(avgApiResponseTime)">
              {{ avgApiResponseTime.toFixed(1) }}
            </div>
          </div>
        </div>

        <!-- Component performance breakdown -->
        <div class="section">
          <h4>Component Performance</h4>
          <div class="component-list">
            <div
              v-for="component in componentMetrics"
              :key="component.name"
              class="component-item"
            >
              <span class="component-name">{{ component.name }}</span>
              <span class="component-renders">{{ component.renderCount }} renders</span>
              <span class="component-time">{{ component.avgTime.toFixed(2) }}ms</span>
            </div>
          </div>
        </div>

        <!-- Performance warnings -->
        <div class="section" v-if="warnings.length > 0">
          <h4>Performance Warnings</h4>
          <div class="warnings">
            <div
              v-for="warning in warnings"
              :key="warning.id"
              class="warning-item"
              :class="warning.severity"
            >
              <el-icon><Warning /></el-icon>
              <span>{{ warning.message }}</span>
            </div>
          </div>
        </div>

        <!-- Performance recommendations -->
        <div class="section">
          <h4>Recommendations</h4>
          <div class="recommendations">
            <div
              v-for="rec in recommendations"
              :key="rec.id"
              class="recommendation-item"
            >
              <el-icon><InfoFilled /></el-icon>
              <span>{{ rec.message }}</span>
              <el-button size="small" @click="applyRecommendation(rec)">
                Apply
              </el-button>
            </div>
          </div>
        </div>

        <!-- Performance controls -->
        <div class="section">
          <h4>Performance Controls</h4>
          <div class="controls">
            <el-button @click="clearCache" size="small">
              Clear Performance Cache
            </el-button>
            <el-button @click="optimizeMemory" size="small" type="warning">
              Optimize Memory
            </el-button>
            <el-button @click="exportMetrics" size="small" type="primary">
              Export Metrics
            </el-button>
            <el-button @click="resetMetrics" size="small" type="danger">
              Reset All
            </el-button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { Expand, Fold, Warning, InfoFilled } from '@element-plus/icons-vue'
import { usePerformanceOptimization } from '@/utils/performance'
import { ElMessage } from 'element-plus'

interface ComponentMetric {
  name: string
  renderCount: number
  avgTime: number
  maxTime: number
  minTime: number
}

interface PerformanceWarning {
  id: string
  message: string
  severity: 'warning' | 'error' | 'info'
  timestamp: number
}

interface Recommendation {
  id: string
  message: string
  action: () => void
  priority: 'low' | 'medium' | 'high'
}

const props = defineProps<{
  showMonitor?: boolean
}>()

const isDevMode = computed(() => process.env.NODE_ENV === 'development')
const expanded = ref(true)
const showMonitor = computed(() => props.showMonitor ?? isDevMode.value)

// Performance optimization utilities
const {
  memoryOptimization,
  performanceMonitor,
  dataPrefetch,
  useMemoize
} = usePerformanceOptimization()

// Reactive state
const currentFps = ref(60)
const memoryUsage = ref<MemoryInfo | null>(null)
const avgRenderTime = ref(0)
const avgApiResponseTime = ref(0)
const componentMetrics = ref<ComponentMetric[]>([])
const warnings = ref<PerformanceWarning[]>([])
const recommendations = ref<Recommendation[]>([])

// FPS calculation
let lastFrameTime = performance.now()
let frameCount = 0
let fpsUpdateInterval: number

const calculateFPS = () => {
  const currentTime = performance.now()
  frameCount++

  if (currentTime - lastFrameTime >= 1000) {
    currentFps.value = frameCount
    frameCount = 0
    lastFrameTime = currentTime
  }

  requestAnimationFrame(calculateFPS)
}

// Memory monitoring
const updateMemoryUsage = () => {
  memoryUsage.value = memoryOptimization.getMemoryUsage()

  // Check for memory warnings
  if (memoryUsage.value) {
    const usedMB = memoryUsage.value.used / (1024 * 1024)
    const totalMB = memoryUsage.value.total / (1024 * 1024)
    const usagePercent = (usedMB / totalMB) * 100

    if (usagePercent > 80) {
      addWarning('HIGH_MEMORY', 'Memory usage is very high', 'error')
      addRecommendation('OPTIMIZE_MEMORY', 'Run memory optimization', () => optimizeMemory(), 'high')
    } else if (usagePercent > 60) {
      addWarning('MEDIUM_MEMORY', 'Memory usage is elevated', 'warning')
      addRecommendation('CLEAN_CACHE', 'Clear unused caches', () => clearCache(), 'medium')
    }
  }
}

// Component performance tracking
const trackComponentRender = (componentName: string, renderTime: number) => {
  let metric = componentMetrics.value.find(m => m.name === componentName)

  if (!metric) {
    metric = {
      name: componentName,
      renderCount: 0,
      avgTime: 0,
      maxTime: 0,
      minTime: Infinity
    }
    componentMetrics.value.push(metric)
  }

  metric.renderCount++
  metric.avgTime = (metric.avgTime * (metric.renderCount - 1) + renderTime) / metric.renderCount
  metric.maxTime = Math.max(metric.maxTime, renderTime)
  metric.minTime = Math.min(metric.minTime, renderTime)

  // Check for performance issues
  if (renderTime > 100) {
    addWarning('SLOW_RENDER', `Component ${componentName} rendered in ${renderTime.toFixed(2)}ms`, 'error')
  } else if (renderTime > 50) {
    addWarning('MODERATE_RENDER', `Component ${componentName} rendered in ${renderTime.toFixed(2)}ms`, 'warning')
  }
}

// API performance tracking
const trackApiResponse = (responseTime: number) => {
  avgApiResponseTime.value = (avgApiResponseTime.value + responseTime) / 2

  if (responseTime > 5000) {
    addWarning('SLOW_API', `API response took ${responseTime}ms`, 'error')
  } else if (responseTime > 2000) {
    addWarning('MODERATE_API', `API response took ${responseTime}ms`, 'warning')
  }
}

// Warning management
const addWarning = (id: string, message: string, severity: 'warning' | 'error' | 'info') => {
  const existing = warnings.value.find(w => w.id === id)
  if (existing) {
    existing.timestamp = Date.now()
    existing.message = message
    existing.severity = severity
  } else {
    warnings.value.push({
      id,
      message,
      severity,
      timestamp: Date.now()
    })
  }

  // Keep only recent warnings (last 50)
  if (warnings.value.length > 50) {
    warnings.value = warnings.value
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, 50)
  }
}

// Recommendation management
const addRecommendation = (
  id: string,
  message: string,
  action: () => void,
  priority: 'low' | 'medium' | 'high'
) => {
  const existing = recommendations.value.find(r => r.id === id)
  if (!existing) {
    recommendations.value.push({
      id,
      message,
      action,
      priority
    })
  }
}

// Performance controls
const clearCache = () => {
  // Clear various caches
  dataPrefetch.clearCache()
  memoryOptimization.cleanExpiredCache()

  // Reset performance metrics
  componentMetrics.value = []
  avgRenderTime.value = 0
  avgApiResponseTime.value = 0

  ElMessage.success('Performance cache cleared')
}

const optimizeMemory = () => {
  // Trigger garbage collection if available
  if (window.gc) {
    window.gc()
  }

  memoryOptimization.cleanExpiredCache()
  updateMemoryUsage()

  ElMessage.success('Memory optimization completed')
}

const exportMetrics = () => {
  const metrics = {
    timestamp: Date.now(),
    fps: currentFps.value,
    memory: memoryUsage.value,
    avgRenderTime: avgRenderTime.value,
    avgApiResponseTime: avgApiResponseTime.value,
    componentMetrics: componentMetrics.value,
    warnings: warnings.value,
    recommendations: recommendations.value
  }

  const blob = new Blob([JSON.stringify(metrics, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `performance-metrics-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)

  ElMessage.success('Performance metrics exported')
}

const resetMetrics = () => {
  currentFps.value = 60
  avgRenderTime.value = 0
  avgApiResponseTime.value = 0
  componentMetrics.value = []
  warnings.value = []
  recommendations.value = []

  ElMessage.info('Performance metrics reset')
}

const applyRecommendation = (rec: Recommendation) => {
  try {
    rec.action()
    recommendations.value = recommendations.value.filter(r => r.id !== rec.id)
    ElMessage.success('Recommendation applied')
  } catch (error) {
    ElMessage.error('Failed to apply recommendation')
  }
}

const toggleExpanded = () => {
  expanded.value = !expanded.value
}

// Utility methods for styling
const getFpsClass = (fps: number) => {
  if (fps >= 50) return 'good'
  if (fps >= 30) return 'moderate'
  return 'poor'
}

const getMemoryClass = (used: number) => {
  const percent = (used / (1024 * 1024 * 100)) * 100 // Assuming 100MB as high threshold
  if (percent <= 60) return 'good'
  if (percent <= 80) return 'moderate'
  return 'poor'
}

const getRenderTimeClass = (time: number) => {
  if (time <= 16) return 'good' // 60fps = 16ms per frame
  if (time <= 33) return 'moderate' // 30fps = 33ms per frame
  return 'poor'
}

const getApiTimeClass = (time: number) => {
  if (time <= 200) return 'good'
  if (time <= 1000) return 'moderate'
  return 'poor'
}

// Lifecycle
onMounted(() => {
  // Start FPS monitoring
  calculateFPS()

  // Update memory usage periodically
  const memoryInterval = setInterval(updateMemoryUsage, 5000)

  // Store interval references for cleanup
  fpsUpdateInterval = memoryInterval

  // Initial memory check
  updateMemoryUsage()

  // Setup global performance tracking
  setupGlobalPerformanceTracking()
})

onUnmounted(() => {
  // Cleanup intervals
  if (fpsUpdateInterval) {
    clearInterval(fpsUpdateInterval)
  }
})

// Global performance tracking
const setupGlobalPerformanceTracking = () => {
  // Track component render times using Vue devtools if available
  if (typeof window !== 'undefined' && (window as any).__VUE_DEVTOOLS_GLOBAL_HOOK__) {
    const hook = (window as any).__VUE_DEVTOOLS_GLOBAL_HOOK__

    const originalRender = hook.on.componentUpdated
    hook.on.componentUpdated = (component: any) => {
      const start = performance.now()

      if (originalRender) {
        originalRender(component)
      }

      const end = performance.now()
      const renderTime = end - start

      if (component.type?.name) {
        trackComponentRender(component.type.name, renderTime)
      }
    }
  }
}

// Expose methods for external usage
defineExpose({
  trackComponentRender,
  trackApiResponse,
  addWarning,
  addRecommendation,
  clearCache,
  optimizeMemory
})
</script>

<style scoped>
.performance-monitor {
  position: fixed;
  top: 10px;
  right: 10px;
  width: 400px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid #dcdfe6;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  z-index: 9999;
  font-size: 12px;
  backdrop-filter: blur(10px);
}

.monitor-header {
  padding: 10px;
  background: #f5f7fa;
  border-radius: 8px 8px 0 0;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 600;
}

.monitor-content {
  padding: 15px;
  max-height: 500px;
  overflow-y: auto;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  margin-bottom: 20px;
}

.metric-card {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 10px;
  text-align: center;
}

.metric-label {
  font-size: 11px;
  color: #909399;
  margin-bottom: 4px;
}

.metric-value {
  font-size: 16px;
  font-weight: 600;
}

.metric-value.good { color: #67c23a; }
.metric-value.moderate { color: #e6a23c; }
.metric-value.poor { color: #f56c6c; }

.section {
  margin-bottom: 20px;
}

.section h4 {
  margin: 0 0 10px 0;
  font-size: 13px;
  color: #303133;
  border-bottom: 1px solid #ebeef5;
  padding-bottom: 5px;
}

.component-list {
  max-height: 150px;
  overflow-y: auto;
}

.component-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #f0f0f0;
}

.component-name {
  flex: 1;
  font-weight: 500;
}

.component-renders {
  color: #909399;
  margin-right: 10px;
}

.component-time {
  color: #606266;
  font-family: monospace;
}

.warnings {
  max-height: 150px;
  overflow-y: auto;
}

.warning-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 4px;
  margin-bottom: 6px;
}

.warning-item.error {
  background: #fef0f0;
  color: #f56c6c;
  border: 1px solid #fbc4c4;
}

.warning-item.warning {
  background: #fdf6ec;
  color: #e6a23c;
  border: 1px solid #f5dab1;
}

.warning-item.info {
  background: #f4f4f5;
  color: #909399;
  border: 1px solid #e4e7ed;
}

.warning-item .el-icon {
  margin-right: 8px;
}

.recommendations {
  max-height: 200px;
  overflow-y: auto;
}

.recommendation-item {
  display: flex;
  align-items: center;
  padding: 10px;
  background: #ecf5ff;
  border: 1px solid #b3d8ff;
  border-radius: 4px;
  margin-bottom: 8px;
}

.recommendation-item .el-icon {
  margin-right: 8px;
  color: #409eff;
}

.recommendation-item span {
  flex: 1;
  margin-right: 10px;
}

.controls {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.slide-down-enter-from,
.slide-down-leave-to {
  max-height: 0;
  opacity: 0;
}

.slide-down-enter-to,
.slide-down-leave-from {
  max-height: 500px;
  opacity: 1;
}
</style>