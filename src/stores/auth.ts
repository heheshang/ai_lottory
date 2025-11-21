import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/api/tauri'
import type { User, UserLogin, UserRegistration } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const isAuthenticated = computed(() => !!user.value)

  const login = async (credentials: UserLogin) => {
    try {
      console.log('🔵 [Auth Store] Starting login for user:', credentials.username)
      const userData = await authApi.login(credentials)
      console.log('🔵 [Auth Store] Login API successful, user data:', userData)

      user.value = userData
      localStorage.setItem('isAuthenticated', 'true')
      localStorage.setItem('userId', userData.id.toString())
      console.log('🔵 [Auth Store] User state and localStorage updated')

      return { success: true }
    } catch (error) {
      console.error('🔴 [Auth Store] Login failed:', error)
      return {
        success: false,
        error: typeof error === 'string' ? error : 'Login failed'
      }
    }
  }

  const register = async (userData: UserRegistration) => {
    try {
      const newUser = await authApi.register(userData)
      user.value = newUser
      localStorage.setItem('isAuthenticated', 'true')
      localStorage.setItem('userId', newUser.id.toString())
      return { success: true }
    } catch (error) {
      console.error('Registration failed:', error)
      return {
        success: false,
        error: typeof error === 'string' ? error : 'Registration failed'
      }
    }
  }

  const logout = async () => {
    try {
      await authApi.logout()
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      user.value = null
      localStorage.removeItem('isAuthenticated')
      localStorage.removeItem('userId')
    }
  }

  const checkAuth = async () => {
    const isAuth = localStorage.getItem('isAuthenticated') === 'true'
    const userId = localStorage.getItem('userId')

    if (isAuth && userId) {
      try {
        const userData = await authApi.getCurrentUser(parseInt(userId))
        if (userData) {
          user.value = userData
        } else {
          logout()
        }
      } catch (error) {
        console.error('Auth check failed:', error)
        logout()
      }
    }
  }

  return {
    user,
    isAuthenticated,
    login,
    register,
    logout,
    checkAuth
  }
})