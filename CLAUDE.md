# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Lottery Prediction App** built with **Rust + Tauri 2 + Vue 3** - a cross-platform desktop application that provides lottery analysis features including user authentication, historical lottery data, and hot/cold number analysis.

## Technology Stack

### Core Technologies
- **Backend**: Rust with Tauri 2 framework for cross-platform desktop app
- **Frontend**: Vue 3 with Composition API and TypeScript
- **Database**: SQLite for local data storage
- **State Management**: Pinia (Vue 3)
- **UI Framework**: Element Plus or Vuetify for Vue 3
- **Build Tool**: Vite for frontend, Cargo for Rust backend

### Key Dependencies
- **Rust**: `tauri`, `serde`, `sqlx`, `tokio`, `anyhow`, `chrono`
- **Vue 3**: `vue`, `vue-router`, `pinia`, `axios`, `element-plus`
- **Development**: `vite`, `typescript`, `@tauri-apps/cli`

## Development Commands

### Project Setup
```bash
# Install dependencies
npm install
cargo build

# Development mode (both frontend and backend)
npm run tauri dev

# Frontend only development
npm run dev

# Build application
npm run tauri build
```

### Testing
```bash
# Run Rust tests
cargo test

# Run frontend tests
npm run test

# Run E2E tests (if configured)
npm run test:e2e
```

### Code Quality
```bash
# Format Rust code
cargo fmt

# Check Rust code
cargo clippy

# Lint frontend code
npm run lint

# Type check frontend
npm run type-check
```

## Project Architecture

### Directory Structure
```
ai_lottory/
├── src-tauri/                 # Rust backend (Tauri app)
│   ├── src/
│   │   ├── main.rs            # Application entry point
│   │   ├── commands/          # Tauri commands exposed to frontend
│   │   │   ├── auth.rs        # Authentication commands
│   │   │   ├── lottery.rs     # Lottery data commands
│   │   │   └── analysis.rs    # Number analysis commands
│   │   ├── models/            # Data models
│   │   │   ├── user.rs        # User model
│   │   │   ├── lottery.rs     # Lottery data model
│   │   │   └── analysis.rs    # Analysis result model
│   │   ├── services/          # Business logic
│   │   │   ├── auth_service.rs
│   │   │   ├── lottery_service.rs
│   │   │   └── analysis_service.rs
│   │   ├── database/          # Database operations
│   │   │   ├── connection.rs
│   │   │   ├── migrations/
│   │   │   └── queries.rs
│   │   └── utils/             # Utility functions
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src/                       # Vue 3 frontend
│   ├── components/            # Vue components
│   │   ├── common/            # Shared components
│   │   ├── auth/              # Authentication components
│   │   ├── lottery/           # Lottery display components
│   │   └── analysis/          # Analysis components
│   ├── views/                 # Page components
│   │   ├── Login.vue
│   │   ├── Dashboard.vue
│   │   ├── History.vue
│   │   ├── HotNumbers.vue
│   │   └── ColdNumbers.vue
│   ├── stores/                # Pinia stores
│   │   ├── auth.ts
│   │   ├── lottery.ts
│   │   └── analysis.ts
│   ├── router/                # Vue Router
│   │   └── index.ts
│   ├── api/                   # API layer
│   │   └── tauri.ts           # Tauri command wrappers
│   ├── utils/                 # Frontend utilities
│   ├── types/                 # TypeScript definitions
│   ├── App.vue
│   └── main.ts
├── database/                  # Database files and migrations
│   ├── lottery.db            # SQLite database
│   └── migrations/           # SQL migration files
├── docs/                     # Documentation
├── tests/                    # Test files
│   ├── unit/                 # Unit tests
│   └── e2e/                  # End-to-end tests
├── package.json
├── vite.config.ts
├── tsconfig.json
├── Cargo.toml
└── CLAUDE.md                 # This file
```

### Application Flow
1. **Authentication**: User login/logout with local credential storage
2. **Data Management**: Lottery historical data storage and retrieval
3. **Analysis Engine**: Hot/cold number analysis algorithms
4. **User Interface**: Vue 3 frontend consuming Tauri commands

## Core Features

### 1. Authentication System
- User registration and login
- Session management with Tauri secure storage
- Password hashing with Rust cryptography
- JWT-like token system for session persistence

### 2. Lottery Historical Data
- Data storage for lottery drawings
- Search and filter capabilities
- Date range queries
- Data import/export functionality

### 3. Hot Number Analysis
- Frequency analysis of winning numbers
- Statistical calculations for "hot" numbers
- Visualization with charts and graphs
- Customizable time periods

### 4. Cold Number Analysis
- Identification of infrequently drawn numbers
- Trend analysis over time
- Predictive indicators
- Pattern recognition

## Tauri Commands (Backend API)

### Authentication Commands
```rust
#[tauri::command]
async fn login(username: String, password: String) -> Result<User, String>

#[tauri::command]
async fn register(user: UserRegistration) -> Result<User, String>

#[tauri::command]
async fn logout() -> Result<(), String>

#[tauri::command]
async fn get_current_user() -> Result<Option<User>, String>
```

### Lottery Data Commands
```rust
#[tauri::command]
async fn get_lottery_history(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<LotteryDraw>, String>

#[tauri::command]
async fn add_lottery_draw(draw: LotteryDraw) -> Result<(), String>

#[tauri::command]
async fn search_lottery_draws(query: LotterySearchQuery) -> Result<Vec<LotteryDraw>, String>
```

### Analysis Commands
```rust
#[tauri::command]
async fn get_hot_numbers(days: Option<u32>) -> Result<Vec<NumberFrequency>, String>

#[tauri::command]
async fn get_cold_numbers(days: Option<u32>) -> Result<Vec<NumberFrequency>, String>

#[tauri::command]
async fn get_number_statistics(number: u32) -> Result<NumberStatistics, String>
```

## Data Models

### User Model
```rust
struct User {
    id: u32,
    username: String,
    email: Option<String>,
    created_at: DateTime<Utc>,
    last_login: Option<DateTime<Utc>>,
}
```

### Lottery Draw Model
```rust
struct LotteryDraw {
    id: u32,
    draw_date: DateTime<Utc>,
    winning_numbers: Vec<u32>,
    bonus_number: Option<u32>,
    jackpot_amount: Option<f64>,
    created_at: DateTime<Utc>,
}
```

### Analysis Models
```rust
struct NumberFrequency {
    number: u32,
    frequency: u32,
    last_drawn: Option<DateTime<Utc>>,
    hot_score: f64,
}

struct NumberStatistics {
    number: u32,
    total_draws: u32,
    frequency: f64,
    average_gap: f64,
    current_gap: u32,
}
```

## Development Guidelines

### Rust Backend
- Use `Result<T, String>` for error handling in Tauri commands
- Implement proper async/await patterns for database operations
- Use `serde` for serialization between Rust and JavaScript
- Follow Rust naming conventions (snake_case for functions, PascalCase for structs)

### Vue 3 Frontend
- Use Composition API with `<script setup>` syntax
- Implement proper TypeScript types for all data
- Use Pinia for state management instead of Vuex
- Follow Vue 3 best practices for reactivity and component design

### Database
- Use SQLx for type-safe database operations
- Implement proper migration system
- Use transactions for multi-table operations
- Include proper indexing for performance

### Testing
- Write unit tests for Rust business logic
- Create component tests for Vue components
- Implement integration tests for Tauri commands
- Use mock data for consistent testing

## Security Considerations

- Hash passwords using Argon2 or bcrypt
- Validate all user inputs on both frontend and backend
- Use Tauri's secure storage for sensitive data
- Implement proper session management
- Sanitize database queries to prevent SQL injection

## Performance Optimization

- Use database indexes for frequent queries
- Implement pagination for large datasets
- Cache analysis results where appropriate
- Optimize Vue component rendering with proper keys
- Use lazy loading for large data sets