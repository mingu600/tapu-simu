//! # Generation 1 Damage Mechanics
//!
//! Generation 1 has unique damage calculation mechanics that differ significantly
//! from later generations, including different critical hit handling and damage formulas.

use crate::core::battle_state::{BattleState};
use crate::core::battle_state::pokemon_state::{Pokemon, Move};
use crate::data::showdown_types::MoveData;
use crate::engine::combat::damage::damage_rolls::DamageRolls;
use crate::engine::combat::damage::modifiers::*;
use crate::engine::combat::type_effectiveness::get_type_effectiveness;
use crate::generation::GenerationMechanics;

/// Generation 1 specific constants
pub mod constants {
    pub const DAMAGE_ROLL_MIN: u16 = 217;
    pub const DAMAGE_ROLL_MAX: u16 = 255;
    pub const DAMAGE_ROLL_AVERAGE: u16 = 236;
    
    pub const CRIT_LEVEL_MULTIPLIER: u16 = 2;
    pub const BASE_DAMAGE_OFFSET: u16 = 2;
    pub const MAX_BASE_DAMAGE: u16 = 997;
}

/// Calculate damage for Generation 1
///
/// Gen 1 has unique mechanics including:
/// - Critical hits double the level before damage calculation
/// - Different damage roll system (217-255 range)
/// - Special attack/defense handling
/// - Type effectiveness applied differently
///
/// ## Parameters
/// - `state`: Current battle state
/// - `attacker`: The attacking Pokemon
/// - `defender`: The defending Pokemon
/// - `move_data`: Move data including base power and type
/// - `is_critical`: Whether this is a critical hit
/// - `damage_rolls`: Which damage roll to use
///
/// ## Returns
/// Final damage value using Gen 1 mechanics
pub fn calculate_damage_gen1(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
) -> i16 {
    // Gen 1 uses level 50 for most calculations
    let level = attacker.level as u16;
    let base_power = move_data.base_power as u16;
    
    // Skip calculation for non-damaging moves
    if base_power == 0 {
        return 0;
    }
    
    // Get attack and defense stats
    let (attack, defense) = get_gen1_attack_defense_stats(
        attacker, 
        defender, 
        &move_data.category, 
        is_critical
    );
    
    // Calculate base damage using Gen 1 formula
    let effective_level = if is_critical {
        level * constants::CRIT_LEVEL_MULTIPLIER // Critical hits double the level
    } else {
        level
    };
    
    // Gen 1 damage formula: floor(floor((floor((2*Level)/5+2) * max(1,Attack) * BP) / max(1,Defense)) / 50) + 2
    let level_factor = (2 * effective_level) / 5 + 2;
    let numerator = level_factor * attack.max(1) * base_power;
    let base_damage = (numerator / defense.max(1)) / 50;
    let base_damage = (base_damage + constants::BASE_DAMAGE_OFFSET).min(constants::MAX_BASE_DAMAGE);
    
    // Apply STAB
    let stab_modifier = get_stab_modifier(attacker, &move_data.move_type);
    let mut damage = (base_damage as f64 * stab_modifier).floor() as u16;
    
    // Apply type effectiveness (Gen 1 applies each type separately)
    let type_effectiveness = get_type_effectiveness_gen1(&move_data.move_type, &defender.types);
    for effectiveness in type_effectiveness {
        damage = ((damage as f64) * effectiveness).floor() as u16;
    }
    
    // Apply weather effects
    let weather_modifier = get_weather_damage_modifier(state, &move_data.move_type, attacker);
    damage = ((damage as f64) * weather_modifier).floor() as u16;
    
    // Apply damage roll
    calculate_final_damage_gen1(damage as f64, damage_rolls)
}

/// Get attack and defense stats for Gen 1 calculation
///
/// Gen 1 has special handling for Special moves and critical hits.
fn get_gen1_attack_defense_stats(
    attacker: &Pokemon,
    defender: &Pokemon,
    move_category: &str,
    is_critical: bool,
) -> (u16, u16) {
    let (attack_stat, defense_stat) = match move_category {
        "Physical" => ("attack", "defense"),
        "Special" => ("special", "special"), // Gen 1 only had one Special stat
        _ => ("attack", "defense"), // Default for status moves (though they shouldn't reach here)
    };
    
    let attack = if is_critical {
        // Critical hits use raw stats (ignore stat changes)
        get_raw_stat(attacker, attack_stat)
    } else {
        get_effective_stat(attacker, attack_stat)
    };
    
    let defense = if is_critical {
        // Critical hits use raw stats (ignore stat changes)
        get_raw_stat(defender, defense_stat)
    } else {
        get_effective_stat(defender, defense_stat)
    };
    
    (attack, defense)
}

/// Get raw stat value without modifications (for critical hits)
fn get_raw_stat(pokemon: &Pokemon, stat_name: &str) -> u16 {
    match stat_name {
        "attack" => pokemon.stats.attack as u16,
        "defense" => pokemon.stats.defense as u16,
        "special" => pokemon.stats.special_attack as u16, // Gen 1 Special
        "speed" => pokemon.stats.speed as u16,
        _ => 100, // Default value
    }
}

/// Get effective stat value with modifications
fn get_effective_stat(pokemon: &Pokemon, stat_name: &str) -> u16 {
    let base_stat = get_raw_stat(pokemon, stat_name);
    
    // Apply stat boosts/drops
    let boost = pokemon.stat_boosts.get(stat_name).unwrap_or(&0);
    let multiplier = get_stat_boost_multiplier(*boost);
    
    (base_stat as f64 * multiplier).floor() as u16
}

/// Get stat boost multiplier for Gen 1
fn get_stat_boost_multiplier(boost: i8) -> f64 {
    match boost {
        -6 => 0.25,
        -5 => 0.28,
        -4 => 0.33,
        -3 => 0.4,
        -2 => 0.5,
        -1 => 0.67,
        0 => 1.0,
        1 => 1.5,
        2 => 2.0,
        3 => 2.5,
        4 => 3.0,
        5 => 3.5,
        6 => 4.0,
        _ => if boost > 6 { 4.0 } else { 0.25 },
    }
}

/// Get type effectiveness for Gen 1 (returns individual multipliers)
fn get_type_effectiveness_gen1(move_type: &str, defender_types: &[crate::engine::combat::type_effectiveness::PokemonType]) -> Vec<f64> {
    // Gen 1 applies type effectiveness for each defending type separately
    defender_types.iter()
        .map(|def_type| {
            // Simplified type effectiveness for Gen 1
            // This would use the actual Gen 1 type chart in a full implementation
            match (move_type, def_type.as_str()) {
                ("Electric", "Flying") => 2.0,
                ("Electric", "Water") => 2.0,
                ("Electric", "Ground") => 0.0,
                ("Water", "Fire") => 2.0,
                ("Water", "Rock") => 2.0,
                ("Water", "Ground") => 2.0,
                ("Water", "Water") => 0.5,
                ("Water", "Grass") => 0.5,
                ("Water", "Dragon") => 0.5,
                ("Fire", "Grass") => 2.0,
                ("Fire", "Ice") => 2.0,
                ("Fire", "Bug") => 2.0,
                ("Fire", "Steel") => 2.0,
                ("Fire", "Fire") => 0.5,
                ("Fire", "Water") => 0.5,
                ("Fire", "Rock") => 0.5,
                ("Fire", "Dragon") => 0.5,
                // Add more type matchups as needed
                _ => 1.0,
            }
        })
        .collect()
}

/// Calculate final damage with Gen 1 damage roll system
///
/// Gen 1 uses a 217-255 range instead of the modern 85-100% system.
pub fn calculate_final_damage_gen1(base_damage: f64, damage_rolls: DamageRolls) -> i16 {
    let damage = base_damage as u16;
    
    // Special case: if damage is 1, skip random factor
    if damage == 1 {
        return 1;
    }
    
    let result = match damage_rolls {
        DamageRolls::Min => (damage * constants::DAMAGE_ROLL_MIN) / constants::DAMAGE_ROLL_MAX,
        DamageRolls::Max => damage,
        DamageRolls::Average => (damage * constants::DAMAGE_ROLL_AVERAGE) / constants::DAMAGE_ROLL_MAX,
        DamageRolls::All => (damage * constants::DAMAGE_ROLL_AVERAGE) / constants::DAMAGE_ROLL_MAX, // Default to average
    };
    
    result.max(1) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::Pokemon;
    use crate::engine::combat::damage::damage_rolls::DamageRolls;

    #[test]
    fn test_gen1_damage_roll_calculation() {
        assert_eq!(calculate_final_damage_gen1(100.0, DamageRolls::Min), 85);
        assert_eq!(calculate_final_damage_gen1(100.0, DamageRolls::Max), 100);
        assert_eq!(calculate_final_damage_gen1(100.0, DamageRolls::Average), 92);
    }

    #[test]
    fn test_gen1_damage_roll_special_case() {
        // When base damage is 1, should always return 1
        assert_eq!(calculate_final_damage_gen1(1.0, DamageRolls::Min), 1);
        assert_eq!(calculate_final_damage_gen1(1.0, DamageRolls::Max), 1);
        assert_eq!(calculate_final_damage_gen1(1.0, DamageRolls::Average), 1);
    }

    #[test]
    fn test_stat_boost_multipliers() {
        assert_eq!(get_stat_boost_multiplier(0), 1.0);
        assert_eq!(get_stat_boost_multiplier(1), 1.5);
        assert_eq!(get_stat_boost_multiplier(-1), 0.67);
        assert_eq!(get_stat_boost_multiplier(6), 4.0);
        assert_eq!(get_stat_boost_multiplier(-6), 0.25);
    }

    #[test]
    fn test_gen1_constants() {
        assert_eq!(constants::DAMAGE_ROLL_MIN, 217);
        assert_eq!(constants::DAMAGE_ROLL_MAX, 255);
        assert_eq!(constants::DAMAGE_ROLL_AVERAGE, 236);
        assert_eq!(constants::CRIT_LEVEL_MULTIPLIER, 2);
    }
}