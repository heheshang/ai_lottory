<!--
Lazy Image Component - Optimized image loading with progressive enhancement

Features:
- Intersection Observer for lazy loading
- Progressive image loading with placeholders
- Blurhash/low quality image placeholder support
- Error handling with fallback
- Performance monitoring
- Accessibility support
- Responsive image support
-->

<template>
  <div
    ref="containerRef"
    class="lazy-image"
    :class="containerClass"
    :style="containerStyle"
    :aria-label="alt"
    role="img"
    @mouseenter="preloadImage"
    @focus="preloadImage"
  >
    <!-- Low quality placeholder -->
    <img
      v-if="placeholderSrc && !isLoaded"
      :src="placeholderSrc"
      :alt="alt"
      :style="placeholderStyle"
      class="lazy-image-placeholder"
      loading="eager"
      aria-hidden="true"
    />

    <!-- Blurhash canvas placeholder -->
    <canvas
      v-if="blurhash && !isLoaded && !placeholderSrc"
      ref="canvasRef"
      class="lazy-image-blurhash"
      :style="blurhashStyle"
      width="width"
      height="height"
      aria-hidden="true"
    />

    <!-- Main image -->
    <img
      ref="imageRef"
      v-show="shouldShowImage"
      :src="currentSrc"
      :srcset="srcset"
      :sizes="sizes"
      :alt="alt"
      :style="imageStyle"
      class="lazy-image-main"
      :loading="eager ? 'eager' : 'lazy'"
      @load="handleImageLoad"
      @error="handleImageError"
      @loadstart="handleLoadStart"
      @loadend="handleLoadEnd"
    />

    <!-- Loading indicator -->
    <div v-if="isLoading" class="lazy-image-loading">
      <div class="loading-spinner"></div>
      <div class="loading-text">{{ loadingText }}</div>
    </div>

    <!-- Error state -->
    <div v-if="hasError && !isLoaded" class="lazy-image-error">
      <div class="error-icon">⚠️</div>
      <div class="error-text">{{ errorText }}</div>
      <button v-if="retryable" class="retry-button" @click="retryLoad">
        Retry
      </button>
    </div>

    <!-- Skeleton loader -->
    <div v-if="showSkeleton && !isLoaded && !hasError" class="lazy-image-skeleton">
      <div class="skeleton-content"></div>
    </div>

    <!-- Progress indicator -->
    <div v-if="showProgress && isLoading" class="lazy-image-progress">
      <div class="progress-bar" :style="{ width: `${loadProgress}%` }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  nextTick,
  watch,
  type PropType
} from 'vue'
import { usePerformanceMonitoring } from '../../composables/usePerformanceMonitoring'
import { useLogger } from '../../composables/useLogger'

interface LazyImageProps {
  src: string
  alt: string
  placeholderSrc?: string
  blurhash?: string
  srcset?: string
  sizes?: string
  width?: number
  height?: number
  aspectRatio?: number
  eager?: boolean
  loadingText?: string
  errorText?: string
  retryable?: boolean
  showSkeleton?: boolean
  showProgress?: boolean
  threshold?: number
  rootMargin?: string
  fadeInDuration?: number
  backgroundColor?: string
  objectFit?: 'cover' | 'contain' | 'fill' | 'none' | 'scale-down'
  maxRetries?: number
  retryDelay?: number
}

const props = withDefaults(defineProps<LazyImageProps>(), {
  placeholderSrc: '',
  blurhash: '',
  srcset: '',
  sizes: '',
  width: 300,
  height: 200,
  aspectRatio: 0,
  eager: false,
  loadingText: 'Loading...',
  errorText: 'Failed to load image',
  retryable: true,
  showSkeleton: true,
  showProgress: false,
  threshold: 0.1,
  rootMargin: '50px',
  fadeInDuration: 300,
  backgroundColor: '#f3f4f6',
  objectFit: 'cover',
  maxRetries: 3,
  retryDelay: 1000
})

const emit = defineEmits([
  'load',
  'error',
  'load-start',
  'load-end',
  'progress',
  'placeholder-load',
  'placeholder-error'
])

// Reactive state
const containerRef = ref<HTMLElement>()
const imageRef = ref<HTMLImageElement>()
const canvasRef = ref<HTMLCanvasElement>()
const observer = ref<IntersectionObserver | null>(null)
const isIntersecting = ref(false)
const isLoaded = ref(false)
const hasError = ref(false)
const isLoading = ref(false)
const loadProgress = ref(0)
const retryCount = ref(0)
const performanceMonitor = usePerformanceMonitoring()
const logger = useLogger('lazy-image')

// Computed properties
const containerClass = computed(() => ({
  'lazy-image--loaded': isLoaded.value,
  'lazy-image--error': hasError.value,
  'lazy-image--loading': isLoading.value,
  'lazy-image--intersecting': isIntersecting.value,
  'lazy-image--skeleton': props.showSkeleton && !isLoaded.value && !hasError.value
}))

const containerStyle = computed(() => ({
  width: props.width ? `${props.width}px` : '100%',
  height: props.height ? `${props.height}px` : 'auto',
  aspectRatio: props.aspectRatio ? `${props.aspectRatio}` : undefined,
  backgroundColor: props.backgroundColor,
  objectFit: props.objectFit
}))

const placeholderStyle = computed(() => ({
  width: '100%',
  height: '100%',
  objectFit: props.objectFit,
  filter: 'blur(10px)',
  transform: 'scale(1.1)',
  transition: 'opacity 0.3s ease'
}))

const blurhashStyle = computed(() => ({
  width: '100%',
  height: '100%',
  objectFit: props.objectFit
}))

const imageStyle = computed(() => ({
  width: '100%',
  height: '100%',
  objectFit: props.objectFit,
  opacity: isLoaded.value ? 1 : 0,
  transition: `opacity ${props.fadeInDuration}ms ease`
}))

const shouldShowImage = computed(() => {
  return (isIntersecting.value || props.eager) && !hasError.value
})

const currentSrc = computed(() => {
  if (hasError.value && retryCount.value > 0) {
    return props.src // Retry with original source
  }
  return props.src
})

// Methods
const decodeBlurhash = (blurhash: string) => {
  try {
    // This is a simplified blurhash decoder
    // In a real implementation, you would use a proper blurhash library
    const size = 32 // Standard blurhash size
    const canvas = canvasRef.value
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    canvas.width = size
    canvas.height = size

    // Create a simple gradient placeholder
    const gradient = ctx.createLinearGradient(0, 0, size, size)
    gradient.addColorStop(0, '#e0e0e0')
    gradient.addColorStop(1, '#f0f0f0')
    ctx.fillStyle = gradient
    ctx.fillRect(0, 0, size, size)

    // Add some noise pattern to simulate blurhash
    const imageData = ctx.getImageData(0, 0, size, size)
    const data = imageData.data

    for (let i = 0; i < data.length; i += 4) {
      const noise = Math.random() * 20 - 10
      data[i] = Math.max(0, Math.min(255, data[i] + noise))
      data[i + 1] = Math.max(0, Math.min(255, data[i + 1] + noise))
      data[i + 2] = Math.max(0, Math.min(255, data[i + 2] + noise))
    }

    ctx.putImageData(imageData, 0, 0)
  } catch (error) {
    logger.error('Failed to decode blurhash', error as Error, {
      blurhash,
      width: props.width,
      height: props.height
    })
  }
}

const setupIntersectionObserver = () => {
  if (props.eager) return

  observer.value = new IntersectionObserver(
    (entries) => {
      const entry = entries[0]
      if (entry.isIntersecting) {
        isIntersecting.value = true
        loadImage()

        // Stop observing once loaded
        if (observer.value) {
          observer.value.unobserve(containerRef.value!)
          observer.value = null
        }
      }
    },
    {
      threshold: props.threshold,
      rootMargin: props.rootMargin
    }
  )

  if (containerRef.value) {
    observer.value.observe(containerRef.value)
  }
}

const preloadImage = () => {
  if (props.eager || isIntersecting.value) return

  // Preload without showing
  const img = new Image()
  img.src = props.src

  img.onload = () => {
    logger.debug('Image preloaded', { src: props.src })
  }

  img.onerror = () => {
    logger.warn('Image preload failed', { src: props.src })
  }
}

const loadImage = () => {
  if (isLoaded.value || isLoading.value) return

  isLoading.value = true
  loadProgress.value = 0
  hasError.value = false

  emit('load-start')

  // Track performance
  const startTime = performance.now()

  if (imageRef.value && imageRef.value.complete) {
    // Already loaded from cache
    handleImageLoad()
    return
  }

  // Simulate progress (in real implementation, you'd use fetch progress)
  if (props.showProgress) {
    const progressInterval = setInterval(() => {
      if (loadProgress.value < 90) {
        loadProgress.value += Math.random() * 20
        emit('progress', loadProgress.value)
      } else {
        clearInterval(progressInterval)
      }
    }, 100)
  }

  // Set a timeout for error handling
  const timeout = setTimeout(() => {
    if (!isLoaded.value && isLoading.value) {
      handleImageError(new Error('Image load timeout'))
    }
  }, 10000) // 10 second timeout

  // Store timeout for cleanup
  if (imageRef.value) {
    imageRef.value.dataset.timeout = timeout.toString()
  }
}

const handleImageLoad = () => {
  const startTime = performanceMonitor.startTimer(
    `image-load-${props.src}`,
    'image-loading',
    { component: 'LazyImage', src: props.src }
  )

  isLoaded.value = true
  isLoading.value = false
  hasError.value = false
  retryCount.value = 0
  loadProgress.value = 100

  if (startTime) {
    performanceMonitor.endTimer(startTime, { success: true })
  }

  emit('load')
  emit('progress', 100)
  emit('load-end')

  logger.info('Image loaded successfully', {
    src: props.src,
    width: props.width,
    height: props.height,
    loadTime: startTime ? performance.now() - startTime : 0
  })

  // Clear timeout
  if (imageRef.value?.dataset.timeout) {
    clearTimeout(parseInt(imageRef.value.dataset.timeout))
    delete imageRef.value.dataset.timeout
  }
}

const handleImageError = (error?: Event) => {
  const errorMessage = error ? (error as Error).message : 'Unknown error'
  const startTime = performanceMonitor.startTimer(
    `image-load-error-${props.src}`,
    'image-loading',
    { component: 'LazyImage', src: props.src, error: errorMessage }
  )

  isLoading.value = false
  hasError.value = true
  loadProgress.value = 0

  if (startTime) {
    performanceMonitor.endTimer(startTime, { success: false, error: errorMessage })
  }

  emit('error', error)
  emit('load-end')

  logger.error('Image load failed', error as Error, {
    src: props.src,
    retryCount: retryCount.value,
    maxRetries: props.maxRetries
  })

  // Clear timeout
  if (imageRef.value?.dataset.timeout) {
    clearTimeout(parseInt(imageRef.value.dataset.timeout))
    delete imageRef.value.dataset.timeout
  }

  // Auto-retry if allowed
  if (props.retryable && retryCount.value < props.maxRetries) {
    retryLoad()
  }
}

const handleLoadStart = () => {
  isLoading.value = true
  emit('load-start')
}

const handleLoadEnd = () => {
  isLoading.value = false
  emit('load-end')
}

const retryLoad = () => {
  if (retryCount.value >= props.maxRetries) {
    logger.warn('Max retries reached', {
      src: props.src,
      retryCount: retryCount.value
    })
    return
  }

  retryCount.value++

  logger.info('Retrying image load', {
    src: props.src,
    retryCount: retryCount.value,
    delay: props.retryDelay
  })

  // Clear current image
  if (imageRef.value) {
    imageRef.value.src = ''
  }

  // Reset state and retry after delay
  setTimeout(() => {
    hasError.value = false
    loadImage()
  }, props.retryDelay * retryCount.value) // Exponential backoff
}

const retryLoadManual = () => {
  retryCount.value = 0
  hasError.value = false
  loadImage()
}

// Watchers
watch(() => props.src, (newSrc, oldSrc) => {
  if (newSrc !== oldSrc) {
    // Reset state when src changes
    isLoaded.value = false
    hasError.value = false
    isLoading.value = false
    loadProgress.value = 0
    retryCount.value = 0

    // Setup new observer
    setupIntersectionObserver()
  }
})

watch(() => props.blurhash, (newBlurhash) => {
  if (newBlurhash && canvasRef.value) {
    nextTick(() => {
      decodeBlurhash(newBlurhash)
    })
  }
}, { immediate: true })

// Lifecycle
onMounted(() => {
  setupIntersectionObserver()

  // Decode blurhash if provided
  if (props.blurhash && canvasRef.value) {
    nextTick(() => {
      decodeBlurhash(props.blurhash)
    })
  }

  logger.debug('LazyImage mounted', {
    src: props.src,
    width: props.width,
    height: props.height,
    eager: props.eager
  })
})

onUnmounted(() => {
  if (observer.value) {
    observer.value.disconnect()
    observer.value = null
  }

  // Clear any pending timeout
  if (imageRef.value?.dataset.timeout) {
    clearTimeout(parseInt(imageRef.value.dataset.timeout))
  }

  logger.debug('LazyImage unmounted', { src: props.src })
})

// Expose methods
defineExpose({
  retryLoad: retryLoadManual,
  preloadImage,
  loadImage
})
</script>

<style scoped>
.lazy-image {
  position: relative;
  display: inline-block;
  overflow: hidden;
  border-radius: 4px;
  background-color: var(--lazy-image-bg, #f3f4f6);
}

.lazy-image--loaded {
  background-color: transparent;
}

.lazy-image--loading {
  background-color: var(--lazy-image-bg, #f3f4f6);
}

.lazy-image--error {
  background-color: var(--lazy-image-error-bg, #fee2e2);
}

.lazy-image-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.lazy-image-blurhash {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.lazy-image-main {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.lazy-image-loading {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.1);
  color: #666;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 8px;
}

.loading-text {
  font-size: 14px;
  text-align: center;
}

.lazy-image-error {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--lazy-image-error-bg, #fee2e2);
  color: var(--lazy-image-error-text, #991b1b);
}

.error-icon {
  font-size: 24px;
  margin-bottom: 8px;
}

.error-text {
  font-size: 14px;
  text-align: center;
  margin-bottom: 12px;
}

.retry-button {
  padding: 6px 12px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.retry-button:hover {
  background: #2563eb;
}

.lazy-image-skeleton {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-content {
  width: 60%;
  height: 20px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
}

.lazy-image-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: rgba(0, 0, 0, 0.1);
}

.progress-bar {
  height: 100%;
  background: #3b82f6;
  transition: width 0.3s ease;
}

/* Animations */
@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .lazy-image--error {
    background-color: #000;
  }

  .error-text {
    color: #fff;
  }

  .loading-text {
    color: #000;
  }
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .loading-spinner {
    animation: none;
    border-top-color: #3b82f6;
  }

  .lazy-image-skeleton {
    animation: none;
    background: #e0e0e0;
  }

  .lazy-image-main {
    transition: none;
  }
}

/* Focus styles */
.lazy-image:focus {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}
</style>