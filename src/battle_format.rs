//! # Battle Format System
//! 
//! This module defines the core battle format system for the V2 engine.
//! It provides format definitions, position management, and format-aware
//! battle mechanics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents different Pokemon battle formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattleFormat {
    /// Single battle format (1v1)
    Singles,
    /// Double battle format (2v2)
    Doubles,
    /// VGC (Video Game Championships) format (2v2 with VGC rules)
    Vgc,
    /// Triple battle format (3v3) - deprecated in modern Pokemon
    Triples,
}

impl BattleFormat {
    /// Returns the number of active Pokemon per side for this format
    pub fn active_pokemon_count(&self) -> usize {
        match self {
            BattleFormat::Singles => 1,
            BattleFormat::Doubles | BattleFormat::Vgc => 2,
            BattleFormat::Triples => 3,
        }
    }

    /// Returns true if this format supports spread moves affecting multiple targets
    pub fn supports_spread_moves(&self) -> bool {
        match self {
            BattleFormat::Singles => false,
            BattleFormat::Doubles | BattleFormat::Vgc | BattleFormat::Triples => true,
        }
    }

    /// Returns the spread move damage multiplier for this format
    pub fn spread_damage_multiplier(&self) -> f32 {
        if self.supports_spread_moves() {
            0.75 // 25% damage reduction for spread moves
        } else {
            1.0
        }
    }

    /// Returns true if ally damage is possible in this format
    pub fn allows_ally_damage(&self) -> bool {
        match self {
            BattleFormat::Singles => false,
            BattleFormat::Doubles | BattleFormat::Vgc | BattleFormat::Triples => true,
        }
    }

    /// Returns all valid slot indices for this format
    pub fn valid_slots(&self) -> Vec<usize> {
        (0..self.active_pokemon_count()).collect()
    }
}

impl Default for BattleFormat {
    fn default() -> Self {
        BattleFormat::Singles
    }
}

impl fmt::Display for BattleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BattleFormat::Singles => write!(f, "Singles"),
            BattleFormat::Doubles => write!(f, "Doubles"),
            BattleFormat::Vgc => write!(f, "VGC"),
            BattleFormat::Triples => write!(f, "Triples"),
        }
    }
}

/// Represents which side of the battle (Player 1 or Player 2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SideReference {
    SideOne,
    SideTwo,
}

impl SideReference {
    /// Returns the opposite side
    pub fn opposite(&self) -> Self {
        match self {
            SideReference::SideOne => SideReference::SideTwo,
            SideReference::SideTwo => SideReference::SideOne,
        }
    }
}

impl fmt::Display for SideReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SideReference::SideOne => write!(f, "Side 1"),
            SideReference::SideTwo => write!(f, "Side 2"),
        }
    }
}

/// Represents a specific position on the battlefield
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BattlePosition {
    pub side: SideReference,
    pub slot: usize,
}

impl BattlePosition {
    /// Creates a new battle position
    pub fn new(side: SideReference, slot: usize) -> Self {
        Self { side, slot }
    }

    /// Returns true if this position is valid for the given format
    pub fn is_valid_for_format(&self, format: &BattleFormat) -> bool {
        self.slot < format.active_pokemon_count()
    }

    /// Returns the ally position for this position (same side, different slot)
    /// Returns None if no ally exists or this is the only Pokemon on the side
    pub fn ally_position(&self, format: &BattleFormat) -> Option<Self> {
        if format.active_pokemon_count() <= 1 {
            return None;
        }

        // For doubles, slot 0 ally is slot 1 and vice versa
        let ally_slot = match self.slot {
            0 => 1,
            1 => 0,
            _ => return None, // Triples would need more complex logic
        };

        if ally_slot < format.active_pokemon_count() {
            Some(BattlePosition::new(self.side, ally_slot))
        } else {
            None
        }
    }

    /// Returns all opponent positions for this position
    pub fn opponent_positions(&self, format: &BattleFormat) -> Vec<Self> {
        let opponent_side = self.side.opposite();
        (0..format.active_pokemon_count())
            .map(|slot| BattlePosition::new(opponent_side, slot))
            .collect()
    }

    /// Returns all positions on the same side as this position
    pub fn same_side_positions(&self, format: &BattleFormat) -> Vec<Self> {
        (0..format.active_pokemon_count())
            .map(|slot| BattlePosition::new(self.side, slot))
            .collect()
    }

    /// Returns all positions on the battlefield for the given format
    pub fn all_positions(format: &BattleFormat) -> Vec<Self> {
        let mut positions = Vec::new();
        for side in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..format.active_pokemon_count() {
                positions.push(BattlePosition::new(side, slot));
            }
        }
        positions
    }
}

impl fmt::Display for BattlePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.side, self.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_format_active_pokemon_count() {
        assert_eq!(BattleFormat::Singles.active_pokemon_count(), 1);
        assert_eq!(BattleFormat::Doubles.active_pokemon_count(), 2);
        assert_eq!(BattleFormat::Vgc.active_pokemon_count(), 2);
        assert_eq!(BattleFormat::Triples.active_pokemon_count(), 3);
    }

    #[test]
    fn test_battle_format_spread_moves() {
        assert!(!BattleFormat::Singles.supports_spread_moves());
        assert!(BattleFormat::Doubles.supports_spread_moves());
        assert!(BattleFormat::Vgc.supports_spread_moves());
        assert!(BattleFormat::Triples.supports_spread_moves());
    }

    #[test]
    fn test_battle_position_validity() {
        let singles = BattleFormat::Singles;
        let doubles = BattleFormat::Doubles;

        assert!(BattlePosition::new(SideReference::SideOne, 0).is_valid_for_format(&singles));
        assert!(!BattlePosition::new(SideReference::SideOne, 1).is_valid_for_format(&singles));

        assert!(BattlePosition::new(SideReference::SideOne, 0).is_valid_for_format(&doubles));
        assert!(BattlePosition::new(SideReference::SideOne, 1).is_valid_for_format(&doubles));
        assert!(!BattlePosition::new(SideReference::SideOne, 2).is_valid_for_format(&doubles));
    }

    #[test]
    fn test_ally_positions() {
        let doubles = BattleFormat::Doubles;
        let singles = BattleFormat::Singles;

        let pos_0 = BattlePosition::new(SideReference::SideOne, 0);
        let pos_1 = BattlePosition::new(SideReference::SideOne, 1);

        // In doubles, allies exist
        assert_eq!(pos_0.ally_position(&doubles), Some(pos_1));
        assert_eq!(pos_1.ally_position(&doubles), Some(pos_0));

        // In singles, no allies
        assert_eq!(pos_0.ally_position(&singles), None);
    }

    #[test]
    fn test_opponent_positions() {
        let doubles = BattleFormat::Doubles;
        let pos = BattlePosition::new(SideReference::SideOne, 0);

        let opponents = pos.opponent_positions(&doubles);
        assert_eq!(opponents.len(), 2);
        assert!(opponents.contains(&BattlePosition::new(SideReference::SideTwo, 0)));
        assert!(opponents.contains(&BattlePosition::new(SideReference::SideTwo, 1)));
    }
}