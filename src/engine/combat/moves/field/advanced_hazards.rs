//! # Advanced Hazard Manipulation

//! 
//! This module contains moves with advanced hazard manipulation effects beyond basic hazard setting/removal.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{SideCondition, PokemonStatus};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, StatusInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::moves::{apply_generic_effects, is_immune_to_poison};
use crate::data::showdown_types::MoveData;

// =============================================================================
// ADVANCED HAZARD MANIPULATION
// =============================================================================

/// Apply Mortal Spin - Rapid Spin + poison damage to adjacent opponents
pub fn apply_mortal_spin(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply normal move damage first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation, true));
    
    // Remove hazards from user's side (like Rapid Spin)
    let user_side_ref = user_position.side;
    let user_side = state.get_side_by_ref(user_side_ref);
    
    let hazards_to_remove = vec![
        SideCondition::Spikes,
        SideCondition::ToxicSpikes,
        SideCondition::StealthRock,
        SideCondition::StickyWeb,
    ];
    
    for condition in hazards_to_remove {
        if let Some(duration) = user_side.side_conditions.get(&condition) {
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                    side: user_side_ref,
                    condition,
                    previous_duration: 0,
                }),
            ]));
        }
    }
    
    // Poison all adjacent opponents (in doubles/multi-battles)
    let opponent_side_ref = match user_side_ref {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    // Get all active opponents and poison them
    let opponent_side = state.get_side_by_ref(opponent_side_ref);
    for (slot, pokemon) in opponent_side.pokemon.iter().enumerate() {
        if let Some(active_slot) = opponent_side.active_pokemon_indices.get(slot) {
            if active_slot.is_some() && !pokemon.is_fainted() {
                let opponent_position = BattlePosition::new(opponent_side_ref, slot);
                
                // Apply poison if not already statused and not immune
                if pokemon.status == PokemonStatus::None && !is_immune_to_poison(pokemon, generation) {
                    instructions.push(BattleInstructions::new(100.0, vec![
                        BattleInstruction::Status(StatusInstruction::Apply {
                            target: opponent_position,
                            status: PokemonStatus::Poison,
                            duration: None,
                            previous_status: Some(pokemon.status),
                            previous_duration: pokemon.status_duration,
                        }),
                    ]));
                }
            }
        }
    }
    
    instructions
}