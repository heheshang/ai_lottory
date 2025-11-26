# Data Models: Comprehensive Code Refactoring

**Date**: 2025-01-25
**Purpose**: Define data models for modularized lottery prediction system

## Core Data Models

### 1. Enhanced Lottery Draw Models

#### SuperLottoDraw (Existing - Enhanced)
```typescript
interface SuperLottoDraw {
  id: string
  drawDate: DateTime
  frontZone: number[]        // 5 numbers, 1-35
  backZone: number[]         // 2 numbers, 1-12
  jackpotAmount?: number
  createdDate: DateTime

  // Enhanced fields for advanced analysis
  drawNumber: number         // Sequential draw number
  season?: string           // Season/period identifier
  specialDraw?: boolean     // Indicates special promotional draws

  // Computed fields (stored in analysis cache)
  sum?: number              // Sum of front zone numbers
  sumRange?: 'very_low' | 'low' | 'medium' | 'high' | 'very_high'
  oddCount?: number         // Count of odd numbers in front zone
  evenCount?: number        // Count of even numbers in front zone
  consecutiveCount?: number // Count of consecutive number pairs
  primeCount?: number       // Count of prime numbers
}
```

#### PatternAnalysisResult (New)
```typescript
interface PatternAnalysisResult {
  id: string
  analysisType: PatternType
  periodDays: number
  sampleSize: number
  confidence: number         // 0.0 to 1.0
  analysisData: unknown      // Flexible data structure for different patterns
  createdDate: DateTime

  // Metadata
  algorithm: string
  version: string
  cacheExpiry?: DateTime
}

enum PatternType {
  ConsecutiveNumbers = 'consecutive_numbers',
  OddEvenDistribution = 'odd_even_distribution',
  SumRanges = 'sum_ranges',
  PrimeNumberDistribution = 'prime_distribution',
  GapAnalysis = 'gap_analysis',
  PositionalPatterns = 'positional_patterns'
}
```

### 2. Prediction System Models

#### Enhanced PredictionResult
```typescript
interface PredictionResult {
  id: string
  algorithm: PredictionAlgorithm
  frontZone: number[]        // 5 predicted numbers (1-35)
  backZone: number[]         // 2 predicted numbers (1-12)
  confidence: number         // 0.0 to 1.0
  reasoning: unknown         // Algorithm-specific reasoning data

  // Enhanced fields
  analysisPeriod: number     // Days of historical data used
  sampleSize: number         // Number of draws analyzed
  predictionScore: number    // Overall prediction quality score
  riskLevel: 'low' | 'medium' | 'high'

  // Timing and metadata
  createdDate: DateTime
  validForDraw?: string      // Draw number this prediction is for
  version: string

  // Validation
  validated: boolean
  validationResult?: ValidationResult
}

enum PredictionAlgorithm {
  HotCold = 'hot_cold',
  Frequency = 'frequency',
  MarkovChain = 'markov_chain',
  PatternBased = 'pattern_based',
  NeuralNetwork = 'neural_network',
  Ensemble = 'ensemble'
}
```

#### MarkovChainData (New)
```typescript
interface MarkovChainData {
  order: number                              // 1, 2, or 3
  transitionMatrix: number[][]              // Probability matrix
  confidence: number
  timeDecayFactor: number
  lastUpdated: DateTime

  // Analysis metadata
  totalTransitions: number
  averageTransitionTime: number              // Average days between transitions

  // State management
  currentState: number[]                     // Most recent numbers
  nextProbabilities: number[]               // Probability for each next number
}

interface TransitionProbability {
  fromNumber: number
  toNumber: number
  probability: number
  frequency: number
  lastSeen: DateTime
}
```

### 3. Analysis System Models

#### NumberStatistics (Enhanced)
```typescript
interface NumberStatistics {
  number: number

  // Basic frequency data
  totalAppearances: number
  totalDraws: number
  frequency: number             // Appearances / total draws
  percentage: number            // frequency * 100

  // Gap analysis
  currentGap: number            // Draws since last appearance
  averageGap: number
  medianGap: number
  maxGap: number
  minGap: number
  gapStandardDeviation: number

  // Position analysis
  positionFrequency: number[]   // Frequency in each position (0-4)
  favoritePositions: number[]   // Most common positions

  // Advanced metrics
  trend: 'increasing' | 'decreasing' | 'stable'
  momentum: number              // Recent performance indicator
  consistency: number           // How regular the appearances are

  // Metadata
  lastUpdated: DateTime
  analysisPeriod: number
  sampleSize: number
}
```

#### HotColdAnalysis (New)
```typescript
interface HotColdAnalysis {
  periodDays: number
  analysisDate: DateTime

  // Hot numbers (frequently appearing)
  hotNumbers: NumberStatistics[]
  hotThreshold: number         // Frequency threshold for "hot" classification

  // Cold numbers (infrequently appearing)
  coldNumbers: NumberStatistics[]
  coldThreshold: number         // Frequency threshold for "cold" classification

  // Distribution analysis
  distributionProfile: {
    hot: number                // Count of hot numbers
    normal: number             // Count of normal numbers
    cold: number               // Count of cold numbers
  }

  // Confidence and quality metrics
  confidence: number
  sampleSize: number
  dataQuality: 'excellent' | 'good' | 'fair' | 'poor'
}
```

### 4. User and Session Models

#### Enhanced User
```typescript
interface User {
  id: string
  username: string
  email?: string

  // Authentication
  lastLogin: DateTime
  sessionToken?: string
  sessionExpiry?: DateTime

  // Preferences
  preferredAlgorithms: PredictionAlgorithm[]
  analysisSettings: AnalysisSettings
  uiPreferences: UIPreferences

  // Activity tracking
  predictionsCreated: number
  successfulPredictions: number
  accuracyRate: number

  // Metadata
  createdDate: DateTime
  lastActive: DateTime
  subscriptionTier: 'free' | 'premium' | 'enterprise'
}

interface AnalysisSettings {
  defaultPeriodDays: number
  hotThreshold: number
  coldThreshold: number
  enableAdvancedFeatures: boolean
  preferredChartTypes: string[]
}

interface UIPreferences {
  theme: 'light' | 'dark' | 'auto'
  language: string
  timezone: string
  defaultView: 'dashboard' | 'analysis' | 'predictions'
  showAdvancedStats: boolean
}
```

#### UserPrediction (Enhanced)
```typescript
interface UserPrediction {
  id: string
  userId: string
  predictionResult: PredictionResult

  // User metadata
  savedDate: DateTime
  notes?: string
  tags: string[]

  // Results tracking (when actual draw occurs)
  actualResult?: string        // Reference to actual draw
  accuracy?: AccuracyResult
  resultDate?: DateTime

  // User interaction
  isFavorite: boolean
  isShared: boolean
  shareCode?: string

  // Analytics
  viewCount: number
  lastViewed: DateTime
}

interface AccuracyResult {
  correctFrontNumbers: number   // 0-5
  correctBackNumbers: number    // 0-2
  totalAccuracy: number         // Percentage
  tier: 'exact' | 'close' | 'partial' | 'miss'
  points: number                // User reward points
}
```

### 5. System and Cache Models

#### AnalysisCache (New)
```typescript
interface AnalysisCache {
  key: string
  dataType: CacheDataType
  data: unknown

  // Cache management
  createdDate: DateTime
  lastAccessed: DateTime
  expiryDate: DateTime
  hitCount: number

  // Performance metrics
  computeTime: number          // Time taken to compute originally
  estimatedSavings: number     // Time saved by using cache

  // Metadata
  version: string
  dependencies: string[]       // Cache keys this depends on
}

enum CacheDataType {
  PatternAnalysis = 'pattern_analysis',
  NumberStatistics = 'number_statistics',
  HotColdAnalysis = 'hot_cold_analysis',
  MarkovMatrix = 'markov_matrix',
  PredictionResult = 'prediction_result'
}
```

#### PerformanceMetrics (New)
```typescript
interface PerformanceMetrics {
  id: string
  operation: string
  startTime: DateTime
  endTime: DateTime
  duration: number             // Duration in milliseconds

  // Resource usage
  memoryUsage: number          // Peak memory usage in MB
  cpuUsage: number             // CPU usage percentage

  // Operation details
  algorithm?: string
  dataSize: number             // Size of processed data
  resultCount: number          // Number of results produced

  // Quality metrics
  success: boolean
  errorMessage?: string
  errorCode?: string

  // Context
  userId?: string
  sessionId: string
  requestId?: string
}
```

## Data Relationships and Flow

### Entity Relationship Diagram

```
User (1) -----> (N) UserPrediction
   |                     |
   |                     | predictionResult
   |                     v
   |              PredictionResult (1) -----> (1) PatternAnalysisResult
   |                     |
   |                     | algorithm
   |                     v
   |              PredictionAlgorithm
   |
   | preferences
   v
AnalysisSettings

SuperLottoDraw (1) -----> (N) NumberStatistics
   |
   | analyzed by
   v
PatternAnalysisResult

AnalysisCache (N) -----> (1) [any data type]
```

### Data Flow Patterns

#### Prediction Generation Flow
```
User Request → Algorithm Selection → Historical Data Retrieval
    → Statistical Analysis → Pattern Detection → Prediction Generation
    → Confidence Calculation → Result Storage → Cache Update
```

#### Analysis Flow
```
Data Request → Cache Check → [if miss] Database Query → Statistical Computation
    → Pattern Detection → Result Storage → Cache Update → Response
```

## State Management Structure

### Store Hierarchy

#### Primary Stores
1. **Auth Store** - User authentication and session management
2. **Lottery Data Store** - Raw lottery draw data management
3. **Analysis Store** - Hot/cold analysis and statistical computations
4. **Prediction Store** - Prediction generation and management
5. **UI Store** - Interface state and user preferences

#### Secondary/Utility Stores
1. **Cache Store** - Analysis cache management
2. **Performance Store** - Performance metrics and monitoring
3. **Settings Store** - User preferences and application settings

### Cross-Store Communication

#### Store Dependencies
```
Auth Store → [user data] → Prediction Store
Lottery Data Store → [draws] → Analysis Store
Analysis Store → [statistics] → Prediction Store
UI Store → [loading states] → [all stores]
Cache Store → [cached data] → [all stores]
```

#### Event-Driven Updates
```
Data Update Event → Cache Invalidation → Store Refresh → UI Update
Prediction Created → Analysis Update → Statistics Refresh
User Login → Data Load → Cache Warm → UI Ready
```

## Validation and Constraints

### Data Validation Rules

#### Lottery Draw Validation
- Front zone: exactly 5 unique numbers, range 1-35
- Back zone: exactly 2 unique numbers, range 1-12
- Date: valid ISO 8601 format, within reasonable range
- Draw number: positive integer, sequential

#### Prediction Validation
- Algorithm: valid enum value
- Confidence: range 0.0 to 1.0
- Front zone: valid lottery numbers
- Back zone: valid lottery numbers
- Reasoning: valid JSON structure

#### User Data Validation
- Username: 3-50 characters, alphanumeric + underscores
- Email: valid email format (if provided)
- Session token: valid UUID format
- Preferences: valid JSON structure

### Business Logic Constraints

#### Analysis Period
- Minimum: 30 days (for statistical significance)
- Maximum: 5 years (performance considerations)
- Default: 365 days (one year)

#### Cache Management
- Maximum age: 24 hours for pattern analysis
- Maximum size: 100MB total cache
- Cleanup threshold: 50% usage trigger

#### Performance Limits
- Maximum concurrent predictions: 10 per user
- Maximum analysis time: 30 seconds
- Maximum data size per request: 10MB

## API Data Contracts

### Request/Response Models

#### API Response Wrapper
```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: ApiError
  metadata?: ResponseMetadata

  // Pagination (if applicable)
  pagination?: {
    page: number
    pageSize: number
    total: number
    totalPages: number
  }

  // Performance
  processingTime: number
  cached: boolean
}

interface ApiError {
  code: string
  message: string
  details?: unknown
  suggestions?: string[]
  timestamp: DateTime
  requestId: string
}
```

#### Batch Operations
```typescript
interface BatchRequest<T> {
  items: T[]
  options?: BatchOptions
}

interface BatchOptions {
  continueOnError?: boolean
  maxConcurrency?: number
  timeout?: number
}

interface BatchResult<T, E = Error> {
  successful: T[]
  failed: FailedItem<E>[]
  totalProcessed: number
  processingTime: number
}

interface FailedItem<E> {
  index: number
  item: unknown
  error: E
}
```

This data model provides a comprehensive foundation for the refactored lottery prediction system, ensuring type safety, performance optimization, and maintainability across all components.