/**
 * Auth Store Tests
 */

import { setActivePinia, createPinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useAuthStore } from '../../src/stores/auth'
import { mockTauriInvoke, createMockUser, createMockError } from '../setup'

describe('Auth Store', () => {
  let authStore: ReturnType<typeof useAuthStore>

  beforeEach(() => {
    const pinia = createPinia()
    setActivePinia(pinia)
    authStore = useAuthStore()
  })

  describe('Initial State', () => {
    it('should have correct initial state', () => {
      expect(authStore.isAuthenticated).toBe(false)
      expect(authStore.user).toBeNull()
      expect(authStore.sessionToken).toBeNull()
      expect(authStore.loading.login).toBe(false)
      expect(authStore.error).toBeNull()
      expect(authStore.lastError).toBeNull()
    })

    it('should have correct computed values', () => {
      expect(authStore.username).toBe('')
      expect(authStore.userId).toBeNull()
      expect(authStore.sessionValid).toBe(false)
      expect(authStore.hasError).toBe(false)
      expect(authStore.isLoggingIn).toBe(false)
    })
  })

  describe('Login', () => {
    it('should login successfully with valid credentials', async () => {
      const mockUser = createMockUser()
      mockTauriInvoke({
        user: mockUser,
        session_token: 'test_token',
        session_expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString()
      })

      const result = await authStore.login({
        username: 'testuser',
        password: 'password123'
      })

      expect(result).toBe(true)
      expect(authStore.isAuthenticated).toBe(true)
      expect(authStore.user).toEqual(mockUser)
      expect(authStore.username).toBe('testuser')
      expect(authStore.sessionToken).toBe('test_token')
      expect(authStore.loading.login).toBe(false)
      expect(authStore.error).toBeNull()
    })

    it('should handle login failure with invalid credentials', async () => {
      mockTauriInvoke({
        error: 'Invalid credentials'
      })

      await expect(
        authStore.login({
          username: 'invalid',
          password: 'wrong'
        })
      ).rejects.toThrow('Invalid credentials')

      expect(authStore.isAuthenticated).toBe(false)
      expect(authStore.user).toBeNull()
      expect(authStore.sessionToken).toBeNull()
      expect(authStore.loading.login).toBe(false)
      expect(authStore.error).toBe('Invalid credentials')
      expect(authStore.lastError).not.toBeNull()
    })

    it('should handle login with validation errors', async () => {
      const result = await authStore.login({
        username: '',
        password: ''
      })

      expect(result).toBe(false)
      expect(authStore.error).toContain('Username is required')
      expect(authStore.error).toContain('Password is required')
    })

    it('should handle API errors during login', async () => {
      mockTauriInvoke({
        code: 'NETWORK_ERROR',
        message: 'Network connection failed'
      })

      await expect(
        authStore.login({
          username: 'testuser',
          password: 'password123'
        })
      ).rejects.toThrow('Network connection failed')

      expect(authStore.error).toBe('Network connection failed')
      expect(authStore.lastError).toEqual(createMockError('Network connection failed', 'NETWORK_ERROR'))
    })

    it('should not login if already authenticated', async () => {
      const mockUser = createMockUser()
      authStore.$patch({
        user: mockUser,
        sessionToken: 'existing_token',
        isAuthenticated: true
      })

      mockTauriInvoke({}) // Should not be called

      await authStore.login({
        username: 'testuser',
        password: 'password123'
      })

      expect(authStore.user).toEqual(mockUser)
      expect(authStore.sessionToken).toBe('existing_token')
    })
  })

  describe('Logout', () => {
    beforeEach(() => {
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        isAuthenticated: true
      })
    })

    it('should logout successfully', async () => {
      mockTauriInvoke({ success: true })

      const result = await authStore.logout()

      expect(result).toBe(true)
      expect(authStore.isAuthenticated).toBe(false)
      expect(authStore.user).toBeNull()
      expect(authStore.sessionToken).toBeNull()
      expect(authStore.loading.logout).toBe(false)
    })

    it('should handle logout failure', async () => {
      mockTauriInvoke({
        error: 'Logout failed'
      })

      await expect(authStore.logout()).rejects.toThrow('Logout failed')

      expect(authStore.error).toBe('Logout failed')
      expect(authStore.lastError).not.toBeNull()
    })

    it('should clear all state on logout', async () => {
      mockTauriInvoke({ success: true })

      await authStore.logout()

      expect(authStore.username).toBe('')
      expect(authStore.userId).toBeNull()
      expect(authStore.sessionValid).toBe(false)
    })
  })

  describe('Session Management', () => {
    it('should detect valid session', () => {
      const futureDate = new Date(Date.now() + 24 * 60 * 60 * 1000)
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        sessionExpiresAt: futureDate.toISOString(),
        isAuthenticated: true
      })

      expect(authStore.sessionValid).toBe(true)
    })

    it('should detect expired session', () => {
      const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000)
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        sessionExpiresAt: pastDate.toISOString(),
        isAuthenticated: true
      })

      expect(authStore.sessionValid).toBe(false)
    })

    it('should validate session correctly', async () => {
      const futureDate = new Date(Date.now() + 24 * 60 * 60 * 1000)
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        sessionExpiresAt: futureDate.toISOString(),
        isAuthenticated: true
      })

      mockTauriInvoke({ valid: true })

      const result = await authStore.validateSession()

      expect(result).toBe(true)
      expect(authStore.sessionValid).toBe(true)
    })

    it('should invalidate session on validation failure', async () => {
      const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000)
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        sessionExpiresAt: pastDate.toISOString(),
        isAuthenticated: true
      })

      mockTauriInvoke({ valid: false })

      const result = await authStore.validateSession()

      expect(result).toBe(false)
      expect(authStore.isAuthenticated).toBe(false)
      expect(authStore.user).toBeNull()
      expect(authStore.sessionToken).toBeNull()
    })
  })

  describe('User Preferences', () => {
    it('should update preferences', () => {
      authStore.$patch({
        user: createMockUser()
      })

      const newPreferences = {
        theme: 'dark',
        language: 'fr',
        notifications: false
      }

      authStore.updatePreferences(newPreferences)

      expect(authStore.user?.preferences).toEqual({
        ...authStore.user?.preferences,
        ...newPreferences
      })
    })

    it('should handle preferences update without user', () => {
      const newPreferences = {
        theme: 'dark'
      }

      // Should not throw error
      expect(() => {
        authStore.updatePreferences(newPreferences)
      }).not.toThrow()
    })
  })

  describe('Session Timeout', () => {
    it('should set session timeout', () => {
      authStore.setSessionTimeout(60)

      expect(authStore.sessionTimeoutMinutes).toBe(60)
    })

    it('should update session expiry', () => {
      const futureDate = new Date(Date.now() + 24 * 60 * 60 * 1000)
      authStore.$patch({
        user: createMockUser(),
        sessionCreatedAt: new Date(Date.now() - 60 * 60 * 1000).toISOString()
      })

      authStore.updateSessionExpiry()

      const expectedExpiry = new Date(Date.now() + 30 * 60 * 1000) // 30 minutes from now
      expect(authStore.sessionExpiresAt).toBeCloseTo(expectedExpiry.toISOString(), 1000)
    })

    it('should detect session timeout', () => {
      const pastDate = new Date(Date.now() - 60 * 60 * 1000)
      authStore.$patch({
        sessionCreatedAt: pastDate.toISOString(),
        sessionTimeoutMinutes: 30
      })

      expect(authStore.sessionExpired).toBe(true)
    })
  })

  describe('Error Handling', () => {
    it('should clear error', () => {
      authStore.$patch({
        error: 'Test error'
      })

      authStore.clearError()

      expect(authStore.error).toBeNull()
    })

    it('should clear last error', () => {
      authStore.$patch({
        lastError: createMockError('Test error')
      })

      authStore.clearLastError()

      expect(authStore.lastError).toBeNull()
    })

    it('should handle multiple errors correctly', async () => {
      // First error
      mockTauriInvoke({
        error: 'First error'
      })

      await expect(
        authStore.login({
          username: 'test',
          password: 'wrong'
        })
      ).rejects.toThrow()

      expect(authStore.error).toBe('First error')
      expect(authStore.lastError?.message).toBe('First error')

      // Second error should replace first
      mockTauriInvoke({
        error: 'Second error'
      })

      await expect(
        authStore.login({
          username: 'test2',
          password: 'wrong2'
        })
      ).rejects.toThrow()

      expect(authStore.error).toBe('Second error')
      expect(authStore.lastError?.message).toBe('Second error')
    })
  })

  describe('State Management', () => {
    it('should reset store correctly', () => {
      authStore.$patch({
        user: createMockUser(),
        sessionToken: 'test_token',
        isAuthenticated: true,
        error: 'Test error'
      })

      authStore.resetStore()

      expect(authStore.isAuthenticated).toBe(false)
      expect(authStore.user).toBeNull()
      expect(authStore.sessionToken).toBeNull()
      expect(authStore.error).toBeNull()
      expect(authStore.lastError).toBeNull()
    })

    it('should save state correctly', () => {
      const mockUser = createMockUser()
      authStore.$patch({
        user: mockUser,
        sessionToken: 'test_token',
        isAuthenticated: true
      })

      // Mock localStorage
      const setItemSpy = vi.spyOn(localStorage, 'setItem')

      authStore.saveState()

      expect(setItemSpy).toHaveBeenCalledWith(
        'auth-store',
        expect.stringContaining(mockUser.username)
      )
    })

    it('should load state correctly', () => {
      const mockUser = createMockUser()
      const savedState = JSON.stringify({
        user: mockUser,
        sessionToken: 'test_token',
        isAuthenticated: true
      })

      // Mock localStorage
      vi.spyOn(localStorage, 'getItem').mockReturnValue(savedState)

      authStore.loadState()

      expect(authStore.user).toEqual(mockUser)
      expect(authStore.sessionToken).toBe('test_token')
      expect(authStore.isAuthenticated).toBe(true)
    })
  })
})