# Frontend Refactoring Guide

## Overview

This document describes the comprehensive refactoring of the frontend state management system from a monolithic 950+ line store to a modular, scalable architecture using Pinia stores and composables.

## Architecture Overview

### Before (Monolithic)
```
src/stores/superLotto.ts (950+ lines)
├── Authentication logic
├── Lottery data management
├── Analysis functionality
├── Prediction algorithms
├── UI state management
└── Mixed concerns
```

### After (Modular)
```
src/
├── stores/                    # Domain-specific Pinia stores
│   ├── auth.ts              # Authentication & session management
│   ├── lottery-data.ts      # Raw lottery draw data
│   ├── analysis.ts          # Hot/cold number analysis
│   ├── predictions.ts       # Prediction algorithms & results
│   ├── ui.ts                # UI state, themes, notifications
│   └── orchestrator.ts      # Workflow coordination
├── composables/              # Cross-store logic
│   ├── useLotteryOperations.ts    # High-level workflows
│   ├── useErrorHandler.ts         # Error handling & monitoring
│   └── useAPIcache.ts            # API caching utilities
├── api/                        # Unified API layer
│   └── unified.ts               # Single interface for all backend calls
└── utils/
    └── cache.ts                 # Smart caching system
```

## Store Architecture

### 1. Auth Store (`stores/auth.ts`)
**Responsibilities:**
- User authentication and session management
- JWT token handling
- Session timeout management
- User preferences storage
- Security state monitoring

**Key Features:**
- Reactive authentication state
- Automatic session validation
- Secure credential storage with Tauri
- Session persistence and restoration

### 2. Lottery Data Store (`stores/lottery-data.ts`)
**Responsibilities:**
- Historical lottery draw data storage
- Data search and filtering
- Pagination and sorting
- Data import/export
- Draw validation

**Key Features:**
- Advanced filtering system
- Pagination with state persistence
- Smart search with caching
- Batch import/export functionality
- Data integrity validation

### 3. Analysis Store (`stores/analysis.ts`)
**Responsibilities:**
- Hot and cold number analysis
- Pattern detection (consecutive, odd/even, sum ranges)
- Statistical calculations
- Frequency analysis
- Trend identification

**Key Features:**
- Multiple analysis algorithms
- Configurable time periods
- Real-time pattern detection
- Statistical confidence scoring
- Analysis result caching

### 4. Predictions Store (`stores/predictions.ts`)
**Responsibilities:**
- Prediction algorithm management
- Prediction result storage
- Algorithm performance tracking
- Batch prediction generation
- Confidence score calculations

**Key Features:**
- Multiple prediction algorithms (Weighted Frequency, Pattern-Based, Markov Chain, etc.)
- Algorithm performance monitoring
- Batch prediction with confidence ranking
- Validation against actual results
- Configurable algorithm parameters

### 5. UI Store (`stores/ui.ts`)
**Responsibilities:**
- Theme management (light/dark/auto)
- Layout configuration
- Notification system
- Modal and overlay management
- Responsive design states
- User interaction state

**Key Features:**
- Comprehensive theme system
- Responsive layout management
- Smart notification system
- Modal/drawer management
- Accessibility support
- User preference persistence

### 6. Orchestrator Store (`stores/orchestrator.ts`)
**Responsibilities:**
- Complex workflow coordination
- Cross-store state synchronization
- Application lifecycle management
- Business process automation
- System health monitoring

**Key Features:**
- High-level workflow orchestration
- Cross-store dependency management
- System health monitoring
- Performance optimization
- Error recovery mechanisms

## Composables Architecture

### 1. useLotteryOperations
**Purpose:** High-level lottery operations that span multiple stores

**Key Functions:**
- `performCompleteAnalysis()` - End-to-end analysis workflow
- `generateSmartPrediction()` - Intelligent prediction generation
- `synchronizeData()` - Data integrity verification and sync
- `batchImportData()` - Batch data import with progress tracking

### 2. useErrorHandler
**Purpose:** Centralized error handling and monitoring

**Key Functions:**
- Global error interception
- Error categorization and retry logic
- Performance metrics collection
- Error trend analysis

### 3. useAPIcache
**Purpose:** Smart API response caching

**Key Functions:**
- Multi-tier caching (memory + localStorage)
- TTL (Time To Live) management
- Cache invalidation strategies
- Performance monitoring

## API Layer

### Unified API (`api/unified.ts`)
**Purpose:** Single interface for all backend operations

**Features:**
- Type-safe API calls
- Automatic error handling and retry
- Response caching and deduplication
- Request/Response interceptors
- Performance monitoring

**Example Usage:**
```typescript
import { api } from '@/api/unified'

// Get lottery history with caching
const history = await api.getLotteryHistory({ limit: 100 })

// Generate batch prediction
const predictions = await api.generateBatchPrediction({
  algorithms: ['weighted_frequency', 'pattern_based', 'markov_chain'],
  analysis_period_days: 90
})

// Export data
const exportPath = await api.exportLotteryData('csv', filters)
```

## Caching System

### Cache Manager (`utils/cache.ts`)
**Features:**
- Multi-tier storage (memory + localStorage)
- Configurable TTL support
- Multiple invalidation strategies (TTL, LRU, LFU, size, priority)
- Performance monitoring
- Compression for large objects
- Dependency management

**Example Usage:**
```typescript
import { createCacheManager } from '@/utils/cache'

const cache = createCacheManager('predictions', {
  defaultTTL: 10 * 60 * 1000, // 10 minutes
  maxSize: 50,
  compress: true
})

// Set with options
cache.set('key', data, {
  ttl: 5 * 60 * 1000,
  priority: 'high',
  tags: ['predictions', 'recent']
})

// Get with automatic cache miss handling
const result = cache.get('key') || await fetchData()
```

## Migration Guide

### From Monolithic Store

**Before:**
```typescript
import { useSuperLottoStore } from '@/stores/superLotto'

const store = useSuperLottoStore()
await store.fetchDraws()
const predictions = await store.generatePrediction()
```

**After:**
```typescript
import { useLotteryDataStore, usePredictionsStore } from '@/stores'
import { useLotteryOperations } from '@/composables'

const dataStore = useLotteryDataStore()
const predictionsStore = usePredictionsStore()
const { performCompleteAnalysis } = useLotteryOperations()

// Fetch data
await dataStore.fetchDraws()

// Generate smart prediction
const result = await predictionsStore.generatePrediction('weighted_frequency')

// Or use high-level workflow
await performCompleteAnalysis()
```

### Store Usage Patterns

#### Authentication
```typescript
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

// Check if authenticated
if (authStore.isAuthenticated) {
  const user = authStore.user
  await authStore.validateSession()
}

// Login
try {
  await authStore.login({ username, password })
  uiStore.showSuccess('Login Successful', `Welcome ${authStore.user?.username}`)
} catch (error) {
  uiStore.showError('Login Failed', error.message)
}
```

#### Data Management
```typescript
import { useLotteryDataStore } from '@/stores/lottery-data'

const dataStore = useLotteryDataStore()

// Fetch with filtering
await dataStore.fetchDraws({ limit: 100 })

// Apply advanced filters
dataStore.setActiveFilters({
  draw_date_range: ['2023-01-01', '2023-12-31'],
  jackpot_min: 1000000
})

// Search
dataStore.setSearchQuery('123')

// Pagination
dataStore.setPage(2)
dataStore.setLimit(25)
```

#### Predictions
```typescript
import { usePredictionsStore } from '@/stores/predictions'

const predictionsStore = usePredictionsStore()

// Generate single prediction
const prediction = await predictionsStore.generatePrediction('markov_chain', {
  analysis_period_days: 180,
  time_decay_factor: 0.9
})

// Generate batch predictions
const batch = await predictionsStore.generateBatchPredictions({
  algorithms: ['weighted_frequency', 'pattern_based', 'ensemble'],
  analysis_period_days: 90,
  sample_size: 1000
})

// Get statistics
const stats = predictionsStore.algorithmStats
```

#### UI Management
```typescript
import { useUIStore } from '@/stores/ui'

const uiStore = useUIStore()

// Theme management
uiStore.setTheme('dark')
uiStore.setColorScheme('purple')

// Notifications
const notificationId = uiStore.showSuccess('Success!', 'Operation completed')

// Modals
const modalId = uiStore.showModal({
  component: 'PredictionSettings',
  title: 'Prediction Configuration',
  size: 'medium'
})

// Layout
uiStore.toggleSidebar()
uiStore.updateGridView({ cards_per_row: 4 })
```

## Performance Optimizations

### 1. Smart Caching
- API responses are cached automatically
- Computed results are cached with TTL
- Cache invalidation based on dependencies
- Compression for large objects

### 2. Lazy Loading
- Large datasets are loaded in batches
- Pagination with progressive loading
- Component-level lazy loading
- Route-based code splitting

### 3. Memory Management
- Automatic cache cleanup
- Size-based eviction
- Memory usage monitoring
- Garbage collection optimization

### 4. Performance Monitoring
- Request timing and success rates
- Cache hit rates and efficiency
- Component render performance
- Memory usage tracking

## Error Handling

### Error Categories
1. **Validation Errors** - User input validation failures
2. **Network Errors** - API communication failures
3. **Runtime Errors** - JavaScript execution errors
4. **Business Errors** - Application logic errors
5. **System Errors** - Infrastructure failures

### Error Recovery
- Automatic retry with exponential backoff
- Fallback to alternative strategies
- Graceful degradation
- User notification and guidance

### Error Monitoring
- Centralized error collection
- Error trend analysis
- Performance impact assessment
- Automated alerting

## Testing Strategy

### Unit Tests
- Individual store function testing
- Composable logic verification
- Utility function validation

### Integration Tests
- Store interaction testing
- Workflow validation
- API integration verification

### E2E Tests
- Complete user workflow testing
- Cross-store interaction validation
- Performance benchmarking

## Development Workflow

### 1. Store Development
```bash
# Create new store
touch src/stores/new-feature.ts

# Implement store with proper typing
# Add reactive state and computed properties
# Implement actions with error handling
# Add persistence where needed
```

### 2. Composable Development
```bash
# Create new composable
touch src/composables/useNewFeature.ts

# Implement cross-store logic
# Handle error cases gracefully
# Add performance monitoring
# Include proper TypeScript types
```

### 3. Integration
```typescript
// Import and use in components
import { useNewFeatureStore } from '@/stores'
import { useNewFeature } from '@/composables'

const store = useNewFeatureStore()
const { performOperation } = useNewFeature()
```

## Best Practices

### 1. Store Design
- Single responsibility principle
- Clear domain boundaries
- Reactive state management
- Proper error handling
- State persistence where appropriate

### 2. Composable Design
- Cross-store logic only
- Reusable functionality
- Clear input/output contracts
- Error handling and retries
- Performance optimization

### 3. API Integration
- Type-safe interfaces
- Automatic error handling
- Response caching
- Request deduplication
- Performance monitoring

### 4. Error Handling
- Categorize errors appropriately
- Provide meaningful user feedback
- Implement recovery strategies
- Monitor error trends
- Log errors for debugging

### 5. Performance
- Implement smart caching
- Use lazy loading strategies
- Monitor memory usage
- Optimize re-renders
- Measure and improve

## Migration Checklist

### Phase 1: Store Migration
- [ ] Create individual stores
- [ ] Move relevant logic
- [ ] Update type definitions
- [ ] Add proper error handling
- [ ] Implement state persistence

### Phase 2: Composable Migration
- [ ] Create cross-store composables
- [ ] Move workflow logic
- [ ] Add error handling
- [ ] Implement performance monitoring

### Phase 3: API Layer Migration
- [ ] Create unified API interface
- [ ] Add interceptors and middleware
- [ ] Implement caching strategy
- [ ] Add error handling and retry

### Phase 4: Component Migration
- [ ] Update component imports
- [ ] Refactor store usage
- [ ] Add composables where needed
- [ ] Update type definitions

### Phase 5: Testing and Validation
- [ ] Add unit tests for stores
- [ ] Add integration tests for workflows
- [ ] Add E2E tests for complete flows
- [ ] Performance testing and optimization
- [ ] Documentation updates

## Conclusion

The refactored architecture provides:

1. **Better Separation of Concerns** - Each store has a clear domain responsibility
2. **Improved Maintainability** - Smaller, focused modules are easier to maintain
3. **Enhanced Testability** - Individual components can be tested in isolation
4. **Better Performance** - Smart caching and optimized reactivity
5. **Scalability** - New features can be added without affecting existing code
6. **Type Safety** - Comprehensive TypeScript coverage throughout
7. **Error Resilience** - Robust error handling and recovery mechanisms
8. **Developer Experience** - Clear patterns and comprehensive tooling

This architecture provides a solid foundation for continued development and maintenance of the lottery prediction application.