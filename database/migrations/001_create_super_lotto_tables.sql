-- Create Super Lotto tables for lottery prediction functionality
-- Migration 001: Create Super Lotto schema

-- Main lottery draws table with computed columns for performance
CREATE TABLE IF NOT EXISTS super_lotto_draws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_date TEXT NOT NULL UNIQUE,  -- ISO 8601 datetime string
    draw_number TEXT,                -- Official draw identifier
    front_zone TEXT NOT NULL,       -- JSON array: [1,2,3,4,5]
    back_zone TEXT NOT NULL,        -- JSON array: [1,2]
    jackpot_amount REAL,             -- Jackpot amount in local currency
    winners_count INTEGER,          -- Number of jackpot winners
    sum_front INTEGER,              -- Pre-computed sum of front numbers
    odd_count_front INTEGER,        -- Pre-computed odd number count
    even_count_front INTEGER,       -- Pre-computed even number count
    has_consecutive_front BOOLEAN,  -- Pre-computed consecutive pattern
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- Constraints for data integrity
    CONSTRAINT valid_front_zone CHECK (
        json_array_length(front_zone) = 5 AND
        json_valid(front_zone) = 1
    ),
    CONSTRAINT valid_back_zone CHECK (
        json_array_length(back_zone) = 2 AND
        json_valid(back_zone) = 1
    ),
    CONSTRAINT valid_draw_date CHECK (draw_date IS NOT NULL AND length(draw_date) > 0)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_date ON super_lotto_draws(draw_date);
CREATE INDEX IF NOT EXISTS idx_super_lotto_sum_front ON super_lotto_draws(sum_front);
CREATE INDEX IF NOT EXISTS idx_super_lotto_odd_even ON super_lotto_draws(odd_count_front, even_count_front);
CREATE INDEX IF NOT EXISTS idx_super_lotto_consecutive ON super_lotto_draws(has_consecutive_front);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draw_number ON super_lotto_draws(draw_number);

-- Number frequency tracking table with hot/cold scores
CREATE TABLE IF NOT EXISTS number_frequencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL,
    zone TEXT NOT NULL CHECK (zone IN ('FRONT', 'BACK')),
    frequency INTEGER DEFAULT 0,
    last_seen DATETIME,
    hot_score REAL DEFAULT 0.0,
    cold_score REAL DEFAULT 0.0,
    average_gap REAL DEFAULT 0.0,
    current_gap INTEGER DEFAULT 0,
    period_days INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(number, zone, period_days)
);

-- Indexes for frequency queries
CREATE INDEX IF NOT EXISTS idx_number_frequencies_hot ON number_frequencies(hot_score) WHERE zone = 'FRONT';
CREATE INDEX IF NOT EXISTS idx_number_frequencies_cold ON number_frequencies(cold_score) WHERE zone = 'FRONT';
CREATE INDEX IF NOT EXISTS idx_number_frequencies_back_hot ON number_frequencies(hot_score) WHERE zone = 'BACK';
CREATE INDEX IF NOT EXISTS idx_number_frequencies_back_cold ON number_frequencies(cold_score) WHERE zone = 'BACK';
CREATE INDEX IF NOT EXISTS idx_number_frequencies_number_zone ON number_frequencies(number, zone);

-- Pattern analysis results table
CREATE TABLE IF NOT EXISTS pattern_analyses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL CHECK (pattern_type IN (
        'CONSECUTIVE_NUMBERS', 'GAP_PATTERNS', 'ODD_EVEN_DISTRIBUTION',
        'SUM_RANGES', 'POSITION_PATTERNS', 'ZONE_PATTERNS'
    )),
    analysis_data TEXT NOT NULL,     -- JSON storage for flexible pattern data
    confidence_score REAL DEFAULT 0.0 CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    sample_size INTEGER NOT NULL,
    period_days INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT valid_analysis_data CHECK (json_valid(analysis_data) = 1)
);

-- Indexes for pattern queries
CREATE INDEX IF NOT EXISTS idx_pattern_analyses_type ON pattern_analyses(pattern_type);
CREATE INDEX IF NOT EXISTS idx_pattern_analyses_confidence ON pattern_analyses(confidence_score);
CREATE INDEX IF NOT EXISTS idx_pattern_analyses_created ON pattern_analyses(created_at);
CREATE INDEX IF NOT EXISTS idx_pattern_analyses_period ON pattern_analyses(period_days);

-- Prediction results table
CREATE TABLE IF NOT EXISTS prediction_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    algorithm TEXT NOT NULL CHECK (algorithm IN (
        'WEIGHTED_FREQUENCY', 'PATTERN_BASED', 'MARKOV_CHAIN',
        'ENSEMBLE', 'HOT_NUMBERS', 'COLD_NUMBERS', 'POSITION_ANALYSIS'
    )),
    front_numbers TEXT NOT NULL,      -- JSON: [1,2,3,4,5]
    back_numbers TEXT NOT NULL,       -- JSON: [1,2]
    confidence_score REAL DEFAULT 0.0 CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    reasoning TEXT NOT NULL,          -- JSON reasoning data
    analysis_period_days INTEGER NOT NULL,
    sample_size INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_validated BOOLEAN DEFAULT FALSE,

    -- Constraints for prediction validation
    CONSTRAINT valid_front_predictions CHECK (
        json_array_length(front_numbers) = 5 AND
        json_valid(front_numbers) = 1
    ),
    CONSTRAINT valid_back_predictions CHECK (
        json_array_length(back_numbers) = 2 AND
        json_valid(back_numbers) = 1
    )
);

-- Indexes for prediction queries
CREATE INDEX IF NOT EXISTS idx_prediction_results_algorithm ON prediction_results(algorithm);
CREATE INDEX IF NOT EXISTS idx_prediction_results_confidence ON prediction_results(confidence_score);
CREATE INDEX IF NOT EXISTS idx_prediction_results_created ON prediction_results(created_at);
CREATE INDEX IF NOT EXISTS idx_prediction_results_validated ON prediction_results(is_validated);

-- Analysis cache table for performance optimization
CREATE TABLE IF NOT EXISTS analysis_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cache_key TEXT NOT NULL UNIQUE,
    analysis_type TEXT NOT NULL CHECK (analysis_type IN (
        'HOT_NUMBERS', 'COLD_NUMBERS', 'PATTERN_ANALYSIS',
        'FREQUENCY_ANALYSIS', 'PREDICTION_GENERATION'
    )),
    result_data TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    hit_count INTEGER DEFAULT 0,

    CONSTRAINT valid_cache_result CHECK (json_valid(result_data) = 1)
);

-- Indexes for cache queries
CREATE INDEX IF NOT EXISTS idx_analysis_cache_expires ON analysis_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_analysis_cache_type ON analysis_cache(analysis_type);
CREATE INDEX IF NOT EXISTS idx_analysis_cache_key ON analysis_cache(cache_key);

-- Create triggers for automatic updates
CREATE TRIGGER IF NOT EXISTS update_number_frequencies_timestamp
AFTER UPDATE ON number_frequencies
FOR EACH ROW
BEGIN
    UPDATE number_frequencies SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Clean up expired cache entries automatically (this would be called by the application)
-- CREATE TRIGGER IF NOT EXISTS cleanup_expired_cache
-- AFTER INSERT ON analysis_cache
-- WHEN NEW.expires_at < CURRENT_TIMESTAMP
-- BEGIN
--     DELETE FROM analysis_cache WHERE expires_at < CURRENT_TIMESTAMP;
-- END;

-- Insert some initial configuration or reference data if needed
INSERT OR IGNORE INTO number_frequencies (number, zone, frequency, period_days) VALUES
-- Front zone numbers (1-35)
(1, 'FRONT', 0, 30), (2, 'FRONT', 0, 30), (3, 'FRONT', 0, 30), (4, 'FRONT', 0, 30), (5, 'FRONT', 0, 30),
(6, 'FRONT', 0, 30), (7, 'FRONT', 0, 30), (8, 'FRONT', 0, 30), (9, 'FRONT', 0, 30), (10, 'FRONT', 0, 30),
(11, 'FRONT', 0, 30), (12, 'FRONT', 0, 30), (13, 'FRONT', 0, 30), (14, 'FRONT', 0, 30), (15, 'FRONT', 0, 30),
(16, 'FRONT', 0, 30), (17, 'FRONT', 0, 30), (18, 'FRONT', 0, 30), (19, 'FRONT', 0, 30), (20, 'FRONT', 0, 30),
(21, 'FRONT', 0, 30), (22, 'FRONT', 0, 30), (23, 'FRONT', 0, 30), (24, 'FRONT', 0, 30), (25, 'FRONT', 0, 30),
(26, 'FRONT', 0, 30), (27, 'FRONT', 0, 30), (28, 'FRONT', 0, 30), (29, 'FRONT', 0, 30), (30, 'FRONT', 0, 30),
(31, 'FRONT', 0, 30), (32, 'FRONT', 0, 30), (33, 'FRONT', 0, 30), (34, 'FRONT', 0, 30), (35, 'FRONT', 0, 30),
-- Back zone numbers (1-12)
(1, 'BACK', 0, 30), (2, 'BACK', 0, 30), (3, 'BACK', 0, 30), (4, 'BACK', 0, 30), (5, 'BACK', 0, 30),
(6, 'BACK', 0, 30), (7, 'BACK', 0, 30), (8, 'BACK', 0, 30), (9, 'BACK', 0, 30), (10, 'BACK', 0, 30),
(11, 'BACK', 0, 30), (12, 'BACK', 0, 30);