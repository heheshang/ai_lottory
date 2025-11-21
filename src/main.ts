import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router from './router'

console.log('🟢 [Main] Application starting up...')

const app = createApp(App)

console.log('🟢 [Main] Vue app created, registering icons...')

// Register Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

console.log('🟢 [Main] Icons registered, adding plugins...')

app.use(createPinia())
app.use(router)
app.use(ElementPlus)

console.log('🟢 [Main] Plugins added, setting up error handlers...')

// Global error handler
app.config.errorHandler = (error, instance, info) => {
  console.error('🔴 [Global Error] Unhandled error:', error)
  console.error('🔴 [Global Error] Component instance:', instance)
  console.error('🔴 [Global Error] Error info:', info)
}

// Handle unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
  console.error('🔴 [Unhandled Rejection]:', event.reason)
})

console.log('🟢 [Main] Error handlers set up, mounting app...')

app.mount('#app')

console.log('🟢 [Main] Application mounted successfully!')