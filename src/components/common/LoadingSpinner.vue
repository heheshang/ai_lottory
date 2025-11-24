<template>
  <div class="loading-spinner" :class="`loading-spinner--${size}`">
    <div class="spinner-circle">
      <div 
        class="spinner-path" 
        :style="{
          width: spinnerSize,
          height: spinnerSize,
          borderTopColor: color,
          borderLeftColor: color
        }"
      ></div>
    </div>
    
    <div v-if="message" class="loading-spinner__message">
      {{ message }}
    </div>
    
    <div v-if="showProgress && progress !== undefined" class="loading-spinner__progress">
      <div class="progress-bar">
        <div 
          class="progress-fill" 
          :style="{ width: `${progress}%`, backgroundColor: color }"
        ></div>
      </div>
      <span class="progress-text">{{ progress }}%</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  size?: 'small' | 'medium' | 'large'
  color?: string
  message?: string
  progress?: number
  showProgress?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 'medium',
  color: '#007bff',
  showProgress: false
})

const spinnerSize = computed(() => {
  const sizes = {
    small: '20px',
    medium: '32px',
    large: '48px'
  }
  return sizes[props.size]
})
</script>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.loading-spinner--small {
  padding: 10px;
}

.loading-spinner--large {
  padding: 30px;
}

.spinner-circle {
  position: relative;
  display: inline-block;
}

.spinner-path {
  border: 3px solid #f3f3f3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-spinner__message {
  margin-top: 12px;
  color: #6c757d;
  font-size: 14px;
  text-align: center;
  line-height: 1.4;
}

.loading-spinner__progress {
  margin-top: 16px;
  width: 200px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: #e9ecef;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: #6c757d;
  font-weight: 500;
}

/* Size variations */
.loading-spinner--small .loading-spinner__message {
  font-size: 12px;
  margin-top: 8px;
}

.loading-spinner--small .loading-spinner__progress {
  width: 150px;
  margin-top: 12px;
}

.loading-spinner--large .loading-spinner__message {
  font-size: 16px;
  margin-top: 16px;
}

.loading-spinner--large .loading-spinner__progress {
  width: 250px;
  margin-top: 20px;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .spinner-path {
    border-color: #495057;
  }
  
  .loading-spinner__message,
  .progress-text {
    color: #adb5bd;
  }
  
  .progress-bar {
    background: #495057;
  }
}
</style>
