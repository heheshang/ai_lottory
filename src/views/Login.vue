<template>
  <div class="login-container">
    <el-card class="login-card">
      <template #header>
        <h2>AI Lottery Prediction</h2>
        <p>Please login or register to continue</p>
      </template>

      <el-tabs v-model="activeTab" class="login-tabs">
        <el-tab-pane label="Login" name="login">
          <el-form
            ref="loginFormRef"
            :model="loginForm"
            :rules="loginRules"
            @submit.prevent="handleLogin"
          >
            <el-form-item prop="username">
              <el-input
                v-model="loginForm.username"
                placeholder="Username"
                size="large"
              />
            </el-form-item>
            <el-form-item prop="password">
              <el-input
                v-model="loginForm.password"
                type="password"
                placeholder="Password"
                size="large"
                show-password
                @keyup.enter="handleLogin"
              />
            </el-form-item>
            <el-form-item>
              <el-button
                type="primary"
                size="large"
                style="width: 100%"
                :loading="loading"
                @click="handleLogin"
              >
                Login
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="Register" name="register">
          <el-form
            ref="registerFormRef"
            :model="registerForm"
            :rules="registerRules"
            @submit.prevent="handleRegister"
          >
            <el-form-item prop="username">
              <el-input
                v-model="registerForm.username"
                placeholder="Username"
                size="large"
              />
            </el-form-item>
            <el-form-item prop="email">
              <el-input
                v-model="registerForm.email"
                placeholder="Email (optional)"
                size="large"
              />
            </el-form-item>
            <el-form-item prop="password">
              <el-input
                v-model="registerForm.password"
                type="password"
                placeholder="Password"
                size="large"
                show-password
              />
            </el-form-item>
            <el-form-item prop="confirmPassword">
              <el-input
                v-model="registerForm.confirmPassword"
                type="password"
                placeholder="Confirm Password"
                size="large"
                show-password
                @keyup.enter="handleRegister"
              />
            </el-form-item>
            <el-form-item>
              <el-button
                type="primary"
                size="large"
                style="width: 100%"
                :loading="loading"
                @click="handleRegister"
              >
                Register
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import type { FormRules, FormInstance } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import type { UserLogin, UserRegistration } from '@/types'

const router = useRouter()
const authStore = useAuthStore()

const activeTab = ref('login')
const loading = ref(false)

// Login form
const loginFormRef = ref<FormInstance>()
const loginForm = reactive<UserLogin>({
  username: '',
  password: ''
})

const loginRules: FormRules = {
  username: [
    { required: true, message: 'Please enter your username', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Please enter your password', trigger: 'blur' }
  ]
}

// Register form
const registerFormRef = ref<FormInstance>()
const registerForm = reactive<UserRegistration & { confirmPassword: string }>({
  username: '',
  email: '',
  password: '',
  confirmPassword: ''
})

const registerRules: FormRules = {
  username: [
    { required: true, message: 'Please enter a username', trigger: 'blur' },
    { min: 3, message: 'Username must be at least 3 characters', trigger: 'blur' }
  ],
  email: [
    { type: 'email', message: 'Please enter a valid email', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Please enter a password', trigger: 'blur' },
    { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
  ],
  confirmPassword: [
    { required: true, message: 'Please confirm your password', trigger: 'blur' },
    {
      validator: (rule, value, callback) => {
        if (value !== registerForm.password) {
          callback(new Error('Passwords do not match'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
}

const handleLogin = async () => {
  if (!loginFormRef.value) return

  try {
    console.log('🔵 [Login] Starting login process for username:', loginForm.username)
    await loginFormRef.value.validate()
    loading.value = true

    console.log('🔵 [Login] Form validation passed, calling auth store...')
    const result = await authStore.login(loginForm)

    console.log('🔵 [Login] Auth store result:', result)

    if (result.success) {
      ElMessage.success('Login successful!')
      console.log('🔵 [Login] Login successful, navigating to dashboard...')
      router.push('/dashboard')
    } else {
      console.error('🔴 [Login] Login failed:', result.error)
      ElMessage.error(result.error || 'Login failed')
    }
  } catch (error) {
    console.error('🔴 [Login] Login validation error:', error)
  } finally {
    loading.value = false
  }
}

const handleRegister = async () => {
  if (!registerFormRef.value) return

  try {
    await registerFormRef.value.validate()
    loading.value = true

    const { confirmPassword, ...userData } = registerForm
    const result = await authStore.register(userData)

    if (result.success) {
      ElMessage.success('Registration successful!')
      router.push('/dashboard')
    } else {
      ElMessage.error(result.error || 'Registration failed')
    }
  } catch (error) {
    console.error('Registration validation error:', error)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.login-tabs {
  margin-top: 20px;
}

.login-card :deep(.el-card__header) {
  text-align: center;
}

.login-card h2 {
  margin: 0 0 10px 0;
  color: #303133;
}

.login-card p {
  margin: 0;
  color: #606266;
  font-size: 14px;
}
</style>