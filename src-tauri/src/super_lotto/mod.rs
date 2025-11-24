//! Super Lotto lottery prediction functionality
//!
//! This module provides comprehensive lottery analysis and prediction capabilities
//! for the Chinese Super Lotto (大乐透) game.

pub mod errors;
pub mod models;
pub mod services;
pub mod utils;
pub mod validation;
pub mod predictions;

// Placeholder modules to be implemented
pub mod analysis;
pub mod commands;

// Re-export main types for easier access
pub use errors::SuperLottoResult as Result;
