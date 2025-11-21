<template>
  <div class="data-import">
    <div class="import-controls">
      <div class="import-section">
        <h3>导入大乐透数据</h3>
        <div class="import-methods">
          <div class="method-section">
            <h4>手动录入</h4>
            <div class="manual-input">
              <div class="input-group">
                <label>开奖日期:</label>
                <input
                  v-model="manualDraw.draw_date"
                  type="date"
                  class="form-input"
                  required
                />
              </div>
              <div class="input-group">
                <label>期号:</label>
                <input
                  v-model="manualDraw.draw_number"
                  type="text"
                  placeholder="例如: 2024001"
                  class="form-input"
                  required
                />
              </div>
              <div class="input-group">
                <label>前区号码 (5个):</label>
                <div class="number-inputs">
                  <input
                    v-for="(num, index) in manualDraw.front_zone"
                    :key="`front-${index}`"
                    v-model.number="manualDraw.front_zone[index]"
                    type="number"
                    min="1"
                    max="35"
                    class="number-input"
                    required
                  />
                </div>
              </div>
              <div class="input-group">
                <label>后区号码 (2个):</label>
                <div class="number-inputs">
                  <input
                    v-for="(num, index) in manualDraw.back_zone"
                    :key="`back-${index}`"
                    v-model.number="manualDraw.back_zone[index]"
                    type="number"
                    min="1"
                    max="12"
                    class="number-input"
                    required
                  />
                </div>
              </div>
              <div class="input-group">
                <label>奖池金额 (可选):</label>
                <input
                  v-model.number="manualDraw.jackpot_amount"
                  type="number"
                  min="0"
                  step="0.01"
                  placeholder="单位: 亿元"
                  class="form-input"
                />
              </div>
              <button
                @click="addManualDraw"
                :disabled="!isManualDrawValid || loading"
                class="btn btn-primary"
              >
                添加开奖数据
              </button>
            </div>
          </div>

          <div class="method-section">
            <h4>文件导入</h4>
            <div class="file-import">
              <div class="file-upload">
                <input
                  ref="fileInput"
                  type="file"
                  accept=".txt,.csv,.json"
                  @change="handleFileChange"
                  style="display: none"
                />
                <button
                  @click="$refs.fileInput.click()"
                  :disabled="loading"
                  class="btn btn-secondary"
                >
                  选择文件
                </button>
                <span v-if="selectedFile" class="file-name">
                  {{ selectedFile.name }}
                </span>
              </div>
              <div class="format-info">
                <p>支持的文件格式:</p>
                <ul>
                  <li>文本文件 (.txt): 每行一组数据，用逗号分隔</li>
                  <li>CSV文件 (.csv): 表格格式</li>
                  <li>JSON文件 (.json): 结构化数据</li>
                </ul>
              </div>
              <button
                @click="importFromFile"
                :disabled="!selectedFile || loading"
                class="btn btn-primary"
              >
                {{ loading ? '导入中...' : '开始导入' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="preview-section" v-if="previewData.length > 0">
        <h4>数据预览</h4>
        <div class="preview-table">
          <table>
            <thead>
              <tr>
                <th>日期</th>
                <th>期号</th>
                <th>前区</th>
                <th>后区</th>
                <th>状态</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(draw, index) in previewData.slice(0, 10)" :key="index">
                <td>{{ formatDate(draw.draw_date) }}</td>
                <td>{{ draw.draw_number }}</td>
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
                <td>
                  <span :class="['status', draw.valid ? 'valid' : 'invalid']">
                    {{ draw.valid ? '有效' : '无效' }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
          <p v-if="previewData.length > 10" class="preview-more">
            还有 {{ previewData.length - 10 }} 条数据...
          </p>
        </div>
        <div class="preview-actions">
          <button
            @click="confirmImport"
            :disabled="loading || !hasValidData"
            class="btn btn-success"
          >
            确认导入 ({{ validCount }} 条有效数据)
          </button>
          <button
            @click="cancelImport"
            :disabled="loading"
            class="btn btn-secondary"
          >
            取消
          </button>
        </div>
      </div>
    </div>

    <div class="import-status" v-if="importResult">
      <div :class="['status-message', importResult.success ? 'success' : 'error']">
        <h4>{{ importResult.success ? '导入成功' : '导入失败' }}</h4>
        <p>{{ importResult.message }}</p>
        <div v-if="importResult.details" class="import-details">
          <p>成功导入: {{ importResult.details.imported }} 条</p>
          <p v-if="importResult.details.skipped > 0">
            跳过: {{ importResult.details.skipped }} 条 (重复或无效)
          </p>
          <p v-if="importResult.details.errors > 0">
            错误: {{ importResult.details.errors }} 条
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSuperLottoStore } from '@/stores/superLotto'

interface ImportResult {
  success: boolean
  message: string
  details?: {
    imported: number
    skipped: number
    errors: number
  }
}

interface PreviewDraw {
  draw_date: string
  draw_number: string
  front_zone: number[]
  back_zone: number[]
  jackpot_amount?: number
  valid: boolean
  error?: string
}

const superLottoStore = useSuperLottoStore()

// State
const loading = ref(false)
const selectedFile = ref<File | null>(null)
const previewData = ref<PreviewDraw[]>([])
const importResult = ref<ImportResult | null>(null)

// Manual draw input
const manualDraw = ref({
  draw_date: '',
  draw_number: '',
  front_zone: [null, null, null, null, null] as (number | null)[],
  back_zone: [null, null] as (number | null)[],
  jackpot_amount: null as number | null
})

// Computed properties
const isManualDrawValid = computed(() => {
  return (
    manualDraw.value.draw_date &&
    manualDraw.value.draw_number &&
    manualDraw.value.front_zone.every(n => n && n >= 1 && n <= 35) &&
    manualDraw.value.back_zone.every(n => n && n >= 1 && n <= 12) &&
    new Set(manualDraw.value.front_zone).size === 5 &&
    new Set(manualDraw.value.back_zone).size === 2
  )
})

const hasValidData = computed(() => {
  return previewData.value.some(draw => draw.valid)
})

const validCount = computed(() => {
  return previewData.value.filter(draw => draw.valid).length
})

// Methods
const addManualDraw = async () => {
  if (!isManualDrawValid.value) return

  try {
    loading.value = true
    await superLottoStore.importSuperLottoDraws([{
      draw_date: manualDraw.value.draw_date,
      draw_number: manualDraw.value.draw_number,
      front_zone: manualDraw.value.front_zone as number[],
      back_zone: manualDraw.value.back_zone as number[],
      jackpot_amount: manualDraw.value.jackpot_amount
    }])

    // Reset form
    manualDraw.value = {
      draw_date: '',
      draw_number: '',
      front_zone: [null, null, null, null, null],
      back_zone: [null, null],
      jackpot_amount: null
    }

    importResult.value = {
      success: true,
      message: '开奖数据添加成功'
    }
  } catch (error) {
    importResult.value = {
      success: false,
      message: error instanceof Error ? error.message : '添加失败'
    }
  } finally {
    loading.value = false
  }
}

const handleFileChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  if (target.files && target.files[0]) {
    selectedFile.value = target.files[0]
    previewFile()
  }
}

const previewFile = () => {
  if (!selectedFile.value) return

  const reader = new FileReader()
  reader.onload = (e) => {
    const content = e.target?.result as string
    parseFileContent(content)
  }
  reader.readAsText(selectedFile.value)
}

const parseFileContent = (content: string) => {
  const lines = content.trim().split('\n')
  const draws: PreviewDraw[] = []

  lines.forEach((line, index) => {
    try {
      const draw = parseDrawLine(line.trim())
      draws.push({
        ...draw,
        valid: validateDraw(draw),
        error: validateDraw(draw) ? undefined : '数据格式不正确'
      })
    } catch (error) {
      draws.push({
        draw_date: '',
        draw_number: '',
        front_zone: [],
        back_zone: [],
        valid: false,
        error: '解析失败'
      })
    }
  })

  previewData.value = draws
}

const parseDrawLine = (line: string): Omit<PreviewDraw, 'valid' | 'error'> => {
  // Try to parse different formats
  const parts = line.split(/[,\t\s]+/)

  // Basic format: date, draw_number, front1, front2, front3, front4, front5, back1, back2
  if (parts.length >= 8) {
    const front = parts.slice(2, 7).map(n => parseInt(n.trim()))
    const back = parts.slice(7, 9).map(n => parseInt(n.trim()))

    return {
      draw_date: parts[0].trim(),
      draw_number: parts[1].trim(),
      front_zone: front,
      back_zone: back,
      jackpot_amount: parts[9] ? parseFloat(parts[9]) : undefined
    }
  }

  throw new Error('Unable to parse line format')
}

const validateDraw = (draw: Omit<PreviewDraw, 'valid' | 'error'>): boolean => {
  return (
    draw.front_zone.length === 5 &&
    draw.back_zone.length === 2 &&
    draw.front_zone.every(n => n >= 1 && n <= 35) &&
    draw.back_zone.every(n => n >= 1 && n <= 12) &&
    new Set(draw.front_zone).size === 5 &&
    new Set(draw.back_zone).size === 2
  )
}

const importFromFile = async () => {
  if (!selectedFile.value || !hasValidData.value) return

  try {
    loading.value = true
    const validDraws = previewData.value
      .filter(draw => draw.valid)
      .map(draw => ({
        draw_date: draw.draw_date,
        draw_number: draw.draw_number,
        front_zone: draw.front_zone,
        back_zone: draw.back_zone,
        jackpot_amount: draw.jackpot_amount
      }))

    await superLottoStore.importSuperLottoDraws(validDraws)

    importResult.value = {
      success: true,
      message: '文件导入成功',
      details: {
        imported: validCount.value,
        skipped: previewData.value.length - validCount.value,
        errors: 0
      }
    }

    // Clear preview
    previewData.value = []
    selectedFile.value = null
  } catch (error) {
    importResult.value = {
      success: false,
      message: error instanceof Error ? error.message : '导入失败'
    }
  } finally {
    loading.value = false
  }
}

const confirmImport = () => {
  importFromFile()
}

const cancelImport = () => {
  previewData.value = []
  selectedFile.value = null
  const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement
  if (fileInput) fileInput.value = ''
}

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString)
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateString
  }
}
</script>

<style scoped>
.data-import {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.import-controls {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.import-section h3 {
  color: #2c3e50;
  margin-bottom: 20px;
}

.import-methods {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 30px;
}

.method-section {
  padding: 20px;
  border: 1px solid #ecf0f1;
  border-radius: 8px;
  background: #fafafa;
}

.method-section h4 {
  color: #2c3e50;
  margin-bottom: 15px;
}

.manual-input {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  color: #2c3e50;
  font-weight: 500;
  font-size: 0.9rem;
}

.form-input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
}

.number-inputs {
  display: flex;
  gap: 10px;
}

.number-input {
  width: 60px;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  text-align: center;
  font-size: 0.9rem;
}

.file-import {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.file-upload {
  display: flex;
  align-items: center;
  gap: 15px;
}

.file-name {
  color: #7f8c8d;
  font-size: 0.9rem;
}

.format-info {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 4px;
  border: 1px solid #ecf0f1;
}

.format-info p {
  margin-bottom: 10px;
  color: #2c3e50;
  font-weight: 500;
}

.format-info ul {
  margin: 0;
  padding-left: 20px;
  color: #7f8c8d;
  font-size: 0.8rem;
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
  background-color: #6c757d;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background-color: #5a6268;
}

.btn-success {
  background-color: #27ae60;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background-color: #229954;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.preview-section {
  margin-top: 30px;
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.preview-section h4 {
  color: #2c3e50;
  margin-bottom: 15px;
}

.preview-table {
  max-height: 400px;
  overflow-y: auto;
}

.preview-table table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.preview-table th,
.preview-table td {
  padding: 8px;
  border-bottom: 1px solid #ecf0f1;
  text-align: left;
}

.preview-table th {
  background: #f8f9fa;
  font-weight: 600;
  color: #2c3e50;
}

.front-zone,
.back-zone {
  display: flex;
  gap: 5px;
}

.number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 25px;
  height: 25px;
  border-radius: 50%;
  font-size: 0.8rem;
  font-weight: bold;
}

.number.front {
  background: #e3f2fd;
  color: #1565c0;
}

.number.back {
  background: #f3e5f5;
  color: #7b1fa2;
}

.status {
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.8rem;
  font-weight: 500;
}

.status.valid {
  background: #e8f5e8;
  color: #27ae60;
}

.status.invalid {
  background: #fdeaea;
  color: #e74c3c;
}

.preview-more {
  margin-top: 10px;
  color: #7f8c8d;
  font-size: 0.8rem;
  font-style: italic;
}

.preview-actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.import-status {
  margin-top: 20px;
}

.status-message {
  padding: 15px;
  border-radius: 8px;
}

.status-message.success {
  background: #e8f5e8;
  color: #27ae60;
  border: 1px solid #c3e6c3;
}

.status-message.error {
  background: #fdeaea;
  color: #e74c3c;
  border: 1px solid #f5c6cb;
}

.status-message h4 {
  margin: 0 0 10px 0;
}

.import-details {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid rgba(0,0,0,0.1);
}

.import-details p {
  margin: 5px 0;
  font-size: 0.9rem;
}

/* Responsive design */
@media (max-width: 768px) {
  .import-methods {
    grid-template-columns: 1fr;
  }

  .number-inputs {
    flex-wrap: wrap;
  }

  .preview-actions {
    flex-direction: column;
  }
}
</style>