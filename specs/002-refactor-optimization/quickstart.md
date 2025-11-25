# Quick Start Guide: Code Refactoring and Optimization

This guide helps developers understand and work with the refactored codebase.

## Prerequisites

- Rust 1.75+ installed
- Node.js 18+ installed
- SQLite development tools
- Git

## Development Setup

### 1. Clone and Setup

```bash
git clone <repository-url>
cd ai_lottory
git checkout 002-refactor-optimization
```

### 2. Backend Setup

```bash
# Install Rust dependencies
cd src-tauri
cargo build

# Run database migrations
cargo run --bin migrate

# Run tests
cargo test

# Start development server
cargo tauri dev
```

### 3. Frontend Setup

```bash
# From project root
npm install

# Run type checking
npm run type-check

# Run linter
npm run lint

# Start frontend dev server
npm run dev
```

## Architecture Overview

### Backend Structure

```
src-tauri/src/
├── super_lotto/
│   ├── analysis/          # Prediction algorithms
│   ├── cache/             # Caching layer
│   ├── errors/            # Error handling
│   ├── models/            # Data models
│   ├── repository/        # Data access layer
│   └── services/          # Business logic
├── cache/                 # General caching utilities
├── error/                 # Central error management
└── performance/           # Performance monitoring
```

### Frontend Structure

```
src/
├── components/
│   ├── common/            # Shared components
│   ├── charts/            # Visualization components
│   └── analysis/          # Analysis UI components
├── composables/           # Vue composition functions
├── stores/                # Pinia state management
├── api/                   # API layer
├── utils/                 # Utility functions
└── types/                 # TypeScript definitions
```

## Key Features

### 1. Caching System

The application implements a three-level caching strategy:

```rust
// Example: Using the cache
use crate::cache::AnalysisCache;

let cache = AnalysisCache::new(pool);
let result = cache.get_or_compute(
    "WEIGHTED_FREQUENCY",
    params,
    || compute_analysis(params)
).await?;
```

### 2. Performance Monitoring

Track performance metrics automatically:

```rust
// Example: Performance tracking
use crate::performance::{track_operation, OperationType};

let result = track_operation(OperationType::Analysis, || {
    expensive_operation()
}).await?;
```

### 3. Error Handling

Centralized error management:

```rust
// Example: Error handling
use crate::error::{AppError, ErrorContext};

match risky_operation() {
    Ok(result) => Ok(result),
    Err(e) => Err(AppError::DatabaseError(e)
        .with_context("Failed to load lottery data")
        .with_user_impact(ErrorImpact::High))
}
```

### 4. Algorithm Plugin System

Add new prediction algorithms:

```rust
// Example: New algorithm
use crate::super_lotto::analysis::PredictionAlgorithm;

pub struct MyCustomAlgorithm;

impl PredictionAlgorithm for MyCustomAlgorithm {
    fn predict(&self, draws: &[Draw]) -> Result<Prediction> {
        // Implementation
    }

    fn name(&self) -> &str {
        "MY_CUSTOM"
    }
}

// Register algorithm
register_algorithm!("MY_CUSTOM", MyCustomAlgorithm);
```

## Testing

### Running Tests

```bash
# Backend tests
cargo test --package ai-lottory

# Frontend tests
npm test

# E2E tests
npm run test:e2e

# Performance benchmarks
cargo bench
```

### Test Coverage

Target: >90% code coverage

```bash
# Generate coverage report
cargo tarpaulin --out Html
npm run test:coverage
```

## Performance Guidelines

### Memory Management

1. Use streaming for large datasets:
   ```rust
   // Good: Streaming
   let mut stream = sqlx::query_as::<_, Draw>("SELECT * FROM draws")
       .fetch(&pool);

   while let Some(draw) = stream.next().await {
       process_draw(draw)?;
   }
   ```

2. Avoid loading all data into memory

### Caching Best Practices

1. Set appropriate TTL values
2. Implement cache invalidation
3. Monitor cache hit rates

### Frontend Optimization

1. Use Vue's `computed` for derived data
2. Implement virtual scrolling for large lists
3. Lazy load chart components

## Debugging

### Backend Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo tauri dev

# Use debugger
rust-gdb target/debug/ai-lottory
```

### Frontend Debugging

```bash
# Vue devtools
# Install browser extension for Vue debugging
```

## Common Tasks

### Adding a New Analysis Type

1. Define data model in `models/`
2. Implement repository in `repository/`
3. Add service layer in `services/`
4. Create API endpoint
5. Build UI components

### Performance Tuning

1. Check metrics dashboard
2. Identify bottlenecks
3. Implement caching
4. Optimize queries
5. Profile memory usage

### Error Investigation

1. Check error logs
2. Correlate with performance metrics
3. Reproduce in development
4. Add more logging if needed
5. Fix and test

## Deployment

### Build for Production

```bash
# Build optimized binary
cargo tauri build

# Build frontend
npm run build
```

### Environment Variables

- `DATABASE_URL`: SQLite database path
- `RUST_LOG`: Logging level
- `CACHE_SIZE_MB`: Maximum cache size
- `ENABLE_METRICS`: Enable performance tracking

## Troubleshooting

### Common Issues

1. **High memory usage**: Check for memory leaks in long-running operations
2. **Slow startup**: Profile initialization code
3. **Cache misses**: Verify cache key generation
4. **Database errors**: Check connection pool settings

### Getting Help

1. Check the error logs
2. Review performance metrics
3. Consult the architecture documentation
4. Ask the development team

## Contributing

1. Fork the repository
2. Create feature branch
3. Make changes
4. Add tests
5. Verify performance impact
6. Submit PR

## Monitoring

### Key Metrics to Watch

- Application startup time
- Memory usage patterns
- Cache hit rates
- Error rates
- API response times

### Alert Thresholds

- Startup time > 5 seconds
- Memory usage > 500MB
- Error rate > 1%
- Cache hit rate < 50%