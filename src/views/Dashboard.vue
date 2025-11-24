<template>
  <div class="dashboard-container">
    <el-container>
      <!-- Header -->
      <el-header class="dashboard-header">
        <div class="header-left">
          <h1>AI Lottery Prediction</h1>
        </div>
        <div class="header-right">
          <span class="welcome-text"
            >Welcome, {{ authStore.user?.username }}!</span
          >
          <el-button type="primary" @click="handleLogout">Logout</el-button>
        </div>
      </el-header>

      <!-- Main Content -->
      <el-main>
        <el-row :gutter="20">
          <!-- Quick Stats -->
          <el-col :span="24">
            <el-card class="stats-card">
              <template #header>
                <h3>Quick Overview</h3>
              </template>
              <el-row :gutter="20">
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.totalDraws }}</div>
                    <div class="stat-label">Total Draws</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.hotNumbersCount }}</div>
                    <div class="stat-label">Hot Numbers</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.coldNumbersCount }}</div>
                    <div class="stat-label">Cold Numbers</div>
                  </div>
                </el-col>
                <el-col :span="6">
                  <div class="stat-item">
                    <div class="stat-number">{{ stats.avgJackpot }}</div>
                    <div class="stat-label">Avg Jackpot</div>
                  </div>
                </el-col>
              </el-row>
            </el-card>
          </el-col>

          <!-- Navigation Cards -->
          <el-col :span="8" v-for="card in navigationCards" :key="card.id">
            <el-card class="nav-card" @click="card.action">
              <div class="nav-content">
                <div
                  class="nav-icon"
                  :style="{ color: card.color }"
                  v-html="card.icon"
                ></div>
                <h3>{{ card.title }}</h3>
                <p>{{ card.description }}</p>
              </div>
            </el-card>
          </el-col>
          <!-- Recent Draws -->
          <el-col :span="24">
            <el-card class="recent-draws-card">
              <template #header>
                <div class="card-header">
                  <h3>Recent Draws</h3>
                  <el-button type="text" @click="goToHistory"
                    >View All</el-button
                  >
                </div>
              </template>
              <div v-loading="loading" class="recent-draws">
                <template v-if="recentDraws.length === 0">
                  <div class="empty-draws">
                    <p>No recent draws available</p>
                  </div>
                </template>
                <template v-else>
                  <div
                    v-for="draw in recentDraws"
                    :key="draw.id"
                    class="draw-item"
                  >
                    <div class="draw-date">
                      {{ formatDate(draw.draw_date) }}
                    </div>
                    <div class="draw-numbers">
                      <template
                        v-if="
                          draw.winning_numbers &&
                          draw.winning_numbers.length > 0
                        "
                      >
                        <el-tag
                          v-for="(number, index) in draw.winning_numbers"
                          :key="'num-' + index"
                          class="number-tag"
                        >
                          {{ number }}
                        </el-tag>
                      </template>
                      <el-tag
                        v-if="draw.bonus_number"
                        type="warning"
                        class="bonus-tag"
                      >
                        {{ draw.bonus_number }}
                      </el-tag>
                    </div>
                    <div class="draw-type">
                      {{ formatLotteryType(draw.lottery_type) }}
                    </div>
                  </div>
                </template>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { useAuthStore } from "@/stores/auth";
import { useDashboard } from "@/composables/useDashboard";

const authStore = useAuthStore();
const {
  loading,
  recentDraws,
  stats,
  navigationCards,
  handleLogout,
  formatLotteryType,
  formatDate
} = useDashboard();
</script>

<style scoped>
.dashboard-container {
  min-height: 100vh;
  background-color: #f5f5f5;
}

.dashboard-header {
  background: white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
}

.header-left h1 {
  margin: 0;
  color: #303133;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 20px;
}

.welcome-text {
  color: #606266;
  font-size: 16px;
}

.stats-card {
  margin-bottom: 20px;
}

.stat-item {
  text-align: center;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 14px;
  color: #606266;
}

.nav-card {
  cursor: pointer;
  transition: all 0.3s ease;
  margin-bottom: 20px;
}

.nav-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.nav-content {
  text-align: center;
  padding: 20px;
}

.nav-icon {
  width: 48px;
  height: 48px;
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-content h3 {
  margin: 15px 0 10px 0;
  color: #303133;
}

.nav-content p {
  margin: 0;
  color: #606266;
  font-size: 14px;
}

.recent-draws-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h3 {
  margin: 0;
}

.recent-draws {
  max-height: 400px;
  overflow-y: auto;
}

.empty-draws {
  text-align: center;
  padding: 40px 20px;
  color: #909399;
}

.empty-draws p {
  margin: 0;
  font-size: 14px;
}

.draw-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 15px 0;
  border-bottom: 1px solid #ebeef5;
}

.draw-item:last-child {
  border-bottom: none;
}

.draw-date {
  font-weight: 500;
  color: #606266;
  min-width: 100px;
}

.draw-numbers {
  display: flex;
  gap: 8px;
  flex: 1;
  justify-content: center;
}

.number-tag,
.bonus-tag {
  font-weight: bold;
}

.bonus-tag {
  border-style: dashed;
}

.draw-type {
  font-size: 12px;
  color: #909399;
  min-width: 100px;
  text-align: right;
}

.el-main {
  padding: 20px;
}
</style>
