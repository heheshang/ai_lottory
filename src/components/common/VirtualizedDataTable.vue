<template>
  <div class="virtualized-data-table" ref="containerRef">
    <!-- Virtual scrolling container -->
    <div
      class="virtual-scroll-container"
      :style="{ height: `${containerHeight}px`, overflow: 'auto' }"
      @scroll="handleScroll"
    >
      <!-- Spacer before visible items -->
      <div :style="{ height: `${offsetY}px`, position: 'relative' }">
        <!-- Visible items -->
        <div
          v-for="item in visibleItems"
          :key="getItemKey(item)"
          :style="{
            position: 'absolute',
            top: `${item.offset}px`,
            width: '100%',
            height: `${itemHeight}px`
          }"
          class="virtual-item"
        >
          <slot name="item" :item="item.item" :index="item.index">
            <div class="default-item">
              {{ item.item }}
            </div>
          </slot>
        </div>
      </div>
    </div>

    <!-- Loading indicator -->
    <div v-if="loading" class="loading-overlay">
      <el-loading :fullscreen="false" />
    </div>

    <!-- Performance indicator (dev mode) -->
    <div v-if="showPerformance" class="performance-indicator">
      <span>Visible: {{ visibleItems.length }}</span>
      <span>Total: {{ totalItems }}</span>
      <span>Render time: {{ renderTime }}ms</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  watch,
  nextTick,
  type Ref,
  type ComputedRef
} from 'vue'
import { useThrottle, useDebounce } from '@/utils/performance'

interface Props {
  items: any[]
  itemHeight: number
  containerHeight: number
  buffer?: number
  overscan?: number
  loading?: boolean
  getItemKey?: (item: any, index: number) => string | number
  showPerformance?: boolean
}

interface VirtualItem {
  item: any
  index: number
  offset: number
}

const props = withDefaults(defineProps<Props>(), {
  buffer: 10,
  overscan: 5,
  loading: false,
  showPerformance: process.env.NODE_ENV === 'development',
  getItemKey: (item: any, index: number) => index
})

const emit = defineEmits<{
  scroll: [{ scrollTop: number, scrollLeft: number }]
  visibleChange: [{ start: number, end: number, visibleCount: number }]
}>()

// Reactive state
const containerRef: Ref<HTMLElement | null>(ref(null))
const scrollTop = ref(0)
const renderTime = ref(0)

// Computed properties
const totalHeight: ComputedRef<number> = computed(() => {
  return props.items.length * props.itemHeight
})

const visibleCount: ComputedRef<number> = computed(() => {
  return Math.ceil(props.containerHeight / props.itemHeight)
})

const startIndex: ComputedRef<number> = computed(() => {
  const index = Math.floor(scrollTop.value / props.itemHeight)
  return Math.max(0, index - props.overscan)
})

const endIndex: ComputedRef<number> = computed(() => {
  const index = startIndex.value + visibleCount.value + props.overscan * 2
  return Math.min(props.items.length, index)
})

const offsetY: ComputedRef<number> = computed(() => {
  return startIndex.value * props.itemHeight
})

const visibleItems: ComputedRef<VirtualItem[]> = computed(() => {
  const startRender = performance.now()

  const items = props.items.slice(startIndex.value, endIndex.value).map((item, index) => ({
    item,
    index: startIndex.value + index,
    offset: (startIndex.value + index) * props.itemHeight
  }))

  const endRender = performance.now()
  renderTime.value = Math.round(endRender - startRender)

  return items
})

// Methods
const handleScroll = useThrottle((event: Event) => {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop

  emit('scroll', {
    scrollTop: target.scrollTop,
    scrollLeft: target.scrollLeft
  })

  emit('visibleChange', {
    start: startIndex.value,
    end: endIndex.value,
    visibleCount: visibleItems.value.length
  })
}, 16) // ~60fps

const scrollToItem = (index: number, alignment: 'start' | 'center' | 'auto' = 'auto') => {
  if (!containerRef.value || index < 0 || index >= props.items.length) return

  const container = containerRef.value
  const itemTop = index * props.itemHeight

  let scrollPosition: number
  switch (alignment) {
    case 'start':
      scrollPosition = itemTop
      break
    case 'center':
      scrollPosition = itemTop - (props.containerHeight - props.itemHeight) / 2
      break
    case 'auto':
    default:
      const currentScrollTop = container.scrollTop
      const currentBottom = currentScrollTop + props.containerHeight

      if (itemTop < currentScrollTop) {
        scrollPosition = itemTop
      } else if (itemTop + props.itemHeight > currentBottom) {
        scrollPosition = itemTop + props.itemHeight - props.containerHeight
      } else {
        return // Item is already visible
      }
      break
  }

  container.scrollTo({
    top: Math.max(0, scrollPosition),
    behavior: 'smooth'
  })
}

const scrollToTop = () => scrollToItem(0, 'start')
const scrollToBottom = () => scrollToItem(props.items.length - 1, 'end')

const getVisibleRange = () => ({
  start: startIndex.value,
  end: endIndex.value,
  count: visibleItems.value.length
})

// Performance optimization: Debounced resize handler
const handleResize = useDebounce(() => {
  // Trigger re-computation of visible items
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop
  }
}, 100)

// Lifecycle
onMounted(() => {
  if (containerRef.value) {
    containerRef.value.addEventListener('resize', handleResize)
  }
})

onUnmounted(() => {
  if (containerRef.value) {
    containerRef.value.removeEventListener('resize', handleResize)
  }
})

// Expose methods for parent component
defineExpose({
  scrollToItem,
  scrollToTop,
  scrollToBottom,
  getVisibleRange
})

// Watch for changes in container height or item height
watch([() => props.containerHeight, () => props.itemHeight], () => {
  nextTick(() => {
    if (containerRef.value) {
      scrollTop.value = containerRef.value.scrollTop
    }
  })
})
</script>

<style scoped>
.virtualized-data-table {
  position: relative;
  width: 100%;
}

.virtual-scroll-container {
  position: relative;
}

.virtual-item {
  box-sizing: border-box;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #f0f0f0;
}

.default-item {
  padding: 12px;
  width: 100%;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.performance-indicator {
  position: absolute;
  top: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 4px 8px;
  font-size: 12px;
  border-radius: 0 0 0 4px;
  z-index: 5;
}

.performance-indicator span {
  margin-right: 8px;
}

/* Smooth scrolling */
.virtual-scroll-container {
  scroll-behavior: smooth;
}

/* Optimize rendering with will-change */
.virtual-item {
  will-change: transform;
}
</style>