//! Tapu Simu - High-performance Pokemon battle simulator
//! 
//! A Rust port of Pokemon Showdown's battle engine with a focus on:
//! - State serialization for AI/RL applications
//! - Efficient turn-level undo functionality 
//! - Deterministic battles with PRNG seeds
//! - Support for all Pokemon battle formats

pub mod battle_state;
pub mod pokemon;
pub mod side;
pub mod action_queue;
pub mod prng;
pub mod dex;
pub mod moves;
pub mod events;
pub mod battle;
pub mod format;
pub mod errors;

// Re-export core types
pub use battle_state::BattleState;
pub use pokemon::Pokemon;
pub use side::{Side, SideId};
pub use battle::Battle;
pub use errors::{BattleError, BattleResult};
pub use format::BattleFormat;

/// Common types used throughout the simulator
pub mod types {
    pub use serde::{Deserialize, Serialize};
    
    /// Pokemon type (Fire, Water, etc.)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Type {
        Normal, Fire, Water, Electric, Grass, Ice, Fighting, Poison,
        Ground, Flying, Psychic, Bug, Rock, Ghost, Dragon, Dark,
        Steel, Fairy,
    }
    
    /// Gender of a Pokemon
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Gender {
        Male, Female, Genderless,
    }
    
    /// Nature affecting stats
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Nature {
        Hardy, Lonely, Brave, Adamant, Naughty,
        Bold, Docile, Relaxed, Impish, Lax,
        Timid, Hasty, Serious, Jolly, Naive,
        Modest, Mild, Quiet, Bashful, Rash,
        Calm, Gentle, Sassy, Careful, Quirky,
    }
    
    /// Move category (Physical, Special, Status)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum MoveCategory {
        Physical, Special, Status,
    }
    
    /// Move targeting
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum MoveTarget {
        // Single targets
        Normal,         // One adjacent foe
        Any,           // Any single Pokemon on the field
        AdjacantAlly,  // One adjacent ally
        AdjacantFoe,   // One adjacent foe
        AdjacantAllyOrSelf, // One adjacent ally or self
        
        // Self targets
        Self_,         // User only
        
        // Multiple targets
        AllAdjacent,   // All adjacent Pokemon
        AllAdjacentFoes, // All adjacent foes
        AllAllies,     // All allies
        AllFoes,       // All foes
        All,           // Every Pokemon on the field
        
        // Field targets
        FoeSide,       // Foe's side of the field
        AllySide,      // User's side of the field
        RandomNormal,  // Random adjacent foe
    }
    
    /// Stats table
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct StatsTable {
        pub hp: u16,
        pub attack: u16,
        pub defense: u16,
        pub special_attack: u16,
        pub special_defense: u16,
        pub speed: u16,
    }
    
    impl Default for StatsTable {
        fn default() -> Self {
            Self {
                hp: 0,
                attack: 0,
                defense: 0,
                special_attack: 0,
                special_defense: 0,
                speed: 0,
            }
        }
    }
    
    /// Boosts table (-6 to +6 for each stat)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct BoostsTable {
        pub attack: i8,
        pub defense: i8,
        pub special_attack: i8,
        pub special_defense: i8,
        pub speed: i8,
        pub accuracy: i8,
        pub evasion: i8,
    }
    
    impl Default for BoostsTable {
        fn default() -> Self {
            Self {
                attack: 0,
                defense: 0,
                special_attack: 0,
                special_defense: 0,
                speed: 0,
                accuracy: 0,
                evasion: 0,
            }
        }
    }
}