<template>
  <div
    ref="container"
    class="virtual-list"
    :style="{ height: containerHeight + 'px' }"
    @scroll="handleScroll"
  >
    <!-- Virtual spacer before visible items -->
    <div :style="{ height: offsetY + 'px' }"></div>

    <!-- Visible items -->
    <div
      v-for="item in visibleItems"
      :key="getItemKey(item)"
      class="virtual-list-item"
      :style="{ height: itemHeight + 'px' }"
    >
      <slot name="item" :item="item" :index="item.index"></slot>
    </div>

    <!-- Virtual spacer after visible items -->
    <div :style="{ height: totalHeight - offsetY - visibleHeight + 'px' }"></div>

    <!-- Loading indicator -->
    <div v-if="loading" class="virtual-list-loading">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && items.length === 0" class="virtual-list-empty">
      <div class="empty-icon">📊</div>
      <p>暂无数据</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { usePerformanceStore } from '@/stores/performance'

interface Props {
  // 数据列表
  items: any[]
  // 每项高度（固定高度模式）
  itemHeight?: number
  // 容器高度
  containerHeight: number
  // 是否动态计算高度
  dynamicHeight?: boolean
  // 缓冲区大小（额外渲染的项目数）
  bufferSize?: number
  // 加载状态
  loading?: boolean
  // 项目唯一标识字段
  itemKey?: string
  // 预加载阈值
  preloadThreshold?: number
}

const props = withDefaults(defineProps<Props>(), {
  itemHeight: 50,
  dynamicHeight: false,
  bufferSize: 5,
  loading: false,
  itemKey: 'id',
  preloadThreshold: 100
})

// Performance tracking
const performanceStore = usePerformanceStore()
const renderStartTime = ref(0)
const scrollDebounceTimer = ref<NodeJS.Timeout>()

// Refs
const container = ref<HTMLElement>()
const offsetY = ref(0)
const scrollPosition = ref(0)

// Computed properties
const totalHeight = computed(() => {
  if (props.dynamicHeight) {
    // 动态高度模式下，这里简化处理，实际应该测量每个项目高度
    return props.items.length * props.itemHeight
  }
  return props.items.length * props.itemHeight
})

const visibleCount = computed(() => {
  return Math.ceil(props.containerHeight / props.itemHeight) + props.bufferSize * 2
})

const startIndex = computed(() => {
  const start = Math.floor(offsetY.value / props.itemHeight) - props.bufferSize
  return Math.max(0, start)
})

const endIndex = computed(() => {
  const end = startIndex.value + visibleCount.value
  return Math.min(props.items.length - 1, end)
})

const visibleItems = computed(() => {
  const items = props.items.slice(startIndex.value, endIndex.value + 1)
  return items.map((item, index) => ({
    ...item,
    index: startIndex.value + index
  }))
})

const visibleHeight = computed(() => {
  return (endIndex.value - startIndex.value + 1) * props.itemHeight
})

// Methods
const getItemKey = (item: any) => {
  if (props.itemKey && typeof item === 'object' && item !== null) {
    return item[props.itemKey]
  }
  return item
}

const handleScroll = (event: Event) => {
  if (!container.value) return

  // Debounce scroll events to improve performance
  if (scrollDebounceTimer.value) {
    clearTimeout(scrollDebounceTimer.value)
  }

  scrollDebounceTimer.value = setTimeout(() => {
    const target = event.target as HTMLElement
    scrollPosition.value = target.scrollTop
    offsetY.value = Math.max(0, target.scrollTop)

    // Check if we need to trigger preload
    checkPreload()

    // Track scroll performance
    performanceStore.recordInteraction('virtual_list_scroll')
  }, 16) // ~60fps
}

const checkPreload = () => {
  if (!container.value) return

  const { scrollTop, scrollHeight, clientHeight } = container.value
  const threshold = props.preloadThreshold

  // Trigger preload when approaching bottom
  if (scrollTop + clientHeight >= scrollHeight - threshold) {
    emit('preload')
  }
}

const scrollToItem = (index: number) => {
  if (!container.value) return

  const targetOffset = index * props.itemHeight
  container.value.scrollTop = targetOffset
  offsetY.value = targetOffset
}

const scrollToTop = () => {
  if (!container.value) return
  container.value.scrollTop = 0
  offsetY.value = 0
}

const scrollToBottom = () => {
  if (!container.value) return
  container.value.scrollTop = totalHeight.value
}

// Performance monitoring
const trackRender = () => {
  renderStartTime.value = performance.now()

  nextTick(() => {
    const renderTime = performance.now() - renderStartTime.value
    performanceStore.recordRenderTime('virtual_list', renderTime)

    console.debug(`VirtualList rendered in ${renderTime.toFixed(2)}ms with ${visibleItems.value.length} items`)
  })
}

// Watch for data changes
watch(() => props.items.length, () => {
  trackRender()
})

watch(() => visibleItems.value.length, () => {
  trackRender()
})

// Lifecycle
onMounted(() => {
  trackRender()

  // Add resize observer for responsive container
  if (window.ResizeObserver && container.value) {
    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        // Emit resize event for parent handling
        emit('resize', {
          height: entry.contentRect.height,
          width: entry.contentRect.width
        })
      }
    })

    resizeObserver.observe(container.value)

    onUnmounted(() => {
      resizeObserver.disconnect()
    })
  }
})

onUnmounted(() => {
  if (scrollDebounceTimer.value) {
    clearTimeout(scrollDebounceTimer.value)
  }
})

// Expose methods to parent
defineExpose({
  scrollToItem,
  scrollToTop,
  scrollToBottom
})

// Events
const emit = defineEmits<{
  preload: []
  resize: [dimensions: { height: number; width: number }]
  itemClick: [item: any, index: number]
}>()
</script>

<style scoped>
.virtual-list {
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
  scrollbar-width: thin;
  scrollbar-color: #ddd transparent;
}

.virtual-list::-webkit-scrollbar {
  width: 6px;
}

.virtual-list::-webkit-scrollbar-track {
  background: transparent;
}

.virtual-list::-webkit-scrollbar-thumb {
  background: #ddd;
  border-radius: 3px;
  transition: background 0.3s;
}

.virtual-list::-webkit-scrollbar-thumb:hover {
  background: #bbb;
}

.virtual-list-item {
  display: flex;
  align-items: center;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.virtual-list-item:hover {
  background-color: #f8f9fa;
}

.virtual-list-loading,
.virtual-list-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #666;
  text-align: center;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .virtual-list-item {
    padding: 8px 12px;
    font-size: 14px;
  }

  .loading-spinner {
    width: 24px;
    height: 24px;
    border-width: 2px;
  }

  .empty-icon {
    font-size: 36px;
  }
}

/* Performance optimizations */
.virtual-list-item {
  contain: layout style paint;
  will-change: background-color;
}
</style>