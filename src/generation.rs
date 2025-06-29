//! # Generation System
//!
//! This module defines generation-specific battle mechanics and data structures.
//! Each generation has different rules, mechanics, and available content.

use serde::{Deserialize, Serialize};

/// Pokemon generations with their specific mechanics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Generation {
    Gen1 = 1,
    Gen2 = 2, 
    Gen3 = 3,
    Gen4 = 4,
    Gen5 = 5,
    Gen6 = 6,
    Gen7 = 7,
    Gen8 = 8,
    Gen9 = 9,
}

impl From<u8> for Generation {
    fn from(value: u8) -> Self {
        match value {
            1 => Generation::Gen1,
            2 => Generation::Gen2,
            3 => Generation::Gen3,
            4 => Generation::Gen4,
            5 => Generation::Gen5,
            6 => Generation::Gen6,
            7 => Generation::Gen7,
            8 => Generation::Gen8,
            9 => Generation::Gen9,
            _ => {
                eprintln!("Warning: Unknown generation number {}, defaulting to Gen9", value);
                Generation::Gen9
            }
        }
    }
}

impl Generation {
    /// Get all generations for iteration
    pub fn all() -> [Self; 9] {
        [
            Self::Gen1, Self::Gen2, Self::Gen3, Self::Gen4, Self::Gen5,
            Self::Gen6, Self::Gen7, Self::Gen8, Self::Gen9,
        ]
    }

    /// Convert from generation number
    pub fn from_number(gen: u8) -> Option<Self> {
        match gen {
            1 => Some(Self::Gen1),
            2 => Some(Self::Gen2),
            3 => Some(Self::Gen3),
            4 => Some(Self::Gen4),
            5 => Some(Self::Gen5),
            6 => Some(Self::Gen6),
            7 => Some(Self::Gen7),
            8 => Some(Self::Gen8),
            9 => Some(Self::Gen9),
            _ => None,
        }
    }

    /// Convert to generation number
    pub fn number(&self) -> u8 {
        *self as u8
    }

    /// Get the generation mechanics
    pub fn get_mechanics(&self) -> GenerationMechanics {
        GenerationMechanics::new(*self)
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gen1 => "Generation 1 (RBY)",
            Self::Gen2 => "Generation 2 (GSC)",
            Self::Gen3 => "Generation 3 (RSE)",
            Self::Gen4 => "Generation 4 (DPPt)",
            Self::Gen5 => "Generation 5 (BW)",
            Self::Gen6 => "Generation 6 (XY)",
            Self::Gen7 => "Generation 7 (SM)",
            Self::Gen8 => "Generation 8 (SwSh)",
            Self::Gen9 => "Generation 9 (SV)",
        }
    }
}

/// Generation-specific battle mechanics and constants
#[derive(Debug, Clone)]
pub struct GenerationMechanics {
    pub generation: Generation,
    
    // Damage calculation mechanics
    pub critical_hit_multiplier: f32,
    pub terrain_multiplier: f32,
    
    // Feature availability
    pub has_physical_special_split: bool,
    pub has_abilities: bool,
    pub has_held_items: bool,
    pub has_steel_type: bool,
    pub has_dark_type: bool,
    pub has_fairy_type: bool,
    pub has_weather: bool,
    pub has_double_battles: bool,
    pub has_z_moves: bool,
    pub has_mega_evolution: bool,
    pub has_dynamax: bool,
    pub has_terastallization: bool,
    
    // Battle mechanics
    pub burn_physical_reduction: f32,
    pub paralysis_speed_reduction: f32,
    pub paralysis_immobilization_chance: f32,
    
    // Type effectiveness changes
    pub ghost_vs_psychic: f32,      // Gen 1: 0.0, Gen 2+: 2.0
    pub bug_vs_poison: f32,         // Gen 1: 2.0, Gen 2+: 0.5
    pub poison_vs_bug: f32,         // Gen 1: 2.0, Gen 2+: 1.0
    pub ice_vs_fire: f32,           // Gen 1: 1.0, Gen 2+: 0.5
}

impl GenerationMechanics {
    /// Create generation mechanics for a specific generation
    pub fn new(generation: Generation) -> Self {
        match generation {
            Generation::Gen1 => Self::gen1_mechanics(),
            Generation::Gen2 => Self::gen2_mechanics(),
            Generation::Gen3 => Self::gen3_mechanics(),
            Generation::Gen4 => Self::gen4_mechanics(),
            Generation::Gen5 => Self::gen5_mechanics(),
            Generation::Gen6 => Self::gen6_mechanics(),
            Generation::Gen7 => Self::gen7_mechanics(),
            Generation::Gen8 => Self::gen8_mechanics(),
            Generation::Gen9 => Self::gen9_mechanics(),
        }
    }

    fn gen1_mechanics() -> Self {
        Self {
            generation: Generation::Gen1,
            critical_hit_multiplier: 2.0,
            terrain_multiplier: 1.0, // No terrain
            has_physical_special_split: false,
            has_abilities: false,
            has_held_items: false,
            has_steel_type: false,
            has_dark_type: false,
            has_fairy_type: false,
            has_weather: false,
            has_double_battles: false,
            has_z_moves: false,
            has_mega_evolution: false,
            has_dynamax: false,
            has_terastallization: false,
            burn_physical_reduction: 0.5,
            paralysis_speed_reduction: 0.25, // 75% reduction
            paralysis_immobilization_chance: 0.25,
            ghost_vs_psychic: 0.0,   // Gen 1 bug: Ghost had no effect on Psychic
            bug_vs_poison: 2.0,      // Gen 1: Bug was super effective vs Poison
            poison_vs_bug: 2.0,      // Gen 1: Poison was super effective vs Bug  
            ice_vs_fire: 1.0,        // Gen 1: Ice was neutral vs Fire
        }
    }

    fn gen2_mechanics() -> Self {
        Self {
            generation: Generation::Gen2,
            critical_hit_multiplier: 2.0,
            terrain_multiplier: 1.0, // No terrain
            has_physical_special_split: false,
            has_abilities: false,
            has_held_items: true,
            has_steel_type: true,
            has_dark_type: true,
            has_fairy_type: false,
            has_weather: true,
            has_double_battles: false,
            has_z_moves: false,
            has_mega_evolution: false,
            has_dynamax: false,
            has_terastallization: false,
            burn_physical_reduction: 0.5,
            paralysis_speed_reduction: 0.5, // 50% reduction
            paralysis_immobilization_chance: 0.25,
            ghost_vs_psychic: 2.0,   // Fixed in Gen 2
            bug_vs_poison: 0.5,      // Changed in Gen 2
            poison_vs_bug: 1.0,      // Changed in Gen 2
            ice_vs_fire: 0.5,        // Changed in Gen 2
        }
    }

    fn gen3_mechanics() -> Self {
        let mut mechanics = Self::gen2_mechanics();
        mechanics.generation = Generation::Gen3;
        mechanics.has_abilities = true;
        mechanics.has_double_battles = true;
        mechanics
    }

    fn gen4_mechanics() -> Self {
        let mut mechanics = Self::gen3_mechanics();
        mechanics.generation = Generation::Gen4;
        mechanics.has_physical_special_split = true;
        mechanics
    }

    fn gen5_mechanics() -> Self {
        let mut mechanics = Self::gen4_mechanics();
        mechanics.generation = Generation::Gen5;
        mechanics
    }

    fn gen6_mechanics() -> Self {
        let mut mechanics = Self::gen5_mechanics();
        mechanics.generation = Generation::Gen6;
        mechanics.critical_hit_multiplier = 1.5; // Changed in Gen 6
        mechanics.has_fairy_type = true;
        mechanics.has_mega_evolution = true;
        mechanics
    }

    fn gen7_mechanics() -> Self {
        let mut mechanics = Self::gen6_mechanics();
        mechanics.generation = Generation::Gen7;
        mechanics.terrain_multiplier = 1.5; // Terrain introduced
        mechanics.has_z_moves = true;
        mechanics
    }

    fn gen8_mechanics() -> Self {
        let mut mechanics = Self::gen7_mechanics();
        mechanics.generation = Generation::Gen8;
        mechanics.terrain_multiplier = 1.3; // Terrain nerfed in Gen 8
        mechanics.has_dynamax = true;
        mechanics
    }

    fn gen9_mechanics() -> Self {
        let mut mechanics = Self::gen8_mechanics();
        mechanics.generation = Generation::Gen9;
        mechanics.has_terastallization = true;
        mechanics
    }
}

/// Trait for generation-aware battle mechanics
pub trait GenerationBattleMechanics {
    fn get_critical_multiplier(&self) -> f32;
    fn get_terrain_multiplier(&self) -> f32;
    fn get_burn_reduction(&self) -> f32;
    fn has_feature(&self, feature: GenerationFeature) -> bool;
    fn get_type_effectiveness_override(&self, attacking: &str, defending: &str) -> Option<f32>;
    fn generation(&self) -> Generation;
}

/// Features that vary by generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationFeature {
    PhysicalSpecialSplit,
    Abilities,
    HeldItems,
    SteelType,
    DarkType,
    FairyType,
    Weather,
    DoubleBattles,
    ZMoves,
    MegaEvolution,
    Dynamax,
    Terastallization,
}

impl GenerationBattleMechanics for GenerationMechanics {
    fn get_critical_multiplier(&self) -> f32 {
        self.critical_hit_multiplier
    }

    fn get_terrain_multiplier(&self) -> f32 {
        self.terrain_multiplier
    }

    fn get_burn_reduction(&self) -> f32 {
        self.burn_physical_reduction
    }

    fn has_feature(&self, feature: GenerationFeature) -> bool {
        match feature {
            GenerationFeature::PhysicalSpecialSplit => self.has_physical_special_split,
            GenerationFeature::Abilities => self.has_abilities,
            GenerationFeature::HeldItems => self.has_held_items,
            GenerationFeature::SteelType => self.has_steel_type,
            GenerationFeature::DarkType => self.has_dark_type,
            GenerationFeature::FairyType => self.has_fairy_type,
            GenerationFeature::Weather => self.has_weather,
            GenerationFeature::DoubleBattles => self.has_double_battles,
            GenerationFeature::ZMoves => self.has_z_moves,
            GenerationFeature::MegaEvolution => self.has_mega_evolution,
            GenerationFeature::Dynamax => self.has_dynamax,
            GenerationFeature::Terastallization => self.has_terastallization,
        }
    }

    fn get_type_effectiveness_override(&self, attacking: &str, defending: &str) -> Option<f32> {
        match (attacking.to_lowercase().as_str(), defending.to_lowercase().as_str()) {
            ("ghost", "psychic") => Some(self.ghost_vs_psychic),
            ("bug", "poison") => Some(self.bug_vs_poison),
            ("poison", "bug") => Some(self.poison_vs_bug),
            ("ice", "fire") => Some(self.ice_vs_fire),
            _ => None,
        }
    }

    fn generation(&self) -> Generation {
        self.generation
    }
}

