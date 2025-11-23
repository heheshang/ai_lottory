<template>
  <div class="empty-state">
    <div class="empty-icon">{{ icon }}</div>
    <h3 v-if="title" class="empty-title">{{ title }}</h3>
    <p v-if="description" class="empty-description">{{ description }}</p>
    <slot name="actions">
      <button v-if="actionText" @click="handleAction" class="btn btn-primary">
        {{ actionText }}
      </button>
    </slot>
  </div>
</template>

<script setup lang="ts">
interface Props {
  icon?: string
  title?: string
  description?: string
  actionText?: string
}

const props = withDefaults(defineProps<Props>(), {
  icon: '📭',
  title: '暂无数据',
  description: ''
})

const emit = defineEmits<{
  action: []
}>()

const handleAction = () => {
  emit('action')
}
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 20px;
  opacity: 0.6;
}

.empty-title {
  color: #2c3e50;
  margin: 0 0 10px 0;
  font-size: 1.3rem;
}

.empty-description {
  color: #7f8c8d;
  margin: 0 0 20px 0;
  font-size: 1rem;
  max-width: 400px;
}

.btn {
  padding: 10px 24px;
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

.btn-primary:hover {
  background-color: #2980b9;
}

@media (max-width: 768px) {
  .empty-state {
    padding: 40px 20px;
  }

  .empty-icon {
    font-size: 3rem;
  }

  .empty-title {
    font-size: 1.1rem;
  }
}
</style>
