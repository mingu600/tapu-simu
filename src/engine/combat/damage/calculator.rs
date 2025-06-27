//! Main damage calculator entry point
//!
//! This module provides the primary interface for damage calculations,
//! dispatching to the appropriate generation-specific calculator while
//! maintaining the existing API.

use crate::core::battle_state::{BattleState, Pokemon};
use crate::data::showdown_types::MoveData;
use crate::engine::combat::damage_context::{DamageContext, AttackerContext, DefenderContext, MoveContext, FieldContext, FormatContext};
use crate::engine::combat::damage_context::{EffectiveStats, AbilityState, ItemEffects};
use crate::types::{MoveId, TypeId};
use crate::core::instructions::MoveCategory;
use super::types::DamageRolls;


/// Calculate damage between two Pokemon with explicit battle positions.
///
/// This is the primary damage calculation function that implements Pokemon's
/// damage formula with full generation support and format awareness. The
/// calculation includes all standard damage modifiers including STAB, type
/// effectiveness, critical hits, abilities, items, and field conditions.
///
/// ## Algorithm Overview
///
/// The damage calculation follows Pokemon's standard damage formula:
/// Damage = ((((2 * Level / 5 + 2) * Power * A / D) / 50) + 2) * Modifiers
///
/// Where modifiers include:
/// - Critical hit multiplier (1.5x for Gen 6+, 2.0x for earlier generations)
/// - Same Type Attack Bonus (STAB) - typically 1.5x
/// - Type effectiveness (0x, 0.25x, 0.5x, 1x, 2x, or 4x)
/// - Random damage roll (85%-100% in 16 discrete steps)
/// - Weather conditions (e.g., rain boosting Water moves)
/// - Abilities (e.g., Adaptability changing STAB to 2x)
/// - Items (e.g., Life Orb adding 30% damage)
/// - Multi-target spread move penalty
/// - Generation-specific mechanics
///
/// ## Parameters
///
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
///
/// The calculated damage as an i16. Returns 0 for moves that deal no damage
/// (e.g., status moves, immune type matchups).
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: usize,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> i16 {
    // Build modern DamageContext
    let attacker_context = AttackerContext {
        pokemon: attacker.clone(),
        position: attacker_position,
        effective_stats: EffectiveStats::from_pokemon(attacker),
        ability_state: AbilityState::from_pokemon(attacker),
        item_effects: ItemEffects::from_pokemon(attacker),
    };

    let defender_context = DefenderContext {
        pokemon: defender.clone(),
        position: defender_position,
        effective_stats: EffectiveStats::from_pokemon(defender),
        ability_state: AbilityState::from_pokemon(defender),
        item_effects: ItemEffects::from_pokemon(defender),
    };

    let move_context = MoveContext {
        name: MoveId::new(&move_data.name),
        base_power: move_data.base_power as u8,
        is_critical,
        is_contact: move_data.flags.contains_key("contact"),
        is_punch: move_data.flags.contains_key("punch"),
        is_sound: move_data.flags.contains_key("sound"),
        is_multihit: move_data.flags.contains_key("multihit"),
        move_type: TypeId::new(move_data.move_type.to_normalized_str()),
        category: MoveCategory::from_str(&move_data.category),
    };

    let field_context = FieldContext {
        weather: state.field.weather.clone(),
        terrain: state.field.terrain.clone(),
        global_effects: state.field.global_effects.clone(),
    };

    let format_context = FormatContext {
        format: state.format.clone(),
        target_count,
    };

    let damage_context = DamageContext {
        attacker: attacker_context,
        defender: defender_context,
        move_info: move_context,
        field: field_context,
        format: format_context,
    };

    // Use the modular damage calculation through the generation dispatch
    super::generations::calculate_damage(&damage_context, damage_rolls).damage
}