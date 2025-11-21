<template>
  <div class="search-filters">
    <div class="filter-controls">
      <div class="filter-group">
        <label for="date-range">日期范围:</label>
        <input
          id="date-range"
          v-model="filters.dateRange"
          type="text"
          placeholder="开始日期 - 结束日期"
          class="filter-input"
          @input="handleDateRangeInput"
        />
      </div>

      <div class="filter-group">
        <label for="draw-number">期号:</label>
        <input
          id="draw-number"
          v-model="filters.drawNumber"
          type="text"
          placeholder="输入期号"
          class="filter-input"
          @input="handleFilterChange"
        />
      </div>

      <div class="filter-group">
        <label for="front-numbers">前区号码:</label>
        <input
          id="front-numbers"
          v-model="filters.frontNumbers"
          type="text"
          placeholder="输入号码，用逗号分隔"
          class="filter-input"
          @input="handleFilterChange"
        />
      </div>

      <div class="filter-group">
        <label for="back-numbers">后区号码:</label>
        <input
          id="back-numbers"
          v-model="filters.backNumbers"
          type="text"
          placeholder="输入号码，用逗号分隔"
          class="filter-input"
          @input="handleFilterChange"
        />
      </div>

      <div class="filter-actions">
        <button
          @click="applyFilters"
          :disabled="loading"
          class="btn btn-primary"
        >
          {{ loading ? '筛选中...' : '应用筛选' }}
        </button>
        <button
          @click="clearFilters"
          :disabled="loading"
          class="btn btn-secondary"
        >
          清除
        </button>
      </div>
    </div>

    <div v-if="hasActiveFilters" class="active-filters">
      <h4>当前筛选条件:</h4>
      <div class="active-filter-tags">
        <span
          v-for="(value, key) in activeFiltersList"
          :key="key"
          class="filter-tag"
        >
          {{ getFilterDisplay(key, value) }}
          <button
            @click="removeFilter(key)"
            class="remove-filter"
            aria-label="移除筛选"
          >
            ×
          </button>
        </span>
      </div>
    </div>

    <div class="filter-stats">
      <span>显示 {{ resultCount }} 条结果</span>
      <span v-if="totalResults > resultCount">
        (共 {{ totalResults }} 条)
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface FilterParams {
  dateRange?: string
  startDate?: string
  endDate?: string
  drawNumber?: string
  frontNumbers?: string
  backNumbers?: string
}

interface Props {
  loading?: boolean
  resultCount?: number
  totalResults?: number
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  resultCount: 0,
  totalResults: 0
})

// Emits
const emit = defineEmits<{
  'filter-change': [filters: FilterParams]
  'clear-filters': []
}>()

// State
const filters = ref<FilterParams>({
  dateRange: '',
  drawNumber: '',
  frontNumbers: '',
  backNumbers: ''
})

const activeFilters = ref<FilterParams>({})

// Computed properties
const hasActiveFilters = computed(() => {
  return Object.keys(activeFilters.value).length > 0
})

const activeFiltersList = computed(() => {
  return Object.entries(activeFilters.value)
})

// Methods
const handleDateRangeInput = () => {
  // Parse date range input
  const dateRange = filters.value.dateRange.trim()
  if (dateRange) {
    const parts = dateRange.split('-').map(part => part.trim())
    if (parts.length === 2) {
      filters.value.startDate = parts[0]
      filters.value.endDate = parts[1]
      filters.value.dateRange = ''
    }
  }
}

const handleFilterChange = () => {
  // Auto-apply filters on input change with debounce
  clearTimeout(filterTimeout)
  filterTimeout = setTimeout(() => {
    applyFilters()
  }, 500)
}

let filterTimeout: number

const applyFilters = () => {
  const newFilters: FilterParams = {}

  if (filters.value.startDate || filters.value.endDate) {
    if (filters.value.startDate) newFilters.startDate = filters.value.startDate
    if (filters.value.endDate) newFilters.endDate = filters.value.endDate
  }

  if (filters.value.drawNumber) {
    newFilters.drawNumber = filters.value.drawNumber.trim()
  }

  if (filters.value.frontNumbers) {
    const numbers = filters.value.frontNumbers
      .split(',')
      .map(n => n.trim())
      .filter(n => n && !isNaN(Number(n)))
      .map(n => Number(n))
      .filter(n => n >= 1 && n <= 35)
    if (numbers.length > 0) {
      newFilters.frontNumbers = numbers.join(',')
    }
  }

  if (filters.value.backNumbers) {
    const numbers = filters.value.backNumbers
      .split(',')
      .map(n => n.trim())
      .filter(n => n && !isNaN(Number(n)))
      .map(n => Number(n))
      .filter(n => n >= 1 && n <= 12)
    if (numbers.length > 0) {
      newFilters.backNumbers = numbers.join(',')
    }
  }

  activeFilters.value = newFilters
  emit('filter-change', newFilters)
}

const clearFilters = () => {
  filters.value = {
    dateRange: '',
    drawNumber: '',
    frontNumbers: '',
    backNumbers: ''
  }
  activeFilters.value = {}
  emit('clear-filters')
}

const removeFilter = (key: string) => {
  const newFilters = { ...activeFilters.value }
  delete newFilters[key as keyof FilterParams]
  activeFilters.value = newFilters

  // Also clear from main filters
  if (key === 'startDate' || key === 'endDate') {
    filters.value.dateRange = ''
    filters.value.startDate = ''
    filters.value.endDate = ''
  } else {
    filters.value[key as keyof FilterParams] = ''
  }

  emit('filter-change', newFilters)
}

const getFilterDisplay = (key: string, value: any): string => {
  const displayMap: Record<string, (value: any) => string> = {
    startDate: (date: string) => `开始: ${formatDate(date)}`,
    endDate: (date: string) => `结束: ${formatDate(date)}`,
    drawNumber: (number: string) => `期号: ${number}`,
    frontNumbers: (numbers: string) => `前区: ${numbers}`,
    backNumbers: (numbers: string) => `后区: ${numbers}`
  }
  return displayMap[key]?.(value) || `${key}: ${value}`
}

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateString
  }
}

// Watch for external filter changes
const modelValue = defineModel<FilterParams>({
  get: () => filters.value,
  set: (value: FilterParams) => {
    filters.value = { ...filters.value, ...value }
    // Auto-apply when setting from parent
    applyFilters()
  }
})
</script>

<style scoped>
.search-filters {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}

.filter-controls {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-group label {
  color: #2c3e50;
  font-weight: 500;
  font-size: 0.9rem;
}

.filter-input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
  transition: border-color 0.3s;
}

.filter-input:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.filter-actions {
  display: flex;
  gap: 10px;
  align-items: flex-end;
}

.active-filters {
  background: #f8f9fa;
  border-radius: 6px;
  padding: 15px;
  margin-bottom: 15px;
  border: 1px solid #e9ecef;
}

.active-filters h4 {
  color: #2c3e50;
  margin: 0 0 10px 0;
  font-size: 1rem;
}

.active-filter-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #e3f2fd;
  color: #1565c0;
  border-radius: 20px;
  font-size: 0.8rem;
}

.remove-filter {
  background: none;
  border: none;
  color: #dc3545;
  cursor: pointer;
  font-size: 1rem;
  font-weight: bold;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: background-color 0.2s;
}

.remove-filter:hover {
  background-color: #dc3545;
  color: white;
}

.filter-stats {
  display: flex;
  gap: 15px;
  color: #6c757d;
  font-size: 0.9rem;
  padding: 10px 0;
  border-top: 1px solid #ecf0f1;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.3s;
}

.btn-primary {
  background-color: #3498db;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #2980b9;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #5a6268;
}

/* Responsive design */
@media (max-width: 768px) {
  .filter-controls {
    grid-template-columns: 1fr;
  }

  .filter-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .active-filter-tags {
    justify-content: center;
  }
}
</style>