//! # Critical Hit System
//!
//! This module implements Pokemon's critical hit mechanics across all generations.
//! Critical hits multiply damage and have different probability calculations
//! depending on the generation.

use crate::core::battle_state::pokemon_state::Pokemon;
use crate::data::showdown_types::MoveData;
use crate::generation::GenerationBattleMechanics;
use crate::utils::normalize_name;

/// Calculate critical hit probability for a Pokemon using a move
///
/// This function handles generation-specific critical hit mechanics and returns
/// the probability as a value between 0.0 and 1.0.
///
/// ## Parameters
/// - `pokemon`: The attacking Pokemon
/// - `move_data`: Data for the move being used
/// - `generation`: The generation mechanics to use
///
/// ## Returns
/// Critical hit probability as a f64 between 0.0 and 1.0
pub fn critical_hit_probability(
    pokemon: &Pokemon,
    move_data: &MoveData,
    generation: &dyn GenerationBattleMechanics,
) -> f64 {
    match generation.generation_number() {
        1 => critical_hit_probability_gen1(pokemon, move_data),
        2 => critical_hit_probability_gen2(pokemon, move_data),
        _ => {
            // Gen 3+ use the stage-based system
            let mut stage = 0i8;
            
            // Check for high critical hit ratio moves
            let move_name = normalize_name(&move_data.name);
            if is_high_crit_move(&move_name, generation.generation_number()) {
                stage += 1;
            }
            
            // Check for guaranteed critical hit moves
            if is_guaranteed_crit_move(&move_name) {
                return 1.0;
            }
            
            // Apply ability modifiers
            if let Some(ref ability) = pokemon.ability {
                let ability_name = normalize_name(&ability.name);
                match ability_name.as_str() {
                    "superluck" => stage += 1,
                    "battlearmor" | "shellarmor" => return 0.0, // No critical hits
                    _ => {}
                }
            }
            
            // Apply item modifiers
            if let Some(ref item) = pokemon.item {
                let item_name = normalize_name(&item.name);
                match item_name.as_str() {
                    "scopelens" | "razorclaw" => stage += 1,
                    "luckypunch" if normalize_name(&pokemon.species_id) == "chansey" => stage += 2,
                    "leek" | "stick" if normalize_name(&pokemon.species_id) == "farfetchd" => stage += 2,
                    _ => {}
                }
            }
            
            calculate_crit_rate_from_stage(stage, generation.generation_number())
        }
    }
}

/// Calculate critical hit probability for Generation 1
///
/// Gen 1 uses a unique system based on the Pokemon's base Speed stat.
/// Formula: floor(base_speed / 2) / 256
///
/// ## Parameters
/// - `pokemon`: The attacking Pokemon
/// - `move_data`: Data for the move being used
///
/// ## Returns
/// Critical hit probability for Gen 1
fn critical_hit_probability_gen1(pokemon: &Pokemon, move_data: &MoveData) -> f64 {
    let base_speed = get_base_speed_for_pokemon(&pokemon.species_id);
    let mut threshold = base_speed / 2;
    
    // High critical hit ratio moves get +76 threshold
    let move_name = normalize_name(&move_data.name);
    if is_high_crit_move(&move_name, 1) {
        threshold += 76;
    }
    
    // Focus Energy actually quarters the critical hit rate in Gen 1 (bug)
    if pokemon.volatile_statuses.contains("focusenergy") {
        threshold /= 4;
    }
    
    (threshold as f64 / 256.0).min(1.0)
}

/// Calculate critical hit probability for Generation 2
///
/// Gen 2 uses a fixed stage system with base rate 17/256.
///
/// ## Parameters
/// - `pokemon`: The attacking Pokemon
/// - `move_data`: Data for the move being used
///
/// ## Returns
/// Critical hit probability for Gen 2
fn critical_hit_probability_gen2(pokemon: &Pokemon, move_data: &MoveData) -> f64 {
    let mut stage = 0i8;
    
    // Check for high critical hit ratio moves
    let move_name = normalize_name(&move_data.name);
    if is_high_crit_move(&move_name, 2) {
        stage += 1;
    }
    
    // Focus Energy adds +1 stage in Gen 2+
    if pokemon.volatile_statuses.contains("focusenergy") {
        stage += 1;
    }
    
    // Gen 2 rates: 17/256, 32/256, 64/256, 85/256
    let threshold = match stage {
        0 => 17,
        1 => 32,
        2 => 64,
        _ => 85, // Stage 3+
    };
    
    threshold as f64 / 256.0
}

/// Convert critical hit stage to probability for Gen 3+
///
/// ## Parameters
/// - `stage`: Critical hit stage (can be negative)
/// - `generation`: Generation number for specific mechanics
///
/// ## Returns
/// Critical hit probability based on stage
fn calculate_crit_rate_from_stage(stage: i8, generation: u8) -> f64 {
    match generation {
        3..=5 => {
            // Gen 3-5: Base rate 1/16 (6.25%)
            let rate = match stage {
                i8::MIN..=-1 => 0,  // Negative stages = no crits
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 6,
                _ => 8, // Stage 4+
            };
            rate as f64 / 16.0
        },
        6 => {
            // Gen 6: Modified rates
            let rate = match stage {
                i8::MIN..=-1 => 0,
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                _ => 12, // Stage 4+
            };
            rate as f64 / 16.0
        },
        _ => {
            // Gen 7+: Reduced base rate 1/24 (~4.17%)
            let rate = match stage {
                i8::MIN..=-1 => 0,
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                _ => 12, // Stage 4+
            };
            rate as f64 / 24.0
        }
    }
}

/// Get base Speed stat for a Pokemon species (used for Gen 1 crits)
///
/// This is a simplified lookup - in a full implementation, this would
/// query the species data repository.
///
/// ## Parameters
/// - `species_id`: The Pokemon's species identifier
///
/// ## Returns
/// Base Speed stat for the species
fn get_base_speed_for_pokemon(species_id: &str) -> u16 {
    // Simplified lookup table for common Pokemon
    // In production, this would query the actual species data
    match normalize_name(species_id).as_str() {
        "pikachu" => 90,
        "charizard" => 100,
        "blastoise" => 78,
        "venusaur" => 80,
        "mewtwo" => 130,
        "mew" => 100,
        "electrode" => 150,
        "ninjask" => 160,
        "crobat" => 130,
        _ => 65, // Default average speed
    }
}

/// Check if a move has high critical hit ratio for a given generation
///
/// ## Parameters
/// - `move_name`: Normalized move name
/// - `generation`: Generation number
///
/// ## Returns
/// True if the move has high critical hit ratio
fn is_high_crit_move(move_name: &str, generation: u8) -> bool {
    match generation {
        1 => matches!(move_name, 
            "karatechop" | "razorleaf" | "crabhammer" | "slash"
        ),
        2..=4 => matches!(move_name,
            "karatechop" | "razorleaf" | "crabhammer" | "slash" | 
            "aerialace" | "blaze kick" | "cross chop" | "leaf blade" |
            "night slash" | "psycho cut" | "razor wind" | "sky attack"
        ),
        5..=6 => matches!(move_name,
            "karatechop" | "razorleaf" | "crabhammer" | "slash" |
            "aerialace" | "air cutter" | "attack order" | "blaze kick" |
            "cross chop" | "leaf blade" | "night slash" | "psycho cut" |
            "razor wind" | "shadow claw" | "sky attack" | "spacial rend" |
            "stone edge"
        ),
        _ => matches!(move_name,
            "karatechop" | "razorleaf" | "crabhammer" | "slash" |
            "aerialace" | "air cutter" | "attack order" | "blaze kick" |
            "cross chop" | "drill run" | "leaf blade" | "night slash" |
            "psycho cut" | "razor wind" | "shadow claw" | "sky attack" |
            "spacial rend" | "stone edge" | "storm throw" | "frost breath"
        ),
    }
}

/// Check if a move always results in a critical hit
///
/// ## Parameters
/// - `move_name`: Normalized move name
///
/// ## Returns
/// True if the move always crits
fn is_guaranteed_crit_move(move_name: &str) -> bool {
    matches!(move_name,
        "frost breath" | "storm throw" | "wicked blow" | "surging strikes"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::pokemon_state::Pokemon;
    use crate::data::showdown_types::{MoveData, AbilityData, ItemData};
    use crate::generation::Generation;

    fn create_test_pokemon() -> Pokemon {
        Pokemon {
            species_id: "pikachu".to_string(),
            ability: None,
            item: None,
            volatile_statuses: std::collections::HashSet::new(),
            // ... other required fields would be filled with defaults
            ..Default::default()
        }
    }

    fn create_test_move(name: &str) -> MoveData {
        MoveData {
            name: name.to_string(),
            // ... other required fields would be filled with defaults
            ..Default::default()
        }
    }

    #[test]
    fn test_base_speed_lookup() {
        assert_eq!(get_base_speed_for_pokemon("pikachu"), 90);
        assert_eq!(get_base_speed_for_pokemon("electrode"), 150);
        assert_eq!(get_base_speed_for_pokemon("unknown"), 65); // Default
    }

    #[test]
    fn test_high_crit_moves() {
        assert!(is_high_crit_move("slash", 1));
        assert!(is_high_crit_move("karatechop", 1));
        assert!(!is_high_crit_move("tackle", 1));
        
        assert!(is_high_crit_move("stone edge", 7));
        assert!(!is_high_crit_move("earthquake", 7));
    }

    #[test]
    fn test_guaranteed_crit_moves() {
        assert!(is_guaranteed_crit_move("frost breath"));
        assert!(is_guaranteed_crit_move("storm throw"));
        assert!(!is_guaranteed_crit_move("ice beam"));
    }

    #[test]
    fn test_crit_stage_conversion_gen3() {
        assert_eq!(calculate_crit_rate_from_stage(0, 3), 1.0 / 16.0);
        assert_eq!(calculate_crit_rate_from_stage(1, 3), 2.0 / 16.0);
        assert_eq!(calculate_crit_rate_from_stage(2, 3), 4.0 / 16.0);
        assert_eq!(calculate_crit_rate_from_stage(-1, 3), 0.0);
    }

    #[test]
    fn test_crit_stage_conversion_gen6() {
        assert_eq!(calculate_crit_rate_from_stage(0, 6), 1.0 / 16.0);
        assert_eq!(calculate_crit_rate_from_stage(3, 6), 8.0 / 16.0);
        assert_eq!(calculate_crit_rate_from_stage(4, 6), 12.0 / 16.0);
    }

    #[test]
    fn test_crit_stage_conversion_gen7() {
        assert_eq!(calculate_crit_rate_from_stage(0, 7), 1.0 / 24.0);
        assert_eq!(calculate_crit_rate_from_stage(1, 7), 2.0 / 24.0);
        assert_eq!(calculate_crit_rate_from_stage(2, 7), 4.0 / 24.0);
    }

    #[test]
    fn test_gen1_crit_calculation() {
        let pokemon = create_test_pokemon();
        let normal_move = create_test_move("tackle");
        let high_crit_move = create_test_move("slash");
        
        let normal_rate = critical_hit_probability_gen1(&pokemon, &normal_move);
        let high_crit_rate = critical_hit_probability_gen1(&pokemon, &high_crit_move);
        
        assert!(high_crit_rate > normal_rate);
        assert!(normal_rate >= 0.0 && normal_rate <= 1.0);
        assert!(high_crit_rate >= 0.0 && high_crit_rate <= 1.0);
    }

    #[test]
    fn test_gen2_crit_calculation() {
        let pokemon = create_test_pokemon();
        let normal_move = create_test_move("tackle");
        let high_crit_move = create_test_move("slash");
        
        let normal_rate = critical_hit_probability_gen2(&pokemon, &normal_move);
        let high_crit_rate = critical_hit_probability_gen2(&pokemon, &high_crit_move);
        
        assert_eq!(normal_rate, 17.0 / 256.0);
        assert_eq!(high_crit_rate, 32.0 / 256.0);
    }

    #[test]
    fn test_guaranteed_crit_detection() {
        let pokemon = create_test_pokemon();
        let frost_breath = create_test_move("frost breath");
        let generation = Generation::gen9();
        
        let rate = critical_hit_probability(&pokemon, &frost_breath, &generation);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_ability_modifiers() {
        let mut pokemon = create_test_pokemon();
        pokemon.ability = Some(AbilityData {
            name: "Super Luck".to_string(),
            ..Default::default()
        });
        
        let move_data = create_test_move("tackle");
        let generation = Generation::gen9();
        
        let rate = critical_hit_probability(&pokemon, &move_data, &generation);
        assert!(rate > 1.0 / 24.0); // Should be higher than base rate
    }

    #[test]
    fn test_item_modifiers() {
        let mut pokemon = create_test_pokemon();
        pokemon.item = Some(ItemData {
            name: "Scope Lens".to_string(),
            ..Default::default()
        });
        
        let move_data = create_test_move("tackle");
        let generation = Generation::gen9();
        
        let rate = critical_hit_probability(&pokemon, &move_data, &generation);
        assert!(rate > 1.0 / 24.0); // Should be higher than base rate
    }
}