//! # Damage Roll System
//!
//! This module implements Pokemon's damage roll system, which applies a random
//! multiplier to final damage calculations. The system uses 16 discrete damage
//! rolls ranging from 85% to 100% of base damage.

use crate::constants::damage::*;
use super::utils::poke_round;

/// DamageRolls enum for consistent damage calculation
/// Matches Pokemon's actual 16-roll system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageRolls {
    Average, // Uses the average of the 8th and 9th damage values (true median)
    Min,     // Uses the minimum damage roll (85%)
    Max,     // Uses the maximum damage roll (100%)
    All,     // Returns all 16 possible damage values
}

impl DamageRolls {
    /// Convert DamageRolls enum to damage multiplier (legacy)
    pub fn as_multiplier(self) -> f32 {
        match self {
            DamageRolls::Average => 0.925, // Keep for backwards compatibility
            DamageRolls::Min => 0.85,
            DamageRolls::Max => 1.0,
            DamageRolls::All => 0.925, // Default to average for All
        }
    }
}

/// Calculate all 16 possible damage rolls for a base damage value.
///
/// Pokemon uses 16 discrete damage multipliers from 85% to 100% of base damage.
/// This function returns all possible damage values for analysis and comparison.
///
/// ## Parameters
/// - `base_damage`: The base damage before applying damage rolls
/// - `enforce_minimum`: Whether to enforce minimum 1 damage (typical for damaging moves)
///
/// ## Returns
/// A Vec<i16> containing all 16 possible damage values in ascending order.
pub fn calculate_all_damage_rolls(base_damage: i16, enforce_minimum: bool) -> Vec<i16> {
    let mut rolls = Vec::with_capacity(DAMAGE_ROLL_COUNT);
    
    for i in 0..DAMAGE_ROLL_COUNT {
        let multiplier = (MIN_DAMAGE_PERCENT + i * DAMAGE_ROLL_STEP) as f64 / 100.0;
        let damage = (base_damage as f64 * multiplier).floor() as i16;
        
        if enforce_minimum && damage == 0 && base_damage > 0 {
            rolls.push(1);
        } else {
            rolls.push(damage);
        }
    }
    
    rolls
}

/// Get specific damage for a DamageRolls variant.
///
/// This function calculates the damage for a specific roll type without
/// generating all possible rolls.
///
/// ## Parameters
/// - `base_damage`: The base damage before applying damage rolls
/// - `rolls`: Which damage roll variant to calculate
/// - `enforce_minimum`: Whether to enforce minimum 1 damage
///
/// ## Returns
/// The damage value for the specified roll type.
pub fn get_damage_for_roll(base_damage: i16, rolls: DamageRolls, enforce_minimum: bool) -> i16 {
    match rolls {
        DamageRolls::Min => {
            let damage = (base_damage as f64 * MIN_DAMAGE_PERCENT as f64 / 100.0).floor() as i16;
            if enforce_minimum && damage == 0 && base_damage > 0 { 1 } else { damage }
        },
        DamageRolls::Max => base_damage,
        DamageRolls::Average => {
            // Use true median (average of 8th and 9th values)
            let all_rolls = calculate_all_damage_rolls(base_damage, enforce_minimum);
            (all_rolls[7] + all_rolls[8]) / 2
        },
        DamageRolls::All => {
            // For compatibility, return average when All is requested in single value context
            get_damage_for_roll(base_damage, DamageRolls::Average, enforce_minimum)
        }
    }
}

/// Legacy random damage roll function (deprecated)
/// 
/// This function is maintained for backwards compatibility but should not be used
/// in new code. Use `get_damage_for_roll` or `calculate_all_damage_rolls` instead.
#[deprecated(note = "Use get_damage_for_roll or calculate_all_damage_rolls instead")]
pub fn random_damage_roll(base_damage: i16) -> i16 {
    get_damage_for_roll(base_damage, DamageRolls::Average, true)
}

/// Compare Pokemon's current health with damage multiples to determine kill potential.
///
/// This function analyzes all possible damage rolls to determine if a move can
/// guarantee a KO, potentially KO, or never KO based on current HP.
///
/// ## Parameters
/// - `current_hp`: Target Pokemon's current HP
/// - `base_damage`: Base damage before rolls
///
/// ## Returns
/// - `guaranteed_ko`: True if all damage rolls result in KO
/// - `potential_ko`: True if some damage rolls result in KO
/// - `max_damage`: Highest possible damage roll
/// - `min_damage`: Lowest possible damage roll
pub fn compare_health_with_damage_multiples(
    current_hp: i16, 
    base_damage: i16
) -> (bool, bool, i16, i16) {
    let all_damages = calculate_all_damage_rolls(base_damage, true);
    let min_damage = all_damages[0];
    let max_damage = all_damages[DAMAGE_ROLL_COUNT - 1];
    
    let guaranteed_ko = min_damage >= current_hp;
    let potential_ko = max_damage >= current_hp;
    
    (guaranteed_ko, potential_ko, max_damage, min_damage)
}

/// Calculate final damage with modern Pokemon rounding (Gen 7+)
///
/// This function applies the damage roll and Pokemon-specific rounding used
/// in modern generations.
///
/// ## Parameters
/// - `base_damage`: Base damage before roll application
/// - `damage_rolls`: Which damage roll to apply
///
/// ## Returns
/// Final damage value with proper rounding applied
pub fn calculate_final_damage_roll(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    match damage_rolls {
        DamageRolls::Min => poke_round(base_damage * MIN_DAMAGE_PERCENT as f64 / 100.0),
        DamageRolls::Max => poke_round(base_damage),
        DamageRolls::Average => {
            // Calculate true median of all possible rolls
            let all_rolls: Vec<i16> = (0..DAMAGE_ROLL_COUNT)
                .map(|i| {
                    let multiplier = (MIN_DAMAGE_PERCENT + i * DAMAGE_ROLL_STEP) as f64 / 100.0;
                    poke_round(base_damage * multiplier)
                })
                .collect();
            (all_rolls[7] + all_rolls[8]) / 2
        },
        DamageRolls::All => {
            // Return average for single value context
            calculate_final_damage_roll(base_damage, DamageRolls::Average)
        }
    }
}

/// Calculate final damage for Generation 1 and 2
///
/// Generations 1 and 2 use a different damage roll system with a 217-255 range
/// instead of the modern 85-100% system.
///
/// ## Parameters
/// - `base_damage`: Base damage before roll application
/// - `damage_rolls`: Which damage roll to apply
///
/// ## Returns
/// Final damage value using Gen 1/2 mechanics
pub fn calculate_final_damage_gen12(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    let damage = base_damage as i16;
    
    match damage_rolls {
        DamageRolls::Min => ((damage * 217) / 255).max(1),
        DamageRolls::Max => damage,
        DamageRolls::Average => ((damage * 236) / 255).max(1), // Average of 217-255
        DamageRolls::All => ((damage * 236) / 255).max(1), // Default to average
    }
}

/// Calculate final damage for Generation 3
///
/// Generation 3 uses floor operations at specific steps in the calculation.
///
/// ## Parameters
/// - `base_damage`: Base damage before roll application
/// - `damage_rolls`: Which damage roll to apply
///
/// ## Returns
/// Final damage value using Gen 3 mechanics
pub fn calculate_final_damage_gen3(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    let damage = base_damage.floor();
    
    match damage_rolls {
        DamageRolls::Min => (damage * MIN_DAMAGE_PERCENT as f64 / 100.0).floor() as i16,
        DamageRolls::Max => damage as i16,
        DamageRolls::Average => (damage * 92.5 / 100.0).floor() as i16, // Average of 85-100
        DamageRolls::All => (damage * 92.5 / 100.0).floor() as i16, // Default to average
    }
}

/// Calculate final damage for Generation 4
///
/// Generation 4 applies floor operations at each step of the damage calculation.
///
/// ## Parameters
/// - `base_damage`: Base damage before roll application
/// - `damage_rolls`: Which damage roll to apply
///
/// ## Returns
/// Final damage value using Gen 4 mechanics
pub fn calculate_final_damage_gen4(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    let damage = base_damage.floor();
    
    match damage_rolls {
        DamageRolls::Min => (damage * MIN_DAMAGE_PERCENT as f64 / 100.0).floor() as i16,
        DamageRolls::Max => damage as i16,
        DamageRolls::Average => (damage * 92.5 / 100.0).floor() as i16,
        DamageRolls::All => (damage * 92.5 / 100.0).floor() as i16,
    }
}

/// Calculate final damage for Generation 5 and 6
///
/// Generations 5 and 6 use the modern damage roll system but without Pokemon-specific rounding.
///
/// ## Parameters
/// - `base_damage`: Base damage before roll application
/// - `damage_rolls`: Which damage roll to apply
///
/// ## Returns
/// Final damage value using Gen 5/6 mechanics
pub fn calculate_final_damage_gen56(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    match damage_rolls {
        DamageRolls::Min => (base_damage * MIN_DAMAGE_PERCENT as f64 / 100.0).floor() as i16,
        DamageRolls::Max => base_damage.floor() as i16,
        DamageRolls::Average => (base_damage * 92.5 / 100.0).floor() as i16,
        DamageRolls::All => (base_damage * 92.5 / 100.0).floor() as i16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_rolls_multipliers() {
        assert_eq!(DamageRolls::Min.as_multiplier(), 0.85);
        assert_eq!(DamageRolls::Max.as_multiplier(), 1.0);
        assert_eq!(DamageRolls::Average.as_multiplier(), 0.925);
    }

    #[test]
    fn test_calculate_all_damage_rolls() {
        let rolls = calculate_all_damage_rolls(100, true);
        assert_eq!(rolls.len(), 16);
        assert_eq!(rolls[0], 85);   // 85% of 100
        assert_eq!(rolls[15], 100); // 100% of 100
    }

    #[test]
    fn test_minimum_damage_enforcement() {
        let rolls = calculate_all_damage_rolls(1, true);
        assert!(rolls.iter().all(|&damage| damage >= 1));
        
        let rolls_no_min = calculate_all_damage_rolls(1, false);
        assert!(rolls_no_min[0] == 0); // 85% of 1 floors to 0
    }

    #[test]
    fn test_get_damage_for_roll() {
        assert_eq!(get_damage_for_roll(100, DamageRolls::Min, true), 85);
        assert_eq!(get_damage_for_roll(100, DamageRolls::Max, true), 100);
        
        // Test minimum enforcement
        assert_eq!(get_damage_for_roll(1, DamageRolls::Min, true), 1);
        assert_eq!(get_damage_for_roll(1, DamageRolls::Min, false), 0);
    }

    #[test]
    fn test_compare_health_with_damage() {
        let (guaranteed, potential, max_dmg, min_dmg) = 
            compare_health_with_damage_multiples(90, 100);
        
        assert!(!guaranteed); // Min damage (85) doesn't KO
        assert!(potential);   // Max damage (100) does KO
        assert_eq!(max_dmg, 100);
        assert_eq!(min_dmg, 85);
    }

    #[test]
    fn test_final_damage_roll_modern() {
        assert_eq!(calculate_final_damage_roll(100.0, DamageRolls::Min), 85);
        assert_eq!(calculate_final_damage_roll(100.0, DamageRolls::Max), 100);
    }

    #[test]
    fn test_generation_specific_damage_rolls() {
        // Test Gen 1/2 mechanics
        assert_eq!(calculate_final_damage_gen12(100.0, DamageRolls::Min), 85);
        
        // Test Gen 3 mechanics
        assert_eq!(calculate_final_damage_gen3(100.0, DamageRolls::Min), 85);
        
        // Test Gen 4 mechanics
        assert_eq!(calculate_final_damage_gen4(100.0, DamageRolls::Min), 85);
        
        // Test Gen 5/6 mechanics
        assert_eq!(calculate_final_damage_gen56(100.0, DamageRolls::Min), 85);
    }
}