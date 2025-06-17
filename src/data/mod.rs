//! # Data Layer
//! 
//! This module provides the data layer for the V2 engine using
//! Pokemon Showdown data as the primary source.

pub mod types;
pub mod ps_types;
pub mod ps_conversion;
pub mod ps_loader;
pub mod ps_generation_loader;
pub mod ps_move_service;
pub mod ps_move_factory;
pub mod choices;
pub mod ps_type_chart_loader;

// Re-exports for convenience
pub use ps_move_service::PSMoveService;
pub use ps_move_factory::PSMoveFactory;
pub use ps_generation_loader::PSGenerationRepository;
pub use ps_types::*;
pub use ps_conversion::*;
pub use ps_loader::PSDataRepository;
pub use choices::*;
pub use ps_type_chart_loader::{PSTypeChartLoader, create_ps_type_chart_loader};

// Legacy types still needed for some compatibility
pub use types::{EngineMoveData, TypeEffectiveness};