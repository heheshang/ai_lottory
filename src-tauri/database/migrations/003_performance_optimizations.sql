-- Performance-optimized database schema for Super Lotto application
-- This migration adds performance enhancements and proper indexing

-- Create optimized indexes for better query performance
-- Composite indexes for common query patterns

-- Super Lotto draws table with enhanced performance
CREATE TABLE IF NOT EXISTS super_lotto_draws_optimized (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_number TEXT NOT NULL UNIQUE,
    draw_date TEXT NOT NULL, -- ISO format: YYYY-MM-DD
    draw_year INTEGER NOT NULL, -- Extracted year for fast filtering
    draw_month INTEGER NOT NULL, -- Extracted month for fast filtering
    front_zone TEXT NOT NULL, -- JSON array of 5 numbers (1-35)
    back_zone TEXT NOT NULL,  -- JSON array of 2 numbers (1-12)
    front_sum INTEGER NOT NULL, -- Pre-computed sum of front numbers
    front_odd_count INTEGER NOT NULL, -- Pre-computed odd count
    front_even_count INTEGER NOT NULL, -- Pre-computed even count
    has_consecutive BOOLEAN DEFAULT 0, -- Pre-computed consecutive flag
    jackpot_amount REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create performance indexes
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_date ON super_lotto_draws_optimized(draw_date DESC);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_year_month ON super_lotto_draws_optimized(draw_year, draw_month);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_number ON super_lotto_draws_optimized(draw_number);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_front_sum ON super_lotto_draws_optimized(front_sum);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_odd_even ON super_lotto_draws_optimized(front_odd_count, front_even_count);

-- Number frequency statistics table (pre-computed)
CREATE TABLE IF NOT EXISTS number_frequency_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL,
    zone TEXT NOT NULL CHECK (zone IN ('front', 'back')),
    period_days INTEGER NOT NULL,
    frequency REAL NOT NULL,
    hot_score REAL NOT NULL,
    cold_score REAL NOT NULL,
    last_seen TEXT,
    average_gap REAL NOT NULL,
    current_gap INTEGER NOT NULL,
    sample_size INTEGER NOT NULL,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(number, zone, period_days)
);

CREATE INDEX IF NOT EXISTS idx_number_freq_lookup ON number_frequency_cache(number, zone, period_days);
CREATE INDEX IF NOT EXISTS idx_number_freq_hot_score ON number_frequency_cache(zone, hot_score DESC);
CREATE INDEX IF NOT EXISTS idx_number_freq_cold_score ON number_frequency_cache(zone, cold_score DESC);
CREATE INDEX IF NOT EXISTS idx_number_freq_computed_at ON number_frequency_cache(computed_at);

-- Pattern analysis cache table
CREATE TABLE IF NOT EXISTS pattern_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL, -- JSON data
    period_days INTEGER NOT NULL,
    sample_size INTEGER NOT NULL,
    confidence_score REAL NOT NULL,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pattern_type, period_days)
);

CREATE INDEX IF NOT EXISTS idx_pattern_cache_lookup ON pattern_cache(pattern_type, period_days);

-- Batch prediction results cache
CREATE TABLE IF NOT EXISTS batch_prediction_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_hash TEXT NOT NULL UNIQUE,
    request_data TEXT NOT NULL, -- JSON of request parameters
    result_data TEXT NOT NULL, -- JSON of prediction results
    algorithms TEXT NOT NULL, -- JSON array of algorithms
    sample_size INTEGER NOT NULL,
    processing_time_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL -- Cache expiration
);

CREATE INDEX IF NOT EXISTS idx_batch_prediction_hash ON batch_prediction_cache(request_hash);
CREATE INDEX IF NOT EXISTS idx_batch_prediction_expires ON batch_prediction_cache(expires_at);

-- Performance metrics table
CREATE TABLE IF NOT EXISTS performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_type TEXT NOT NULL, -- 'query', 'command', 'prediction', etc.
    operation TEXT NOT NULL, -- specific operation name
    duration_ms INTEGER NOT NULL,
    sample_size INTEGER,
    cache_hit BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_perf_metrics_type_date ON performance_metrics(metric_type, created_at);
CREATE INDEX IF NOT EXISTS idx_perf_metrics_operation ON performance_metrics(operation);

-- Migration: Copy data from old table to new optimized table
INSERT INTO super_lotto_draws_optimized (
    id, draw_number, draw_date, draw_year, draw_month,
    front_zone, back_zone, front_sum, front_odd_count, front_even_count,
    jackpot_amount, created_at, updated_at
)
SELECT
    id, draw_number, draw_date,
    CAST(strftime('%Y', draw_date) AS INTEGER),
    CAST(strftime('%m', draw_date) AS INTEGER),
    front_zone, back_zone,
    -- Pre-compute derived fields
    CAST(json_extract(front_zone, '$[0]') AS INTEGER) +
    CAST(json_extract(front_zone, '$[1]') AS INTEGER) +
    CAST(json_extract(front_zone, '$[2]') AS INTEGER) +
    CAST(json_extract(front_zone, '$[3]') AS INTEGER) +
    CAST(json_extract(front_zone, '$[4]') AS INTEGER),
    -- Count odd numbers in front zone
    (CAST(json_extract(front_zone, '$[0]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[1]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[2]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[3]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[4]') AS INTEGER) % 2),
    5 - ((CAST(json_extract(front_zone, '$[0]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[1]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[2]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[3]') AS INTEGER) % 2) +
    (CAST(json_extract(front_zone, '$[4]') AS INTEGER) % 2)),
    0, -- has_consecutive default
    jackpot_amount, created_at, updated_at
FROM super_lotto_draws;

-- Update has_consecutive flag
UPDATE super_lotto_draws_optimized SET has_consecutive = 1
WHERE (
    ABS(CAST(json_extract(front_zone, '$[0]') AS INTEGER) - CAST(json_extract(front_zone, '$[1]') AS INTEGER)) = 1 OR
    ABS(CAST(json_extract(front_zone, '$[1]') AS INTEGER) - CAST(json_extract(front_zone, '$[2]') AS INTEGER)) = 1 OR
    ABS(CAST(json_extract(front_zone, '$[2]') AS INTEGER) - CAST(json_extract(front_zone, '$[3]') AS INTEGER)) = 1 OR
    ABS(CAST(json_extract(front_zone, '$[3]') AS INTEGER) - CAST(json_extract(front_zone, '$[4]') AS INTEGER)) = 1
);

-- Create triggers for automatic maintenance
-- Trigger to update derived fields on insert
CREATE TRIGGER IF NOT EXISTS update_derived_fields_insert
AFTER INSERT ON super_lotto_draws_optimized
BEGIN
    UPDATE super_lotto_draws_optimized SET
        draw_year = CAST(strftime('%Y', NEW.draw_date) AS INTEGER),
        draw_month = CAST(strftime('%m', NEW.draw_date) AS INTEGER),
        front_sum = CAST(json_extract(NEW.front_zone, '$[0]') AS INTEGER) +
                   CAST(json_extract(NEW.front_zone, '$[1]') AS INTEGER) +
                   CAST(json_extract(NEW.front_zone, '$[2]') AS INTEGER) +
                   CAST(json_extract(NEW.front_zone, '$[3]') AS INTEGER) +
                   CAST(json_extract(NEW.front_zone, '$[4]') AS INTEGER),
        front_odd_count = (CAST(json_extract(NEW.front_zone, '$[0]') AS INTEGER) % 2) +
                        (CAST(json_extract(NEW.front_zone, '$[1]') AS INTEGER) % 2) +
                        (CAST(json_extract(NEW.front_zone, '$[2]') AS INTEGER) % 2) +
                        (CAST(json_extract(NEW.front_zone, '$[3]') AS INTEGER) % 2) +
                        (CAST(json_extract(NEW.front_zone, '$[4]') AS INTEGER) % 2),
        front_even_count = 5 - ((CAST(json_extract(NEW.front_zone, '$[0]') AS INTEGER) % 2) +
                              (CAST(json_extract(NEW.front_zone, '$[1]') AS INTEGER) % 2) +
                              (CAST(json_extract(NEW.front_zone, '$[2]') AS INTEGER) % 2) +
                              (CAST(json_extract(NEW.front_zone, '$[3]') AS INTEGER) % 2) +
                              (CAST(json_extract(NEW.front_zone, '$[4]') AS INTEGER) % 2))
    WHERE id = NEW.id;
END;

-- Trigger to clean expired cache entries
CREATE TRIGGER IF NOT EXISTS clean_expired_cache
AFTER INSERT ON batch_prediction_cache
BEGIN
    DELETE FROM batch_prediction_cache WHERE expires_at < datetime('now');
END;

-- Trigger to log performance metrics (optional - can be toggled)
-- This can be enabled in development mode
-- PRAGMA table_info(performance_metrics);