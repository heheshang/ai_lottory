//! Service Layer Abstraction
//!
//! Provides a clean separation between business logic and data access layers.

pub mod traits;
pub mod base;
pub mod lottery;
pub mod analysis;
pub mod auth;
pub mod events;
pub mod errors;

pub use traits::*;
pub use base::*;
pub use lottery::*;
pub use analysis::*;
pub use auth::*;
pub use events::*;
pub use errors::*;