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
pub use types::EVStatType;

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
    
    impl StatsTable {
        /// Create a StatsTable with max IVs (31 for all stats)
        pub fn max() -> Self {
            Self {
                hp: 31,
                attack: 31,
                defense: 31,
                special_attack: 31,
                special_defense: 31,
                speed: 31,
            }
        }
        
        /// Create a StatsTable with competitive EVs (252/252/4 spread)
        pub fn competitive_evs(max_stat1: EVStatType, max_stat2: EVStatType) -> Self {
            let mut evs = Self::default();
            
            match max_stat1 {
                EVStatType::Hp => evs.hp = 252,
                EVStatType::Attack => evs.attack = 252,
                EVStatType::Defense => evs.defense = 252,
                EVStatType::SpecialAttack => evs.special_attack = 252,
                EVStatType::SpecialDefense => evs.special_defense = 252,
                EVStatType::Speed => evs.speed = 252,
            }
            
            match max_stat2 {
                EVStatType::Hp => evs.hp = 252,
                EVStatType::Attack => evs.attack = 252,
                EVStatType::Defense => evs.defense = 252,
                EVStatType::SpecialAttack => evs.special_attack = 252,
                EVStatType::SpecialDefense => evs.special_defense = 252,
                EVStatType::Speed => evs.speed = 252,
            }
            
            // Put remaining 4 EVs in HP if it's not already maxed
            if evs.hp < 252 {
                evs.hp = 4;
            }
            
            evs
        }
    }
    
    /// Stat type enumeration for EV spreading
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EVStatType {
        Hp,
        Attack,
        Defense,
        SpecialAttack,
        SpecialDefense,
        Speed,
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