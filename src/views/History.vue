<template>
  <div class="history-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <h2>Lottery History</h2>
          <div class="header-actions">
            <el-button @click="exportData" :disabled="loading">Export</el-button>
            <el-button type="primary" @click="goToDashboard">Back to Dashboard</el-button>
          </div>
        </div>
      </template>

      <!-- Enhanced filters with search -->
      <div class="filters">
        <el-row :gutter="16">
          <el-col :span="4">
            <el-input
              v-model="searchQuery"
              placeholder="Search numbers..."
              clearable
              @input="debouncedSearch"
            >
              <template #prefix>
                <span>🔍</span>
              </template>
            </el-input>
          </el-col>
          <el-col :span="4">
            <el-select v-model="selectedLotteryType" placeholder="Lottery Type" clearable>
              <el-option label="All Types" value="" />
              <el-option label="Powerball" value="powerball" />
              <el-option label="Mega Millions" value="megamillions" />
              <el-option label="Lotto" value="lotto" />
            </el-select>
          </el-col>
          <el-col :span="6">
            <el-date-picker
              v-model="dateRange"
              type="daterange"
              range-separator="To"
              start-placeholder="Start date"
              end-placeholder="End date"
              format="YYYY-MM-DD"
              value-format="YYYY-MM-DD"
              clearable
            />
          </el-col>
          <el-col :span="4">
            <el-button type="primary" @click="loadHistory" :loading="loading">
              Search
            </el-button>
            <el-button @click="resetFilters">Reset</el-button>
          </el-col>
        </el-row>

        <!-- Quick filters -->
        <div class="quick-filters">
          <el-tag
            v-for="filter in quickFilters"
            :key="filter.key"
            :type="activeQuickFilter === filter.key ? 'primary' : ''"
            :effect="activeQuickFilter === filter.key ? 'dark' : 'plain'"
            class="quick-filter-tag"
            @click="applyQuickFilter(filter)"
          >
            {{ filter.label }}
          </el-tag>
        </div>
      </div>

      <!-- Virtual table for large datasets -->
      <div class="virtual-table-wrapper">
        <VirtualTable
          ref="virtualTableRef"
          :items="filteredHistoryData"
          :columns="tableColumns"
          :container-height="tableHeight"
          :row-height="60"
          :loading="loading"
          :show-footer="true"
          :show-pagination="false"
          :selected-count="0"
          :filtered-count="filteredHistoryData.length"
          @preload="handlePreload"
          @row-click="handleRowClick"
        >
          <!-- Custom cell renderers -->
          <template #cell-draw_date="{ value }">
            <span class="date-cell">{{ formatDate(value) }}</span>
          </template>

          <template #cell-lottery_type="{ value }">
            <el-tag :type="getLotteryTypeTagType(value)" size="small">
              {{ formatLotteryType(value) }}
            </el-tag>
          </template>

          <template #cell-winning_numbers="{ record }">
            <div class="numbers-cell">
              <span
                v-for="(number, index) in record.winning_numbers"
                :key="index"
                class="number-ball"
                :class="{ 'selected': isNumberSelected(number) }"
              >
                {{ String(number).padStart(2, '0') }}
              </span>
              <span
                v-if="record.bonus_number"
                class="bonus-ball"
              >
                {{ String(record.bonus_number).padStart(2, '0') }}
              </span>
            </div>
          </template>

          <template #cell-jackpot_amount="{ value }">
            <span v-if="value" class="jackpot-cell">
              ${{ formatNumber(value) }}M
            </span>
            <span v-else class="no-jackpot">N/A</span>
          </template>
        </VirtualTable>
      </div>

      <!-- Statistics panel -->
      <div v-if="!loading && filteredHistoryData.length > 0" class="statistics-panel">
        <h3>Statistics</h3>
        <el-row :gutter="16">
          <el-col :span="6">
            <el-statistic title="Total Draws" :value="filteredHistoryData.length" />
          </el-col>
          <el-col :span="6">
            <el-statistic
              title="Average Jackpot"
              :value="averageJackpot"
              :precision="2"
              prefix="$"
              suffix="M"
            />
          </el-col>
          <el-col :span="6">
            <el-statistic title="Most Common Number" :value="mostCommonNumber" />
          </el-col>
          <el-col :span="6">
            <el-statistic title="Date Range" :value="dateRangeText" />
          </el-col>
        </el-row>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { lotteryApi } from '@/api/tauri'
import { formatDate } from '@/utils/formatters'
import VirtualTable from '@/components/common/VirtualTable.vue'
import { usePerformanceStore } from '@/stores/performance'
import type { LotteryDraw } from '@/types'
import type { TableColumn } from '@/components/common/VirtualTable.vue'

const router = useRouter()
const performanceStore = usePerformanceStore()

// Core data
const loading = ref(false)
const historyData = ref<LotteryDraw[]>([])
const virtualTableRef = ref()

// Filters
const searchQuery = ref('')
const selectedLotteryType = ref('')
const dateRange = ref<[string, string] | null>(null)
const activeQuickFilter = ref('')

// Table configuration
const tableHeight = ref(500)

// Quick filter definitions
const quickFilters = [
  { key: 'today', label: 'Today', days: 0 },
  { key: 'week', label: 'This Week', days: 7 },
  { key: 'month', label: 'This Month', days: 30 },
  { key: 'year', label: 'This Year', days: 365 },
  { key: 'large_jackpots', label: 'Large Jackpots', threshold: 100 },
]

// Table columns definition
const tableColumns: TableColumn[] = [
  {
    key: 'draw_date',
    title: 'Date',
    width: 120,
    sortable: true,
    align: 'left'
  },
  {
    key: 'lottery_type',
    title: 'Type',
    width: 100,
    sortable: true,
    align: 'center'
  },
  {
    key: 'winning_numbers',
    title: 'Winning Numbers',
    width: 400,
    sortable: false,
    align: 'center'
  },
  {
    key: 'jackpot_amount',
    title: 'Jackpot',
    width: 120,
    sortable: true,
    align: 'right',
    formatter: (value: number) => value ? `$${value}M` : 'N/A'
  }
]

// Computed properties
const filteredHistoryData = computed(() => {
  let filtered = [...historyData.value]

  // Apply search filter
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(draw => {
      // Search in winning numbers
      const numbersMatch = draw.winning_numbers.some((n: number) =>
        String(n).includes(query)
      )
      // Search in bonus number
      const bonusMatch = draw.bonus_number && String(draw.bonus_number).includes(query)
      // Search in lottery type
      const typeMatch = draw.lottery_type.toLowerCase().includes(query)
      // Search in date
      const dateMatch = formatDate(draw.draw_date).includes(query)

      return numbersMatch || bonusMatch || typeMatch || dateMatch
    })
  }

  // Apply lottery type filter
  if (selectedLotteryType.value) {
    filtered = filtered.filter(draw => draw.lottery_type === selectedLotteryType.value)
  }

  // Apply date range filter
  if (dateRange.value) {
    const [startDate, endDate] = dateRange.value
    filtered = filtered.filter(draw => {
      const drawDate = draw.draw_date.split('T')[0]
      return drawDate >= startDate && drawDate <= endDate
    })
  }

  // Apply quick filter
  if (activeQuickFilter.value) {
    const filter = quickFilters.find(f => f.key === activeQuickFilter.value)
    if (filter) {
      if (filter.days !== undefined) {
        // Date-based filter
        const cutoffDate = new Date()
        cutoffDate.setDate(cutoffDate.getDate() - filter.days)
        filtered = filtered.filter(draw => new Date(draw.draw_date) >= cutoffDate)
      } else if (filter.threshold) {
        // Jackpot filter
        filtered = filtered.filter(draw =>
          draw.jackpot_amount && draw.jackpot_amount >= filter.threshold
        )
      }
    }
  }

  return filtered
})

const averageJackpot = computed(() => {
  const jackpots = filteredHistoryData.value
    .filter(draw => draw.jackpot_amount)
    .map(draw => draw.jackpot_amount)

  if (jackpots.length === 0) return 0
  return jackpots.reduce((sum, amount) => sum + amount, 0) / jackpots.length
})

const mostCommonNumber = computed(() => {
  const numberFrequency: Record<number, number> = {}

  filteredHistoryData.value.forEach(draw => {
    draw.winning_numbers.forEach((number: number) => {
      numberFrequency[number] = (numberFrequency[number] || 0) + 1
    })
  })

  let mostCommon = 0
  let maxFrequency = 0

  for (const [number, frequency] of Object.entries(numberFrequency)) {
    if (frequency > maxFrequency) {
      maxFrequency = frequency
      mostCommon = parseInt(number)
    }
  }

  return mostCommon || 'N/A'
})

const dateRangeText = computed(() => {
  if (filteredHistoryData.value.length === 0) return 'No data'

  const dates = filteredHistoryData.value
    .map(draw => new Date(draw.draw_date))
    .sort((a, b) => a.getTime() - b.getTime())

  if (dates.length === 0) return 'N/A'

  const first = dates[0].toLocaleDateString()
  const last = dates[dates.length - 1].toLocaleDateString()

  return first === last ? first : `${first} - ${last}`
})

// Debounced search function
let searchTimeout: NodeJS.Timeout
const debouncedSearch = () => {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    performanceStore.recordInteraction('history_search')
  }, 300)
}

// Methods
const loadHistory = async () => {
  try {
    loading.value = true
    const startTime = performance.now()

    const data = await lotteryApi.getLotteryHistory(
      selectedLotteryType.value || undefined,
      10000, // Load more data for virtual scrolling
      0
    )

    historyData.value = data

    const loadTime = performance.now() - startTime
    performanceStore.recordApiTime('lottery_history_load', loadTime)
    console.debug(`Loaded ${data.length} history records in ${loadTime.toFixed(2)}ms`)

  } catch (error) {
    console.error('Failed to load history:', error)
    ElMessage.error('Failed to load lottery history')
    historyData.value = []
  } finally {
    loading.value = false
  }
}

const resetFilters = () => {
  searchQuery.value = ''
  selectedLotteryType.value = ''
  dateRange.value = null
  activeQuickFilter.value = ''
}

const applyQuickFilter = (filter: typeof quickFilters[0]) => {
  if (activeQuickFilter.value === filter.key) {
    activeQuickFilter.value = ''
  } else {
    activeQuickFilter.value = filter.key
  }
  performanceStore.recordInteraction('quick_filter')
}

const handlePreload = () => {
  console.debug('Virtual table requesting more data...')
  // Could implement pagination/infinite scroll here
}

const handleRowClick = (record: LotteryDraw) => {
  console.debug('Row clicked:', record)
  performanceStore.recordInteraction('history_row_click')
}

const exportData = async () => {
  try {
    await ElMessageBox.confirm(
      `Export ${filteredHistoryData.value.length} records to CSV?`,
      'Export Data',
      {
        confirmButtonText: 'Export',
        cancelButtonText: 'Cancel',
        type: 'info'
      }
    )

    // Generate CSV content
    const headers = ['Date', 'Type', 'Numbers', 'Bonus', 'Jackpot']
    const rows = filteredHistoryData.value.map(draw => [
      formatDate(draw.draw_date),
      draw.lottery_type,
      draw.winning_numbers.join(' '),
      draw.bonus_number || '',
      draw.jackpot_amount || ''
    ])

    const csvContent = [headers, ...rows]
      .map(row => row.join(','))
      .join('\n')

    // Create download
    const blob = new Blob([csvContent], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `lottery_history_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
    URL.revokeObjectURL(url)

    ElMessage.success('Data exported successfully')
    performanceStore.recordInteraction('history_export')

  } catch (error) {
    // User cancelled
    if (error !== 'cancel') {
      console.error('Export failed:', error)
      ElMessage.error('Export failed')
    }
  }
}

const goToDashboard = () => {
  router.push('/dashboard')
}

const formatLotteryType = (type: string) => {
  return type.charAt(0).toUpperCase() + type.slice(1).replace(/([A-Z])/g, ' $1')
}

const getLotteryTypeTagType = (type: string) => {
  const types: Record<string, string> = {
    'powerball': 'danger',
    'megamillions': 'warning',
    'lotto': 'success'
  }
  return types[type.toLowerCase()] || ''
}

const formatNumber = (num: number) => {
  return num.toFixed(2)
}

const isNumberSelected = (number: number) => {
  // Could implement selection logic here
  return false
}

// Watch for window resize
const updateTableHeight = () => {
  nextTick(() => {
    const container = document.querySelector('.virtual-table-wrapper')
    if (container) {
      const rect = container.getBoundingClientRect()
      tableHeight.value = Math.max(400, window.innerHeight - rect.top - 200)
    }
  })
}

// Lifecycle
onMounted(() => {
  loadHistory()
  updateTableHeight()
  window.addEventListener('resize', updateTableHeight)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateTableHeight)
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
})
</script>

<style scoped>
.history-container {
  padding: 20px;
  height: calc(100vh - 40px);
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  color: #2c3e50;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.filters {
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.quick-filters {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.quick-filter-tag {
  cursor: pointer;
  transition: all 0.2s;
  font-size: 12px;
}

.quick-filter-tag:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.virtual-table-wrapper {
  flex: 1;
  margin-bottom: 20px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}

/* Virtual table custom styles */
.virtual-table-wrapper :deep(.virtual-table-container) {
  height: 100%;
}

.virtual-table-wrapper :deep(.table-cell) {
  display: flex;
  align-items: center;
  padding: 12px 8px;
}

.virtual-table-wrapper :deep(.cell-content) {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.date-cell {
  font-family: 'Courier New', monospace;
  font-weight: 500;
  color: #666;
}

.numbers-cell {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: center;
  flex-wrap: wrap;
}

.number-ball {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(145deg, #ffffff, #e6e6e6);
  border: 2px solid #1890ff;
  color: #1890ff;
  font-weight: bold;
  font-size: 12px;
  font-family: 'Courier New', monospace;
  transition: all 0.2s;
}

.number-ball:hover {
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(24, 144, 255, 0.3);
}

.number-ball.selected {
  background: #1890ff;
  color: white;
}

.bonus-ball {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(145deg, #ff9800, #f57c00);
  border: 2px solid #ff9800;
  color: white;
  font-weight: bold;
  font-size: 12px;
  font-family: 'Courier New', monospace;
  transition: all 0.2s;
  position: relative;
}

.bonus-ball::after {
  content: '★';
  position: absolute;
  top: -8px;
  right: -8px;
  font-size: 10px;
  color: #ff9800;
}

.jackpot-cell {
  font-weight: 600;
  color: #27ae60;
  font-family: 'Courier New', monospace;
}

.no-jackpot {
  color: #999;
  font-style: italic;
}

.statistics-panel {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 24px;
  border-radius: 12px;
  margin-top: 20px;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.15);
}

.statistics-panel h3 {
  margin: 0 0 20px 0;
  font-size: 1.2rem;
  font-weight: 600;
  text-align: center;
  color: white;
}

.statistics-panel :deep(.el-statistic__content) {
  color: white;
}

.statistics-panel :deep(.el-statistic__head) {
  color: rgba(255, 255, 255, 0.8);
}

/* Performance optimizations */
.virtual-table-wrapper {
  contain: layout style paint;
}

.virtual-table-wrapper :deep(.table-row) {
  contain: layout style paint;
  will-change: background-color;
}

/* Responsive design */
@media (max-width: 768px) {
  .history-container {
    padding: 12px;
    height: calc(100vh - 24px);
  }

  .card-header {
    flex-direction: column;
    gap: 12px;
    text-align: center;
  }

  .header-actions {
    width: 100%;
    justify-content: center;
  }

  .filters {
    padding: 12px;
  }

  .quick-filters {
    justify-content: center;
  }

  .number-ball,
  .bonus-ball {
    width: 24px;
    height: 24px;
    font-size: 10px;
  }

  .numbers-cell {
    gap: 2px;
  }

  .statistics-panel {
    padding: 16px;
  }

  .statistics-panel .el-row {
    --el-row-gap: 12px;
  }
}

/* Animation for loading states */
.history-container :deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
}

/* Scrollbar styling for virtual table */
.virtual-table-wrapper :deep(.virtual-list) {
  scrollbar-width: thin;
  scrollbar-color: #ddd transparent;
}

.virtual-table-wrapper :deep(.virtual-list::-webkit-scrollbar) {
  width: 8px;
}

.virtual-table-wrapper :deep(.virtual-list::-webkit-scrollbar-track) {
  background: transparent;
}

.virtual-table-wrapper :deep(.virtual-list::-webkit-scrollbar-thumb) {
  background: #ddd;
  border-radius: 4px;
  transition: background 0.3s;
}

.virtual-table-wrapper :deep(.virtual-list::-webkit-scrollbar-thumb:hover) {
  background: #bbb;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .history-container {
    background-color: #1a1a1a;
    color: #e0e0e0;
  }

  .filters {
    background-color: #2a2a2a;
    border-color: #404040;
  }

  .virtual-table-wrapper {
    border-color: #404040;
  }

  .date-cell {
    color: #999;
  }

  .no-jackpot {
    color: #666;
  }

  .statistics-panel {
    background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
  }
}
</style>