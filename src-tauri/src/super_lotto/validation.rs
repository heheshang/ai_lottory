use serde::{Deserialize, Serialize};

/// Super Lotto number validation utilities
pub struct SuperLottoValidation;

/// Validation result with detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            errors: vec![message.into()],
            warnings: Vec::new(),
        }
    }

    pub fn errors(messages: Vec<impl Into<String>>) -> Self {
        Self {
            is_valid: false,
            errors: messages.into_iter().map(|m| m.into()).collect(),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, message: impl Into<String>) -> Self {
        self.warnings.push(message.into());
        self
    }

    pub fn with_warnings(mut self, warnings: Vec<impl Into<String>>) -> Self {
        for warning in warnings {
            self.warnings.push(warning.into());
        }
        self
    }
}

/// Super Lotto draw data validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawValidation {
    pub draw_number: String,
    pub draw_date: String,
    pub front_zone: Vec<u32>,
    pub back_zone: Vec<u32>,
    pub jackpot: Option<f64>,
}

impl SuperLottoValidation {
    /// Validate a complete Super Lotto draw
    pub fn validate_draw(draw: &DrawValidation) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Validate draw number
        if let Err(error) = Self::validate_draw_number(&draw.draw_number) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        // Validate draw date
        if let Err(error) = Self::validate_draw_date(&draw.draw_date) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        // Validate front zone numbers
        if let Err(error) = Self::validate_front_zone(&draw.front_zone) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        // Validate back zone numbers
        if let Err(error) = Self::validate_back_zone(&draw.back_zone) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        // Validate jackpot amount
        if let Some(jackpot) = draw.jackpot {
            if let Err(error) = Self::validate_jackpot(jackpot) {
                result.is_valid = false;
                result.errors.extend(error.errors);
            }
        }

        // Add warnings for unusual patterns
        result.warnings.extend(Self::check_unusual_patterns(
            &draw.front_zone,
            &draw.back_zone,
        ));

        result
    }

    /// Validate draw number format
    pub fn validate_draw_number(draw_number: &str) -> Result<(), ValidationResult> {
        if draw_number.trim().is_empty() {
            return Err(ValidationResult::error("期号不能为空"));
        }

        // Check if draw number contains only digits
        if !draw_number.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationResult::error("期号只能包含数字"));
        }

        // Check reasonable length (typically 3-8 digits for lottery draws)
        if draw_number.len() < 3 || draw_number.len() > 8 {
            return Err(ValidationResult::error("期号长度应在3-8位之间"));
        }

        Ok(())
    }

    /// Validate draw date format
    pub fn validate_draw_date(date_str: &str) -> Result<(), ValidationResult> {
        if date_str.trim().is_empty() {
            return Err(ValidationResult::error("开奖日期不能为空"));
        }

        // Try to parse date in common formats
        let formats = ["%Y-%m-%d", "%Y/%m/%d", "%Y-%m-%d %H:%M:%S", "%Y%m%d"];

        for format in formats.iter() {
            if chrono::NaiveDateTime::parse_from_str(date_str, format).is_ok()
                || chrono::NaiveDate::parse_from_str(date_str, format).is_ok()
            {
                return Ok(());
            }
        }

        Err(ValidationResult::error(
            "日期格式无效，请使用YYYY-MM-DD格式",
        ))
    }

    /// Validate front zone numbers (5 numbers from 1-35)
    pub fn validate_front_zone(numbers: &[u32]) -> Result<(), ValidationResult> {
        let mut errors = Vec::new();

        if numbers.len() != 5 {
            errors.push("前区号码必须为5个".to_string());
        }

        // Check range
        for (i, &num) in numbers.iter().enumerate() {
            if num < 1 || num > 35 {
                errors.push(format!("前区第{}个号码{}超出范围(1-35)", i + 1, num));
            }
        }

        // Check for duplicates
        let unique_numbers: std::collections::HashSet<u32> = numbers.iter().cloned().collect();
        if unique_numbers.len() != numbers.len() {
            errors.push("前区号码不能重复".to_string());
        }

        if !errors.is_empty() {
            Err(ValidationResult::errors(errors))
        } else {
            Ok(())
        }
    }

    /// Validate back zone numbers (2 numbers from 1-12)
    pub fn validate_back_zone(numbers: &[u32]) -> Result<(), ValidationResult> {
        let mut errors = Vec::new();

        if numbers.len() != 2 {
            errors.push("后区号码必须为2个".to_string());
        }

        // Check range
        for (i, &num) in numbers.iter().enumerate() {
            if num < 1 || num > 12 {
                errors.push(format!("后区第{}个号码{}超出范围(1-12)", i + 1, num));
            }
        }

        // Check for duplicates
        if numbers.len() == 2 && numbers[0] == numbers[1] {
            errors.push("后区号码不能重复".to_string());
        }

        if !errors.is_empty() {
            Err(ValidationResult::errors(errors))
        } else {
            Ok(())
        }
    }

    /// Validate jackpot amount
    pub fn validate_jackpot(amount: f64) -> Result<(), ValidationResult> {
        if amount < 0.0 {
            return Err(ValidationResult::error("奖金金额不能为负数"));
        }

        if amount > 10_000_000_000.0 {
            return Err(ValidationResult::error("奖金金额超出合理范围"));
        }

        // Check for reasonable precision (2 decimal places)
        let rounded = (amount * 100.0).round() / 100.0;
        if (amount - rounded).abs() > 0.000001 {
            return Err(ValidationResult::error("奖金金额最多保留2位小数"));
        }

        Ok(())
    }

    /// Check for unusual patterns in numbers
    fn check_unusual_patterns(front_zone: &[u32], _back_zone: &[u32]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for consecutive numbers in front zone
        let mut sorted_front = front_zone.to_vec();
        sorted_front.sort();

        for window in sorted_front.windows(2) {
            if window[1] - window[0] == 1 {
                warnings.push("前区包含连续号码".to_string());
                break;
            }
        }

        // Check for arithmetic sequences
        if sorted_front.len() >= 3 {
            let diff1 = sorted_front[1] - sorted_front[0];
            let diff2 = sorted_front[2] - sorted_front[1];
            if diff1 == diff2 {
                warnings.push("前区可能包含等差数列".to_string());
            }
        }

        // Check for all odd or all even numbers
        let odd_count = front_zone.iter().filter(|&x| x % 2 == 1).count();
        if odd_count == 0 {
            warnings.push("前区全为偶数".to_string());
        } else if odd_count == front_zone.len() {
            warnings.push("前区全为奇数".to_string());
        }

        // Check for extreme sum values
        let sum: u32 = front_zone.iter().sum();
        if sum < 50 {
            warnings.push("前区和值偏小".to_string());
        } else if sum > 150 {
            warnings.push("前区和值偏大".to_string());
        }

        warnings
    }

    /// Validate user input for number selection
    pub fn validate_user_selection(front: &[u32], back: &[u32]) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Validate front zone
        if let Err(error) = Self::validate_front_zone(front) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        // Validate back zone
        if let Err(error) = Self::validate_back_zone(back) {
            result.is_valid = false;
            result.errors.extend(error.errors);
        }

        result
    }

    /// Validate date range for analysis
    pub fn validate_date_range(start_date: &str, end_date: &str) -> Result<(), ValidationResult> {
        // Parse dates
        let start = match chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return Err(ValidationResult::error("开始日期格式无效")),
        };

        let end = match chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return Err(ValidationResult::error("结束日期格式无效")),
        };

        // Check if start is before end
        if start > end {
            return Err(ValidationResult::error("开始日期不能晚于结束日期"));
        }

        // Check if date range is reasonable (not too far in the past or future)
        let now = chrono::Utc::now().date_naive();
        let earliest_reasonable = now - chrono::Duration::days(365 * 20); // 20 years ago
        let latest_reasonable = now + chrono::Duration::days(365); // 1 year in future

        if start < earliest_reasonable || end < earliest_reasonable {
            return Err(ValidationResult::error("日期范围不能早于20年前"));
        }

        if start > latest_reasonable || end > latest_reasonable {
            return Err(ValidationResult::error("日期范围不能晚于1年后"));
        }

        Ok(())
    }

    /// Validate analysis parameters
    pub fn validate_analysis_params(
        days: Option<u32>,
        min_occurrences: Option<u32>,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        if let Some(days_val) = days {
            if days_val == 0 {
                result.is_valid = false;
                result.errors.push("分析天数不能为0".to_string());
            } else if days_val > 3650 {
                result.is_valid = false;
                result.errors.push("分析天数不能超过10年".to_string());
            }
        }

        if let Some(occurrences) = min_occurrences {
            if occurrences == 0 {
                result.is_valid = false;
                result.errors.push("最小出现次数不能为0".to_string());
            } else if occurrences > 1000 {
                result.is_valid = false;
                result.errors.push("最小出现次数设置过大".to_string());
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_draw_number() {
        assert!(SuperLottoValidation::validate_draw_number("2023001").is_ok());
        assert!(SuperLottoValidation::validate_draw_number("001").is_ok());
        assert!(SuperLottoValidation::validate_draw_number("12345678").is_ok());

        assert!(SuperLottoValidation::validate_draw_number("").is_err());
        assert!(SuperLottoValidation::validate_draw_number("ABC").is_err());
        assert!(SuperLottoValidation::validate_draw_number("12").is_err());
        assert!(SuperLottoValidation::validate_draw_number("123456789").is_err());
    }

    #[test]
    fn test_validate_front_zone() {
        assert!(SuperLottoValidation::validate_front_zone(&[1, 2, 3, 4, 5]).is_ok());
        assert!(SuperLottoValidation::validate_front_zone(&[35, 34, 33, 32, 31]).is_ok());

        assert!(SuperLottoValidation::validate_front_zone(&[1, 2, 3, 4]).is_err()); // Not enough
        assert!(SuperLottoValidation::validate_front_zone(&[1, 2, 3, 4, 5, 6]).is_err()); // Too many
        assert!(SuperLottoValidation::validate_front_zone(&[0, 2, 3, 4, 5]).is_err()); // Too small
        assert!(SuperLottoValidation::validate_front_zone(&[1, 2, 3, 4, 36]).is_err()); // Too large
        assert!(SuperLottoValidation::validate_front_zone(&[1, 2, 3, 4, 1]).is_err());
        // Duplicate
    }

    #[test]
    fn test_validate_back_zone() {
        assert!(SuperLottoValidation::validate_back_zone(&[1, 2]).is_ok());
        assert!(SuperLottoValidation::validate_back_zone(&[11, 12]).is_ok());

        assert!(SuperLottoValidation::validate_back_zone(&[1]).is_err()); // Not enough
        assert!(SuperLottoValidation::validate_back_zone(&[1, 2, 3]).is_err()); // Too many
        assert!(SuperLottoValidation::validate_back_zone(&[0, 2]).is_err()); // Too small
        assert!(SuperLottoValidation::validate_back_zone(&[1, 13]).is_err()); // Too large
        assert!(SuperLottoValidation::validate_back_zone(&[5, 5]).is_err()); // Duplicate
    }

    #[test]
    fn test_validate_jackpot() {
        assert!(SuperLottoValidation::validate_jackpot(1000000.0).is_ok());
        assert!(SuperLottoValidation::validate_jackpot(12345678.90).is_ok());

        assert!(SuperLottoValidation::validate_jackpot(-100.0).is_err());
        assert!(SuperLottoValidation::validate_jackpot(12345678.901).is_err());
    }
}
