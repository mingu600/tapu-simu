//! # Battle Format System
//! 
//! This module defines the core battle format system for the V2 engine.
//! It provides format definitions, position management, and format-aware
//! battle mechanics.

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::generation::Generation;
use crate::types::{PokemonName, Moves, Items, Abilities};

/// Format types for battle mechanics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BanList {
    pub species: Vec<PokemonName>,
    pub moves: Vec<Moves>,
    pub items: Vec<Items>,
    pub abilities: Vec<Abilities>,
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
            species: species.into_iter().map(|s| crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(&s)).unwrap_or(PokemonName::NONE)).collect(),
            moves: moves.into_iter().map(|s| crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(&s)).unwrap_or(Moves::NONE)).collect(),
            items: items.into_iter().map(|s| crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(&s)).unwrap_or(Items::NONE)).collect(),
            abilities: abilities.into_iter().map(|s| crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(&s)).unwrap_or(Abilities::NONE)).collect(),
        }
    }
}

/// Comprehensive battle format with generation, rules, and mechanics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn new_with_settings(
        name: String,
        generation: Generation,
        format_type: FormatType,
        team_size: usize,
        active_per_side: usize,
    ) -> Self {
        Self {
            name,
            generation,
            format_type,
            team_size,
            active_per_side,
            clauses: Vec::new(),
            ban_list: BanList::empty(),
        }
    }

    /// Serialize the battle format to a compact string format
    /// Format: name|generation|format_type|team_size|active_per_side|clauses|ban_list
    pub fn serialize(&self) -> String {
        let clauses = self.clauses.iter()
            .map(|clause| (*clause as u8).to_string())
            .collect::<Vec<_>>()
            .join("~");
        
        let ban_list = format!("{}#{}#{}#{}",
            self.ban_list.species.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("~"),
            self.ban_list.moves.iter().map(|m| m.as_str()).collect::<Vec<_>>().join("~"),
            self.ban_list.items.iter().map(|i| i.as_str()).collect::<Vec<_>>().join("~"),
            self.ban_list.abilities.iter().map(|a| a.as_str()).collect::<Vec<_>>().join("~")
        );

        format!("{}|{}|{}|{}|{}|{}|{}",
            self.name,
            self.generation as u8,
            self.format_type as u8,
            self.team_size,
            self.active_per_side,
            clauses,
            ban_list
        )
    }

    /// Check if this format allows switching during battle
    pub fn allows_switching(&self) -> bool {
        // Most formats allow switching, only specific rule sets might not
        // For now, return true for all formats unless a specific clause forbids it
        true
    }

    /// Check if this format allows switching for a specific clause
    pub fn allows_switching_with_clauses(&self) -> bool {
        // TODO: Check for specific clauses that might forbid switching
        // For example, a theoretical "No Switch" clause
        !self.clauses.iter().any(|clause| matches!(clause, FormatClause::EndlessBattleClause))
    }

    /// Deserialize a battle format from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let parts: Vec<&str> = serialized.split('|').collect();
        if parts.len() != 7 {
            return Err(format!("Invalid battle format: expected 7 parts, got {}", parts.len()));
        }

        let name = parts[0].to_string();
        let generation_id = parts[1].parse::<u8>()
            .map_err(|_| format!("Invalid generation ID: {}", parts[1]))?;
        let generation = Generation::from(generation_id);
        
        let format_type_id = parts[2].parse::<u8>()
            .map_err(|_| format!("Invalid format type ID: {}", parts[2]))?;
        let format_type = match format_type_id {
            0 => FormatType::Singles,
            1 => FormatType::Doubles,
            2 => FormatType::Vgc,
            3 => FormatType::Triples,
            _ => return Err(format!("Invalid format type ID: {}", format_type_id)),
        };

        let team_size = parts[3].parse::<usize>()
            .map_err(|_| format!("Invalid team size: {}", parts[3]))?;
        let active_per_side = parts[4].parse::<usize>()
            .map_err(|_| format!("Invalid active per side: {}", parts[4]))?;

        // Parse clauses
        let mut clauses = Vec::new();
        if !parts[5].is_empty() {
            for clause_str in parts[5].split('~') {
                let clause_id = clause_str.parse::<u8>()
                    .map_err(|_| format!("Invalid clause ID: {}", clause_str))?;
                let clause = match clause_id {
                    0 => FormatClause::SleepClause,
                    1 => FormatClause::FreezeClause,
                    2 => FormatClause::SpeciesClause,
                    3 => FormatClause::ItemClause,
                    4 => FormatClause::EvasionClause,
                    5 => FormatClause::OhkoClause,
                    6 => FormatClause::MoodyClause,
                    7 => FormatClause::SwaggerClause,
                    8 => FormatClause::BatonPassClause,
                    9 => FormatClause::EndlessBattleClause,
                    _ => return Err(format!("Invalid clause ID: {}", clause_id)),
                };
                clauses.push(clause);
            }
        }

        // Parse ban list
        let ban_parts: Vec<&str> = parts[6].split('#').collect();
        if ban_parts.len() != 4 {
            return Err(format!("Invalid ban list format: {}", parts[6]));
        }
        
        let species = if ban_parts[0].is_empty() { Vec::new() } else { ban_parts[0].split('~').map(|s| s.to_string()).collect() };
        let moves = if ban_parts[1].is_empty() { Vec::new() } else { ban_parts[1].split('~').map(|s| s.to_string()).collect() };
        let items = if ban_parts[2].is_empty() { Vec::new() } else { ban_parts[2].split('~').map(|s| s.to_string()).collect() };
        let abilities = if ban_parts[3].is_empty() { Vec::new() } else { ban_parts[3].split('~').map(|s| s.to_string()).collect() };
        
        let ban_list = BanList::new(species, moves, items, abilities);

        Ok(Self {
            name,
            generation,
            format_type,
            team_size,
            active_per_side,
            clauses,
            ban_list,
        })
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
    pub fn is_species_banned(&self, species_id: &PokemonName) -> bool {
        self.ban_list.species.contains(species_id)
    }

    /// Check if a move is banned
    pub fn is_move_banned(&self, move_id: &Moves) -> bool {
        self.ban_list.moves.contains(move_id)
    }

    /// Check if an item is banned
    pub fn is_item_banned(&self, item_id: &Items) -> bool {
        self.ban_list.items.contains(item_id)
    }

    /// Check if an ability is banned
    pub fn is_ability_banned(&self, ability_id: &Abilities) -> bool {
        self.ban_list.abilities.contains(ability_id)
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

    pub fn gen1_ou() -> Self {
        Self::new(
            "Gen 1 OU".to_string(),
            Generation::Gen1,
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

    pub fn gen2_ou() -> Self {
        Self::new(
            "Gen 2 OU".to_string(),
            Generation::Gen2,
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

    pub fn gen3_ou() -> Self {
        Self::new(
            "Gen 3 OU".to_string(),
            Generation::Gen3,
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

    pub fn gen5_ou() -> Self {
        Self::new(
            "Gen 5 OU".to_string(),
            Generation::Gen5,
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

    pub fn gen6_ou() -> Self {
        Self::new(
            "Gen 6 OU".to_string(),
            Generation::Gen6,
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

    pub fn gen7_ou() -> Self {
        Self::new(
            "Gen 7 OU".to_string(),
            Generation::Gen7,
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

    pub fn gen8_ou() -> Self {
        Self::new(
            "Gen 8 OU".to_string(),
            Generation::Gen8,
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

    /// Random Battle Formats - These use predetermined teams from data files
    pub fn gen9_random_battle() -> Self {
        Self::new(
            "Gen 9 Random Battle".to_string(),
            Generation::Gen9,
            FormatType::Singles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
        ])
    }

    pub fn gen9_random_doubles() -> Self {
        Self::new(
            "Gen 9 Random Doubles".to_string(),
            Generation::Gen9,
            FormatType::Doubles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
        ])
    }

    pub fn gen9_vgc() -> Self {
        Self::new(
            "Gen 9 VGC".to_string(),
            Generation::Gen9,
            FormatType::Doubles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
            FormatClause::ItemClause,
        ])
    }

    pub fn gen8_random_battle() -> Self {
        Self::new(
            "Gen 8 Random Battle".to_string(),
            Generation::Gen8,
            FormatType::Singles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
        ])
    }

    pub fn gen8_random_doubles() -> Self {
        Self::new(
            "Gen 8 Random Doubles".to_string(),
            Generation::Gen8,
            FormatType::Doubles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
        ])
    }

    pub fn gen7_random_battle() -> Self {
        Self::new(
            "Gen 7 Random Battle".to_string(),
            Generation::Gen7,
            FormatType::Singles,
        )
        .with_clauses(vec![
            FormatClause::SpeciesClause,
        ])
    }

    /// Returns all available random battle formats
    pub fn random_battle_formats() -> Vec<Self> {
        vec![
            Self::gen9_random_battle(),
            Self::gen9_random_doubles(),
            Self::gen8_random_battle(),
            Self::gen8_random_doubles(),
            Self::gen7_random_battle(),
        ]
    }

    /// Check if this format is a random battle format
    pub fn is_random_battle(&self) -> bool {
        self.name.contains("Random")
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

    /// Convert to string for logging/debugging
    pub fn to_string(&self) -> String {
        match self {
            SideReference::SideOne => "S1".to_string(),
            SideReference::SideTwo => "S2".to_string(),
        }
    }

    /// Convert to index for array access
    pub fn to_index(&self) -> usize {
        match self {
            SideReference::SideOne => 0,
            SideReference::SideTwo => 1,
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

