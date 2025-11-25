<template>
  <div class="virtual-table-container">
    <!-- Table header -->
    <div class="virtual-table-header">
      <table class="table">
        <thead>
          <tr>
            <th
              v-for="column in columns"
              :key="column.key"
              :style="{ width: column.width + 'px', minWidth: column.minWidth + 'px' }"
              :class="{ 'sortable': column.sortable }"
              @click="handleSort(column)"
            >
              <div class="header-cell">
                <span>{{ column.title }}</span>
                <span v-if="column.sortable" class="sort-icon">
                  <span v-if="sortColumn === column.key" class="sort-direction">
                    {{ sortOrder === 'asc' ? '↑' : '↓' }}
                  </span>
                  <span v-else class="sort-indicator">↕</span>
                </span>
              </div>
            </th>
          </tr>
        </thead>
      </table>
    </div>

    <!-- Virtual table body -->
    <VirtualList
      ref="virtualListRef"
      :items="sortedItems"
      :container-height="containerHeight"
      :item-height="rowHeight"
      :buffer-size="bufferSize"
      :loading="loading"
      :preload-threshold="preloadThreshold"
      @preload="handlePreload"
    >
      <template #item="{ item }">
        <div class="table-row">
          <div
            v-for="column in columns"
            :key="column.key"
            class="table-cell"
            :style="{ width: column.width + 'px', minWidth: column.minWidth + 'px' }"
          >
            <slot
              :name="`cell-${column.key}`"
              :record="item"
              :value="getColumnValue(item, column)"
              :column="column"
            >
              <span class="cell-content">
                {{ formatCellContent(getColumnValue(item, column), column) }}
              </span>
            </slot>
          </div>
        </div>
      </template>
    </VirtualList>

    <!-- Table footer with statistics -->
    <div v-if="showFooter && !loading && items.length > 0" class="virtual-table-footer">
      <div class="footer-stats">
        <span class="stat-item">
          总计: {{ items.length }} 条记录
        </span>
        <span v-if="filteredCount !== items.length" class="stat-item">
          筛选: {{ filteredCount }} 条
        </span>
        <span v-if="selectedCount > 0" class="stat-item">
          已选择: {{ selectedCount }} 条
        </span>
      </div>
      <div v-if="showPagination" class="footer-pagination">
        <button
          class="pagination-btn"
          :disabled="currentPage === 1"
          @click="changePage(currentPage - 1)"
        >
          上一页
        </button>
        <span class="page-info">
          {{ currentPage }} / {{ totalPages }}
        </span>
        <button
          class="pagination-btn"
          :disabled="currentPage === totalPages"
          @click="changePage(currentPage + 1)"
        >
          下一页
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import VirtualList from './VirtualList.vue'
import { usePerformanceStore } from '@/stores/performance'

interface TableColumn {
  key: string
  title: string
  width?: number
  minWidth?: number
  sortable?: boolean
  formatter?: (value: any, record: any) => string
  align?: 'left' | 'center' | 'right'
}

interface Props {
  // 数据列表
  items: any[]
  // 表格列配置
  columns: TableColumn[]
  // 容器高度
  containerHeight: number
  // 行高
  rowHeight?: number
  // 缓冲区大小
  bufferSize?: number
  // 加载状态
  loading?: boolean
  // 是否显示页脚
  showFooter?: boolean
  // 是否显示分页
  showPagination?: boolean
  // 每页大小
  pageSize?: number
  // 预加载阈值
  preloadThreshold?: number
  // 已选择的项目数量
  selectedCount?: number
  // 筛选后的数量
  filteredCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  rowHeight: 40,
  bufferSize: 10,
  loading: false,
  showFooter: true,
  showPagination: false,
  pageSize: 100,
  preloadThreshold: 100,
  selectedCount: 0,
  filteredCount: 0,
})

// Performance tracking
const performanceStore = usePerformanceStore()

// Refs
const virtualListRef = ref()
const sortColumn = ref<string>('')
const sortOrder = ref<'asc' | 'desc'>('asc')
const currentPage = ref(1)

// Computed properties
const totalPages = computed(() => {
  return Math.ceil(props.items.length / props.pageSize)
})

const sortedItems = computed(() => {
  if (!sortColumn.value) {
    return props.items
  }

  return [...props.items].sort((a, b) => {
    const aValue = getColumnValue(a, { key: sortColumn.value } as TableColumn)
    const bValue = getColumnValue(b, { key: sortColumn.value } as TableColumn)

    let comparison = 0
    if (aValue < bValue) comparison = -1
    if (aValue > bValue) comparison = 1

    return sortOrder.value === 'desc' ? -comparison : comparison
  })
})

const paginatedItems = computed(() => {
  if (!props.showPagination) {
    return sortedItems.value
  }

  const start = (currentPage.value - 1) * props.pageSize
  const end = start + props.pageSize
  return sortedItems.value.slice(start, end)
})

// Methods
const getColumnValue = (record: any, column: TableColumn): any => {
  const keys = column.key.split('.')
  let value = record

  for (const key of keys) {
    value = value?.[key]
  }

  return value
}

const formatCellContent = (value: any, column: TableColumn): string => {
  if (column.formatter) {
    return column.formatter(value, null)
  }

  // Default formatting
  if (value === null || value === undefined) {
    return '-'
  }

  if (typeof value === 'number') {
    return value.toLocaleString()
  }

  if (value instanceof Date) {
    return value.toLocaleDateString()
  }

  return String(value)
}

const handleSort = (column: TableColumn) => {
  if (!column.sortable) return

  const startTime = performance.now()

  if (sortColumn.value === column.key) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = column.key
    sortOrder.value = 'asc'
  }

  performanceStore.recordInteraction('table_sort')

  const sortTime = performance.now() - startTime
  console.debug(`Table sort completed in ${sortTime.toFixed(2)}ms`)
}

const handlePreload = () => {
  emit('preload')
}

const changePage = (page: number) => {
  if (page < 1 || page > totalPages.value) return

  currentPage.value = page
  emit('page-change', page)
}

const scrollToTop = () => {
  virtualListRef.value?.scrollToTop()
}

const scrollToBottom = () => {
  virtualListRef.value?.scrollToBottom()
}

const scrollToRow = (index: number) => {
  virtualListRef.value?.scrollToItem(index)
}

const clearSort = () => {
  sortColumn.value = ''
  sortOrder.value = 'asc'
}

// Watch for data changes
watch(() => props.items.length, (newLength, oldLength) => {
  if (oldLength !== newLength) {
    // Reset to first page when data changes
    currentPage.value = 1
    performanceStore.recordInteraction('table_data_change')
  }
})

onMounted(() => {
  performanceStore.recordRenderTime('virtual_table', performance.now())
})

// Expose methods
defineExpose({
  scrollToTop,
  scrollToBottom,
  scrollToRow,
  clearSort,
  changePage
})

// Events
const emit = defineEmits<{
  preload: []
  'page-change': [page: number]
  'row-click': [record: any, index: number]
  'sort-change': [column: string, order: 'asc' | 'desc']
}>()

// Watch sort changes
watch([sortColumn, sortOrder], () => {
  if (sortColumn.value) {
    emit('sort-change', sortColumn.value, sortOrder.value)
  }
})
</script>

<style scoped>
.virtual-table-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: white;
  overflow: hidden;
}

.virtual-table-header {
  flex-shrink: 0;
  background: #fafafa;
  border-bottom: 2px solid #e0e0e0;
  position: sticky;
  top: 0;
  z-index: 10;
}

.table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

.table th {
  padding: 12px 8px;
  text-align: left;
  font-weight: 600;
  color: #333;
  background: inherit;
  user-select: none;
}

.sortable {
  cursor: pointer;
  transition: background-color 0.2s;
}

.sortable:hover {
  background: #f0f0f0;
}

.header-cell {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sort-icon {
  font-size: 12px;
  color: #666;
  margin-left: 4px;
}

.sort-direction {
  color: #1890ff;
  font-weight: bold;
}

.table-row {
  display: flex;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.table-row:hover {
  background-color: #f8f9fa;
}

.table-cell {
  display: flex;
  align-items: center;
  padding: 8px;
  flex-shrink: 0;
  overflow: hidden;
}

.cell-content {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.virtual-table-footer {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #fafafa;
  border-top: 1px solid #e0e0e0;
  font-size: 14px;
}

.footer-stats {
  display: flex;
  gap: 16px;
  color: #666;
}

.stat-item {
  white-space: nowrap;
}

.footer-pagination {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pagination-btn {
  padding: 6px 12px;
  border: 1px solid #d0d0d0;
  background: white;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.pagination-btn:hover:not(:disabled) {
  border-color: #1890ff;
  color: #1890ff;
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-info {
  font-weight: 500;
  color: #333;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .table th,
  .table-cell {
    padding: 8px 4px;
    font-size: 14px;
  }

  .virtual-table-footer {
    flex-direction: column;
    gap: 8px;
    padding: 8px;
  }

  .footer-stats {
    justify-content: center;
  }
}

/* Performance optimizations */
.table-row {
  contain: layout style paint;
  will-change: background-color;
}

.table-cell {
  contain: layout style;
}
</style>