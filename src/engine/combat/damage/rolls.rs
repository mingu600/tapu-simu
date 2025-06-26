//! Damage roll calculation system
//!
//! This module handles all aspects of Pokemon's 16-roll damage system,
//! including the calculation of all possible damage rolls and selection
//! of specific roll variants.

use crate::constants::moves::{DAMAGE_ROLL_COUNT, MIN_DAMAGE_PERCENT, MIN_DAMAGE, MAX_DAMAGE_ROLL_INDEX};

use super::types::DamageRolls;

/// Calculate all 16 possible damage rolls for a base damage value
/// 
/// This matches Pokemon's actual damage calculation: random number 0-15 added to 85%.
/// The damage formula applies a random multiplier from 85% to 100% in 16 discrete steps.
/// 
/// ## Parameters
/// - `base_damage_no_roll`: The base damage before applying the random roll
/// 
/// ## Returns
/// A vector of 16 damage values, one for each possible roll (85%-100%)
pub fn calculate_all_damage_rolls(base_damage_no_roll: f32) -> Vec<i16> {
    let mut damage_values = Vec::with_capacity(DAMAGE_ROLL_COUNT);

    // Generate all 16 possible damage rolls (85% + 0% through 85% + 15%)
    for roll in 0..DAMAGE_ROLL_COUNT {
        let multiplier = (MIN_DAMAGE_PERCENT + roll as u8) as f32 / 100.0;
        let damage_float = base_damage_no_roll * multiplier;
        // Pokemon uses floor for damage rolls, not rounding
        let damage = damage_float.floor() as i16;
        damage_values.push(damage.max(MIN_DAMAGE)); // Minimum 1 damage
    }

    damage_values
}

/// Get the specific damage value for a given DamageRolls variant
/// 
/// ## Parameters
/// - `base_damage_no_roll`: The base damage before applying the random roll
/// - `roll_type`: Which specific roll variant to calculate
/// 
/// ## Returns
/// The damage value for the specified roll type
pub fn get_damage_for_roll(base_damage_no_roll: f32, roll_type: DamageRolls) -> i16 {
    let all_rolls = calculate_all_damage_rolls(base_damage_no_roll);
    
    match roll_type {
        DamageRolls::Min => all_rolls[0],  // 85% roll
        DamageRolls::Max => all_rolls[MAX_DAMAGE_ROLL_INDEX], // 100% roll
        DamageRolls::Average => {
            // True median - average of 8th and 9th values (indices 7 and 8)
            let mid_low = all_rolls[7] as f32;
            let mid_high = all_rolls[8] as f32;
            ((mid_low + mid_high) / 2.0).round() as i16
        }
        DamageRolls::All => {
            // Return average when All is used in single-value context
            let mid_low = all_rolls[7] as f32;
            let mid_high = all_rolls[8] as f32;
            ((mid_low + mid_high) / 2.0).round() as i16
        }
    }
}

/// Compare health with damage multiples for KO calculations
/// 
/// This utility function helps determine how many hits are needed to KO
/// a Pokemon given a specific damage amount.
/// 
/// ## Parameters
/// - `max_damage`: The maximum damage that can be dealt
/// - `health`: The current health of the target Pokemon
/// 
/// ## Returns
/// A tuple of (hits_to_ko, guaranteed_damage):
/// - `hits_to_ko`: Number of hits needed to guarantee a KO
/// - `guaranteed_damage`: The guaranteed damage per hit
pub fn compare_health_with_damage_multiples(max_damage: i16, health: i16) -> (i16, i16) {
    if max_damage == 0 {
        return (0, 0);
    }
    
    let damage_to_ko = health;
    let hits_to_ko = (damage_to_ko as f32 / max_damage as f32).ceil() as i16;
    let guaranteed_damage = max_damage;
    
    (hits_to_ko, guaranteed_damage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_rolls_calculation() {
        let base_damage = 100.0;
        let rolls = calculate_all_damage_rolls(base_damage);
        
        // Should have exactly 16 rolls
        assert_eq!(rolls.len(), 16);
        
        // First roll should be 85% (minimum)
        assert_eq!(rolls[0], 85);
        
        // Last roll should be 100% (maximum)
        assert_eq!(rolls[15], 100);
        
        // All rolls should be at least 1
        for roll in rolls {
            assert!(roll >= 1);
        }
    }

    #[test]
    fn test_get_damage_for_roll() {
        let base_damage = 100.0;
        
        assert_eq!(get_damage_for_roll(base_damage, DamageRolls::Min), 85);
        assert_eq!(get_damage_for_roll(base_damage, DamageRolls::Max), 100);
        
        // Average should be between min and max
        let avg = get_damage_for_roll(base_damage, DamageRolls::Average);
        assert!(avg >= 85 && avg <= 100);
    }

    #[test]
    fn test_minimum_damage() {
        // Even with 0 base damage, should always deal at least 1
        let rolls = calculate_all_damage_rolls(0.0);
        for roll in rolls {
            assert_eq!(roll, 1);
        }
    }
}