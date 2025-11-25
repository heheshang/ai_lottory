-- Performance optimization indexes
-- Migration: 003_add_performance_indexes.sql
-- Purpose: Add indexes to improve query performance for lottery data and analysis

-- Indexes for lottery draws table
CREATE INDEX IF NOT EXISTS idx_lottery_draws_draw_date ON lottery_draws(draw_date DESC);
CREATE INDEX IF NOT EXISTS idx_lottery_draws_lottery_type ON lottery_draws(lottery_type);
CREATE INDEX IF NOT EXISTS idx_lottery_draws_type_date ON lottery_draws(lottery_type, draw_date DESC);
CREATE INDEX IF NOT EXISTS idx_lottery_draws_jackpot_amount ON lottery_draws(jackpot_amount DESC);

-- Composite index for common queries (type + date range)
CREATE INDEX IF NOT EXISTS idx_lottery_draws_type_date_range ON lottery_draws(lottery_type, draw_date DESC) WHERE draw_date >= date('now', '-2 years');

-- Indexes for analysis cache table
CREATE INDEX IF NOT EXISTS idx_analysis_cache_algorithm ON analysis_cache(algorithm);
CREATE INDEX IF NOT EXISTS idx_analysis_cache_created_at ON analysis_cache(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analysis_cache_expires_at ON analysis_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_analysis_cache_algorithm_data_hash ON analysis_cache(algorithm, data_hash);

-- Performance index for expired cache cleanup
CREATE INDEX IF NOT EXISTS idx_analysis_cache_expired ON analysis_cache(expires_at) WHERE expires_at < datetime('now');

-- Indexes for number frequency analysis
CREATE INDEX IF NOT EXISTS idx_lottery_draws_numbers_gin ON lottery_draws USING gin(winning_numbers);
-- Note: SQLite doesn't have GIN indexes, this would be for PostgreSQL
-- For SQLite, we'll create individual number indexes with triggers

-- Create individual number indexes for SQLite (simulate array indexing)
-- We'll create triggers to maintain these indexes when data changes

-- Helper table for number frequency (for faster analysis)
CREATE TABLE IF NOT EXISTS number_frequency (
    number INTEGER NOT NULL,
    lottery_type TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    last_drawn_at DATETIME NOT NULL,
    draw_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (number, lottery_type)
);

-- Indexes for number frequency table
CREATE INDEX IF NOT EXISTS idx_number_frequency_frequency ON number_frequency(frequency DESC);
CREATE INDEX IF NOT EXISTS idx_number_frequency_last_drawn ON number_frequency(last_drawn_at DESC);
CREATE INDEX IF NOT EXISTS idx_number_frequency_type_frequency ON number_frequency(lottery_type, frequency DESC);

-- Indexes for user activity tracking (if user tables exist)
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login DESC);

-- Indexes for predictions table (if exists)
CREATE INDEX IF NOT EXISTS idx_predictions_created_at ON predictions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_predictions_algorithm ON predictions(algorithm);
CREATE INDEX IF NOT EXISTS idx_predictions_user_algorithm ON predictions(user_id, algorithm, created_at DESC);

-- Specialized indexes for hot/cold number analysis
CREATE INDEX IF NOT EXISTS idx_lottery_draws_recent ON lottery_draws(draw_date DESC) WHERE draw_date >= date('now', '-90 days');

-- Create materialized view-like table for frequently accessed statistics
CREATE TABLE IF NOT EXISTS lottery_statistics (
    lottery_type TEXT NOT NULL PRIMARY KEY,
    total_draws INTEGER NOT NULL DEFAULT 0,
    avg_jackpot REAL DEFAULT 0,
    max_jackpot REAL DEFAULT 0,
    min_jackpot REAL DEFAULT 0,
    last_draw_date DATETIME,
    most_frequent_numbers TEXT, -- JSON array
    least_frequent_numbers TEXT, -- JSON array
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for statistics table
CREATE INDEX IF NOT EXISTS idx_lottery_statistics_updated_at ON lottery_statistics(updated_at DESC);

-- Triggers to maintain number frequency table
CREATE TRIGGER IF NOT EXISTS update_number_frequency_insert
AFTER INSERT ON lottery_draws
BEGIN
    -- Update frequency for each winning number
    INSERT OR REPLACE INTO number_frequency (number, lottery_type, frequency, last_drawn_at, draw_count)
    VALUES
        (new.winning_numbers[0], new.lottery_type, 1, new.draw_date, 1),
        (new.winning_numbers[1], new.lottery_type, 1, new.draw_date, 1),
        (new.winning_numbers[2], new.lottery_type, 1, new.draw_date, 1),
        (new.winning_numbers[3], new.lottery_type, 1, new.draw_date, 1),
        (new.winning_numbers[4], new.lottery_type, 1, new.draw_date, 1)
    ON CONFLICT(number, lottery_type) DO UPDATE SET
        frequency = frequency + 1,
        last_drawn_at = new.draw_date,
        draw_count = draw_count + 1;

    -- Handle bonus number if exists
    INSERT OR REPLACE INTO number_frequency (number, lottery_type, frequency, last_drawn_at, draw_count)
    VALUES (new.bonus_number, new.lottery_type || '_bonus', 1, new.draw_date, 1)
    ON CONFLICT(number, lottery_type) DO UPDATE SET
        frequency = frequency + 1,
        last_drawn_at = new.draw_date,
        draw_count = draw_count + 1;
END;

-- Trigger to update statistics table
CREATE TRIGGER IF NOT EXISTS update_lottery_statistics
AFTER INSERT ON lottery_draws
BEGIN
    INSERT OR REPLACE INTO lottery_statistics (
        lottery_type,
        total_draws,
        avg_jackpot,
        max_jackpot,
        min_jackpot,
        last_draw_date,
        updated_at
    )
    SELECT
        new.lottery_type,
        COUNT(*),
        AVG(CASE WHEN jackpot_amount IS NOT NULL THEN jackpot_amount ELSE 0 END),
        MAX(COALESCE(jackpot_amount, 0)),
        MIN(COALESCE(jackpot_amount, 0)),
        MAX(draw_date),
        CURRENT_TIMESTAMP
    FROM lottery_draws
    WHERE lottery_type = new.lottery_type;
END;

-- Cleanup trigger for expired cache entries
CREATE TRIGGER IF NOT EXISTS cleanup_expired_cache
AFTER INSERT ON analysis_cache
WHEN NEW.expires_at < datetime('now')
BEGIN
    DELETE FROM analysis_cache WHERE expires_at < datetime('now');
END;

-- Performance monitoring table for slow queries
CREATE TABLE IF NOT EXISTS query_performance_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_type TEXT NOT NULL,
    execution_time_ms REAL NOT NULL,
    row_count INTEGER DEFAULT 0,
    parameters TEXT, -- JSON
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance monitoring
CREATE INDEX IF NOT EXISTS idx_query_performance_type ON query_performance_log(query_type);
CREATE INDEX IF NOT EXISTS idx_query_performance_execution_time ON query_performance_log(execution_time_ms DESC);
CREATE INDEX IF NOT EXISTS idx_query_performance_created_at ON query_performance_log(created_at DESC);

-- View for slow queries (executions taking more than 100ms)
CREATE VIEW IF NOT EXISTS slow_queries AS
SELECT
    query_type,
    AVG(execution_time_ms) as avg_time_ms,
    MAX(execution_time_ms) as max_time_ms,
    COUNT(*) as execution_count,
    MAX(created_at) as last_executed
FROM query_performance_log
WHERE execution_time_ms > 100
GROUP BY query_type
ORDER BY avg_time_ms DESC;

-- Additional optimizations

-- Set PRAGMA settings for performance
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456; -- 256MB

-- Create optimized covering indexes for common query patterns

-- Covering index for recent lottery draws with pagination
CREATE INDEX IF NOT EXISTS idx_lottery_draws_pagination_covering ON lottery_draws(
    draw_date DESC,
    lottery_type,
    winning_numbers,
    bonus_number,
    jackpot_amount
) WHERE draw_date >= date('now', '-1 year');

-- Covering index for analysis cache lookups
CREATE INDEX IF NOT EXISTS idx_analysis_cache_lookup_covering ON analysis_cache(
    algorithm,
    data_hash,
    result,
    expires_at,
    created_at
) WHERE expires_at > datetime('now');

-- Partial indexes for hot/cold analysis
CREATE INDEX IF NOT EXISTS idx_hot_numbers_recent ON number_frequency(
    lottery_type,
    frequency DESC
) WHERE last_drawn_at >= date('now', '-30 days');

CREATE INDEX IF NOT EXISTS idx_cold_numbers_old ON number_frequency(
    lottery_type,
    last_drawn_at ASC
) WHERE last_drawn_at < date('now', '-90 days');

-- Log migration completion
INSERT OR REPLACE INTO migration_log (migration_name, executed_at, success)
VALUES ('003_add_performance_indexes', CURRENT_TIMESTAMP, 1);