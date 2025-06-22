//! # Simple Move Effects

//! 
//! This module contains simple move implementations with straightforward effects.

use crate::core::battle_state::{Pokemon, MoveCategory};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat, Weather, SideCondition, Terrain};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction,
    FieldInstruction, StatsInstruction,
};
use crate::data::ps::repository::Repository;
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

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
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Accuracy, -1);
        
        let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: target_position,
            stat_changes: stat_boosts,
            previous_boosts: HashMap::new(),
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
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
    let instruction = BattleInstruction::Status(StatusInstruction::Remove {
        target: user_position,
        status: PokemonStatus::None, // Remove any status
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Wish - heals 50% of user's max HP after 2 turns
pub fn apply_wish(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let heal_amount = user.max_hp / 2; // Heals 50% of user's max HP
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::SetWish {
            target: user_position,
            heal_amount,
            turns_remaining: 2, // Activates after 2 turns
            previous_wish: None,
        });
        
        vec![BattleInstructions::new(100.0, vec![instruction])]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
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
    let mut instruction_list = Vec::new();
    
    // Heal user
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let heal_amount = user.max_hp / 4; // Heals 25%
        instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
            target: user_position,
            amount: heal_amount,
            previous_hp: Some(user.hp),
        }));
    }
    
    // Heal targets (ally in doubles)
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let heal_amount = target.max_hp / 4;
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: target_position,
                amount: heal_amount,
                previous_hp: Some(target.hp),
            }));
        }
    }
    
    vec![BattleInstructions::new(100.0, instruction_list)]
}

/// Apply No Retreat - boosts all stats but prevents switching
pub fn apply_no_retreat(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instruction_list = Vec::new();
    
    // Boost all stats by 1 stage
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Defense, 1);
    stat_boosts.insert(Stat::SpecialAttack, 1);
    stat_boosts.insert(Stat::SpecialDefense, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    instruction_list.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: user_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    }));
    
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
        
        // Adjust user's HP
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
        
        // Adjust target's HP
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
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instruction_list = Vec::new();
    
    // Lower target's Attack and Special Attack by 1 stage
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -1);
        stat_boosts.insert(Stat::SpecialAttack, -1);
        
        instruction_list.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: target_position,
            stat_changes: stat_boosts,
            previous_boosts: HashMap::new(),
        }));
    }
    
    vec![BattleInstructions::new(100.0, instruction_list)]
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
) -> Vec<BattleInstructions> {
    // Use the shared implementation from the main moves module
    super::apply_generic_effects(state, move_data, user_position, target_positions, generation)
}

// Fallback function removed - moves should either be implemented or return errors