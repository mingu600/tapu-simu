//! # Data Layer
//! 
//! This module provides the data layer for the V2 engine, integrating with
//! rustemon (Rust wrapper for PokeAPI) to fetch Pokemon data.

pub mod rustemon_client;
pub mod types;
pub mod ps_types;
pub mod conversion;
pub mod ps_conversion;
pub mod move_service;
pub mod move_factory;
pub mod choices;

// Re-exports for convenience
pub use rustemon_client::RustemonClient;
pub use types::*;
pub use conversion::*;
pub use move_service::*;
pub use move_factory::*;
pub use choices::*;