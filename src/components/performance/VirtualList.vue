<!--
Virtual List Component - High-performance scrolling for large datasets

Features:
- Virtual scrolling for optimal memory usage
- Dynamic item heights support
- Smooth scrolling with momentum
- Accessibility support
- Keyboard navigation
- Performance monitoring
-->

<template>
  <div
    ref="containerRef"
    class="virtual-list"
    :style="containerStyle"
    :aria-label="ariaLabel"
    role="grid"
    @scroll="handleScroll"
    @keydown="handleKeyDown"
    tabindex="0"
  >
    <!-- Spacing above visible items -->
    <div :style="spacerAboveStyle" class="virtual-list-spacer-above" />

    <!-- Visible items -->
    <div
      v-for="item in visibleItems"
      :key="getItemKey(item)"
      :ref="setItemRef"
      class="virtual-list-item"
      :style="getItemStyle(item)"
      :aria-rowindex="getItemIndex(item)"
      :aria-selected="isSelected(item)"
      :class="{ 'selected': isSelected(item) }"
      role="row"
      @click="selectItem(item)"
      @dblclick="activateItem(item)"
    >
      <slot name="item" :item="item" :index="getItemIndex(item)">
        <div class="default-item-content">
          {{ getItemContent(item) }}
        </div>
      </slot>
    </div>

    <!-- Spacing below visible items -->
    <div :style="spacerBelowStyle" class="virtual-list-spacer-below" />

    <!-- Loading indicator -->
    <div v-if="loading" class="virtual-list-loading">
      <div class="loading-spinner"></div>
      <div class="loading-text">{{ loadingText }}</div>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && items.length === 0" class="virtual-list-empty">
      <div class="empty-icon">📋</div>
      <div class="empty-title">{{ emptyTitle }}</div>
      <div class="empty-description">{{ emptyDescription }}</div>
      <slot name="empty">
        <button v-if="emptyAction" class="empty-action" @click="$emit('empty-action')">
          {{ emptyAction }}
        </button>
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  nextTick,
  watch,
  type PropType
} from 'vue'
import { usePerformanceMonitoring } from '../../composables/usePerformanceMonitoring'
import { useLogger } from '../../composables/useLogger'

interface VirtualListItem {
  id: string | number
  height?: number
  data?: any
}

interface PerformanceMetrics {
  renderTime: number
  scrollTime: number
  itemCount: number
  visibleCount: number
  bufferSize: number
}

const props = defineProps({
  items: {
    type: Array as PropType<VirtualListItem[]>,
    default: () => []
  },
  itemHeight: {
    type: Number,
    default: 48
  },
  containerHeight: {
    type: Number,
    default: 400
  },
  overscan: {
    type: Number,
    default: 5
  },
  keyField: {
    type: String,
    default: 'id'
  },
  loading: {
    type: Boolean,
    default: false
  },
  loadingText: {
    type: String,
    default: 'Loading...'
  },
  emptyTitle: {
    type: String,
    default: 'No items'
  },
  emptyDescription: {
    type: String,
    default: 'There are no items to display'
  },
  emptyAction: {
    type: String,
    default: ''
  },
  ariaLabel: {
    type: String,
    default: 'Virtual List'
  },
  enableKeyboardNavigation: {
    type: Boolean,
    default: true
  },
  enablePerformanceMonitoring: {
    type: Boolean,
    default: true
  },
  smoothScrolling: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits([
  'item-click',
  'item-activate',
  'item-select',
  'scroll',
  'reach-end',
  'reach-start',
  'empty-action'
])

// Reactive state
const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)
const containerHeight = ref(props.containerHeight)
const itemHeights = ref<Map<string | number, number>>(new Map())
const itemRefs = ref<Map<string | number, HTMLElement>>(new Map())
const selectedItems = ref<Set<string | number>>(new Set())
const focusedIndex = ref(-1)

// Performance monitoring
const logger = useLogger('virtual-list')
const performanceMonitor = usePerformanceMonitoring()
const performanceMetrics = ref<PerformanceMetrics>({
  renderTime: 0,
  scrollTime: 0,
  itemCount: 0,
  visibleCount: 0,
  bufferSize: 0
})

// Computed properties
const totalHeight = computed(() => {
  if (props.items.length === 0) return 0

  return Array.from(itemHeights.value.values()).reduce((sum, height) => sum + height, 0) +
    (props.items.length - itemHeights.value.size) * props.itemHeight
})

const visibleRange = computed(() => {
  if (props.items.length === 0) return { start: 0, end: 0 }

  let currentHeight = 0
  let startIndex = 0
  let endIndex = 0

  // Find start index based on scroll position
  for (let i = 0; i < props.items.length; i++) {
    const itemHeight = getItemHeight(i)
    if (currentHeight + itemHeight > scrollTop.value) {
      startIndex = Math.max(0, i - props.overscan)
      break
    }
    currentHeight += itemHeight
  }

  // Find end index based on visible area
  currentHeight = 0
  let visibleHeight = 0
  for (let i = startIndex; i < props.items.length; i++) {
    const itemHeight = getItemHeight(i)
    if (visibleHeight >= containerHeight.value) {
      endIndex = Math.min(props.items.length - 1, i + props.overscan)
      break
    }
    visibleHeight += itemHeight
    currentHeight += itemHeight
  }

  return { start: startIndex, end: Math.max(startIndex, endIndex) }
})

const visibleItems = computed(() => {
  if (props.items.length === 0) return []

  return props.items.slice(visibleRange.value.start, visibleRange.value.end + 1)
})

const spacerAboveStyle = computed(() => ({
  height: `${getOffsetHeight(visibleRange.value.start)}px`
}))

const spacerBelowStyle = computed(() => {
  const endIndex = visibleRange.value.end
  const belowHeight = totalHeight.value - getOffsetHeight(endIndex + 1)
  return {
    height: `${belowHeight}px`
  }
})

const containerStyle = computed(() => ({
  height: `${containerHeight.value}px`,
  overflow: 'auto',
  position: 'relative'
}))

// Methods
const getItemHeight = (index: number): number => {
  const item = props.items[index]
  if (!item) return props.itemHeight

  const itemKey = getItemKey(item)
  return itemHeights.value.get(itemKey) || item.height || props.itemHeight
}

const getOffsetHeight = (index: number): number => {
  let height = 0
  for (let i = 0; i < index && i < props.items.length; i++) {
    height += getItemHeight(i)
  }
  return height
}

const getItemKey = (item: VirtualListItem): string | number => {
  return item[props.keyField] || item.id
}

const getItemIndex = (item: VirtualListItem): number => {
  return props.items.findIndex(i => getItemKey(i) === getItemKey(item))
}

const getItemStyle = (item: VirtualListItem): Record<string, string> => {
  const index = getItemIndex(item)
  return {
    position: 'absolute',
    top: `${getOffsetHeight(index)}px`,
    left: '0',
    right: '0',
    height: `${getItemHeight(index)}px`,
    minHeight: `${props.itemHeight}px`
  }
}

const getItemContent = (item: VirtualListItem): string => {
  if (typeof item.data === 'string') return item.data
  if (typeof item.data === 'number') return item.data.toString()
  if (item.data && item.data.title) return item.data.title
  if (item.data && item.data.name) return item.data.name
  return JSON.stringify(item.data || item)
}

const isSelected = (item: VirtualListItem): boolean => {
  return selectedItems.value.has(getItemKey(item))
}

const setItemRef = (el: HTMLElement) => {
  if (!el) return

  // Find the item index
  const itemElement = el.closest('.virtual-list-item')
  if (!itemElement) return

  const index = parseInt(itemElement.getAttribute('aria-rowindex') || '0')
  const item = props.items[index]
  if (item) {
    itemRefs.value.set(getItemKey(item), el)

    // Measure actual height
    if (props.enablePerformanceMonitoring) {
      nextTick(() => {
        const actualHeight = el.getBoundingClientRect().height
        const currentHeight = getItemHeight(index)

        if (Math.abs(actualHeight - currentHeight) > 1) {
          itemHeights.value.set(getItemKey(item), actualHeight)
          logger.debug('Item height updated', {
            itemKey: getItemKey(item),
            oldHeight: currentHeight,
            newHeight: actualHeight
          })
        }
      })
    }
  }
}

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  const oldScrollTop = scrollTop.value
  scrollTop.value = target.scrollTop

  // Emit scroll event
  emit('scroll', {
    scrollTop: scrollTop.value,
    scrollLeft: target.scrollLeft,
    scrollTopDelta: scrollTop.value - oldScrollTop,
    target
  })

  // Check if reached ends
  if (scrollTop.value + containerHeight.value >= totalHeight.value - 10) {
    emit('reach-end')
  }

  if (scrollTop.value <= 10) {
    emit('reach-start')
  }

  // Performance monitoring
  if (props.enablePerformanceMonitoring) {
    performanceMonitor.trackComponentPerformance(
      'virtual-list-scroll',
      Math.abs(scrollTop.value - oldScrollTop),
      'scroll'
    )
  }
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (!props.enableKeyboardNavigation) return

  let newIndex = focusedIndex.value

  switch (event.key) {
    case 'ArrowUp':
    case 'ArrowLeft':
      event.preventDefault()
      newIndex = Math.max(0, focusedIndex.value - 1)
      break
    case 'ArrowDown':
    case 'ArrowRight':
      event.preventDefault()
      newIndex = Math.min(props.items.length - 1, focusedIndex.value + 1)
      break
    case 'Home':
      event.preventDefault()
      newIndex = 0
      break
    case 'End':
      event.preventDefault()
      newIndex = props.items.length - 1
      break
    case 'Enter':
    case ' ':
      event.preventDefault()
      if (focusedIndex.value >= 0) {
        const item = props.items[focusedIndex.value]
        activateItem(item)
      }
      break
    default:
      return
  }

  if (newIndex !== focusedIndex.value) {
    focusedIndex.value = newIndex
    scrollToItem(newIndex)

    const item = props.items[newIndex]
    if (item) {
      emit('item-select', { item, index: newIndex, source: 'keyboard' })
    }
  }
}

const selectItem = (item: VirtualListItem) => {
  const itemKey = getItemKey(item)
  const index = getItemIndex(item)

  emit('item-click', { item, index })

  if (event.shiftKey) {
    // Multi-select
    selectedItems.value.add(itemKey)
  } else {
    // Single select
    selectedItems.value.clear()
    selectedItems.value.add(itemKey)
  }

  focusedIndex.value = index
  emit('item-select', { item, index, selected: Array.from(selectedItems.value) })
}

const activateItem = (item: VirtualListItem) => {
  const index = getItemIndex(item)
  emit('item-activate', { item, index })
}

const scrollToItem = (index: number, behavior: ScrollBehavior = 'smooth') => {
  if (!containerRef.value) return

  const offset = getOffsetHeight(index)
  containerRef.value.scrollTo({
    top: offset,
    behavior: props.smoothScrolling ? behavior : 'auto'
  })
}

const scrollToTop = () => {
  if (containerRef.value) {
    containerRef.value.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

const scrollToBottom = () => {
  if (containerRef.value) {
    containerRef.value.scrollTo({
      top: totalHeight.value,
      behavior: 'smooth'
    })
  }
}

const selectAll = () => {
  selectedItems.value.clear()
  props.items.forEach(item => selectedItems.value.add(getItemKey(item)))
}

const clearSelection = () => {
  selectedItems.value.clear()
}

const getSelectedItems = (): VirtualListItem[] => {
  return props.items.filter(item => selectedItems.value.has(getItemKey(item)))
}

// Performance monitoring
const updatePerformanceMetrics = () => {
  performanceMetrics.value = {
    renderTime: performanceMonitor.metrics.value.components.renderTime['virtual-list'] || 0,
    scrollTime: performanceMonitor.metrics.value.performanceMetrics.averageDuration,
    itemCount: props.items.length,
    visibleCount: visibleItems.value.length,
    bufferSize: (visibleRange.value.end - visibleRange.value.start + 1)
  }
}

// Watchers
watch(() => props.items, () => {
  if (props.enablePerformanceMonitoring) {
    const startTime = performance.now()

    nextTick(() => {
      const renderTime = performance.now() - startTime
      performanceMonitor.trackComponentPerformance(
        'virtual-list-render',
        renderTime,
        'render'
      )
      updatePerformanceMetrics()
    })
  }
}, { deep: true })

watch(visibleRange, (newRange, oldRange) => {
  if (props.enablePerformanceMonitoring) {
    logger.debug('Visible range changed', {
      oldRange,
      newRange,
      visibleCount: newRange.end - newRange.start + 1
    })
  }
})

// Lifecycle
onMounted(() => {
  if (containerRef.value) {
    // Initialize container height
    const rect = containerRef.value.getBoundingClientRect()
    if (rect.height > 0) {
      containerHeight.value = rect.height
    }

    // Add resize observer
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.target === containerRef.value) {
          containerHeight.value = entry.contentRect.height
        }
      }
    })

    resizeObserver.observe(containerRef.value)

    // Cleanup on unmount
    onUnmounted(() => {
      resizeObserver.disconnect()
    })
  }

  logger.info('Virtual list mounted', {
    itemCount: props.items.length,
    itemHeight: props.itemHeight,
    containerHeight: containerHeight.value,
    overscan: props.overscan
  })
})

// Expose methods
defineExpose({
  scrollToItem,
  scrollToTop,
  scrollToBottom,
  selectAll,
  clearSelection,
  getSelectedItems,
  containerRef,
  performanceMetrics
})
</script>

<style scoped>
.virtual-list {
  position: relative;
  overflow: auto;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  background: #fff;
  outline: none;
}

.virtual-list:focus {
  border-color: #4a90e2;
  box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.2);
}

.virtual-list-spacer-above,
.virtual-list-spacer-below {
  position: absolute;
  left: 0;
  right: 0;
  pointer-events: none;
}

.virtual-list-item {
  position: absolute;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s ease;
}

.virtual-list-item:hover {
  background-color: #f5f5f5;
}

.virtual-list-item.selected {
  background-color: #e3f2fd;
  color: #1976d2;
}

.virtual-list-item:focus {
  outline: 2px solid #4a90e2;
  outline-offset: -2px;
}

.default-item-content {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.virtual-list-loading {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.8);
  z-index: 1;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f0f0f0;
  border-top: 3px solid #4a90e2;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

.loading-text {
  color: #666;
  font-size: 14px;
}

.virtual-list-empty {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
  text-align: center;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 8px;
}

.empty-description {
  font-size: 14px;
  margin-bottom: 24px;
  max-width: 300px;
  line-height: 1.5;
}

.empty-action {
  padding: 8px 16px;
  background: #4a90e2;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s ease;
}

.empty-action:hover {
  background: #357abd;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .virtual-list-item {
    border-bottom-color: #000;
  }

  .virtual-list-item:hover {
    background-color: #f0f0f0;
  }

  .virtual-list-item.selected {
    background-color: #000;
    color: #fff;
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .virtual-list-item {
    transition: none;
  }

  .loading-spinner {
    animation: none;
  }

  .empty-action {
    transition: none;
  }
}
</style>