-- Initial database migration for Super Lotto application
-- This migration creates the basic tables for lottery data management

-- Super Lotto draws table
CREATE TABLE IF NOT EXISTS super_lotto_draws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_number TEXT NOT NULL UNIQUE,
    draw_date TEXT NOT NULL, -- ISO format: YYYY-MM-DD
    front_zone TEXT NOT NULL, -- JSON array of 5 numbers (1-35)
    back_zone TEXT NOT NULL,  -- JSON array of 2 numbers (1-12)
    jackpot_amount REAL,     -- Optional jackpot amount
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Number statistics table
CREATE TABLE IF NOT EXISTS number_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL,
    zone TEXT NOT NULL CHECK (zone IN ('front', 'back')),
    total_occurrences INTEGER NOT NULL DEFAULT 0,
    last_occurrence TEXT,
    average_gap REAL NOT NULL DEFAULT 0.0,
    hot_score REAL NOT NULL DEFAULT 0.0,
    cold_score REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(number, zone)
);

-- Pattern analysis results table
CREATE TABLE IF NOT EXISTS pattern_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL,
    pattern_data TEXT NOT NULL, -- JSON data for the pattern
    sample_size INTEGER NOT NULL,
    confidence_score REAL NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    analysis_period_days INTEGER NOT NULL,
    min_occurrences INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- User predictions table (for future feature)
CREATE TABLE IF NOT EXISTS user_predictions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    draw_number TEXT NOT NULL,
    predicted_front TEXT NOT NULL, -- JSON array
    predicted_back TEXT NOT NULL,  -- JSON array
    prediction_date TEXT NOT NULL DEFAULT (datetime('now')),
    is_correct INTEGER DEFAULT 0, -- 0: not evaluated, 1: correct, -1: incorrect
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_date ON super_lotto_draws(draw_date);
CREATE INDEX IF NOT EXISTS idx_super_lotto_draws_number ON super_lotto_draws(draw_number);
CREATE INDEX IF NOT EXISTS idx_number_stats_zone ON number_statistics(zone);
CREATE INDEX IF NOT EXISTS idx_number_stats_number ON number_statistics(number);
CREATE INDEX IF NOT EXISTS idx_pattern_type ON pattern_analysis(pattern_type);
CREATE INDEX IF NOT EXISTS idx_user_predictions_user ON user_predictions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_predictions_draw ON user_predictions(draw_number);

-- Insert some default data for testing
INSERT OR IGNORE INTO number_statistics (number, zone, total_occurrences) VALUES
-- Front zone numbers (1-35)
(1, 'front', 0), (2, 'front', 0), (3, 'front', 0), (4, 'front', 0), (5, 'front', 0),
(6, 'front', 0), (7, 'front', 0), (8, 'front', 0), (9, 'front', 0), (10, 'front', 0),
(11, 'front', 0), (12, 'front', 0), (13, 'front', 0), (14, 'front', 0), (15, 'front', 0),
(16, 'front', 0), (17, 'front', 0), (18, 'front', 0), (19, 'front', 0), (20, 'front', 0),
(21, 'front', 0), (22, 'front', 0), (23, 'front', 0), (24, 'front', 0), (25, 'front', 0),
(26, 'front', 0), (27, 'front', 0), (28, 'front', 0), (29, 'front', 0), (30, 'front', 0),
(31, 'front', 0), (32, 'front', 0), (33, 'front', 0), (34, 'front', 0), (35, 'front', 0),
-- Back zone numbers (1-12)
(1, 'back', 0), (2, 'back', 0), (3, 'back', 0), (4, 'back', 0), (5, 'back', 0),
(6, 'back', 0), (7, 'back', 0), (8, 'back', 0), (9, 'back', 0), (10, 'back', 0),
(11, 'back', 0), (12, 'back', 0);