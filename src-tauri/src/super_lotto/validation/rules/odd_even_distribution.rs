use crate::super_lotto::{
    models::{ValidationRequest, ValidationError},
    errors::SuperLottoError,
};

use super::super_lotto::validation::{ValidationBuilder, ValidationRule, ValidationCache};

/// Validates odd/even distribution in lottery numbers
pub struct OddEvenDistributionRule;

impl ValidationRule for OddEvenDistributionRule {
    fn validate(&self, request: &ValidationRequest) -> Result<(), ValidationError> {
        match &request.data {
            serde_json::Value::Array(front_numbers) => {
                let odd_count = request.data.as_array().unwrap()
                    .iter()
                    .filter(|&n| n % 2 == 1)
                    .count();

                let even_count = 5 - odd_count;

                // Check for reasonable distribution (not too skewed)
                if odd_count < 1 || odd_count > 4 {
                    return Err(ValidationError::InvalidJson(String::from(
                        "Unbalanced odd/even distribution. Expected 2-3 odd numbers."
                    )));
                }

                Ok(())
            }
            _ => Err(ValidationError::InvalidJson(String::from(
                "Expected front zone array for odd/even analysis"
            ))),
        }
    }
}