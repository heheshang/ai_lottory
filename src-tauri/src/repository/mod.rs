//! Repository Pattern Implementation
//!
//! This module provides a repository abstraction layer for data access,
//! promoting clean separation between business logic and data storage concerns.

pub mod traits;
pub mod base;
pub mod queries;
pub mod transactions;
pub mod cache;
pub mod errors;
pub mod lottery;

pub use traits::*;
pub use base::*;
pub use queries::*;
pub use transactions::*;
pub use cache::*;
pub use errors::*;
pub use lottery::*;