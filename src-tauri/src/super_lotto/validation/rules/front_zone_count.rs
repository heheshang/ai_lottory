use crate::super_lotto::{
    models::{ValidationRequest, ValidationError},
    errors::SuperLottoError,
};

use super::super_lotto::validation::{ValidationBuilder, ValidationRule, ValidationCache};

/// Validates front zone contains exactly 5 numbers (1-35)
pub struct FrontZoneCountRule;

impl ValidationRule for FrontZoneCountRule {
    fn validate(&self, request: &ValidationRequest) -> Result<(), ValidationError> {
        match &request.data {
            serde_json::Value::Array(front_numbers) => {
                if front_numbers.len() != 5 {
                    return Err(ValidationError::InvalidFrontZoneCount(front_numbers.len()));
                } else {
                    Ok(())
                }
            }
            _ => Err(ValidationError::InvalidJson(String::from(
                "Expected front zone array"
            ))),
        }
    }
}