//! # Data Management Architecture
//!
//! This module provides a comprehensive data management system for Tapu Simu,
//! built around Pokemon Showdown's data format as the primary source. The data
//! layer handles loading, conversion, and access to all Pokemon-related data
//! including species, moves, abilities, items, and generation-specific mechanics.
//!
//! ## Architecture Components
//!
//! - **Repository** (`repository`): Central data access layer that provides
//!   efficient lookup and caching of Pokemon data with normalize_name support.
//!
//! - **Generation Loader** (`generation_loader`): Handles loading of
//!   generation-specific data and mechanics, supporting multiple Pokemon
//!   generations with their unique rule sets.
//!
//!
//! - **Showdown Types** (`showdown_types`): Type definitions that match
//!   Pokemon Showdown's data format, ensuring compatibility and accuracy.
//!
//! - **Random Team Loader** (`random_team_loader`): Provides functionality
//!   for generating random teams for testing and battles, with format-specific
//!   team generation capabilities.
//!
//! ## Key Features
//!
//! - **Format-Aware Loading**: Data loading respects different battle formats
//!   and their specific restrictions and requirements.
//!
//! - **Efficient Caching**: Repository pattern with intelligent caching to
//!   minimize data loading overhead during battles.
//!
//! - **Name Normalization**: Consistent name handling that matches Pokemon
//!   Showdown's normalization rules for reliable data lookup.
//!
//! - **Generation Support**: Full support for different Pokemon generations
//!   with their unique data structures and mechanics.
//!
//! ## Usage Example
//!
//! Load the data repository and look up Pokemon data or generate random teams.

pub mod repositories;
pub mod generation_loader;
pub mod showdown_types;
pub mod random_team_loader;
pub mod types;

// Re-exports for convenience
pub use repositories::{GameDataRepository, MoveRepository, PokemonRepository, ItemRepository, RepositoryStats};
pub use generation_loader::GenerationRepository;
pub use showdown_types::*;
pub use random_team_loader::{RandomTeamLoader, RandomPokemonSet, RandomTeam};

// Core types still needed for compatibility
