//! Field condition damage modifiers (screens, etc.)
//!
//! This module handles damage modifications from field conditions like
//! Light Screen, Reflect, and Aurora Veil.

use crate::core::battle_state::{BattleState, Pokemon, MoveCategory};
use crate::core::instructions::SideCondition;
use crate::generation::GenerationMechanics;

/// Calculate screen damage modifier (Reflect, Light Screen, Aurora Veil)
pub fn get_screen_damage_modifier(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_category: &MoveCategory,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::engine::mechanics::abilities::{apply_ability_effect, AbilityContext};
    use crate::types::AbilityId;
    use crate::core::battle_format::{BattlePosition, SideReference};

    // Check if attacker has ability to bypass screens (Infiltrator, etc.)
    // First find the attacker's position
    let attacker_position = {
        let mut found_position = None;
        for (side_index, side) in state.sides.iter().enumerate() {
            for (slot_index, pokemon) in side.pokemon.iter().enumerate() {
                if std::ptr::eq(pokemon, attacker) {
                    found_position = Some(BattlePosition::new(
                        if side_index == 0 { SideReference::SideOne } else { SideReference::SideTwo },
                        slot_index
                    ));
                    break;
                }
            }
            if found_position.is_some() { break; }
        }
        found_position.unwrap_or(BattlePosition::new(SideReference::SideOne, 0))
    };

    let ability_id = AbilityId::from(attacker.ability.as_str());
    let context = AbilityContext {
        user_position: attacker_position,
        target_position: None,
        move_type: None,
        move_id: None,
        base_power: None,
        is_critical: false,
        is_contact: false,
        state: state,
    };
    
    if apply_ability_effect(&ability_id, context).bypasses_screens {
        return 1.0;
    }

    // Determine defending side by finding which side contains the defender
    let defending_side = if state.sides[0]
        .pokemon
        .iter()
        .any(|p| std::ptr::eq(p, defender))
    {
        &state.sides[0]
    } else {
        &state.sides[1]
    };

    // Check for Aurora Veil (affects both physical and special moves)
    if defending_side
        .side_conditions
        .contains_key(&SideCondition::AuroraVeil)
    {
        // Aurora Veil: 0.5x in singles, 0.66x in doubles
        return if state.format.supports_spread_moves() {
            2.0 / 3.0 // 0.66x
        } else {
            0.5
        };
    }

    // Check for specific screens based on move category
    match move_category {
        MoveCategory::Physical => {
            if defending_side
                .side_conditions
                .contains_key(&SideCondition::Reflect)
            {
                // Reflect: 0.5x in singles, 0.66x in doubles
                if state.format.supports_spread_moves() {
                    2.0 / 3.0 // 0.66x
                } else {
                    0.5
                }
            } else {
                1.0
            }
        }
        MoveCategory::Special => {
            if defending_side
                .side_conditions
                .contains_key(&SideCondition::LightScreen)
            {
                // Light Screen: 0.5x in singles, 0.66x in doubles
                if state.format.supports_spread_moves() {
                    2.0 / 3.0 // 0.66x
                } else {
                    0.5
                }
            } else {
                1.0
            }
        }
        MoveCategory::Status => 1.0, // Status moves aren't affected by screens
    }
}