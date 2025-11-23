<template>
  <div :class="['base-card', variant, { hover: hoverable, clickable }]" @click="handleClick">
    <div v-if="$slots.header || title" class="card-header">
      <slot name="header">
        <h3 v-if="title" class="card-title">{{ title }}</h3>
      </slot>
      <div v-if="$slots.actions" class="card-actions">
        <slot name="actions"></slot>
      </div>
    </div>
    
    <div class="card-body" :class="{ padding: !noPadding }">
      <slot></slot>
    </div>
    
    <div v-if="$slots.footer" class="card-footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger'
  hoverable?: boolean
  clickable?: boolean
  noPadding?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  hoverable: false,
  clickable: false,
  noPadding: false
})

const emit = defineEmits<{
  click: []
}>()

const handleClick = () => {
  if (props.clickable) {
    emit('click')
  }
}
</script>

<style scoped>
.base-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  transition: all 0.3s ease;
}

.base-card.hover:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.base-card.clickable {
  cursor: pointer;
}

.base-card.primary {
  border: 2px solid #3498db;
  background: #f8f9ff;
}

.base-card.success {
  border: 2px solid #27ae60;
  background: #f0f9f4;
}

.base-card.warning {
  border: 2px solid #f39c12;
  background: #fffaf0;
}

.base-card.danger {
  border: 2px solid #e74c3c;
  background: #fff5f5;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #ecf0f1;
}

.card-title {
  margin: 0;
  font-size: 1.1rem;
  color: #2c3e50;
  font-weight: 600;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-body {
  color: #2c3e50;
}

.card-body.padding {
  padding: 20px;
}

.card-footer {
  padding: 16px 20px;
  border-top: 1px solid #ecf0f1;
  background: #f8f9fa;
}

@media (max-width: 768px) {
  .card-header,
  .card-body.padding,
  .card-footer {
    padding: 12px 16px;
  }
}
</style>
