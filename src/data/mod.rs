//! # Data Layer
//!
//! This module provides the data layer for the V2 engine using
//! Pokemon Showdown data as the primary source.

pub mod ps_conversion;
pub mod ps_generation_loader;
pub mod ps_loader;
pub mod ps_move_factory;
pub mod ps_move_service;
pub mod ps_type_chart_loader;
pub mod ps_types;
pub mod types;

// Re-exports for convenience
pub use ps_conversion::*;
pub use ps_generation_loader::PSGenerationRepository;
pub use ps_loader::PSDataRepository;
pub use ps_move_factory::PSMoveFactory;
pub use ps_move_service::PSMoveService;
pub use ps_type_chart_loader::{create_ps_type_chart_loader, PSTypeChartLoader};
pub use ps_types::*;

// Legacy types still needed for some compatibility
pub use types::{EngineMoveData, TypeEffectiveness};
