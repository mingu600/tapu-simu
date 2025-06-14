//! Battle event system
//! 
//! This module will handle battle events and processing
//! in future implementation phases.

use serde::{Deserialize, Serialize};
use crate::side::SideId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleEvent {
    TurnStart(u32),
    PokemonDamage { 
        target: (SideId, usize), 
        damage: u16, 
        source: Option<(SideId, usize)> 
    },
    PokemonFaint((SideId, usize)),
    MoveUse { 
        user: (SideId, usize), 
        move_id: String, 
        targets: Vec<(SideId, usize)> 
    },
    // More events will be added in future phases
}

// Placeholder for future implementation