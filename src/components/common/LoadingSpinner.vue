<template>
  <div class="loading-container" :class="{ overlay: isOverlay }">
    <div class="loading-content">
      <div class="spinner" :class="size"></div>
      <p v-if="message" class="loading-message">{{ message }}</p>
      <div v-if="showProgress && progress !== undefined" class="progress-bar">
        <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  message?: string
  size?: 'small' | 'medium' | 'large'
  isOverlay?: boolean
  showProgress?: boolean
  progress?: number
}

withDefaults(defineProps<Props>(), {
  size: 'medium',
  isOverlay: false,
  showProgress: false
})
</script>

<style scoped>
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

.loading-container.overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  z-index: 9999;
}

.loading-content {
  text-align: center;
}

.spinner {
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

.spinner.small {
  width: 24px;
  height: 24px;
  border-width: 3px;
}

.spinner.medium {
  width: 40px;
  height: 40px;
}

.spinner.large {
  width: 60px;
  height: 60px;
  border-width: 6px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-message {
  color: #7f8c8d;
  margin: 0 0 20px 0;
  font-size: 0.95rem;
}

.progress-bar {
  width: 200px;
  height: 4px;
  background: #ecf0f1;
  border-radius: 2px;
  overflow: hidden;
  margin: 0 auto;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3498db, #2ecc71);
  transition: width 0.3s ease;
}

@media (max-width: 768px) {
  .loading-container {
    padding: 30px 20px;
  }

  .spinner.large {
    width: 50px;
    height: 50px;
  }

  .progress-bar {
    width: 150px;
  }
}
</style>
