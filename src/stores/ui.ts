import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

/**
 * UI Store - Interface state management
 *
 * Responsibilities:
 * - Theme management (light/dark/auto)
 * - Layout configuration (sidebar, panels, grids)
 * - Notification system
 * - Modal and overlay management
 * - Loading states and progress indicators
 * - Responsive design states
 */
export const useUIStore = defineStore('ui', () => {
  // =============================================================================
  // Theme Management
  // =============================================================================

  type Theme = 'light' | 'dark' | 'auto'
  type ColorScheme = 'default' | 'blue' | 'green' | 'purple' | 'orange'

  const theme = ref<Theme>('auto')
  const colorScheme = ref<ColorScheme>('default')
  const systemTheme = ref<Theme>('light')
  const effectiveTheme = computed(() =>
    theme.value === 'auto' ? systemTheme.value : theme.value
  )

  // =============================================================================
  // Layout Configuration
  // =============================================================================

  type LayoutType = 'sidebar' | 'top-nav' | 'compact' | 'full'
  type SidebarState = 'expanded' | 'collapsed' | 'hidden'

  const layout = ref<LayoutType>('sidebar')
  const sidebarState = ref<SidebarState>('expanded')
  const sidebarWidth = ref(280)
  const headerHeight = ref(64)

  // Panel states
  const panels = ref({
    analysis: true,
    predictions: true,
    history: true,
    statistics: false,
    settings: false
  })

  // Grid view preferences
  const gridView = ref({
    cards_per_row: 3,
    card_height: 'auto',
    show_descriptions: true,
    show_metadata: true,
    compact_mode: false
  })

  // Table view preferences
  const tableView = ref({
    page_size: 25,
    show_borders: true,
    striped_rows: true,
    sortable: true,
    filterable: true
  })

  // =============================================================================
  // Responsive Design
  // =============================================================================

  const screenSize = ref({
    width: 1920,
    height: 1080,
    is_mobile: false,
    is_tablet: false,
    is_desktop: true,
    breakpoints: {
      mobile: 768,
      tablet: 1024,
      desktop: 1440,
      wide: 1920
    }
  })

  const responsiveLayout = computed<LayoutType>(() => {
    if (screenSize.value.is_mobile) return 'compact'
    if (screenSize.value.is_tablet) return 'top-nav'
    return layout.value
  })

  // =============================================================================
  // Notification System
  // =============================================================================

  interface Notification {
    id: string
    type: 'success' | 'error' | 'warning' | 'info' | 'loading'
    title: string
    message?: string
    duration?: number
    persistent?: boolean
    actions?: Array<{
      label: string
      action: () => void
      primary?: boolean
    }>
    timestamp: Date
    read: boolean
  }

  const notifications = ref<Notification[]>([])
  const notificationSettings = ref({
    max_visible: 5,
    default_duration: 5000,
    enable_sound: true,
    enable_desktop: true,
    show_progress: true
  })

  // =============================================================================
  // Modal and Overlay Management
  // =============================================================================

  interface Modal {
    id: string
    component: string
    title?: string
    props?: Record<string, any>
    size?: 'small' | 'medium' | 'large' | 'fullscreen'
    closable?: boolean
    backdrop_closable?: boolean
    persistent?: boolean
  }

  const activeModals = ref<Modal[]>([])
  const overlayVisible = ref(false)

  // Drawer management
  const activeDrawers = ref<Array<{
    id: string
    side: 'left' | 'right'
    component: string
    props?: Record<string, any>
  }>>([])

  // =============================================================================
  // Loading States
  // =============================================================================

  const globalLoading = ref(false)
  const loadingOperations = ref<Record<string, boolean>>({})
  const loadingProgress = ref<Record<string, number>>({})

  // Progress tracking
  const progressIndicators = ref<Array<{
    id: string
    label: string
    progress: number
    total: number
    status: 'loading' | 'success' | 'error'
    cancellable?: boolean
    onCancel?: () => void
  }>>([])

  // =============================================================================
  // Navigation and Routing
  // =============================================================================

  const navigation = ref({
    current_route: '/',
    previous_route: '/',
    breadcrumb: [] as Array<{
      label: string
      route: string
    }>,
    navigation_history: [] as Array<{
      route: string
      timestamp: Date
    }>
  })

  // Active navigation items
  const activeMenuItems = ref<string[]>([])
  const navigationCollapsed = ref(false)

  // =============================================================================
  // User Preferences
  // =============================================================================

  const userPreferences = ref({
    language: 'en',
    timezone: 'UTC',
    date_format: 'YYYY-MM-DD',
    time_format: '24h',
    number_format: 'en-US',
    currency: 'USD',
    auto_save: true,
    auto_save_interval: 30000, // 30 seconds
    show_tooltips: true,
    keyboard_shortcuts: true,
    animations: true,
    reduced_motion: false
  })

  // =============================================================================
  // Accessibility
  // =============================================================================

  const accessibility = ref({
    high_contrast: false,
    large_text: false,
    focus_visible: true,
    screen_reader: false,
    keyboard_navigation: true,
    aria_labels: true
  })

  // =============================================================================
  // Component States
  // =============================================================================

  const componentStates = ref<Record<string, any>>({})

  // Focus management
  const focusManagement = ref({
    active_element: null as HTMLElement | null,
    trap_enabled: false,
    restore_focus: true
  })

  // =============================================================================
  // Computed Properties
  // =============================================================================

  // Theme classes
  const themeClasses = computed(() => ({
    'theme-light': effectiveTheme.value === 'light',
    'theme-dark': effectiveTheme.value === 'dark',
    [`color-scheme-${colorScheme.value}`]: true,
    'high-contrast': accessibility.value.high_contrast,
    'large-text': accessibility.value.large_text
  }))

  // Layout classes
  const layoutClasses = computed(() => ({
    [`layout-${responsiveLayout.value}`]: true,
    [`sidebar-${sidebarState.value}`]: true,
    'compact-mode': gridView.value.compact_mode,
    'animations-disabled': !userPreferences.value.animations,
    'reduced-motion': accessibility.value.reduced_motion
  }))

  // Responsive helpers
  const isMobile = computed(() => screenSize.value.is_mobile)
  const isTablet = computed(() => screenSize.value.is_tablet)
  const isDesktop = computed(() => screenSize.value.is_desktop)

  // Notification filters
  const unreadNotifications = computed(() =>
    notifications.value.filter(n => !n.read)
  )

  const visibleNotifications = computed(() =>
    notifications.value
      .filter(n => !n.read || n.persistent)
      .slice(0, notificationSettings.value.max_visible)
  )

  const hasActiveModals = computed(() => activeModals.value.length > 0)
  const hasActiveDrawers = computed(() => activeDrawers.value.length > 0)

  // =============================================================================
  // Theme Management Actions
  // =============================================================================

  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme
    applyTheme()
    saveState()
  }

  const setColorScheme = (scheme: ColorScheme) => {
    colorScheme.value = scheme
    applyTheme()
    saveState()
  }

  const updateSystemTheme = () => {
    if (typeof window !== 'undefined' && window.matchMedia) {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      systemTheme.value = mediaQuery.matches ? 'dark' : 'light'
    }
  }

  const applyTheme = () => {
    if (typeof document !== 'undefined') {
      const root = document.documentElement

      // Apply theme
      root.setAttribute('data-theme', effectiveTheme.value)
      root.setAttribute('data-color-scheme', colorScheme.value)

      // Apply accessibility options
      root.setAttribute('data-high-contrast', accessibility.value.high_contrast.toString())
      root.setAttribute('data-large-text', accessibility.value.large_text.toString())
      root.setAttribute('data-reduced-motion', accessibility.value.reduced_motion.toString())
    }
  }

  // =============================================================================
  // Layout Management Actions
  // =============================================================================

  const setLayout = (newLayout: LayoutType) => {
    layout.value = newLayout
    saveState()
  }

  const setSidebarState = (state: SidebarState) => {
    sidebarState.value = state
    saveState()
  }

  const toggleSidebar = () => {
    if (sidebarState.value === 'expanded') {
      sidebarState.value = 'collapsed'
    } else if (sidebarState.value === 'collapsed') {
      sidebarState.value = 'hidden'
    } else {
      sidebarState.value = 'expanded'
    }
    saveState()
  }

  const updateSidebarWidth = (width: number) => {
    sidebarWidth.value = Math.max(200, Math.min(500, width))
    saveState()
  }

  const togglePanel = (panel: keyof typeof panels.value) => {
    panels.value[panel] = !panels.value[panel]
    saveState()
  }

  const updateGridView = (updates: Partial<typeof gridView.value>) => {
    gridView.value = { ...gridView.value, ...updates }
    saveState()
  }

  const updateTableView = (updates: Partial<typeof tableView.value>) => {
    tableView.value = { ...tableView.value, ...updates }
    saveState()
  }

  // =============================================================================
  // Responsive Design Actions
  // =============================================================================

  const updateScreenSize = (width: number, height: number) => {
    screenSize.value.width = width
    screenSize.value.height = height
    screenSize.value.is_mobile = width < screenSize.value.breakpoints.tablet
    screenSize.value.is_tablet = width >= screenSize.value.breakpoints.tablet && width < screenSize.value.breakpoints.desktop
    screenSize.value.is_desktop = width >= screenSize.value.breakpoints.desktop

    // Auto-adjust layout for mobile
    if (screenSize.value.is_mobile && sidebarState.value !== 'hidden') {
      sidebarState.value = 'hidden'
    }
  }

  // =============================================================================
  // Notification System Actions
  // =============================================================================

  const addNotification = (notification: Omit<Notification, 'id' | 'timestamp' | 'read'>) => {
    const newNotification: Notification = {
      id: `notification_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: new Date(),
      read: false,
      ...notification
    }

    notifications.value.unshift(newNotification)

    // Auto-remove non-persistent notifications
    if (!newNotification.persistent && newNotification.duration) {
      setTimeout(() => {
        removeNotification(newNotification.id)
      }, newNotification.duration)
    }

    // Play sound if enabled
    if (notificationSettings.value.enable_sound && typeof window !== 'undefined') {
      // Play notification sound
    }

    return newNotification.id
  }

  const removeNotification = (id: string) => {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }

  const markNotificationAsRead = (id: string) => {
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.read = true
    }
  }

  const markAllNotificationsAsRead = () => {
    notifications.value.forEach(n => n.read = true)
  }

  const clearNotifications = () => {
    notifications.value = []
  }

  // Convenience methods
  const showSuccess = (title: string, message?: string, duration?: number) =>
    addNotification({ type: 'success', title, message, duration })

  const showError = (title: string, message?: string, persistent = true) =>
    addNotification({ type: 'error', title, message, persistent })

  const showWarning = (title: string, message?: string, duration?: number) =>
    addNotification({ type: 'warning', title, message, duration })

  const showInfo = (title: string, message?: string, duration?: number) =>
    addNotification({ type: 'info', title, message, duration })

  const showLoading = (title: string, message?: string) =>
    addNotification({ type: 'loading', title, message, persistent: true })

  // =============================================================================
  // Modal and Overlay Management
  // =============================================================================

  const showModal = (modal: Omit<Modal, 'id'>) => {
    const newModal: Modal = {
      id: `modal_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      ...modal
    }

    activeModals.value.push(newModal)
    overlayVisible.value = true

    return newModal.id
  }

  const hideModal = (id: string) => {
    const index = activeModals.value.findIndex(m => m.id === id)
    if (index !== -1) {
      activeModals.value.splice(index, 1)
    }

    if (activeModals.value.length === 0) {
      overlayVisible.value = false
    }
  }

  const hideAllModals = () => {
    activeModals.value = []
    overlayVisible.value = false
  }

  const showDrawer = (drawer: {
    side: 'left' | 'right'
    component: string
    props?: Record<string, any>
  }) => {
    const newDrawer = {
      id: `drawer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      ...drawer
    }

    activeDrawers.value.push(newDrawer)
    return newDrawer.id
  }

  const hideDrawer = (id: string) => {
    const index = activeDrawers.value.findIndex(d => d.id === id)
    if (index !== -1) {
      activeDrawers.value.splice(index, 1)
    }
  }

  const hideAllDrawers = () => {
    activeDrawers.value = []
  }

  // =============================================================================
  // Loading State Management
  // =============================================================================

  const setGlobalLoading = (loading: boolean) => {
    globalLoading.value = loading
  }

  const setLoadingOperation = (operation: string, loading: boolean) => {
    loadingOperations.value[operation] = loading
  }

  const isLoadingOperation = (operation: string) =>
    loadingOperations.value[operation] || false

  const setLoadingProgress = (operation: string, progress: number, total = 100) => {
    loadingProgress.value[operation] = progress
  }

  const getLoadingProgress = (operation: string) =>
    loadingProgress.value[operation] || 0

  const addProgressIndicator = (indicator: {
    id: string
    label: string
    total: number
    status?: 'loading' | 'success' | 'error'
    cancellable?: boolean
    onCancel?: () => void
  }) => {
    progressIndicators.value.push({
      ...indicator,
      progress: 0
    })
  }

  const updateProgressIndicator = (id: string, progress: number, status?: 'loading' | 'success' | 'error') => {
    const indicator = progressIndicators.value.find(i => i.id === id)
    if (indicator) {
      indicator.progress = progress
      if (status) indicator.status = status
    }
  }

  const removeProgressIndicator = (id: string) => {
    const index = progressIndicators.value.findIndex(i => i.id === id)
    if (index !== -1) {
      progressIndicators.value.splice(index, 1)
    }
  }

  // =============================================================================
  // Navigation Management
  // =============================================================================

  const setCurrentRoute = (route: string) => {
    navigation.value.previous_route = navigation.value.current_route
    navigation.value.current_route = route
    navigation.value.navigation_history.push({
      route,
      timestamp: new Date()
    })

    // Keep only last 50 history entries
    if (navigation.value.navigation_history.length > 50) {
      navigation.value.navigation_history = navigation.value.navigation_history.slice(-50)
    }
  }

  const setBreadcrumb = (breadcrumb: Array<{ label: string; route: string }>) => {
    navigation.value.breadcrumb = breadcrumb
  }

  const setActiveMenuItems = (items: string[]) => {
    activeMenuItems.value = items
  }

  const toggleNavigationCollapsed = () => {
    navigationCollapsed.value = !navigationCollapsed.value
  }

  // =============================================================================
  // User Preferences Management
  // =============================================================================

  const updateUserPreferences = (updates: Partial<typeof userPreferences.value>) => {
    userPreferences.value = { ...userPreferences.value, ...updates }
    saveState()
  }

  const updateAccessibility = (updates: Partial<typeof accessibility.value>) => {
    accessibility.value = { ...accessibility.value, ...updates }
    applyTheme()
    saveState()
  }

  // =============================================================================
  // Component State Management
  // =============================================================================

  const setComponentState = (component: string, state: any) => {
    componentStates.value[component] = state
  }

  const getComponentState = (component: string) => {
    return componentStates.value[component]
  }

  const clearComponentState = (component: string) => {
    delete componentStates.value[component]
  }

  // =============================================================================
  // Focus Management
  // =============================================================================

  const setFocusTrap = (enabled: boolean, restoreFocus = true) => {
    focusManagement.value.trap_enabled = enabled
    focusManagement.value.restore_focus = restoreFocus
  }

  const setActiveElement = (element: HTMLElement | null) => {
    focusManagement.value.active_element = element
  }

  // =============================================================================
  // Data Persistence
  // =============================================================================

  const saveState = () => {
    try {
      const state = {
        theme: theme.value,
        colorScheme: colorScheme.value,
        layout: layout.value,
        sidebarState: sidebarState.value,
        sidebarWidth: sidebarWidth.value,
        panels: panels.value,
        gridView: gridView.value,
        tableView: tableView.value,
        userPreferences: userPreferences.value,
        accessibility: accessibility.value,
        navigationCollapsed: navigationCollapsed.value,
        notificationSettings: notificationSettings.value
      }

      localStorage.setItem('ui-store', JSON.stringify(state))
    } catch (error) {
      console.warn('Failed to save UI store state:', error)
    }
  }

  const loadState = () => {
    try {
      const savedState = localStorage.getItem('ui-store')
      if (savedState) {
        const state = JSON.parse(savedState)

        if (state.theme) theme.value = state.theme
        if (state.colorScheme) colorScheme.value = state.colorScheme
        if (state.layout) layout.value = state.layout
        if (state.sidebarState) sidebarState.value = state.sidebarState
        if (state.sidebarWidth) sidebarWidth.value = state.sidebarWidth
        if (state.panels) panels.value = { ...panels.value, ...state.panels }
        if (state.gridView) gridView.value = { ...gridView.value, ...state.gridView }
        if (state.tableView) tableView.value = { ...tableView.value, ...state.tableView }
        if (state.userPreferences) userPreferences.value = { ...userPreferences.value, ...state.userPreferences }
        if (state.accessibility) accessibility.value = { ...accessibility.value, ...state.accessibility }
        if (state.navigationCollapsed !== undefined) navigationCollapsed.value = state.navigationCollapsed
        if (state.notificationSettings) notificationSettings.value = { ...notificationSettings.value, ...state.notificationSettings }
      }
    } catch (error) {
      console.warn('Failed to load UI store state:', error)
    }
  }

  const resetStore = () => {
    theme.value = 'auto'
    colorScheme.value = 'default'
    layout.value = 'sidebar'
    sidebarState.value = 'expanded'
    sidebarWidth.value = 280
    panels.value = {
      analysis: true,
      predictions: true,
      history: true,
      statistics: false,
      settings: false
    }

    notifications.value = []
    activeModals.value = []
    activeDrawers.value = []
    globalLoading.value = false
    loadingOperations.value = {}
    loadingProgress.value = {}
    progressIndicators.value = []

    localStorage.removeItem('ui-store')
  }

  // =============================================================================
  // Initialization
  // =============================================================================

  const initializeStore = () => {
    // Load saved state
    loadState()

    // Update system theme
    updateSystemTheme()
    applyTheme()

    // Set up system theme listener
    if (typeof window !== 'undefined' && window.matchMedia) {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      mediaQuery.addEventListener('change', updateSystemTheme)
    }

    // Update screen size
    if (typeof window !== 'undefined') {
      updateScreenSize(window.innerWidth, window.innerHeight)
      window.addEventListener('resize', () => {
        updateScreenSize(window.innerWidth, window.innerHeight)
      })
    }
  }

  return {
    // State
    theme,
    colorScheme,
    systemTheme,
    effectiveTheme,
    layout,
    sidebarState,
    sidebarWidth,
    headerHeight,
    panels,
    gridView,
    tableView,
    screenSize,
    responsiveLayout,
    notifications,
    notificationSettings,
    activeModals,
    overlayVisible,
    activeDrawers,
    globalLoading,
    loadingOperations,
    loadingProgress,
    progressIndicators,
    navigation,
    activeMenuItems,
    navigationCollapsed,
    userPreferences,
    accessibility,
    componentStates,
    focusManagement,

    // Computed
    themeClasses,
    layoutClasses,
    isMobile,
    isTablet,
    isDesktop,
    unreadNotifications,
    visibleNotifications,
    hasActiveModals,
    hasActiveDrawers,

    // Theme Management
    setTheme,
    setColorScheme,
    updateSystemTheme,
    applyTheme,

    // Layout Management
    setLayout,
    setSidebarState,
    toggleSidebar,
    updateSidebarWidth,
    togglePanel,
    updateGridView,
    updateTableView,

    // Responsive Design
    updateScreenSize,

    // Notification System
    addNotification,
    removeNotification,
    markNotificationAsRead,
    markAllNotificationsAsRead,
    clearNotifications,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    showLoading,

    // Modal and Overlay Management
    showModal,
    hideModal,
    hideAllModals,
    showDrawer,
    hideDrawer,
    hideAllDrawers,

    // Loading State Management
    setGlobalLoading,
    setLoadingOperation,
    isLoadingOperation,
    setLoadingProgress,
    getLoadingProgress,
    addProgressIndicator,
    updateProgressIndicator,
    removeProgressIndicator,

    // Navigation Management
    setCurrentRoute,
    setBreadcrumb,
    setActiveMenuItems,
    toggleNavigationCollapsed,

    // User Preferences
    updateUserPreferences,
    updateAccessibility,

    // Component State Management
    setComponentState,
    getComponentState,
    clearComponentState,

    // Focus Management
    setFocusTrap,
    setActiveElement,

    // Data Persistence
    saveState,
    loadState,
    resetStore,
    initializeStore
  }
})