import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { ElMessage } from 'element-plus'
import Login from '@/views/Login.vue'

// Mock Tauri commands
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('Authentication', () => {
  let wrapper: any
  let pinia: any

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    // Reset mocks
    vi.clearAllMocks()

    wrapper = mount(Login, {
      global: {
        plugins: [pinia],
        stubs: {
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-button': true,
          'el-link': true
        }
      }
    })
  })

  it('should render login form correctly', () => {
    expect(wrapper.find('[data-testid="login-form"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="username-input"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="password-input"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="login-button"]').exists()).toBe(true)
  })

  it('should validate required fields', async () => {
    const loginButton = wrapper.find('[data-testid="login-button"]')

    // Try to login with empty form
    await loginButton.trigger('click')

    // Should show validation error
    expect(wrapper.vm.formValid).toBe(false)
  })

  it('should handle successful login', async () => {
    const mockLogin = vi.fn().mockResolvedValue({
      id: 1,
      username: 'testuser',
      email: 'test@example.com'
    })

    vi.doMock('@/api/auth', () => ({
      login: mockLogin
    }))

    wrapper.vm.loginForm.username = 'testuser'
    wrapper.vm.loginForm.password = 'password123'

    await wrapper.vm.handleLogin()

    expect(mockLogin).toHaveBeenCalledWith({
      username: 'testuser',
      password: 'password123'
    })
  })

  it('should handle login error gracefully', async () => {
    const mockLogin = vi.fn().mockRejectedValue(new Error('Invalid credentials'))

    vi.doMock('@/api/auth', () => ({
      login: mockLogin
    }))

    wrapper.vm.loginForm.username = 'wronguser'
    wrapper.vm.loginForm.password = 'wrongpass'

    await wrapper.vm.handleLogin()

    expect(ElMessage.error).toHaveBeenCalledWith('登录失败，请重试')
  })

  it('should validate email format', async () => {
    const wrapper = mount(Login, {
      global: {
        plugins: [pinia],
        stubs: {
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-button': true,
          'el-link': true
        }
      }
    })

    // Switch to registration mode
    await wrapper.vm.switchMode('register')

    // Test invalid email
    wrapper.vm.registerForm.email = 'invalid-email'
    const isValid = wrapper.vm.validateEmail(wrapper.vm.registerForm.email)

    expect(isValid).toBe(false)
  })

  it('should enforce password strength requirements', async () => {
    const wrapper = mount(Login, {
      global: {
        plugins: [pinia],
        stubs: {
          'el-form': true,
          'el-form-item': true,
          'el-input': true,
          'el-button': true,
          'el-link': true
        }
      }
    })

    // Switch to registration mode
    await wrapper.vm.switchMode('register')

    // Test weak password
    wrapper.vm.registerForm.password = '123'
    wrapper.vm.registerForm.confirmPassword = '123'

    const isPasswordValid = wrapper.vm.validatePassword()

    expect(isPasswordValid).toBe(false)
  })
})