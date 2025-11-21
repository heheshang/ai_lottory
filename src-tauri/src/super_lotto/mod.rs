//! Super Lotto lottery prediction functionality
//!
//! This module provides comprehensive lottery analysis and prediction capabilities
//! for the Chinese Super Lotto (大乐透) game.

pub mod models;
pub mod services;
pub mod utils;
pub mod validation;
pub mod errors;

// Placeholder modules to be implemented
pub mod commands;
pub mod analysis;

// Re-export main types for easier access
pub use models::*;
pub use errors::SuperLottoError;
pub use errors::SuperLottoResult as Result;