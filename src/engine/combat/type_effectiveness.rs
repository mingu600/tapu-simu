//! # Type Effectiveness System
//! 
//! This module implements the complete Pokemon type effectiveness chart
//! with generation-specific variations and special cases.

use std::collections::HashMap;
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};
use crate::types::Moves;
use crate::types::PokemonType;
// Removed old service layer import - type chart loading now handled differently

/// Global cache for generation-specific type charts
/// This eliminates the expensive recreation of type charts for every damage calculation
static TYPE_CHART_CACHE: LazyLock<HashMap<u8, TypeChart>> = LazyLock::new(|| {
    let mut cache = HashMap::new();
    
    // Pre-compute all generation type charts
    for generation in 1..=9 {
        cache.insert(generation, TypeChart::new_uncached(generation));
    }
    
    cache
});


/// Type effectiveness chart with generation support
#[derive(Debug, Clone)]
pub struct TypeChart {
    /// 19x19 effectiveness matrix [attacking_type][defending_type]
    effectiveness: [[f32; 19]; 19],
    /// Generation this chart applies to
    generation: u8,
    /// Special case overrides for specific move-type combinations
    special_cases: HashMap<(Moves, PokemonType), f32>,
}

impl TypeChart {
    /// Get a cached type chart for the specified generation (preferred method)
    /// 
    /// This is the recommended way to access type charts as it avoids expensive
    /// recreation of the effectiveness matrix for every damage calculation.
    /// 
    /// ## Performance Benefits
    /// - **Memory**: Reuses pre-computed charts instead of allocating new ones
    /// - **CPU**: Eliminates 500+ operations per damage calculation
    /// - **Scalability**: Shared across all battle calculations
    /// 
    /// ## Usage
    /// ```rust
    /// let type_chart = TypeChart::get_cached(8); // Generation 8
    /// let effectiveness = type_chart.get_effectiveness(PokemonType::Fire, PokemonType::Water);
    /// ```
    pub fn get_cached(generation: u8) -> &'static TypeChart {
        TYPE_CHART_CACHE.get(&generation)
            .unwrap_or_else(|| &TYPE_CHART_CACHE[&9]) // Default to Gen 9 if invalid generation
    }
    
    /// Create a new type chart for the specified generation (internal use only)
    /// 
    /// This method creates a new TypeChart instance and should only be used internally
    /// for populating the cache. Use `get_cached()` for normal operations.
    fn new_uncached(generation: u8) -> Self {
        let mut chart = Self {
            effectiveness: [[1.0; 19]; 19],
            generation,
            special_cases: HashMap::new(),
        };
        
        // Use hardcoded effectiveness matrix (simplified after removing service layer)
        chart.initialize_effectiveness_matrix();
        
        chart.add_special_cases();
        
        // Type effectiveness validation complete
        
        chart
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
                if let Some(&override_multiplier) = self.special_cases.get(&(crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(move_name)).unwrap_or(Moves::NONE), *target_type)) {
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
        self.special_cases.insert((Moves::FREEZEDRY, PokemonType::Water), 2.0);
        
        // Flying Press is Fighting-type but hits like Fighting + Flying
        // This is handled in move-specific logic, not here
        
        // Thousand Arrows hits Flying types for neutral damage despite being Ground
        self.special_cases.insert((Moves::THOUSANDARROWS, PokemonType::Flying), 1.0);
    }
}

/// Default type chart for the current generation (Gen 9)
impl Default for TypeChart {
    fn default() -> Self {
        Self::new_uncached(9)
    }
}

