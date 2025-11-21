<template>
  <div class="data-table">
    <div class="table-container">
      <table class="lottery-table">
        <thead>
          <tr>
            <th>开奖日期</th>
            <th>期号</th>
            <th>前区</th>
            <th>后区</th>
            <th>和值</th>
            <th>奇偶</th>
            <th>连续</th>
            <th>奖池</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="draw in draws"
            :key="draw.id"
            @click="$emit('draw-select', draw)"
            class="data-row"
            :class="{ selected: selectedDraw?.id === draw.id }"
          >
            <td>{{ formatDate(draw.draw_date) }}</td>
            <td>{{ draw.draw_number || '-' }}</td>
            <td class="front-zone">
              <span
                v-for="number in draw.front_zone"
                :key="number"
                class="number front"
              >
                {{ number }}
              </span>
            </td>
            <td class="back-zone">
              <span
                v-for="number in draw.back_zone"
                :key="number"
                class="number back"
              >
                {{ number }}
              </span>
            </td>
            <td>{{ draw.get_sum_front() }}</td>
            <td>{{ draw.get_odd_count_front() }}/{{ draw.get_even_count_front() }}</td>
            <td>
              <span
                :class="['consecutive-indicator', {
                  'has-consecutive': draw.get_has_consecutive_front()
                }]"
              >
                {{ draw.get_has_consecutive_front() ? '是' : '否' }}
              </span>
            </td>
            <td>{{ formatCurrency(draw.jackpot_amount) }}</td>
            <td>
              <button
                @click.stop="$emit('show-details', draw)"
                class="btn btn-small"
              >
                详情
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="draws.length === 0 && !loading" class="empty-state">
      <div class="empty-icon">📊</div>
      <p>暂无数据</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { SuperLottoDraw } from '@/stores/superLotto'

// Props
interface Props {
  draws: SuperLottoDraw[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// Emits
const emit = defineEmits<{
  'draw-select': [draw: SuperLottoDraw]
  'show-details': [draw: SuperLottoDraw]
}>()

// State
const selectedDraw = ref<SuperLottoDraw | null>(null)

// Methods
const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateString
  }
}

const formatCurrency = (amount: number | null | undefined) => {
  if (amount === null || amount === undefined) return '-'
  return `¥${amount.toLocaleString('zh-CN')}`
}

const selectDraw = (draw: SuperLottoDraw) => {
  selectedDraw.value = draw
  emit('draw-select', draw)
}
</script>

<style scoped>
.data-table {
  width: 100%;
  overflow-x: auto;
}

.table-container {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
}

.lottery-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.lottery-table th {
  background: #f8f9fa;
  color: #2c3e50;
  padding: 12px 8px;
  text-align: left;
  font-weight: 600;
  border-bottom: 2px solid #ecf0f1;
}

.lottery-table td {
  padding: 12px 8px;
  border-bottom: 1px solid #ecf0f1;
  vertical-align: middle;
}

.data-row {
  cursor: pointer;
  transition: background-color 0.2s;
}

.data-row:hover {
  background-color: #f8f9fa;
}

.data-row.selected {
  background-color: #e8f4fd;
  border-left: 4px solid #3498db;
}

.front-zone {
  display: flex;
  gap: 8px;
}

.back-zone {
  display: flex;
  gap: 8px;
}

.number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  font-weight: bold;
  font-size: 0.9rem;
  transition: all 0.3s;
}

.number.front {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  border: 2px solid #5a6fd8;
}

.number.back {
  background: linear-gradient(135deg, #f093fb, #f5576c);
  color: white;
  border: 2px solid #e84393;
}

.number:hover {
  transform: scale(1.1);
}

.consecutive-indicator {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 500;
}

.consecutive-indicator.has-consecutive {
  background-color: #e8f5e8;
  color: #27ae60;
}

.consecutive-indicator:not(.has-consecutive) {
  background-color: #fdeaea;
  color: #e74c3c;
}

.btn {
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  transition: background-color 0.2s;
}

.btn-small {
  background-color: #3498db;
  color: white;
}

.btn-small:hover {
  background-color: #2980b9;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #7f8c8d;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 15px;
}

/* Responsive design */
@media (max-width: 768px) {
  .lottery-table {
    font-size: 0.8rem;
  }

  .lottery-table th,
  .lottery-table td {
    padding: 8px 6px;
  }

  .number {
    width: 25px;
    height: 25px;
    font-size: 0.8rem;
  }
}

@media (max-width: 640px) {
  .lottery-table {
    font-size: 0.7rem;
  }

  .lottery-table th,
  .lottery-table td {
    padding: 6px 4px;
  }

  .front-zone,
  .back-zone {
    gap: 4px;
  }

  .number {
    width: 20px;
    height: 20px;
    font-size: 0.7rem;
  }
}
</style>