-- Create lottery_draws table
CREATE TABLE IF NOT EXISTS lottery_draws (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    draw_date TEXT NOT NULL, -- ISO 8601 datetime string
    winning_numbers TEXT NOT NULL, -- JSON array of numbers
    bonus_number INTEGER,
    jackpot_amount REAL,
    lottery_type TEXT NOT NULL, -- e.g., "powerball", "megamillions"
    created_at TEXT NOT NULL -- ISO 8601 datetime string
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_lottery_draws_date ON lottery_draws(draw_date);
CREATE INDEX IF NOT EXISTS idx_lottery_draws_type ON lottery_draws(lottery_type);
CREATE INDEX IF NOT EXISTS idx_lottery_draws_date_type ON lottery_draws(draw_date, lottery_type);