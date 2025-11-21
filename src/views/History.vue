<template>
  <div class="history-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <h2>Lottery History</h2>
          <el-button type="primary" @click="goToDashboard">Back to Dashboard</el-button>
        </div>
      </template>

      <div class="filters">
        <el-row :gutter="20">
          <el-col :span="6">
            <el-select v-model="selectedLotteryType" placeholder="Select Lottery Type">
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
            />
          </el-col>
          <el-col :span="6">
            <el-button type="primary" @click="loadHistory">Search</el-button>
            <el-button @click="resetFilters">Reset</el-button>
          </el-col>
        </el-row>
      </div>

      <div v-loading="loading" class="history-table">
        <el-table :data="historyData" stripe>
          <el-table-column prop="draw_date" label="Date" width="120">
            <template #default="{ row }">
              {{ formatDate(row.draw_date) }}
            </template>
          </el-table-column>
          <el-table-column prop="lottery_type" label="Type" width="120">
            <template #default="{ row }">
              <el-tag>{{ formatLotteryType(row.lottery_type) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="Winning Numbers" width="300">
            <template #default="{ row }">
              <el-tag
                v-for="number in row.winning_numbers"
                :key="number"
                class="number-tag"
              >
                {{ number }}
              </el-tag>
              <el-tag
                v-if="row.bonus_number"
                type="warning"
                class="bonus-tag"
              >
                {{ row.bonus_number }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="jackpot_amount" label="Jackpot" width="120">
            <template #default="{ row }">
              {{ row.jackpot_amount ? `$${row.jackpot_amount}M` : 'N/A' }}
            </template>
          </el-table-column>
        </el-table>

        <el-pagination
          v-if="total > 0"
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { lotteryApi } from '@/api/tauri'
import type { LotteryDraw } from '@/types'

const router = useRouter()

const loading = ref(false)
const historyData = ref<LotteryDraw[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)

const selectedLotteryType = ref('')
const dateRange = ref<[string, string] | null>(null)

const loadHistory = async () => {
  try {
    loading.value = true
    const data = await lotteryApi.getLotteryHistory(
      selectedLotteryType.value || undefined,
      pageSize.value,
      (currentPage.value - 1) * pageSize.value
    )
    historyData.value = data
    total.value = data.length
  } catch (error) {
    console.error('Failed to load history:', error)
    ElMessage.error('Failed to load lottery history')
  } finally {
    loading.value = false
  }
}

const resetFilters = () => {
  selectedLotteryType.value = ''
  dateRange.value = null
  currentPage.value = 1
  loadHistory()
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadHistory()
}

const handleCurrentChange = (page: number) => {
  currentPage.value = page
  loadHistory()
}

const goToDashboard = () => {
  router.push('/dashboard')
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString()
}

const formatLotteryType = (type: string) => {
  return type.charAt(0).toUpperCase() + type.slice(1).replace(/([A-Z])/g, ' $1')
}

onMounted(() => {
  loadHistory()
})
</script>

<style scoped>
.history-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
}

.filters {
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 4px;
}

.history-table {
  margin-top: 20px;
}

.number-tag {
  margin-right: 8px;
  font-weight: bold;
}

.bonus-tag {
  margin-left: 8px;
  font-weight: bold;
  border-style: dashed;
}

.el-pagination {
  margin-top: 20px;
  text-align: center;
}
</style>