import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

// Import views (will create these next)
const Login = () => import('@/views/Login.vue')
const Dashboard = () => import('@/views/Dashboard.vue')
const History = () => import('@/views/History.vue')
const HotNumbers = () => import('@/views/HotNumbers.vue')
const ColdNumbers = () => import('@/views/ColdNumbers.vue')

// Super Lotto views
const SuperLottoHistory = () => import('@/views/SuperLottoHistory.vue')
const HotColdAnalysis = () => import('@/views/HotColdAnalysis.vue')
const PredictionDashboard = () => import('@/views/PredictionDashboard.vue')
const PatternAnalysis = () => import('@/views/PatternAnalysis.vue')
const OneClickPrediction = () => import('@/components/super-lotto/OneClickPrediction.vue')

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/dashboard'
  },
  {
    path: '/login',
    name: 'Login',
    component: Login,
    meta: {
      requiresAuth: false,
      title: 'Login - AI Lottery Prediction'
    }
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: Dashboard,
    meta: {
      requiresAuth: true,
      title: 'Dashboard - AI Lottery Prediction'
    }
  },
  {
    path: '/history',
    name: 'History',
    component: History,
    meta: {
      requiresAuth: true,
      title: 'Lottery History - AI Lottery Prediction'
    }
  },
  {
    path: '/hot-numbers',
    name: 'HotNumbers',
    component: HotNumbers,
    meta: {
      requiresAuth: true,
      title: 'Hot Numbers - AI Lottery Prediction'
    }
  },
  {
    path: '/cold-numbers',
    name: 'ColdNumbers',
    component: ColdNumbers,
    meta: {
      requiresAuth: true,
      title: 'Cold Numbers - AI Lottery Prediction'
    }
  },
  // Super Lotto routes
  {
    path: '/super-lotto/history',
    name: 'SuperLottoHistory',
    component: SuperLottoHistory,
    meta: {
      requiresAuth: true,
      title: 'Super Lotto History - AI Lottery Prediction'
    }
  },
  {
    path: '/super-lotto/hot-cold',
    name: 'HotColdAnalysis',
    component: HotColdAnalysis,
    meta: {
      requiresAuth: true,
      title: 'Hot & Cold Analysis - AI Lottery Prediction'
    }
  },
  {
    path: '/super-lotto/prediction',
    name: 'PredictionDashboard',
    component: PredictionDashboard,
    meta: {
      requiresAuth: true,
      title: 'Prediction Dashboard - AI Lottery Prediction'
    }
  },
  {
    path: '/super-lotto/patterns',
    name: 'PatternAnalysis',
    component: PatternAnalysis,
    meta: {
      requiresAuth: true,
      title: 'Pattern Analysis - AI Lottery Prediction'
    }
  },
  {
    path: '/super-lotto/one-click-prediction',
    name: 'OneClickPrediction',
    component: OneClickPrediction,
    meta: {
      requiresAuth: true,
      title: 'One-Click Prediction - AI Lottery Prediction'
    }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Navigation guards
router.beforeEach((to, from, next) => {
  console.log('🟡 [Router] Navigation from', from.path, 'to', to.path)

  // Set document title
  if (to.meta?.title) {
    document.title = to.meta.title as string
  }

  // Check authentication
  const isAuthenticated = localStorage.getItem('isAuthenticated') === 'true'
  console.log('🟡 [Router] Auth check:', { isAuthenticated, requiresAuth: to.meta?.requiresAuth, path: to.path })

  if (to.meta?.requiresAuth && !isAuthenticated) {
    console.log('🟡 [Router] Redirecting to login (auth required)')
    next('/login')
  } else if (to.path === '/login' && isAuthenticated) {
    console.log('🟡 [Router] Redirecting to dashboard (already authenticated)')
    next('/dashboard')
  } else {
    console.log('🟡 [Router] Navigation allowed')
    next()
  }
})

router.afterEach((to) => {
  console.log('🟢 [Router] Navigation completed to:', to.path)
})

export default router