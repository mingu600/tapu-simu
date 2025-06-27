//! # Simple Move Effects
//! 
//! This module contains simple move implementations with straightforward effects.

use std::collections::HashMap;

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::core::battle_state::{BattleState, MoveCategory, Pokemon};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, PokemonInstruction, PokemonStatus,
    SideCondition, Stat, StatusInstruction, StatsInstruction, Terrain, VolatileStatus, Weather,
};
use crate::data::showdown_types::MoveData;
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::generation::GenerationMechanics;

/// Generate damage instructions with substitute tracking
/// Returns (damage_instructions, hit_substitute)
pub fn generate_substitute_aware_damage_with_tracking(
    state: &BattleState,
    target_position: BattlePosition,
    damage: i16,
) -> (Vec<BattleInstruction>, bool) {
    if let Some(target) = state.get_pokemon_at_position(target_position) {
        // Check if target has a substitute
        if target.volatile_statuses.contains(&VolatileStatus::Substitute) && target.substitute_health > 0 {
            let mut instructions = Vec::new();
            let substitute_damage = damage.min(target.substitute_health);
            let new_substitute_health = target.substitute_health - substitute_damage;
            
            // Generate ChangeSubstituteHealth instruction
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeSubstituteHealth {
                target: target_position,
                new_health: new_substitute_health,
                previous_health: target.substitute_health,
            }));
            
            // If substitute is broken, remove it (but don't carry over damage)
            if new_substitute_health <= 0 {
                // Remove substitute volatile status
                instructions.push(BattleInstruction::Status(crate::core::instructions::StatusInstruction::RemoveVolatile {
                    target: target_position,
                    status: VolatileStatus::Substitute,
                    previous_duration: None,
                }));
                
                // In Pokemon, the substitute absorbs the entire hit that breaks it
                // No remaining damage is dealt to the Pokemon from the same hit
            }
            
            return (instructions, true);
        }
    }
    
    // Hit Pokemon directly
    let instructions = vec![
        BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target_position,
            amount: damage,
            previous_hp: None, // Will be filled in by battle state
        })
    ];
    (instructions, false)
}

// =============================================================================
// SIMPLE MOVES WITH STRAIGHTFORWARD EFFECTS
// =============================================================================

/// Apply Splash - does nothing
pub fn apply_splash(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Splash does nothing - return empty instructions
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Kinesis - lowers accuracy by 1 stage
pub fn apply_kinesis(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::status_moves::{
        stat_modification_move, create_stat_changes
    };
    
    let stat_changes = create_stat_changes(&[(Stat::Accuracy, -1)]);
    let instructions = stat_modification_move(state, target_positions, &stat_changes, None);
    
    if instructions.is_empty() {
        vec![BattleInstructions::new(100.0, vec![])]
    } else {
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

/// Apply Quick Attack - priority +1 physical move
/// Note: Priority is handled by the PS move data, this just handles any special effects
pub fn apply_quick_attack(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Quick Attack is just a priority move with no special effects
    // Priority is handled by the instruction generator
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Tailwind - doubles speed for side for 4 turns
pub fn apply_tailwind(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let side = user_position.side;
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side,
        condition: SideCondition::Tailwind,
        duration: 4,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Trick Room - reverses speed priority for 5 turns
pub fn apply_trick_room(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Toggle trick room state - if active, turn off; if inactive, turn on for 5 turns
    let instruction = BattleInstruction::Field(FieldInstruction::TrickRoom {
        active: true, // Will be properly handled by state application
        turns: Some(5),
        source: None,
        previous_active: false,
        previous_turns: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Refresh - cures user's status condition
pub fn apply_refresh(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::status_moves::status_cure_move;
    
    let instructions = status_cure_move(&[user_position], Some(user_position));
    
    vec![BattleInstructions::new(100.0, instructions)]
}


/// Apply Healing Wish - user faints, fully heals replacement
pub fn apply_healing_wish(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instruction_list = Vec::new();
    
    // Get the user's current HP before fainting
    let user_current_hp = state.get_pokemon_at_position(user_position)
        .map(|pokemon| pokemon.hp)
        .unwrap_or(0);
    
    // Faint the user
    instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Faint {
        target: user_position,
        previous_hp: user_current_hp,
        previous_status: None,
    }));
    
    // Set up healing for next Pokemon
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: user_position.side,
        condition: SideCondition::HealingWish,
        duration: 1,
        previous_duration: None,
    });
    instruction_list.push(instruction);
    
    vec![BattleInstructions::new(100.0, instruction_list)]
}

/// Apply Life Dew - heals user and ally by 25%
pub fn apply_life_dew(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::status_moves::healing_move;
    
    let mut instruction_list = Vec::new();
    
    // Heal user using the proper composer
    let user_heal_instructions = healing_move(state, user_position, 0.25, Some(user_position));
    instruction_list.extend(user_heal_instructions);
    
    // Heal targets (ally in doubles) using the proper composer
    for &target_position in target_positions {
        let target_heal_instructions = healing_move(state, target_position, 0.25, Some(user_position));
        instruction_list.extend(target_heal_instructions);
    }
    
    vec![BattleInstructions::new(100.0, instruction_list)]
}

/// Apply No Retreat - boosts all stats but prevents switching
pub fn apply_no_retreat(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::status_moves::{
        self_stat_boost_move, create_stat_changes
    };
    
    let mut instruction_list = Vec::new();
    
    // Boost all stats by 1 stage using the composer
    let stat_changes = create_stat_changes(&[
        (Stat::Attack, 1),
        (Stat::Defense, 1),
        (Stat::SpecialAttack, 1),
        (Stat::SpecialDefense, 1),
        (Stat::Speed, 1),
    ]);
    
    let stat_instructions = self_stat_boost_move(state, user_position, &stat_changes);
    instruction_list.extend(stat_instructions);
    
    // Apply No Retreat status
    instruction_list.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: user_position,
        status: VolatileStatus::NoRetreat,
        duration: None, // Permanent
        previous_had_status: false,
        previous_duration: None,
    }));
    
    vec![BattleInstructions::new(100.0, instruction_list)]
}

/// Apply Pain Split - averages HP between user and target
pub fn apply_pain_split(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if target_positions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let target_position = target_positions[0];
    
    if let (Some(user), Some(target)) = (
        state.get_pokemon_at_position(user_position),
        state.get_pokemon_at_position(target_position)
    ) {
        let total_hp = user.hp + target.hp;
        let new_hp = total_hp / 2;
        
        let mut instruction_list = Vec::new();
        
        // Adjust user's HP (properly using core instructions)
        let user_hp_change = new_hp - user.hp;
        if user_hp_change > 0 {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: user_position,
                amount: user_hp_change,
                previous_hp: Some(user.hp),
            }));
        } else if user_hp_change < 0 {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: -user_hp_change,
                previous_hp: Some(user.hp),
            }));
        }
        
        // Adjust target's HP (properly using core instructions)
        let target_hp_change = new_hp - target.hp;
        if target_hp_change > 0 {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: target_position,
                amount: target_hp_change,
                previous_hp: Some(target.hp),
            }));
        } else if target_hp_change < 0 {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: -target_hp_change,
                previous_hp: Some(target.hp),
            }));
        }
        
        vec![BattleInstructions::new(100.0, instruction_list)]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Parting Shot - lowers opponent's stats then switches
pub fn apply_parting_shot(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::status_moves::{
        enemy_stat_reduction_move, create_stat_changes
    };
    
    // Lower target's Attack and Special Attack by 1 stage using the composer
    let stat_changes = create_stat_changes(&[
        (Stat::Attack, -1),
        (Stat::SpecialAttack, -1),
    ]);
    
    let instructions = enemy_stat_reduction_move(state, target_positions, &stat_changes, user_position);
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Perish Song - all Pokemon on field faint in 3 turns
pub fn apply_perish_song(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instruction_list = Vec::new();
    
    // Apply Perish 3 to user
    instruction_list.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: user_position,
        status: VolatileStatus::Perish3,
        duration: Some(3),
        previous_had_status: false,
        previous_duration: None,
    }));
    
    // Apply Perish 3 to all targets
    for &target_position in target_positions {
        instruction_list.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Perish3,
            duration: Some(3),
            previous_had_status: false,
            previous_duration: None,
        }));
    }
    
    vec![BattleInstructions::new(100.0, instruction_list)]
}

// =============================================================================
// GENERIC EFFECTS AND HELPER FUNCTIONS
// =============================================================================

/// Apply generic move effects based on move data
/// This is the fallback function for moves without specific implementations
pub fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Use the shared implementation from the main moves module
    super::apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

// Fallback function removed - moves should either be implemented or return errors