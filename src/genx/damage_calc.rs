//! # Damage Calculation
//! 
//! This module provides damage calculation for Pokemon moves.

use crate::state::Pokemon;
use crate::data::types::EngineMoveData;

/// Calculate damage for a move
pub fn calculate_damage(
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
) -> i16 {
    // Basic damage calculation formula
    let level = attacker.level as f32;
    let power = move_data.base_power.unwrap_or(0) as f32;
    
    if power == 0.0 {
        return 0; // Status moves don't deal damage
    }

    let attack_stat = match move_data.category {
        crate::state::MoveCategory::Physical => attacker.get_effective_stat(crate::instruction::Stat::Attack),
        crate::state::MoveCategory::Special => attacker.get_effective_stat(crate::instruction::Stat::SpecialAttack),
        crate::state::MoveCategory::Status => return 0,
    } as f32;

    let defense_stat = match move_data.category {
        crate::state::MoveCategory::Physical => defender.get_effective_stat(crate::instruction::Stat::Defense),
        crate::state::MoveCategory::Special => defender.get_effective_stat(crate::instruction::Stat::SpecialDefense),
        crate::state::MoveCategory::Status => return 0,
    } as f32;

    // Base damage calculation
    let base_damage = ((((2.0 * level / 5.0 + 2.0) * power * attack_stat / defense_stat) / 50.0) + 2.0);
    
    // Apply modifiers
    let mut damage = base_damage;
    
    // Critical hit multiplier
    if is_critical {
        damage *= 1.5;
    }
    
    // Damage roll (0.85 to 1.0)
    damage *= damage_roll;
    
    // TODO: Add more modifiers (STAB, type effectiveness, weather, abilities, items, etc.)
    
    damage as i16
}

/// Generate a random damage roll
pub fn random_damage_roll() -> f32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0.85..=1.0)
}

/// Calculate critical hit probability
pub fn critical_hit_probability(_attacker: &Pokemon, _move_data: &EngineMoveData) -> f32 {
    // Base critical hit rate is 1/24 (about 4.17%)
    let mut crit_rate: f32 = 1.0 / 24.0;
    
    // TODO: Add modifiers for high crit ratio moves, abilities, items, etc.
    
    // Cap at 50% (1/2)
    crit_rate.min(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Pokemon, MoveCategory};
    use crate::data::types::EngineMoveData;

    fn create_test_pokemon() -> Pokemon {
        let mut pokemon = Pokemon::new("Test".to_string());
        pokemon.stats.attack = 100;
        pokemon.stats.defense = 100;
        pokemon.stats.special_attack = 100;
        pokemon.stats.special_defense = 100;
        pokemon
    }

    fn create_test_move() -> EngineMoveData {
        EngineMoveData {
            id: 1,
            name: "Test Move".to_string(),
            base_power: Some(80),
            accuracy: Some(100),
            pp: 10,
            move_type: "Normal".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::types::MoveTarget::SpecificMove,
            effect_chance: None,
            effect_description: String::new(),
            flags: vec![],
        }
    }

    #[test]
    fn test_basic_damage_calculation() {
        let attacker = create_test_pokemon();
        let defender = create_test_pokemon();
        let move_data = create_test_move();

        let damage = calculate_damage(&attacker, &defender, &move_data, false, 1.0);
        assert!(damage > 0);
    }

    #[test]
    fn test_critical_hit_damage() {
        let attacker = create_test_pokemon();
        let defender = create_test_pokemon();
        let move_data = create_test_move();

        let normal_damage = calculate_damage(&attacker, &defender, &move_data, false, 1.0);
        let crit_damage = calculate_damage(&attacker, &defender, &move_data, true, 1.0);
        
        assert!(crit_damage > normal_damage);
    }

    #[test]
    fn test_status_move_no_damage() {
        let attacker = create_test_pokemon();
        let defender = create_test_pokemon();
        let mut move_data = create_test_move();
        move_data.category = MoveCategory::Status;

        let damage = calculate_damage(&attacker, &defender, &move_data, false, 1.0);
        assert_eq!(damage, 0);
    }
}