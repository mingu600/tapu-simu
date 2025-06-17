//! # Battle Format System
//! 
//! This module defines the core battle format system for the V2 engine.
//! It provides format definitions, position management, and format-aware
//! battle mechanics.

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::generation::Generation;

/// Format types for battle mechanics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatType {
    /// Single battle format (1v1)
    Singles,
    /// Double battle format (2v2)
    Doubles,
    /// VGC (Video Game Championships) format (2v2 with VGC rules)
    Vgc,
    /// Triple battle format (3v3) - deprecated in modern Pokemon
    Triples,
}

impl FormatType {
    /// Returns the number of active Pokemon per side for this format type
    pub fn active_pokemon_count(&self) -> usize {
        match self {
            FormatType::Singles => 1,
            FormatType::Doubles | FormatType::Vgc => 2,
            FormatType::Triples => 3,
        }
    }

    /// Returns true if this format supports spread moves affecting multiple targets
    pub fn supports_spread_moves(&self) -> bool {
        match self {
            FormatType::Singles => false,
            FormatType::Doubles | FormatType::Vgc | FormatType::Triples => true,
        }
    }

    /// Returns true if ally damage is possible in this format
    pub fn allows_ally_damage(&self) -> bool {
        match self {
            FormatType::Singles => false,
            FormatType::Doubles | FormatType::Vgc | FormatType::Triples => true,
        }
    }
}

impl fmt::Display for FormatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatType::Singles => write!(f, "Singles"),
            FormatType::Doubles => write!(f, "Doubles"),
            FormatType::Vgc => write!(f, "VGC"),
            FormatType::Triples => write!(f, "Triples"),
        }
    }
}

/// Clauses that can be applied to a battle format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatClause {
    /// Sleep Clause: Only one Pokemon per team can be asleep at a time
    SleepClause,
    /// Freeze Clause: Only one Pokemon per team can be frozen at a time (older gens)
    FreezeClause,
    /// Species Clause: No duplicate species on a team
    SpeciesClause,
    /// Item Clause: No duplicate items on a team
    ItemClause,
    /// Evasion Clause: Evasion-boosting moves banned
    EvasionClause,
    /// OHKO Clause: One-hit KO moves banned
    OhkoClause,
    /// Moody Clause: Moody ability banned
    MoodyClause,
    /// Swagger Clause: Swagger move banned
    SwaggerClause,
    /// Baton Pass Clause: Baton Pass move restrictions
    BatonPassClause,
    /// Endless Battle Clause: Strategies that create endless battles banned
    EndlessBattleClause,
}

/// Specific Pokemon, moves, items, or abilities that are banned
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BanList {
    pub species: Vec<String>,
    pub moves: Vec<String>,
    pub items: Vec<String>,
    pub abilities: Vec<String>,
}

impl BanList {
    /// Create an empty ban list
    pub fn empty() -> Self {
        Self {
            species: Vec::new(),
            moves: Vec::new(),
            items: Vec::new(),
            abilities: Vec::new(),
        }
    }

    /// Create a ban list with specific entries
    pub fn new(
        species: Vec<String>,
        moves: Vec<String>,
        items: Vec<String>,
        abilities: Vec<String>,
    ) -> Self {
        Self {
            species: species.into_iter().map(|s| s.to_lowercase()).collect(),
            moves: moves.into_iter().map(|s| s.to_lowercase()).collect(),
            items: items.into_iter().map(|s| s.to_lowercase()).collect(),
            abilities: abilities.into_iter().map(|s| s.to_lowercase()).collect(),
        }
    }
}

/// Comprehensive battle format with generation, rules, and mechanics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BattleFormat {
    /// Display name for the format (e.g., "Gen 9 OU", "Gen 4 Uber")
    pub name: String,
    /// Pokemon generation this format uses
    pub generation: Generation,
    /// Basic format type (Singles, Doubles, etc.)
    pub format_type: FormatType,
    /// Team size (typically 6 for most formats)
    pub team_size: usize,
    /// Number of active Pokemon per side
    pub active_per_side: usize,
    /// Clauses that apply to this format
    pub clauses: Vec<FormatClause>,
    /// Banned content for this format
    pub ban_list: BanList,
}

impl BattleFormat {
    /// Create a new battle format
    pub fn new(
        name: String,
        generation: Generation,
        format_type: FormatType,
    ) -> Self {
        let active_per_side = format_type.active_pokemon_count();
        Self {
            name,
            generation,
            format_type,
            team_size: 6, // Standard team size
            active_per_side,
            clauses: Vec::new(),
            ban_list: BanList::empty(),
        }
    }

    /// Add clauses to this format
    pub fn with_clauses(mut self, clauses: Vec<FormatClause>) -> Self {
        self.clauses = clauses;
        self
    }

    /// Add ban list to this format
    pub fn with_bans(mut self, ban_list: BanList) -> Self {
        self.ban_list = ban_list;
        self
    }

    /// Returns the number of active Pokemon per side for this format
    pub fn active_pokemon_count(&self) -> usize {
        self.active_per_side
    }

    /// Returns true if this format supports spread moves affecting multiple targets
    pub fn supports_spread_moves(&self) -> bool {
        self.format_type.supports_spread_moves()
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
        self.format_type.allows_ally_damage()
    }

    /// Returns all valid slot indices for this format
    pub fn valid_slots(&self) -> Vec<usize> {
        (0..self.active_pokemon_count()).collect()
    }

    /// Check if a specific clause is active
    pub fn has_clause(&self, clause: &FormatClause) -> bool {
        self.clauses.contains(clause)
    }

    /// Check if a species is banned
    pub fn is_species_banned(&self, species: &str) -> bool {
        self.ban_list.species.contains(&species.to_lowercase())
    }

    /// Check if a move is banned
    pub fn is_move_banned(&self, move_name: &str) -> bool {
        self.ban_list.moves.contains(&move_name.to_lowercase())
    }

    /// Check if an item is banned
    pub fn is_item_banned(&self, item: &str) -> bool {
        self.ban_list.items.contains(&item.to_lowercase())
    }

    /// Check if an ability is banned
    pub fn is_ability_banned(&self, ability: &str) -> bool {
        self.ban_list.abilities.contains(&ability.to_lowercase())
    }

    /// Create standard competitive formats
    pub fn gen9_ou() -> Self {
        Self::new(
            "Gen 9 OU".to_string(),
            Generation::Gen9,
            FormatType::Singles,
        )
        .with_clauses(vec![
            FormatClause::SleepClause,
            FormatClause::SpeciesClause,
            FormatClause::EvasionClause,
            FormatClause::OhkoClause,
            FormatClause::EndlessBattleClause,
        ])
    }

    pub fn gen4_ou() -> Self {
        Self::new(
            "Gen 4 OU".to_string(),
            Generation::Gen4,
            FormatType::Singles,
        )
        .with_clauses(vec![
            FormatClause::SleepClause,
            FormatClause::FreezeClause,
            FormatClause::SpeciesClause,
            FormatClause::EvasionClause,
            FormatClause::OhkoClause,
        ])
    }

    pub fn vgc2024() -> Self {
        Self::new(
            "VGC 2024".to_string(),
            Generation::Gen9,
            FormatType::Vgc,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
            FormatClause::ItemClause,
        ])
    }

    pub fn doubles() -> Self {
        Self::new(
            "Doubles".to_string(),
            Generation::Gen9,
            FormatType::Doubles,
        )
    }

}

impl Default for BattleFormat {
    fn default() -> Self {
        BattleFormat::gen9_ou()
    }
}

impl fmt::Display for BattleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
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
        let singles = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
        let doubles = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let vgc = BattleFormat::new("VGC".to_string(), Generation::Gen9, FormatType::Vgc);
        let triples = BattleFormat::new("Triples".to_string(), Generation::Gen9, FormatType::Triples);
        
        assert_eq!(singles.active_pokemon_count(), 1);
        assert_eq!(doubles.active_pokemon_count(), 2);
        assert_eq!(vgc.active_pokemon_count(), 2);
        assert_eq!(triples.active_pokemon_count(), 3);
    }

    #[test]
    fn test_battle_format_spread_moves() {
        let singles = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
        let doubles = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let vgc = BattleFormat::new("VGC".to_string(), Generation::Gen9, FormatType::Vgc);
        let triples = BattleFormat::new("Triples".to_string(), Generation::Gen9, FormatType::Triples);
        
        assert!(!singles.supports_spread_moves());
        assert!(doubles.supports_spread_moves());
        assert!(vgc.supports_spread_moves());
        assert!(triples.supports_spread_moves());
    }

    #[test]
    fn test_battle_position_validity() {
        let singles = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
        let doubles = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);

        assert!(BattlePosition::new(SideReference::SideOne, 0).is_valid_for_format(&singles));
        assert!(!BattlePosition::new(SideReference::SideOne, 1).is_valid_for_format(&singles));

        assert!(BattlePosition::new(SideReference::SideOne, 0).is_valid_for_format(&doubles));
        assert!(BattlePosition::new(SideReference::SideOne, 1).is_valid_for_format(&doubles));
        assert!(!BattlePosition::new(SideReference::SideOne, 2).is_valid_for_format(&doubles));
    }

    #[test]
    fn test_ally_positions() {
        let doubles = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let singles = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);

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
        let doubles = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let pos = BattlePosition::new(SideReference::SideOne, 0);

        let opponents = pos.opponent_positions(&doubles);
        assert_eq!(opponents.len(), 2);
        assert!(opponents.contains(&BattlePosition::new(SideReference::SideTwo, 0)));
        assert!(opponents.contains(&BattlePosition::new(SideReference::SideTwo, 1)));
    }

    #[test]
    fn test_generation_format_integration() {
        let gen9_ou = BattleFormat::gen9_ou();
        let gen4_ou = BattleFormat::gen4_ou();
        let vgc2024 = BattleFormat::vgc2024();

        assert_eq!(gen9_ou.generation, Generation::Gen9);
        assert_eq!(gen4_ou.generation, Generation::Gen4);
        assert_eq!(vgc2024.generation, Generation::Gen9);

        assert!(gen9_ou.has_clause(&FormatClause::SleepClause));
        assert!(gen4_ou.has_clause(&FormatClause::FreezeClause));
        assert!(!gen9_ou.has_clause(&FormatClause::FreezeClause));

        assert_eq!(gen9_ou.format_type, FormatType::Singles);
        assert_eq!(vgc2024.format_type, FormatType::Vgc);
    }

    #[test]
    fn test_ban_list_functionality() {
        let ban_list = BanList::new(
            vec!["mewtwo".to_string()],
            vec!["swagger".to_string()],
            vec!["brightpowder".to_string()],
            vec!["moody".to_string()],
        );

        let format = BattleFormat::new("Test".to_string(), Generation::Gen9, FormatType::Singles)
            .with_bans(ban_list);

        assert!(format.is_species_banned("mewtwo"));
        assert!(format.is_species_banned("MEWTWO")); // Case insensitive
        assert!(!format.is_species_banned("pikachu"));

        assert!(format.is_move_banned("swagger"));
        assert!(!format.is_move_banned("tackle"));

        assert!(format.is_item_banned("brightpowder"));
        assert!(!format.is_item_banned("leftovers"));

        assert!(format.is_ability_banned("moody"));
        assert!(!format.is_ability_banned("levitate"));
    }
}