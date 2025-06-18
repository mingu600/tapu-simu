//! # Type Effectiveness System
//! 
//! This module implements the complete Pokemon type effectiveness chart
//! with generation-specific variations and special cases.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::data::services::type_chart::{PSTypeChartLoader, create_ps_type_chart_loader};

/// Pokemon types with numeric indices for the effectiveness matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PokemonType {
    Normal = 0,
    Fire = 1,
    Water = 2,
    Electric = 3,
    Grass = 4,
    Ice = 5,
    Fighting = 6,
    Poison = 7,
    Ground = 8,
    Flying = 9,
    Psychic = 10,
    Bug = 11,
    Rock = 12,
    Ghost = 13,
    Dragon = 14,
    Dark = 15,
    Steel = 16,
    Fairy = 17,
    Typeless = 18, // Internal type for moves like Struggle
}

impl PokemonType {
    /// Convert from string representation
    pub fn from_str(type_str: &str) -> Option<Self> {
        match type_str.to_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "fire" => Some(Self::Fire),
            "water" => Some(Self::Water),
            "electric" => Some(Self::Electric),
            "grass" => Some(Self::Grass),
            "ice" => Some(Self::Ice),
            "fighting" => Some(Self::Fighting),
            "poison" => Some(Self::Poison),
            "ground" => Some(Self::Ground),
            "flying" => Some(Self::Flying),
            "psychic" => Some(Self::Psychic),
            "bug" => Some(Self::Bug),
            "rock" => Some(Self::Rock),
            "ghost" => Some(Self::Ghost),
            "dragon" => Some(Self::Dragon),
            "dark" => Some(Self::Dark),
            "steel" => Some(Self::Steel),
            "fairy" => Some(Self::Fairy),
            "typeless" | "???" => Some(Self::Typeless),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Electric => "Electric",
            Self::Grass => "Grass",
            Self::Ice => "Ice",
            Self::Fighting => "Fighting",
            Self::Poison => "Poison",
            Self::Ground => "Ground",
            Self::Flying => "Flying",
            Self::Psychic => "Psychic",
            Self::Bug => "Bug",
            Self::Rock => "Rock",
            Self::Ghost => "Ghost",
            Self::Dragon => "Dragon",
            Self::Dark => "Dark",
            Self::Steel => "Steel",
            Self::Fairy => "Fairy",
            Self::Typeless => "Typeless",
        }
    }

    /// Get all types (for iteration)
    pub fn all_types() -> [Self; 18] {
        [
            Self::Normal, Self::Fire, Self::Water, Self::Electric,
            Self::Grass, Self::Ice, Self::Fighting, Self::Poison,
            Self::Ground, Self::Flying, Self::Psychic, Self::Bug,
            Self::Rock, Self::Ghost, Self::Dragon, Self::Dark,
            Self::Steel, Self::Fairy,
        ]
    }
}

/// Type effectiveness chart with generation support
#[derive(Debug, Clone)]
pub struct TypeChart {
    /// 19x19 effectiveness matrix [attacking_type][defending_type]
    effectiveness: [[f32; 19]; 19],
    /// Generation this chart applies to
    generation: u8,
    /// Special case overrides for specific move-type combinations
    special_cases: HashMap<(String, PokemonType), f32>,
}

impl TypeChart {
    /// Create a new type chart for the specified generation
    pub fn new(generation: u8) -> Self {
        let mut chart = Self {
            effectiveness: [[1.0; 19]; 19],
            generation,
            special_cases: HashMap::new(),
        };
        
        // Try to load from PS data first
        let loader = create_ps_type_chart_loader();
        if let Ok(ps_effectiveness) = loader.load_type_chart(generation) {
            chart.load_from_ps_data(ps_effectiveness);
        } else {
            // Fallback to hardcoded chart only if PS data fails
            eprintln!("Warning: Failed to load PS type chart for gen {}, using fallback", generation);
            chart.initialize_effectiveness_matrix();
        }
        
        chart.add_special_cases();
        chart
    }

    /// Load effectiveness data from PS type chart
    fn load_from_ps_data(&mut self, ps_data: HashMap<(PokemonType, PokemonType), f32>) {
        // Initialize all to neutral
        for row in &mut self.effectiveness {
            row.fill(1.0);
        }
        
        // Apply PS data
        for ((attacking, defending), multiplier) in ps_data {
            self.effectiveness[attacking as usize][defending as usize] = multiplier;
        }
    }

    /// Get type effectiveness multiplier between two types
    pub fn get_effectiveness(&self, attacking_type: PokemonType, defending_type: PokemonType) -> f32 {
        self.effectiveness[attacking_type as usize][defending_type as usize]
    }

    /// Calculate damage multiplier for a move against target types
    pub fn calculate_damage_multiplier(
        &self,
        move_type: PokemonType,
        target_types: (PokemonType, PokemonType),
        tera_type: Option<PokemonType>,
        move_name: Option<&str>,
    ) -> f32 {
        // Check for special case overrides first
        if let Some(move_name) = move_name {
            for target_type in [target_types.0, target_types.1].iter() {
                if let Some(&override_multiplier) = self.special_cases.get(&(move_name.to_string(), *target_type)) {
                    return override_multiplier;
                }
            }
        }

        // Use Tera type if Terastallized, otherwise use normal types
        let effective_types = if let Some(tera) = tera_type {
            (tera, tera) // Terastallized Pokemon become single-type
        } else {
            target_types
        };

        let multiplier1 = self.get_effectiveness(move_type, effective_types.0);
        let multiplier2 = if effective_types.0 == effective_types.1 {
            1.0 // Single type or duplicate types
        } else {
            self.get_effectiveness(move_type, effective_types.1)
        };

        multiplier1 * multiplier2
    }

    /// Calculate STAB (Same Type Attack Bonus) multiplier
    pub fn calculate_stab_multiplier(
        &self,
        move_type: PokemonType,
        user_types: (PokemonType, PokemonType),
        tera_type: Option<PokemonType>,
        has_adaptability: bool,
    ) -> f32 {
        let types_to_check = if let Some(tera) = tera_type {
            vec![tera] // Terastallized Pokemon only get STAB from Tera type
        } else {
            if user_types.0 == user_types.1 {
                vec![user_types.0] // Single type
            } else {
                vec![user_types.0, user_types.1] // Dual type
            }
        };

        let has_stab = types_to_check.contains(&move_type);
        
        if !has_stab {
            1.0
        } else if has_adaptability {
            2.0 // Adaptability ability doubles STAB
        } else {
            1.5 // Normal STAB
        }
    }

    /// Initialize the complete type effectiveness matrix
    fn initialize_effectiveness_matrix(&mut self) {
        // Initialize all to 1.0 (neutral)
        for row in &mut self.effectiveness {
            row.fill(1.0);
        }

        // Super effective (2.0x) matchups
        let super_effective = [
            // Normal: No super effective
            
            // Fire
            (PokemonType::Fire, PokemonType::Grass),
            (PokemonType::Fire, PokemonType::Ice),
            (PokemonType::Fire, PokemonType::Bug),
            (PokemonType::Fire, PokemonType::Steel),
            
            // Water
            (PokemonType::Water, PokemonType::Fire),
            (PokemonType::Water, PokemonType::Ground),
            (PokemonType::Water, PokemonType::Rock),
            
            // Electric
            (PokemonType::Electric, PokemonType::Water),
            (PokemonType::Electric, PokemonType::Flying),
            
            // Grass
            (PokemonType::Grass, PokemonType::Water),
            (PokemonType::Grass, PokemonType::Ground),
            (PokemonType::Grass, PokemonType::Rock),
            
            // Ice
            (PokemonType::Ice, PokemonType::Grass),
            (PokemonType::Ice, PokemonType::Ground),
            (PokemonType::Ice, PokemonType::Flying),
            (PokemonType::Ice, PokemonType::Dragon),
            
            // Fighting
            (PokemonType::Fighting, PokemonType::Normal),
            (PokemonType::Fighting, PokemonType::Ice),
            (PokemonType::Fighting, PokemonType::Rock),
            (PokemonType::Fighting, PokemonType::Dark),
            (PokemonType::Fighting, PokemonType::Steel),
            
            // Poison
            (PokemonType::Poison, PokemonType::Grass),
            (PokemonType::Poison, PokemonType::Fairy),
            
            // Ground
            (PokemonType::Ground, PokemonType::Fire),
            (PokemonType::Ground, PokemonType::Electric),
            (PokemonType::Ground, PokemonType::Poison),
            (PokemonType::Ground, PokemonType::Rock),
            (PokemonType::Ground, PokemonType::Steel),
            
            // Flying
            (PokemonType::Flying, PokemonType::Grass),
            (PokemonType::Flying, PokemonType::Fighting),
            (PokemonType::Flying, PokemonType::Bug),
            
            // Psychic
            (PokemonType::Psychic, PokemonType::Fighting),
            (PokemonType::Psychic, PokemonType::Poison),
            
            // Bug
            (PokemonType::Bug, PokemonType::Grass),
            (PokemonType::Bug, PokemonType::Psychic),
            (PokemonType::Bug, PokemonType::Dark),
            
            // Rock
            (PokemonType::Rock, PokemonType::Fire),
            (PokemonType::Rock, PokemonType::Ice),
            (PokemonType::Rock, PokemonType::Flying),
            (PokemonType::Rock, PokemonType::Bug),
            
            // Ghost
            (PokemonType::Ghost, PokemonType::Psychic),
            (PokemonType::Ghost, PokemonType::Ghost),
            
            // Dragon
            (PokemonType::Dragon, PokemonType::Dragon),
            
            // Dark
            (PokemonType::Dark, PokemonType::Psychic),
            (PokemonType::Dark, PokemonType::Ghost),
            
            // Steel
            (PokemonType::Steel, PokemonType::Ice),
            (PokemonType::Steel, PokemonType::Rock),
            (PokemonType::Steel, PokemonType::Fairy),
            
            // Fairy
            (PokemonType::Fairy, PokemonType::Fighting),
            (PokemonType::Fairy, PokemonType::Dragon),
            (PokemonType::Fairy, PokemonType::Dark),
        ];

        for (attacking, defending) in super_effective {
            self.effectiveness[attacking as usize][defending as usize] = 2.0;
        }

        // Not very effective (0.5x) matchups
        let not_very_effective = [
            // Normal
            (PokemonType::Normal, PokemonType::Rock),
            (PokemonType::Normal, PokemonType::Steel),
            
            // Fire
            (PokemonType::Fire, PokemonType::Fire),
            (PokemonType::Fire, PokemonType::Water),
            (PokemonType::Fire, PokemonType::Rock),
            (PokemonType::Fire, PokemonType::Dragon),
            
            // Water
            (PokemonType::Water, PokemonType::Water),
            (PokemonType::Water, PokemonType::Grass),
            (PokemonType::Water, PokemonType::Dragon),
            
            // Electric
            (PokemonType::Electric, PokemonType::Electric),
            (PokemonType::Electric, PokemonType::Grass),
            (PokemonType::Electric, PokemonType::Dragon),
            
            // Grass
            (PokemonType::Grass, PokemonType::Fire),
            (PokemonType::Grass, PokemonType::Grass),
            (PokemonType::Grass, PokemonType::Poison),
            (PokemonType::Grass, PokemonType::Flying),
            (PokemonType::Grass, PokemonType::Bug),
            (PokemonType::Grass, PokemonType::Dragon),
            (PokemonType::Grass, PokemonType::Steel),
            
            // Ice
            (PokemonType::Ice, PokemonType::Fire),
            (PokemonType::Ice, PokemonType::Water),
            (PokemonType::Ice, PokemonType::Ice),
            (PokemonType::Ice, PokemonType::Steel),
            
            // Fighting
            (PokemonType::Fighting, PokemonType::Flying),
            (PokemonType::Fighting, PokemonType::Psychic),
            (PokemonType::Fighting, PokemonType::Bug),
            (PokemonType::Fighting, PokemonType::Fairy),
            
            // Poison
            (PokemonType::Poison, PokemonType::Poison),
            (PokemonType::Poison, PokemonType::Ground),
            (PokemonType::Poison, PokemonType::Rock),
            (PokemonType::Poison, PokemonType::Ghost),
            
            // Ground
            (PokemonType::Ground, PokemonType::Grass),
            (PokemonType::Ground, PokemonType::Bug),
            
            // Flying
            (PokemonType::Flying, PokemonType::Rock),
            (PokemonType::Flying, PokemonType::Steel),
            (PokemonType::Flying, PokemonType::Electric),
            
            // Psychic
            (PokemonType::Psychic, PokemonType::Psychic),
            (PokemonType::Psychic, PokemonType::Steel),
            
            // Bug
            (PokemonType::Bug, PokemonType::Fire),
            (PokemonType::Bug, PokemonType::Fighting),
            (PokemonType::Bug, PokemonType::Poison),
            (PokemonType::Bug, PokemonType::Flying),
            (PokemonType::Bug, PokemonType::Ghost),
            (PokemonType::Bug, PokemonType::Steel),
            (PokemonType::Bug, PokemonType::Fairy),
            
            // Rock
            (PokemonType::Rock, PokemonType::Fighting),
            (PokemonType::Rock, PokemonType::Ground),
            (PokemonType::Rock, PokemonType::Steel),
            
            // Ghost
            (PokemonType::Ghost, PokemonType::Dark),
            
            // Dragon
            (PokemonType::Dragon, PokemonType::Steel),
            
            // Dark
            (PokemonType::Dark, PokemonType::Fighting),
            (PokemonType::Dark, PokemonType::Dark),
            (PokemonType::Dark, PokemonType::Fairy),
            
            // Steel
            (PokemonType::Steel, PokemonType::Fire),
            (PokemonType::Steel, PokemonType::Water),
            (PokemonType::Steel, PokemonType::Electric),
            (PokemonType::Steel, PokemonType::Steel),
            
            // Fairy
            (PokemonType::Fairy, PokemonType::Fire),
            (PokemonType::Fairy, PokemonType::Poison),
            (PokemonType::Fairy, PokemonType::Steel),
        ];

        for (attacking, defending) in not_very_effective {
            self.effectiveness[attacking as usize][defending as usize] = 0.5;
        }

        // No effect (0.0x) matchups
        let no_effect = [
            (PokemonType::Normal, PokemonType::Ghost),
            (PokemonType::Fighting, PokemonType::Ghost),
            (PokemonType::Poison, PokemonType::Steel),
            (PokemonType::Ground, PokemonType::Flying),
            (PokemonType::Psychic, PokemonType::Dark),
            (PokemonType::Ghost, PokemonType::Normal),
        ];

        for (attacking, defending) in no_effect {
            self.effectiveness[attacking as usize][defending as usize] = 0.0;
        }

        // Apply generation-specific changes
        self.apply_generation_changes();
    }

    /// Apply generation-specific type chart changes
    fn apply_generation_changes(&mut self) {
        if self.generation >= 6 {
            // Gen 6+: Steel lost resistance to Ghost and Dark
            self.effectiveness[PokemonType::Ghost as usize][PokemonType::Steel as usize] = 1.0;
            self.effectiveness[PokemonType::Dark as usize][PokemonType::Steel as usize] = 1.0;
        }

        if self.generation < 6 {
            // Pre-Gen 6: No Fairy type
            for i in 0..19 {
                self.effectiveness[PokemonType::Fairy as usize][i] = 1.0;
                self.effectiveness[i][PokemonType::Fairy as usize] = 1.0;
            }
        }
    }

    /// Add special case overrides for specific moves
    fn add_special_cases(&mut self) {
        // Freeze-Dry is super effective against Water despite being Ice-type
        self.special_cases.insert(("freeze-dry".to_string(), PokemonType::Water), 2.0);
        
        // Flying Press is Fighting-type but hits like Fighting + Flying
        // This is handled in move-specific logic, not here
        
        // Thousand Arrows hits Flying types for neutral damage despite being Ground
        self.special_cases.insert(("thousand-arrows".to_string(), PokemonType::Flying), 1.0);
    }
}

/// Default type chart for the current generation (Gen 9)
impl Default for TypeChart {
    fn default() -> Self {
        Self::new(9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_effectiveness() {
        let chart = TypeChart::default();
        
        // Super effective
        assert_eq!(chart.get_effectiveness(PokemonType::Fire, PokemonType::Grass), 2.0);
        assert_eq!(chart.get_effectiveness(PokemonType::Water, PokemonType::Fire), 2.0);
        
        // Not very effective
        assert_eq!(chart.get_effectiveness(PokemonType::Fire, PokemonType::Water), 0.5);
        assert_eq!(chart.get_effectiveness(PokemonType::Grass, PokemonType::Fire), 0.5);
        
        // No effect
        assert_eq!(chart.get_effectiveness(PokemonType::Normal, PokemonType::Ghost), 0.0);
        assert_eq!(chart.get_effectiveness(PokemonType::Ground, PokemonType::Flying), 0.0);
        
        // Neutral
        assert_eq!(chart.get_effectiveness(PokemonType::Normal, PokemonType::Normal), 1.0);
    }

    #[test]
    fn test_dual_type_effectiveness() {
        let chart = TypeChart::default();
        
        // Fire vs Grass/Poison (Bulbasaur line) = 2.0 * 1.0 = 2.0 (Fire neutral vs Poison according to PS)
        let multiplier = chart.calculate_damage_multiplier(
            PokemonType::Fire,
            (PokemonType::Grass, PokemonType::Poison),
            None,
            None,
        );
        assert_eq!(multiplier, 2.0);
        
        // Rock vs Fire/Flying (Charizard) = 2.0 * 2.0 = 4.0
        let multiplier = chart.calculate_damage_multiplier(
            PokemonType::Rock,
            (PokemonType::Fire, PokemonType::Flying),
            None,
            None,
        );
        assert_eq!(multiplier, 4.0);
    }

    #[test]
    fn test_stab_calculation() {
        let chart = TypeChart::default();
        
        // Normal STAB
        let stab = chart.calculate_stab_multiplier(
            PokemonType::Fire,
            (PokemonType::Fire, PokemonType::Flying),
            None,
            false,
        );
        assert_eq!(stab, 1.5);
        
        // Adaptability STAB
        let stab = chart.calculate_stab_multiplier(
            PokemonType::Fire,
            (PokemonType::Fire, PokemonType::Flying),
            None,
            true,
        );
        assert_eq!(stab, 2.0);
        
        // No STAB
        let stab = chart.calculate_stab_multiplier(
            PokemonType::Water,
            (PokemonType::Fire, PokemonType::Flying),
            None,
            false,
        );
        assert_eq!(stab, 1.0);
    }

    #[test]
    fn test_tera_type_effectiveness() {
        let chart = TypeChart::default();
        
        // Terastallized Pokemon becomes single-type
        let multiplier = chart.calculate_damage_multiplier(
            PokemonType::Water,
            (PokemonType::Fire, PokemonType::Flying), // Original types
            Some(PokemonType::Electric), // Tera type
            None,
        );
        // Water vs Electric = 1.0 (neutral according to PS data)
        assert_eq!(multiplier, 1.0);
    }

    #[test]
    fn test_special_cases() {
        let chart = TypeChart::default();
        
        // Freeze-Dry vs Water should be 2.0 despite being Ice-type
        let multiplier = chart.calculate_damage_multiplier(
            PokemonType::Ice,
            (PokemonType::Water, PokemonType::Water),
            None,
            Some("freeze-dry"),
        );
        assert_eq!(multiplier, 2.0);
    }

    #[test]
    fn test_type_string_conversion() {
        assert_eq!(PokemonType::from_str("fire"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_str("WATER"), Some(PokemonType::Water));
        assert_eq!(PokemonType::from_str("Electric"), Some(PokemonType::Electric));
        assert_eq!(PokemonType::from_str("invalid"), None);
        
        assert_eq!(PokemonType::Fire.to_str(), "Fire");
        assert_eq!(PokemonType::Water.to_str(), "Water");
    }
}