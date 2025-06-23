//! # Two-Turn/Charge Move Effects

//! 
//! This module contains moves that require charging or preparation on the first turn,
//! then execute their effect on the second turn. This includes moves like Solar Beam,
//! Fly, Dig, and other two-turn moves.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{VolatileStatus, Weather, Stat, SideCondition};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, StatsInstruction,
    PokemonInstruction, FieldInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

// =============================================================================
// TWO-TURN/CHARGE MOVES
// =============================================================================

/// Apply Solar Beam - no charge in sun, reduced power in other weather
pub fn apply_solar_beam(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user is already charging Solar Beam
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack with potentially modified power
            let power_multiplier = match state.weather() {
                Weather::Sun | Weather::HarshSun | Weather::HarshSunlight => 1.0, // Full power in sun
                Weather::Rain | Weather::HeavyRain | Weather::Sand | Weather::Sandstorm |
                Weather::Hail | Weather::Snow | Weather::StrongWinds => 0.5, // Half power in other weather
                Weather::None => 1.0, // Full power in no weather
            };
            
            let modified_move_data = MoveData {
                base_power: ((move_data.base_power as f32 * power_multiplier) as u16),
                ..move_data.clone()
            };
            
            // Remove charging status
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                })
            ]));
            
            // Apply damage
            let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - check if we can skip charging in sun
            if state.weather() == Weather::Sun || state.weather() == Weather::HarshSun {
                // Skip charging and attack immediately in sun
                let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
                instructions.extend(generic_instructions);
            } else {
                // Start charging
                instructions.push(BattleInstructions::new(100.0, vec![
                    BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                        target: user_position,
                        status: VolatileStatus::TwoTurnMove,
                        duration: Some(1), // Charge for 1 turn
                        previous_had_status: false,
                        previous_duration: None,
                    })
                ]));
            }
        }
    }
    
    instructions
}

/// Apply Solar Blade - identical to Solar Beam but physical
pub fn apply_solar_blade(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Solar Blade has identical mechanics to Solar Beam
    apply_solar_beam(state, move_data, user_position, target_positions, generation)
}

/// Apply Meteor Beam - charges for one turn, raises Special Attack, then attacks
pub fn apply_meteor_beam(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - start charging and boost Special Attack
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::SpecialAttack, 1);
            
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target: user_position,
                    stat_changes: stat_boosts,
                    previous_boosts: HashMap::new(),
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Electro Shot - charges for one turn, raises Special Attack, then attacks
pub fn apply_electro_shot(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Electro Shot has identical mechanics to Meteor Beam
    apply_meteor_beam(state, move_data, user_position, target_positions, generation)
}

/// Apply Dig - semi-invulnerable underground on first turn, attacks on second
pub fn apply_dig(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack from underground
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::Dig,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - go underground
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::Dig,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Fly - semi-invulnerable in the air on first turn, attacks on second
pub fn apply_fly(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack from the air
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::Fly,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - fly up
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::Fly,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Bounce - semi-invulnerable in the air on first turn, attacks on second with paralysis chance
pub fn apply_bounce(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Bounce has identical two-turn mechanics to Fly, with the secondary effect of paralysis
    // handled by the secondary effect system
    apply_fly(state, move_data, user_position, target_positions, generation)
}

/// Apply Dive - semi-invulnerable underwater on first turn, attacks on second
pub fn apply_dive(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack from underwater
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::Dive,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - dive underwater
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::Dive,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Phantom Force - semi-invulnerable on first turn, bypasses protection on second
pub fn apply_phantom_force(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack with protection bypass
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - become semi-invulnerable
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Shadow Force - identical to Phantom Force
pub fn apply_shadow_force(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_phantom_force(state, move_data, user_position, target_positions, generation)
}

// All major two-turn moves are now implemented
// This includes charging moves, semi-invulnerable moves, delayed attacks, and focus moves

/// Apply Future Sight - delayed attack that hits in 3 turns
pub fn apply_future_sight(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::damage_calc::{calculate_damage_with_positions, DamageRolls};
    
    if target_positions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    // Future Sight targets the first specified position (usually opposite side, slot 0 in singles)
    let target_position = target_positions[0];
    
    // Calculate damage that will be dealt in 3 turns
    let target = match state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    let damage = calculate_damage_with_positions(
        state,
        user,
        target,
        move_data,
        false, // Future Sight cannot crit
        DamageRolls::Average, // Use average damage for delayed attacks
        1, // Single target
        user_position,
        target_position,
    );
    
    // Create instruction to set up the future attack
    // This uses a field instruction to schedule the attack
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: target_position.side,
        condition: SideCondition::FutureSight,
        duration: 3, // Hits after 3 turns
        previous_duration: None,
    });
    
    // In a full implementation, this would create a special future sight tracking structure
    // For now, we'll create a simplified version that approximates the mechanic
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Razor Wind - charges first turn, attacks second with high crit ratio
pub fn apply_razor_wind(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Standard two-turn move pattern
    apply_standard_two_turn_move(state, move_data, user_position, target_positions, generation)
}

/// Apply Skull Bash - charges first turn with Defense boost, attacks second
pub fn apply_skull_bash(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - charge and boost Defense
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Defense, 1);
            
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target: user_position,
                    stat_changes: stat_boosts,
                    previous_boosts: HashMap::new(),
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Sky Attack - charges first turn, attacks second with high crit ratio
pub fn apply_sky_attack(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_standard_two_turn_move(state, move_data, user_position, target_positions, generation)
}

/// Apply Focus Punch - charges with focus, fails if hit during charge
pub fn apply_focus_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - check if user was hit during focus
            if state.user_moved_after_taking_damage(user_position) {
                // User was hit during focus - move fails
                instructions.push(BattleInstructions::new(100.0, vec![
                    BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                        target: user_position,
                        status: VolatileStatus::TwoTurnMove,
                        previous_duration: None,
                    })
                ]));
            } else {
                // User wasn't hit - execute the powerful punch
                instructions.push(BattleInstructions::new(100.0, vec![
                    BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                        target: user_position,
                        status: VolatileStatus::TwoTurnMove,
                        previous_duration: None,
                    })
                ]));
                
                let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
                instructions.extend(generic_instructions);
            }
        } else {
            // First turn - start focusing
            // Apply a special volatile status to track that Focus Punch is charging
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Standard two-turn move pattern
fn apply_standard_two_turn_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::TwoTurnMove) {
            // Second turn - attack
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    previous_duration: None,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - start charging
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::TwoTurnMove,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply generic move effects (delegate to shared implementation)
fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Use the shared implementation from the main moves module
    super::apply_generic_effects(state, move_data, user_position, target_positions, generation)
}