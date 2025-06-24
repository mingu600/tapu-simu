//! # Complex Move Effects
//! 
//! This module contains complex move implementations that have unique mechanics
//! and cannot be easily categorized into other groups.

use crate::core::battle_state::{Pokemon, MoveCategory};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat, Weather, SideCondition, Terrain};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction,
    FieldInstruction, StatsInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use std::collections::HashMap;

// =============================================================================
// COMPLEX MOVES WITH UNIQUE MECHANICS
// =============================================================================

/// Apply Baton Pass - enables stat boost passing when switching
pub fn apply_baton_pass(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Baton Pass enables stat boost passing when switching
    vec![BattleInstructions::new(100.0, vec![
        BattleInstruction::Field(FieldInstruction::ToggleBatonPassing {
            side: user_position.side,
            active: true,
            previous_state: false, // Assume it was false before
        })
    ])]
}

/// Apply Belly Drum - maximizes Attack at cost of 50% HP
pub fn apply_belly_drum(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let cost = pokemon.max_hp / 2;
        if pokemon.hp > cost {
            let mut instructions = Vec::new();
            
            // Damage user for 50% of max HP
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: cost,
                previous_hp: Some(pokemon.hp),
            }));
            
            // Maximize Attack (set to +6)
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Attack, 6);
            
            instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: target_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            }));
            
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            // Not enough HP - move fails
            vec![BattleInstructions::new(100.0, vec![])]
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Curse - different effects for Ghost vs non-Ghost types
pub fn apply_curse(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        if user.types.iter().any(|t| t.to_lowercase() == "ghost") {
            // Ghost type: Curses target, user loses 50% HP
            if let Some(&target_position) = target_positions.first() {
                let mut instructions = Vec::new();
                
                // Damage user for 50% HP
                let damage = user.max_hp / 2;
                instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: user_position,
                    amount: damage,
                    previous_hp: Some(user.hp),
                }));
                
                // Apply curse to target
                instructions.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: target_position,
                    status: VolatileStatus::Curse,
                    duration: None, // Lasts until target switches
                    previous_had_status: false,
                    previous_duration: None,
                }));
                
                vec![BattleInstructions::new(100.0, instructions)]
            } else {
                vec![BattleInstructions::new(100.0, vec![])]
            }
        } else {
            // Non-Ghost type: Raises Attack and Defense, lowers Speed
            let target_position = if target_positions.is_empty() {
                user_position
            } else {
                target_positions[0]
            };
            
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Attack, 1);
            stat_boosts.insert(Stat::Defense, 1);
            stat_boosts.insert(Stat::Speed, -1);
            
            let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: target_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            });
            
            vec![BattleInstructions::new(100.0, vec![instruction])]
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Destiny Bond - if user faints, opponent also faints
pub fn apply_destiny_bond(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: user_position,
        status: VolatileStatus::DestinyBond,
        duration: Some(1), // Lasts until end of turn
        previous_had_status: false,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Encore - forces opponent to repeat last move
pub fn apply_encore(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Encore,
            duration: Some(3), // Lasts 3 turns
            previous_had_status: false,
            previous_duration: None,
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Leech Seed - drains HP every turn
pub fn apply_leech_seed(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::LeechSeed,
            duration: None, // Lasts until Pokemon switches
            previous_had_status: false,
            previous_duration: None,
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Rest - fully heals and puts user to sleep
pub fn apply_rest(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let mut instructions = Vec::new();
        
        // Heal to full HP
        let heal_amount = pokemon.max_hp - pokemon.hp;
        if heal_amount > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: target_position,
                amount: heal_amount,
                previous_hp: Some(pokemon.hp),
            }));
        }
        
        // Clear any existing status
        if pokemon.status != PokemonStatus::None {
            instructions.push(BattleInstruction::Status(StatusInstruction::Apply {
                target: target_position,
                status: PokemonStatus::None,
                duration: None,
                previous_status: Some(pokemon.status),
                previous_duration: pokemon.status_duration,
            }));
        }
        
        // Put to sleep for 2 turns
        instructions.push(BattleInstruction::Status(StatusInstruction::Apply {
            target: target_position,
            status: PokemonStatus::Sleep,
            duration: Some(2),
            previous_status: Some(PokemonStatus::None),
            previous_duration: None,
        }));
        
        instructions.push(BattleInstruction::Status(StatusInstruction::SetRestTurns {
            target: target_position,
            turns: 2,
            previous_turns: None,
        }));
        
        vec![BattleInstructions::new(100.0, instructions)]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Sleep Talk - uses random move while asleep
pub fn apply_sleep_talk(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(pokemon) = state.get_pokemon_at_position(user_position) {
        if pokemon.status == PokemonStatus::Sleep {
            // Move succeeds - actual move selection handled by turn system
            vec![BattleInstructions::new(100.0, vec![])]
        } else {
            // Move fails if not asleep
            vec![BattleInstructions::new(100.0, vec![])]
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Taunt - prevents status moves
pub fn apply_taunt(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Taunt,
            duration: Some(3), // Lasts 3 turns
            previous_had_status: false,
            previous_duration: None,
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Whirlwind - forces opponent to switch
pub fn apply_whirlwind(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Force switch for opposing side
    let opposing_side = match user_position.side {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    let force_switch_instruction = BattleInstruction::Field(FieldInstruction::ToggleForceSwitch {
        side: opposing_side,
        active: true,
        previous_state: false,
    });
    
    vec![BattleInstructions::new(100.0, vec![force_switch_instruction])]
}

/// Apply Yawn - causes sleep next turn
pub fn apply_yawn(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Yawn,
            duration: Some(2), // Sleep occurs after 1 turn
            previous_had_status: false,
            previous_duration: None,
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}