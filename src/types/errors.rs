use crate::types::{AbilityId, ItemId, MoveId, SpeciesId};
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for battle simulation
#[derive(Debug, Error)]
pub enum BattleError {
    #[error("Invalid move choice: {reason}")]
    InvalidMoveChoice { reason: String },
    
    #[error("Pokemon {species} not found")]
    PokemonNotFound { species: SpeciesId },
    
    #[error("Move {move_id} not found")]
    MoveNotFound { move_id: MoveId },
    
    #[error("Ability {ability} not found")]
    AbilityNotFound { ability: AbilityId },
    
    #[error("Item {item} not found")]
    ItemNotFound { item: ItemId },
    
    #[error("Data loading failed")]
    DataLoad(#[from] DataError),
    
    #[error("Format validation failed")]
    FormatValidation(#[from] FormatError),
    
    #[error("Team validation failed")]
    TeamValidation(#[from] TeamError),
    
    #[error("Invalid battle state: {reason}")]
    InvalidState { reason: String },
    
    #[error("Battle execution failed: {reason}")]
    ExecutionFailed { reason: String },
}

/// Errors related to data loading and access
#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to read file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid JSON in {file}")]
    JsonParse {
        file: String,
        #[source]
        source: serde_json::Error,
    },
    
    #[error("Species {species} not found in data")]
    SpeciesNotFound { species: SpeciesId },
    
    #[error("Move {move_id} not found in data")]
    MoveNotFound { move_id: MoveId },
    
    #[error("Ability {ability} not found in data")]
    AbilityNotFound { ability: AbilityId },
    
    #[error("Item {item} not found in data")]
    ItemNotFound { item: ItemId },
    
    #[error("Data directory not found: {path}")]
    DataDirNotFound { path: PathBuf },
    
    #[error("Required data file missing: {file}")]
    RequiredFileMissing { file: String },
}

/// Errors related to battle format validation
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Invalid format name: {name}")]
    InvalidName { name: String },
    
    #[error("Unsupported generation: {generation}")]
    UnsupportedGeneration { generation: u8 },
    
    #[error("Invalid team size: {size} (expected {expected})")]
    InvalidTeamSize { size: usize, expected: usize },
    
    #[error("Banned species: {species}")]
    BannedSpecies { species: SpeciesId },
    
    #[error("Banned move: {move_id}")]
    BannedMove { move_id: MoveId },
    
    #[error("Banned ability: {ability}")]
    BannedAbility { ability: AbilityId },
    
    #[error("Banned item: {item}")]
    BannedItem { item: ItemId },
    
    #[error("Format rule violation: {rule}")]
    RuleViolation { rule: String },
}

/// Errors related to team validation and generation
#[derive(Debug, Error)]
pub enum TeamError {
    #[error("Invalid team size: {size}")]
    InvalidSize { size: usize },
    
    #[error("Duplicate species: {species}")]
    DuplicateSpecies { species: SpeciesId },
    
    #[error("Invalid Pokemon configuration: {reason}")]
    InvalidPokemon { reason: String },
    
    #[error("Random team generation failed: {reason}")]
    RandomGenerationFailed { reason: String },
    
    #[error("Team loading failed: {reason}")]
    LoadingFailed { reason: String },
    
    #[error("Format violation: {reason}")]
    FormatViolation { reason: String },
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Invalid configuration format")]
    InvalidFormat(#[from] serde_json::Error),
    
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },
}

/// Simulator initialization errors
#[derive(Debug, Error)]
pub enum SimulatorError {
    #[error("Simulator initialization failed")]
    InitializationFailed(#[from] DataError),
    
    #[error("Configuration error")]
    Config(#[from] ConfigError),
    
    #[error("Data repository unavailable")]
    DataUnavailable,
}

/// Type alias for common Result pattern
pub type BattleResult<T> = Result<T, BattleError>;
pub type DataResult<T> = Result<T, DataError>;
pub type FormatResult<T> = Result<T, FormatError>;
pub type TeamResult<T> = Result<T, TeamError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type SimulatorResult<T> = Result<T, SimulatorError>;