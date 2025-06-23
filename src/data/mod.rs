//! # Data Layer
//!
//! This module provides the modernized data layer using
//! Pokemon Showdown data as the primary source.

pub mod conversion;
pub mod repository;
pub mod generation_loader;
pub mod showdown_types;
pub mod random_team_loader;
pub mod types;

// Re-exports for convenience
pub use conversion::*;
pub use repository::{Repository, RepositoryStats};
pub use generation_loader::GenerationRepository;
pub use showdown_types::*;
pub use random_team_loader::{RandomTeamLoader, RandomPokemonSet, RandomTeam};

// Core types still needed for compatibility
