//! Battle format definitions

use serde::{Deserialize, Serialize};

/// Supported battle formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattleFormat {
    Singles,
    Doubles,
    Triples,
    Multi,
    FreeForAll,
}

impl BattleFormat {
    /// Number of active Pokemon per side
    pub fn active_per_side(&self) -> usize {
        match self {
            BattleFormat::Singles => 1,
            BattleFormat::Doubles => 2,
            BattleFormat::Triples => 3,
            BattleFormat::Multi => 1, // per player, but 2 players per side
            BattleFormat::FreeForAll => 1,
        }
    }
    
    /// Maximum number of sides supported
    pub fn max_sides(&self) -> usize {
        match self {
            BattleFormat::Singles => 2,
            BattleFormat::Doubles => 2,
            BattleFormat::Triples => 2,
            BattleFormat::Multi => 4,
            BattleFormat::FreeForAll => 4,
        }
    }
    
    /// Whether this format uses position-based targeting
    pub fn uses_position_targeting(&self) -> bool {
        match self {
            BattleFormat::Singles => false,
            BattleFormat::Doubles => true,
            BattleFormat::Triples => true,
            BattleFormat::Multi => true,
            BattleFormat::FreeForAll => true,
        }
    }
    
    /// Get adjacent positions for a given position
    pub fn adjacent_positions(&self, position: usize, side_positions: usize) -> Vec<usize> {
        match self {
            BattleFormat::Singles => vec![],
            BattleFormat::Doubles => {
                match position {
                    0 => vec![1],
                    1 => vec![0],
                    _ => vec![],
                }
            }
            BattleFormat::Triples => {
                match position {
                    0 => vec![1],
                    1 => vec![0, 2],
                    2 => vec![1],
                    _ => vec![],
                }
            }
            BattleFormat::Multi => {
                // Similar to doubles but with team coordination
                match position {
                    0 => vec![1],
                    1 => vec![0],
                    _ => vec![],
                }
            }
            BattleFormat::FreeForAll => vec![], // No adjacency in FFA
        }
    }
}

impl Default for BattleFormat {
    fn default() -> Self {
        BattleFormat::Singles
    }
}

/// Format-specific rules and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRules {
    pub team_size: usize,
    pub allow_megas: bool,
    pub allow_z_moves: bool,
    pub allow_dynamax: bool,
    pub allow_terastalization: bool,
    pub timer_per_turn: Option<u32>, // seconds
    pub generation: u8,
}

impl Default for FormatRules {
    fn default() -> Self {
        Self {
            team_size: 6,
            allow_megas: true,
            allow_z_moves: true,
            allow_dynamax: true,
            allow_terastalization: true,
            timer_per_turn: None,
            generation: 9,
        }
    }
}