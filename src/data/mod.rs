//! # Data Layer
//!
//! This module provides the data layer for the V2 engine using
//! Pokemon Showdown data as the primary source.

pub mod conversion;
pub mod loader;
pub mod ps_generation_loader;
pub mod ps_move_factory;
pub mod ps_types;
pub mod services;
pub mod types;

// Re-exports for convenience
pub use conversion::*;
pub use loader::PSDataRepository;
pub use ps_generation_loader::PSGenerationRepository;
pub use ps_move_factory::PSMoveFactory;
pub use ps_types::*;
pub use services::move_service::PSMoveService;
pub use services::type_chart::{create_ps_type_chart_loader, PSTypeChartLoader};

// Legacy types still needed for some compatibility
pub use types::{EngineMoveData, TypeEffectiveness};
