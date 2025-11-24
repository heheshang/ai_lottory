<template>
  <div class="empty-state">
    <div v-if="icon" class="empty-state__icon">
      {{ icon }}
    </div>
    
    <div v-else-if="showDefaultIcon" class="empty-state__icon">
      <svg width="64" height="64" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" fill="#currentColor"/>
      </svg>
    </div>
    
    <h3 v-if="title" class="empty-state__title">
      {{ title }}
    </h3>
    
    <p v-if="description" class="empty-state__description">
      {{ description }}
    </p>
    
    <div v-if="$slots.default" class="empty-state__content">
      <slot />
    </div>
    
    <button 
      v-if="actionText" 
      @click="$emit('action')" 
      class="empty-state__action"
      :disabled="actionDisabled"
    >
      {{ actionText }}
    </button>
  </div>
</template>

<script setup lang="ts">
interface Props {
  icon?: string
  title?: string
  description?: string
  actionText?: string
  actionDisabled?: boolean
  showDefaultIcon?: boolean
}

withDefaults(defineProps<Props>(), {
  showDefaultIcon: true,
  actionDisabled: false
})

defineEmits<{
  action: []
}>()
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  color: #6c757d;
}

.empty-state__icon {
  margin-bottom: 20px;
  font-size: 4rem;
  color: #dee2e6;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state__icon svg {
  width: 64px;
  height: 64px;
  color: #dee2e6;
}

.empty-state__title {
  margin: 0 0 10px 0;
  color: #495057;
  font-size: 1.25rem;
  font-weight: 500;
}

.empty-state__description {
  margin: 0 0 20px 0;
  color: #6c757d;
  font-size: 1rem;
  line-height: 1.5;
  max-width: 400px;
}

.empty-state__content {
  margin-bottom: 20px;
}

.empty-state__action {
  padding: 10px 20px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.empty-state__action:hover:not(:disabled) {
  background: #0056b3;
}

.empty-state__action:disabled {
  background: #6c757d;
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
