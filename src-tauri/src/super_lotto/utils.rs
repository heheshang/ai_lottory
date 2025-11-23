use chrono::{DateTime, Utc};
use rand::Rng;

/// Utility functions for Super Lotto operations
pub struct SuperLottoUtils;

impl SuperLottoUtils {
    /// Generate a unique ID for Super Lotto draws
    pub fn generate_draw_id(draw_number: &str, draw_date: &str) -> String {
        format!("sl_{}_{}", draw_number, draw_date.replace("-", ""))
    }

    /// Validate front zone numbers (1-35, no duplicates, exactly 5 numbers)
    pub fn validate_front_zone(numbers: &[u32]) -> bool {
        if numbers.len() != 5 {
            return false;
        }

        // Check range
        for &num in numbers {
            if num < 1 || num > 35 {
                return false;
            }
        }

        // Check for duplicates
        let unique_numbers: std::collections::HashSet<u32> = numbers.iter().cloned().collect();
        unique_numbers.len() == 5
    }

    /// Validate back zone numbers (1-12, no duplicates, exactly 2 numbers)
    pub fn validate_back_zone(numbers: &[u32]) -> bool {
        if numbers.len() != 2 {
            return false;
        }

        // Check range
        for &num in numbers {
            if num < 1 || num > 12 {
                return false;
            }
        }

        // Check for duplicates
        let unique_numbers: std::collections::HashSet<u32> = numbers.iter().cloned().collect();
        unique_numbers.len() == 2
    }

    /// Calculate sum of front zone numbers
    pub fn calculate_front_sum(numbers: &[u32]) -> u32 {
        numbers.iter().sum()
    }

    /// Calculate odd/even count for front zone
    pub fn calculate_odd_even_counts(numbers: &[u32]) -> (u32, u32) {
        let odd_count = numbers.iter().filter(|&x| x % 2 == 1).count() as u32;
        let even_count = numbers.len() as u32 - odd_count;
        (odd_count, even_count)
    }

    /// Check if there are consecutive numbers in the array
    pub fn has_consecutive_numbers(numbers: &[u32]) -> bool {
        if numbers.len() < 2 {
            return false;
        }

        let mut sorted_numbers = numbers.to_vec();
        sorted_numbers.sort();

        for i in 0..sorted_numbers.len() - 1 {
            if sorted_numbers[i + 1] - sorted_numbers[i] == 1 {
                return true;
            }
        }

        false
    }

    /// Format date for display
    pub fn format_date(date: &DateTime<Utc>) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// Parse date string to DateTime
    pub fn parse_date(date_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
        let dt = DateTime::parse_from_str(date_str, "%Y-%m-%d")?;
        Ok(dt.with_timezone(&Utc))
    }

    /// Generate a random timestamp for test data
    pub fn generate_random_timestamp() -> DateTime<Utc> {
        let start = DateTime::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap();
        let start_utc = start.with_timezone(&Utc);
        let end = Utc::now();
        let duration = end.signed_duration_since(start_utc);
        let random_duration =
            duration.num_milliseconds() * rand::thread_rng().gen_range(0..=1) as i64;
        start_utc + chrono::Duration::milliseconds(random_duration)
    }

    /// Calculate hot score based on frequency and recency
    pub fn calculate_hot_score(frequency: u32, last_seen_days: u32) -> f64 {
        let base_score = frequency as f64;
        let recency_factor = 1.0 / (1.0 + (last_seen_days as f64 / 30.0)); // Decay over 30 days
        base_score * recency_factor
    }

    /// Calculate cold score based on days since last appearance
    pub fn calculate_cold_score(frequency: u32, last_seen_days: u32) -> f64 {
        let base_score = last_seen_days as f64;
        let frequency_factor = 1.0 / (1.0 + (frequency as f64 / 10.0)); // Adjust for frequency
        base_score * frequency_factor
    }

    /// Calculate gap between two consecutive appearances of a number
    pub fn calculate_gap(current_date: &DateTime<Utc>, previous_date: &DateTime<Utc>) -> u32 {
        let duration = current_date.signed_duration_since(*previous_date);
        duration.num_days() as u32
    }
}
