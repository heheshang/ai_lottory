<template>
  <div 
    class="base-card"
    :class="[
      `base-card--${variant}`,
      {
        'base-card--loading': loading,
        'base-card--bordered': bordered,
        'base-card--shadow': shadow,
        'base-card--hoverable': hoverable
      }
    ]"
  >
    <div v-if="$slots.header" class="base-card__header">
      <slot name="header" />
    </div>
    
    <div class="base-card__body" :class="{ 'base-card__body--padded': padded }">
      <div v-if="loading" class="base-card__loading">
        <div class="loading-spinner"></div>
        <span v-if="loadingText" class="loading-text">{{ loadingText }}</span>
      </div>
      <slot v-else />
    </div>
    
    <div v-if="$slots.footer" class="base-card__footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info'
  loading?: boolean
  loadingText?: string
  bordered?: boolean
  shadow?: boolean
  hoverable?: boolean
  padded?: boolean
}

withDefaults(defineProps<Props>(), {
  variant: 'default',
  loading: false,
  bordered: true,
  shadow: true,
  hoverable: false,
  padded: true
})
</script>

<style scoped>
.base-card {
  background: white;
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.base-card--bordered {
  border: 1px solid #e1e5e9;
}

.base-card--shadow {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.base-card--hoverable:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.base-card__header {
  padding: 16px 20px;
  border-bottom: 1px solid #e1e5e9;
  background: #f8f9fa;
}

.base-card__body {
  position: relative;
}

.base-card__body--padded {
  padding: 20px;
}

.base-card__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #6c757d;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-text {
  font-size: 14px;
  margin-top: 8px;
}

.base-card__footer {
  padding: 16px 20px;
  border-top: 1px solid #e1e5e9;
  background: #f8f9fa;
}

/* Variant styles */
.base-card--primary {
  border-color: #007bff;
}

.base-card--primary .base-card__header,
.base-card--primary .base-card__footer {
  background: #e7f3ff;
  border-color: #007bff;
}

.base-card--success {
  border-color: #28a745;
}

.base-card--success .base-card__header,
.base-card--success .base-card__footer {
  background: #e8f5e8;
  border-color: #28a745;
}

.base-card--warning {
  border-color: #ffc107;
}

.base-card--warning .base-card__header,
.base-card--warning .base-card__footer {
  background: #fff8e1;
  border-color: #ffc107;
}

.base-card--danger {
  border-color: #dc3545;
}

.base-card--danger .base-card__header,
.base-card--danger .base-card__footer {
  background: #fde8e8;
  border-color: #dc3545;
}

.base-card--info {
  border-color: #17a2b8;
}

.base-card--info .base-card__header,
.base-card--info .base-card__footer {
  background: #e8f4f8;
  border-color: #17a2b8;
}
</style>
