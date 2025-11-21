//! Super Lotto validation utilities
//!
//! Comprehensive validation for Super Lotto data.

use crate::super_lotto::{
    models::{SuperLottoDraw, CreateSuperLottoDraw, PredictionResult, NumberFrequency},
    errors::{Result, SuperLottoError, ValidationError},
};

pub struct SuperLottoValidator;

impl SuperLottoValidator {
    pub fn validate_draw_data(draw: &CreateSuperLottoDraw) -> Result<()> {
        draw.validate()
    }

    pub fn validate_draw_object(draw: &SuperLottoDraw) -> Result<()> {
        draw.validate()
    }

    pub fn validate_prediction(prediction: &PredictionResult) -> Result<()> {
        // Validate front numbers
        if prediction.front_numbers.len() != 5 {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidFrontZoneCount(prediction.front_numbers.len())
            ));
        }

        if !prediction.front_numbers.iter().all(|&n| n >= 1 && n <= 35) {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidFrontZoneRange(
                    prediction.front_numbers.iter().find(|&&n| n < 1 || n > 35).copied().unwrap_or(0)
                )
            ));
        }

        // Validate back numbers
        if prediction.back_numbers.len() != 2 {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidBackZoneCount(prediction.back_numbers.len())
            ));
        }

        if !prediction.back_numbers.iter().all(|&n| n >= 1 && n <= 12) {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidBackZoneRange(
                    prediction.back_numbers.iter().find(|&&n| n < 1 || n > 12).copied().unwrap_or(0)
                )
            ));
        }

        // Validate confidence score
        if prediction.confidence_score < 0.0 || prediction.confidence_score > 1.0 {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidJson("Confidence score must be between 0.0 and 1.0".to_string())
            ));
        }

        Ok(())
    }

    pub fn validate_frequency(frequency: &NumberFrequency) -> Result<()> {
        if frequency.number == 0 {
            return Err(SuperLottoError::Validation(
                ValidationError::InvalidJson("Number cannot be zero".to_string())
            ));
        }

        match frequency.zone.as_str() {
            "FRONT" => {
                if frequency.number > 35 {
                    return Err(SuperLottoError::Validation(
                        ValidationError::InvalidFrontZoneRange(frequency.number)
                    ));
                }
            },
            "BACK" => {
                if frequency.number > 12 {
                    return Err(SuperLottoError::Validation(
                        ValidationError::InvalidBackZoneRange(frequency.number)
                    ));
                }
            },
            _ => {
                return Err(SuperLottoError::Validation(
                    ValidationError::InvalidJson(format!("Invalid zone: {}", frequency.zone))
                ));
            }
        }

        Ok(())
    }

    pub fn validate_analysis_parameters(days: u32, min_draws: u32) -> Result<()> {
        if days == 0 {
            return Err(SuperLottoError::InvalidInput("Days must be greater than 0".to_string()));
        }

        if days > 365 * 5 { // 5 years max
            return Err(SuperLottoError::InvalidInput("Days cannot exceed 5 years".to_string()));
        }

        // This would check against actual database count
        // TODO: Implement database query to check draw count
        if days < min_draws {
            return Err(SuperLottoError::InsufficientData(
                format!("Insufficient data: need at least {} days for analysis", min_draws)
            ));
        }

        Ok(())
    }
}