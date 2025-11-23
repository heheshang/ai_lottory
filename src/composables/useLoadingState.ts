/**
 * Composable for managing loading states
 * Provides consistent loading state management across components
 */

import { ref, computed } from 'vue'

export interface LoadingStateOptions {
  initialLoading?: boolean
  defaultMessage?: string
}

export function useLoadingState(options: LoadingStateOptions = {}) {
  const {
    initialLoading = false,
    defaultMessage = 'Loading...'
  } = options

  const isLoading = ref(initialLoading)
  const loadingMessage = ref<string | undefined>(defaultMessage)
  const progress = ref<number | undefined>()

  const setLoading = (loading: boolean, message?: string, progressValue?: number) => {
    isLoading.value = loading
    loadingMessage.value = message
    progress.value = progressValue
  }

  const startLoading = (message?: string) => {
    setLoading(true, message)
  }

  const stopLoading = () => {
    setLoading(false)
  }

  const withLoading = async <T>(
    operation: () => Promise<T>,
    message?: string
  ): Promise<T> => {
    startLoading(message)
    try {
      return await operation()
    } finally {
      stopLoading()
    }
  }

  return {
    isLoading: computed(() => isLoading.value),
    loadingMessage: computed(() => loadingMessage.value),
    progress: computed(() => progress.value),
    setLoading,
    startLoading,
    stopLoading,
    withLoading
  }
}
