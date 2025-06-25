//! # Core Damage Calculation
//!
//! This module provides the main entry points for damage calculation,
//! dispatching to generation-specific implementations as needed.

use crate::core::battle_state::{BattleState, Move, Gender, MoveCategory};
use crate::core::battle_state::pokemon_state::Pokemon;
use crate::core::battle_format::BattlePosition;
use crate::data::showdown_types::MoveData;
use crate::generation::{GenerationMechanics, GenerationBattleMechanics};
use super::damage_rolls::DamageRolls;
use super::generation_mechanics::*;
use super::critical_hits::critical_hit_probability;

/// Calculate damage between two Pokemon with explicit battle positions.
///
/// This is the primary damage calculation function that implements Pokemon's
/// damage formula with full generation support and format awareness.
///
/// ## Parameters
/// - `state`: The current battle state containing field conditions
/// - `attacker`: The Pokemon using the move
/// - `defender`: The Pokemon receiving the damage
/// - `move_data`: Complete move information including base power and type
/// - `is_critical`: Whether this is a critical hit
/// - `damage_rolls`: Which damage roll variant to use (min/max/average/all)
/// - `target_count`: Number of targets (affects spread move damage)
/// - `attacker_position`: Battle position of the attacking Pokemon
/// - `defender_position`: Battle position of the defending Pokemon
///
/// ## Returns
/// The calculated damage as an i16. Returns 0 for moves that deal no damage.
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: usize,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
) -> i16 {
    // Development-time validation
    debug_assert!(
        attacker.current_hp > 0, 
        "Attacker {} has 0 HP, cannot calculate damage", attacker.species_id
    );
    debug_assert!(
        defender.current_hp > 0, 
        "Defender {} has 0 HP, cannot receive damage", defender.species_id
    );
    debug_assert!(
        move_data.base_power >= 0, 
        "Move {} has negative base power: {}", move_data.name, move_data.base_power
    );
    debug_assert!(
        target_count > 0, 
        "Target count must be positive, got {}", target_count
    );

    // Dispatch to generation-specific calculation
    let generation = state.format.generation();
    
    match generation.generation_number() {
        1 => gen1::calculate_damage_gen1(state, attacker, defender, move_data, is_critical, damage_rolls),
        2 => gen2::calculate_damage_gen2(state, attacker, defender, move_data, is_critical, damage_rolls),
        3 => gen3::calculate_damage_gen3(state, attacker, defender, move_data, is_critical, damage_rolls),
        4 => gen4::calculate_damage_gen4(state, attacker, defender, move_data, is_critical, damage_rolls),
        5..=6 => gen56::calculate_damage_gen56(state, attacker, defender, move_data, is_critical, damage_rolls),
        _ => modern::calculate_damage_modern_gen789(state, attacker, defender, move_data, is_critical, damage_rolls),
    }
}

/// Calculate critical hit probability for a move
///
/// ## Parameters
/// - `pokemon`: The attacking Pokemon
/// - `move_data`: Data for the move being used
/// - `generation`: The generation mechanics to use
///
/// ## Returns
/// Critical hit probability as a f64 between 0.0 and 1.0
pub fn calculate_critical_hit_probability(
    pokemon: &Pokemon,
    move_data: &MoveData,
    generation: &dyn GenerationBattleMechanics,
) -> f64 {
    critical_hit_probability(pokemon, move_data, generation)
}

/// Determine if a critical hit should occur based on probability
///
/// ## Parameters
/// - `probability`: Critical hit probability (0.0 to 1.0)
/// - `rng`: Random number generator (0.0 to 1.0)
///
/// ## Returns
/// True if a critical hit should occur
pub fn should_critical_hit(probability: f64, rng: f64) -> bool {
    rng < probability
}

/// Calculate all possible damage rolls for analysis
///
/// This function is useful for AI decision making and damage range analysis.
///
/// ## Parameters
/// - `state`: Current battle state
/// - `attacker`: The attacking Pokemon
/// - `defender`: The defending Pokemon
/// - `move_data`: Move data
/// - `is_critical`: Whether this is a critical hit
/// - `target_count`: Number of targets
/// - `attacker_position`: Attacker's battle position
/// - `defender_position`: Defender's battle position
///
/// ## Returns
/// Vec<i16> containing all possible damage values
pub fn calculate_all_damage_possibilities(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    target_count: usize,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
) -> Vec<i16> {
    // Calculate damage for each roll type
    vec![
        calculate_damage_with_positions(
            state, attacker, defender, move_data, is_critical, 
            DamageRolls::Min, target_count, attacker_position, defender_position
        ),
        calculate_damage_with_positions(
            state, attacker, defender, move_data, is_critical, 
            DamageRolls::Average, target_count, attacker_position, defender_position
        ),
        calculate_damage_with_positions(
            state, attacker, defender, move_data, is_critical, 
            DamageRolls::Max, target_count, attacker_position, defender_position
        ),
    ]
}

/// Calculate damage range summary for quick analysis
///
/// ## Parameters
/// - `state`: Current battle state
/// - `attacker`: The attacking Pokemon
/// - `defender`: The defending Pokemon
/// - `move_data`: Move data
/// - `target_count`: Number of targets
/// - `attacker_position`: Attacker's battle position
/// - `defender_position`: Defender's battle position
///
/// ## Returns
/// DamageSummary containing min, max, average damage and KO information
pub fn calculate_damage_summary(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    target_count: usize,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
) -> DamageSummary {
    let min_damage = calculate_damage_with_positions(
        state, attacker, defender, move_data, false, 
        DamageRolls::Min, target_count, attacker_position, defender_position
    );
    
    let avg_damage = calculate_damage_with_positions(
        state, attacker, defender, move_data, false, 
        DamageRolls::Average, target_count, attacker_position, defender_position
    );
    
    let max_damage = calculate_damage_with_positions(
        state, attacker, defender, move_data, false, 
        DamageRolls::Max, target_count, attacker_position, defender_position
    );
    
    let crit_damage = calculate_damage_with_positions(
        state, attacker, defender, move_data, true, 
        DamageRolls::Average, target_count, attacker_position, defender_position
    );
    
    let current_hp = defender.current_hp;
    let guaranteed_ko = min_damage >= current_hp;
    let potential_ko = max_damage >= current_hp;
    let guaranteed_crit_ko = crit_damage >= current_hp;
    
    DamageSummary {
        min_damage,
        avg_damage,
        max_damage,
        crit_damage,
        guaranteed_ko,
        potential_ko,
        guaranteed_crit_ko,
        damage_percentage: ((avg_damage as f64 / current_hp as f64) * 100.0) as u8,
    }
}

/// Summary of damage calculation results
#[derive(Debug, Clone, PartialEq)]
pub struct DamageSummary {
    pub min_damage: i16,
    pub avg_damage: i16,
    pub max_damage: i16,
    pub crit_damage: i16,
    pub guaranteed_ko: bool,
    pub potential_ko: bool,
    pub guaranteed_crit_ko: bool,
    pub damage_percentage: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::{BattleState};
use crate::core::battle_state::pokemon_state::{Pokemon, Move};
    use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
    use crate::data::showdown_types::MoveData;

    fn create_test_pokemon() -> Pokemon {
        Pokemon {
            species_id: "pikachu".to_string(),
            level: 50,
            current_hp: 100,
            // ... other fields with defaults
            ..Default::default()
        }
    }

    fn create_test_move() -> MoveData {
        MoveData {
            name: "Thunderbolt".to_string(),
            base_power: 90,
            move_type: "Electric".to_string(),
            category: "Special".to_string(),
            // ... other fields with defaults
            ..Default::default()
        }
    }

    fn create_test_state() -> BattleState {
        BattleState {
            format: BattleFormat::gen9_ou(),
            // ... other fields with defaults
            ..Default::default()
        }
    }

    #[test]
    fn test_damage_calculation_basic() {
        let state = create_test_state();
        let attacker = create_test_pokemon();
        let defender = create_test_pokemon();
        let move_data = create_test_move();
        
        let damage = calculate_damage_with_positions(
            &state,
            &attacker,
            &defender,
            &move_data,
            false,
            DamageRolls::Average,
            1,
            BattlePosition::new(SideReference::SideOne, 0),
            BattlePosition::new(SideReference::SideTwo, 0),
        );
        
        assert!(damage > 0);
        assert!(damage <= move_data.base_power as i16);
    }

    #[test]
    fn test_critical_hit_probability() {
        let pokemon = create_test_pokemon();
        let move_data = create_test_move();
        let generation = crate::generation::Generation::gen9();
        
        let probability = calculate_critical_hit_probability(&pokemon, &move_data, &generation);
        
        assert!(probability >= 0.0);
        assert!(probability <= 1.0);
    }

    #[test]
    fn test_damage_summary() {
        let state = create_test_state();
        let attacker = create_test_pokemon();
        let defender = create_test_pokemon();
        let move_data = create_test_move();
        
        let summary = calculate_damage_summary(
            &state,
            &attacker,
            &defender,
            &move_data,
            1,
            BattlePosition::new(SideReference::SideOne, 0),
            BattlePosition::new(SideReference::SideTwo, 0),
        );
        
        assert!(summary.min_damage <= summary.avg_damage);
        assert!(summary.avg_damage <= summary.max_damage);
        assert!(summary.damage_percentage <= 100);
    }

    #[test]
    fn test_should_critical_hit() {
        assert!(should_critical_hit(1.0, 0.5)); // 100% probability
        assert!(!should_critical_hit(0.0, 0.5)); // 0% probability
        assert!(should_critical_hit(0.5, 0.25)); // Should crit with low RNG
        assert!(!should_critical_hit(0.5, 0.75)); // Should not crit with high RNG
    }
}