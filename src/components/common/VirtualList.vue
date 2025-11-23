<template>
  <div 
    ref="containerRef" 
    class="virtual-list-container" 
    :style="{ height: containerHeight }"
    @scroll="handleScroll"
  >
    <div 
      class="virtual-list-phantom" 
      :style="{ height: `${totalHeight}px` }"
    ></div>
    
    <div 
      class="virtual-list-content" 
      :style="{ transform: `translateY(${offsetY}px)` }"
    >
      <div
        v-for="item in visibleItems"
        :key="getItemKey(item)"
        class="virtual-list-item"
        :style="{ height: `${itemHeight}px` }"
      >
        <slot :item="item" :index="item._index"></slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface VirtualListProps {
  items: any[]
  itemHeight: number
  containerHeight?: string
  buffer?: number
  keyField?: string
}

const props = withDefaults(defineProps<VirtualListProps>(), {
  containerHeight: '500px',
  buffer: 5,
  keyField: 'id'
})

const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)

// Add index to items for tracking
const itemsWithIndex = computed(() => 
  props.items.map((item, index) => ({
    ...item,
    _index: index
  }))
)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleCount = computed(() => {
  if (!containerRef.value) return 0
  return Math.ceil(parseInt(props.containerHeight) / props.itemHeight)
})

const startIndex = computed(() => {
  const index = Math.floor(scrollTop.value / props.itemHeight) - props.buffer
  return Math.max(0, index)
})

const endIndex = computed(() => {
  const index = startIndex.value + visibleCount.value + props.buffer * 2
  return Math.min(itemsWithIndex.value.length, index)
})

const visibleItems = computed(() => {
  return itemsWithIndex.value.slice(startIndex.value, endIndex.value)
})

const offsetY = computed(() => startIndex.value * props.itemHeight)

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop
}

const getItemKey = (item: any) => {
  return item[props.keyField] ?? item._index
}

// Scroll to specific index
const scrollToIndex = (index: number) => {
  if (!containerRef.value) return
  const targetScrollTop = index * props.itemHeight
  containerRef.value.scrollTop = targetScrollTop
}

// Reset scroll position
const resetScroll = () => {
  if (!containerRef.value) return
  containerRef.value.scrollTop = 0
  scrollTop.value = 0
}

// Watch items change and reset scroll if needed
watch(() => props.items.length, (newLength, oldLength) => {
  if (newLength < oldLength && scrollTop.value > totalHeight.value) {
    resetScroll()
  }
})

defineExpose({
  scrollToIndex,
  resetScroll
})
</script>

<style scoped>
.virtual-list-container {
  overflow-y: auto;
  position: relative;
  will-change: scroll-position;
}

.virtual-list-phantom {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: -1;
}

.virtual-list-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  will-change: transform;
}

.virtual-list-item {
  overflow: hidden;
}

/* Custom scrollbar styling */
.virtual-list-container::-webkit-scrollbar {
  width: 8px;
}

.virtual-list-container::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.virtual-list-container::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 4px;
}

.virtual-list-container::-webkit-scrollbar-thumb:hover {
  background: #555;
}
</style>
