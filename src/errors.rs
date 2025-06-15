//! Error types for the battle simulator

use thiserror::Error;

pub type BattleResult<T> = Result<T, BattleError>;

#[derive(Debug, Error)]
pub enum BattleError {
    #[error("Invalid move choice: {0}")]
    InvalidMove(String),
    
    #[error("Invalid switch choice: {0}")]
    InvalidSwitch(String),
    
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    
    #[error("Battle has already ended")]
    BattleEnded,
    
    #[error("Side {side} not found")]
    SideNotFound { side: crate::side::SideId },
    
    #[error("Pokemon not found at position {position}")]
    PokemonNotFound { position: usize },
    
    #[error("Invalid Pokemon: {0}")]
    InvalidPokemon(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid PRNG seed: {0}")]
    InvalidSeed(String),
    
    #[error("Turn {turn} not found in history")]
    TurnNotFound { turn: u32 },
    
    #[error("Cannot undo: {reason}")]
    CannotUndo { reason: String },
    
    #[error("Data error: {0}")]
    DataError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Event system stack overflow")]
    EventStackOverflow,
    
    #[error("Event system infinite loop detected")]
    EventInfiniteLoop,
}