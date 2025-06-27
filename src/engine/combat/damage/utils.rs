//! Shared utility functions for damage calculation
//!
//! This module contains utility functions used across the damage calculation
//! system, including Pokemon-specific rounding and final damage roll calculation.

use super::types::DamageRolls;

/// Pokemon's rounding function: rounds down at exactly 0.5
/// 
/// This matches the damage-calc implementation: 
/// `num % 1 > 0.5 ? Math.ceil(num) : Math.floor(num)`
/// 
/// Pokemon uses a specific rounding rule where exactly 0.5 rounds down,
/// which differs from standard mathematical rounding.
/// 
/// ## Parameters
/// - `num`: The number to round
/// 
/// ## Returns
/// The rounded number using Pokemon's rounding rules
pub fn poke_round(num: f32) -> f32 {
    let fractional_part = num - num.floor();
    if fractional_part > 0.5 {
        num.ceil()
    } else {
        num.floor()
    }
}

/// Calculate final damage roll following the exact sequence from damage-calc getFinalDamage
/// 
/// This matches: `getFinalDamage(baseAmount, i, effectiveness, isBurned, stabMod, finalMod, protect)`
/// 
/// The function applies damage modifiers in the exact order used by Pokemon:
/// 1. Apply damage roll (85-100%)
/// 2. Apply STAB (Same Type Attack Bonus)
/// 3. Apply type effectiveness with pokeRound then floor
/// 4. Apply burn status effect
/// 5. Apply final modifiers with pokeRound
/// 
/// ## Parameters
/// - `base_amount`: The base damage before final modifiers
/// - `damage_rolls`: Which damage roll variant to use
/// - `effectiveness`: Type effectiveness multiplier
/// - `is_burned`: Whether the attacker is burned (affects physical moves)
/// - `stab_mod`: STAB modifier as 4096-based value
/// - `final_mod`: Final modifier as 4096-based value
/// 
/// ## Returns
/// The final damage value after all modifiers
pub fn calculate_final_damage_roll(
    base_amount: f32,
    damage_rolls: DamageRolls,
    effectiveness: f32,
    is_burned: bool,
    stab_mod: u32,
    final_mod: u32,
) -> i16 {
    // Get the specific damage roll we want (0-15)
    let roll_index = match damage_rolls {
        DamageRolls::Min => 0,     // 85% roll
        DamageRolls::Max => 15,    // 100% roll 
        DamageRolls::Average => 7, // ~92% roll (index 7)
        DamageRolls::All => 7,     // Default to average
    };
    
    // Step 1: Apply damage roll (85 + i) / 100
    let mut damage_amount = (base_amount * (85.0 + roll_index as f32) / 100.0).floor();
    
    // Step 2: Apply STAB (if not 4096 to avoid unnecessary calculation)
    if stab_mod != 4096 {
        damage_amount = damage_amount * stab_mod as f32 / 4096.0;
    }
    
    // Step 3: Apply type effectiveness with pokeRound then floor
    damage_amount = (poke_round(damage_amount) * effectiveness).floor();
    
    // Step 4: Apply burn (floor division by 2)
    if is_burned {
        damage_amount = (damage_amount / 2.0).floor();
    }
    
    // Step 5: Apply final modifiers with pokeRound
    let final_damage = poke_round((damage_amount * final_mod as f32 / 4096.0).max(1.0));
    
    final_damage as i16
}

/// Calculate Gen 1/2 damage using the old 217-255 random range
/// 
/// Gen 1 and 2 use a different damage roll system compared to modern generations.
/// Instead of 85-100%, they use a 217-255 range applied differently.
/// 
/// ## Parameters
/// - `base_damage`: The base damage value
/// - `damage_rolls`: Which damage roll variant to use
/// - `generation`: The Pokemon generation (Gen 1 or Gen 2)
/// 
/// ## Returns
/// The damage value using the Gen 1/2 calculation method
pub fn calculate_final_damage_gen12(
    base_damage: f32,
    damage_rolls: DamageRolls,
    generation: crate::generation::Generation,
) -> i16 {
    // Gen 1/2 use range 217-255 instead of 85-100
    let roll_index = match damage_rolls {
        DamageRolls::Min => 217,     // Min roll
        DamageRolls::Max => 255,     // Max roll 
        DamageRolls::Average => 236, // Average roll
        DamageRolls::All => 236,     // Default to average
    };
    
    // Apply the Gen 1/2 damage roll formula
    let final_damage = (base_damage * roll_index as f32 / 255.0).floor().max(1.0);
    
    final_damage as i16
}

/// Calculate Gen 5/6 final damage without pokeRound
/// 
/// Gen 5 and 6 use a similar system to modern generations but without
/// the pokeRound function in the final step.
/// 
/// ## Parameters
/// - `base_amount`: The base damage before final modifiers
/// - `damage_rolls`: Which damage roll variant to use
/// - `effectiveness`: Type effectiveness multiplier
/// - `is_burned`: Whether the attacker is burned
/// - `stab_mod`: STAB modifier as 4096-based value
/// - `final_mod`: Final modifier as 4096-based value
/// 
/// ## Returns
/// The final damage value using Gen 5/6 calculation
pub fn calculate_final_damage_gen56(
    base_amount: f32,
    damage_rolls: DamageRolls,
    effectiveness: f32,
    is_burned: bool,
    stab_mod: u32,
    final_mod: u32,
) -> i16 {
    // Get the specific damage roll we want (0-15)
    let roll_index = match damage_rolls {
        DamageRolls::Min => 0,     // 85% roll
        DamageRolls::Max => 15,    // 100% roll 
        DamageRolls::Average => 7, // ~92% roll (index 7)
        DamageRolls::All => 7,     // Default to average
    };
    
    // Step 1: Apply damage roll (85 + i) / 100
    let mut damage_amount = (base_amount * (85.0 + roll_index as f32) / 100.0).floor();
    
    // Step 2: Apply STAB (if not 4096 to avoid unnecessary calculation)
    if stab_mod != 4096 {
        damage_amount = damage_amount * stab_mod as f32 / 4096.0;
    }
    
    // Step 3: Apply type effectiveness with pokeRound then floor
    damage_amount = (poke_round(damage_amount) * effectiveness).floor();
    
    // Step 4: Apply burn (floor division by 2)
    if is_burned {
        damage_amount = (damage_amount / 2.0).floor();
    }
    
    // Step 5: Apply final modifiers with floor (no pokeRound in Gen 5-6)
    let final_damage = (damage_amount * final_mod as f32 / 4096.0).max(1.0).floor();
    
    final_damage as i16
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poke_round() {
        // Test exact 0.5 rounds down
        assert_eq!(poke_round(2.5), 2.0);
        assert_eq!(poke_round(3.5), 3.0);
        
        // Test above 0.5 rounds up
        assert_eq!(poke_round(2.6), 3.0);
        assert_eq!(poke_round(3.7), 4.0);
        
        // Test below 0.5 rounds down
        assert_eq!(poke_round(2.4), 2.0);
        assert_eq!(poke_round(3.3), 3.0);
        
        // Test whole numbers
        assert_eq!(poke_round(5.0), 5.0);
        assert_eq!(poke_round(10.0), 10.0);
    }

    #[test]
    fn test_calculate_final_damage_roll() {
        let base_damage = 100.0;
        let effectiveness = 2.0; // Super effective
        let stab_mod = 6144; // 1.5x STAB as 4096-based
        let final_mod = 4096; // No additional modifiers
        
        let min_damage = calculate_final_damage_roll(
            base_damage,
            DamageRolls::Min,
            effectiveness,
            false,
            stab_mod,
            final_mod,
        );
        
        let max_damage = calculate_final_damage_roll(
            base_damage,
            DamageRolls::Max,
            effectiveness,
            false,
            stab_mod,
            final_mod,
        );
        
        // Max damage should be higher than min damage
        assert!(max_damage > min_damage);
        
        // Both should be positive
        assert!(min_damage > 0);
        assert!(max_damage > 0);
    }

    #[test]
    fn test_burn_effect() {
        let base_damage = 100.0;
        let effectiveness = 1.0;
        let stab_mod = 4096;
        let final_mod = 4096;
        
        let normal_damage = calculate_final_damage_roll(
            base_damage,
            DamageRolls::Average,
            effectiveness,
            false,
            stab_mod,
            final_mod,
        );
        
        let burned_damage = calculate_final_damage_roll(
            base_damage,
            DamageRolls::Average,
            effectiveness,
            true,
            stab_mod,
            final_mod,
        );
        
        // Burned damage should be less than normal damage
        assert!(burned_damage < normal_damage);
    }

}