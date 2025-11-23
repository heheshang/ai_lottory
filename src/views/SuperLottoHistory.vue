<template>
  <div class="super-lotto-history">
    <div class="page-header">
      <h1>大乐透历史数据</h1>
      <p class="page-description">浏览和搜索大乐透历史开奖结果</p>
    </div>

    <div class="controls-section">
      <div class="import-section">
        <DataImport @import-success="handleImportSuccess" />
      </div>

      <div class="filters-section">
        <SearchFilters
          @filter-change="handleFilterChange"
          :loading="loading"
        />
      </div>
    </div>

    <div class="data-section">
      <div v-if="loading" class="loading-container">
        <div class="loading-spinner"></div>
        <p>加载历史数据中...</p>
      </div>

      <div v-else-if="error" class="error-container">
        <div class="error-message">
          <i class="icon-error"></i>
          <p>{{ error }}</p>
          <button @click="clearError" class="btn btn-primary">重试</button>
        </div>
      </div>

      <div v-else-if="draws.length === 0" class="empty-state">
        <div class="empty-icon">📊</div>
        <h3>暂无历史数据</h3>
        <p>请导入大乐透历史开奖数据以开始分析</p>
        <button @click="showImportDialog" class="btn btn-primary">导入数据</button>
      </div>

      <div v-else class="data-container">
        <div class="data-header">
          <h3>历史开奖记录</h3>
          <div class="data-stats">
            <span>共 {{ totalDraws }} 条记录</span>
            <span>第 {{ currentPage }} 页，共 {{ totalPages }} 页</span>
          </div>
        </div>

        <DataTable
          :draws="draws"
          :loading="loading"
          @draw-select="handleDrawSelect"
        />

        <div class="pagination" v-if="totalPages > 1">
          <button
            @click="previousPage"
            :disabled="currentPage <= 1"
            class="btn btn-secondary"
          >
            上一页
          </button>
          <span class="page-info">
            第 {{ currentPage }} / {{ totalPages }} 页
          </span>
          <button
            @click="nextPage"
            :disabled="currentPage >= totalPages"
            class="btn btn-secondary"
          >
            下一页
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useSuperLottoStore } from '@/stores/superLotto'
import type { SuperLottoDraw } from '@/types/superLotto'
import DataTable from '@/components/super-lotto/DataTable.vue'
import SearchFilters from '@/components/super-lotto/SearchFilters.vue'
import DataImport from '@/components/super-lotto/DataImport.vue'

interface HistoryParams {
  page?: number
  limit?: number
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
  start_date?: string
  end_date?: string
  front_numbers?: number[]
  back_numbers?: number[]
}

const superLottoStore = useSuperLottoStore()

// Reactive state
const currentPage = ref(1)
const pageSize = ref(100)

// Computed properties
const loading = computed(() => superLottoStore.loading)
const error = computed(() => superLottoStore.error)
const draws = computed(() => superLottoStore.draws)
const totalDraws = computed(() => superLottoStore.totalDraws)
const totalPages = computed(() => superLottoStore.totalPages)

// Methods
const loadDraws = async (params: Partial<HistoryParams> = {}) => {
  try {
    await superLottoStore.fetchDraws({
      limit: pageSize.value,
      offset: (currentPage.value - 1) * pageSize.value,
      ...params
    })
  } catch (err) {
    console.error('Failed to load draws:', err)
  }
}

const handleImportSuccess = async () => {
  // Reload data after successful import
  currentPage.value = 1
  await loadDraws()
}

const handleFilterChange = async (filters: HistoryParams) => {
  currentPage.value = 1
  await loadDraws(filters)
}

const handleDrawSelect = (draw: SuperLottoDraw) => {
  // Handle draw selection (show details, navigate to analysis, etc.)
  console.log('Selected draw:', draw)
}

const previousPage = async () => {
  if (currentPage.value > 1) {
    currentPage.value--
    await loadDraws()
  }
}

const nextPage = async () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
    await loadDraws()
  }
}

const clearError = () => {
  superLottoStore.clearError()
}

const showImportDialog = () => {
  // Show import dialog - this would be implemented with a modal or similar UI
  console.log('Show import dialog')
}

// Lifecycle
onMounted(async () => {
  await loadDraws()
})
</script>

<style scoped>
.super-lotto-history {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.page-header {
  text-align: center;
  margin-bottom: 30px;
}

.page-header h1 {
  color: #2c3e50;
  font-size: 2.5rem;
  margin-bottom: 10px;
}

.page-description {
  color: #7f8c8d;
  font-size: 1.1rem;
}

.controls-section {
  display: flex;
  gap: 20px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.import-section,
.filters-section {
  flex: 1;
  min-width: 250px;
}

.data-section {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  overflow: hidden;
}

.loading-container,
.error-container,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-message {
  color: #e74c3c;
  text-align: center;
}

.error-message .icon-error {
  font-size: 3rem;
  margin-bottom: 15px;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 20px;
}

.empty-state h3 {
  color: #2c3e50;
  margin-bottom: 10px;
}

.data-header {
  padding: 20px;
  border-bottom: 1px solid #ecf0f1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.data-header h3 {
  color: #2c3e50;
  margin: 0;
}

.data-stats {
  color: #7f8c8d;
  font-size: 0.9rem;
}

.data-stats span {
  margin-left: 15px;
}

.pagination {
  padding: 20px;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 15px;
  border-top: 1px solid #ecf0f1;
}

.page-info {
  color: #7f8c8d;
  font-weight: 500;
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

.btn-secondary {
  background-color: #95a5a6;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #7f8c8d;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Responsive design */
@media (max-width: 768px) {
  .controls-section {
    flex-direction: column;
  }

  .data-header {
    flex-direction: column;
    gap: 10px;
    text-align: center;
  }

  .data-stats span {
    margin: 0;
    display: block;
  }

  .pagination {
    flex-direction: column;
    gap: 10px;
  }
}
</style>