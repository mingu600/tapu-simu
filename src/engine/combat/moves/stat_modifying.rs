//! # Stat Modifying Move Effects

//! 
//! This module contains stat modification move effects extracted from move_effects.rs.
//! These functions handle moves that boost or lower Pokemon stats.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{Stat, Weather};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatsInstruction, PokemonInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

// =============================================================================
// STAT-BOOSTING MOVES (SELF-TARGETING)
// =============================================================================

/// Apply Swords Dance - raises Attack by 2 stages
pub fn apply_swords_dance(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position // Self-targeting move
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 2);
    
    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: target_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Dragon Dance - raises Attack and Speed by 1 stage each
pub fn apply_dragon_dance(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: target_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Nasty Plot - raises Special Attack by 2 stages
pub fn apply_nasty_plot(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::SpecialAttack, 2);
    
    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: target_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Agility - raises Speed by 2 stages
pub fn apply_agility(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Speed, 2);
    
    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: target_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Growth - raises Attack and Special Attack
/// Generation-aware: Enhanced in sun weather
pub fn apply_growth(
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
    
    let mut stat_boosts = HashMap::new();
    
    // Enhanced in sun weather
    let boost_amount = match state.weather() {
        Weather::Sun | Weather::HarshSun => 2,
        _ => 1,
    };
    
    stat_boosts.insert(Stat::Attack, boost_amount);
    stat_boosts.insert(Stat::SpecialAttack, boost_amount);
    
    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: target_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

// =============================================================================
// STAT-LOWERING MOVES (TARGET-AFFECTING)
// =============================================================================

/// Apply Growl - lowers target's Attack by 1 stage
pub fn apply_growl(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -1);
        
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

/// Apply Leer - lowers target's Defense by 1 stage
pub fn apply_leer(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Defense, -1);
        
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

/// Apply Tail Whip - lowers target's Defense by 1 stage
pub fn apply_tail_whip(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_leer(state, user_position, target_positions, generation) // Same effect as Leer
}

/// Apply String Shot - lowers target's Speed by 2 stages
/// Generation-aware: Effect may change in earlier generations
pub fn apply_string_shot(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // In Gen 1, String Shot only lowered Speed by 1 stage
    let speed_reduction = if generation.generation.number() == 1 { -1 } else { -2 };
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Speed, speed_reduction);
        
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

/// Apply Charm - lowers target's Attack by 2 stages
pub fn apply_charm(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -2);
        
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

// =============================================================================
// SPECIAL STAT-MODIFYING MOVES WITH SECONDARY EFFECTS
// =============================================================================

/// Apply Acid - deals damage with chance to lower Defense
/// Generation-aware: 33.2% chance in Gen 1, 10% in later generations
pub fn apply_acid(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Acid deals damage AND has a secondary effect
    let effect_chance = if generation.generation.number() == 1 { 33 } else { 10 };
    
    apply_probability_based_secondary_effects(
        state,
        move_data,
        user_position,
        target_positions,
        generation,
        effect_chance,
    )
}

// =============================================================================
// HELPER FUNCTIONS FOR STAT MODIFICATION
// =============================================================================

/// Helper function for moves with probability-based secondary effects
pub fn apply_probability_based_secondary_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    effect_chance: i16,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Calculate probabilities
    let effect_probability = effect_chance as f32;
    let no_effect_probability = 100.0 - effect_probability;
    
    // Create no-effect branch (most common case)
    if no_effect_probability > 0.0 {
        instructions.push(BattleInstructions::new(no_effect_probability, vec![]));
    }
    
    // Create effect branch
    if effect_probability > 0.0 {
        if let Some(effect_instructions) = determine_secondary_effect_from_move(
            state, 
            move_data, 
            user_position, 
            target_positions, 
            generation
        ) {
            instructions.push(BattleInstructions::new(effect_probability, effect_instructions));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Determine what secondary effect a move should have based on its properties
/// This function maps move types and names to their appropriate secondary effects
pub fn determine_secondary_effect_from_move(
    _state: &BattleState,
    move_data: &MoveData,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Option<Vec<BattleInstruction>> {
    let move_name = move_data.name.to_lowercase();
    
    // Move-specific secondary effects for stat modifications
    match move_name.as_str() {
        "acid" => {
            Some(create_defense_lowering_instructions(target_positions))
        }
        _ => None,
    }
}

/// Create instructions for lowering Defense by 1 stage
fn create_defense_lowering_instructions(target_positions: &[BattlePosition]) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .map(|&position| {
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Defense, -1);
            
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            })
        })
        .collect()
}

// =============================================================================
// HIGH-COST STAT-BOOSTING MOVES
// =============================================================================

/// Apply Fillet Away - boosts offensive stats but costs 1/2 HP
pub fn apply_fillet_away(
    state: &BattleState,
    _move_data: &MoveData,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let half_hp = user_pokemon.max_hp / 2;
        
        let mut instruction_list = vec![
            // Damage user for half their max HP
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: half_hp,
                previous_hp: Some(user_pokemon.hp),
            }),
            // Boost Attack, Special Attack, and Speed by 2 stages each
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: user_position,
                stat_changes: {
                    let mut boosts = HashMap::new();
                    boosts.insert(Stat::Attack, 2);
                    boosts.insert(Stat::SpecialAttack, 2);
                    boosts.insert(Stat::Speed, 2);
                    boosts
                },
                previous_boosts: HashMap::new(),
            }),
        ];
        
        instructions.push(BattleInstructions::new(100.0, instruction_list));
    }
    
    instructions
}

/// Apply Clangorous Soul - boosts all stats but costs 1/3 HP
pub fn apply_clangorous_soul(
    state: &BattleState,
    _move_data: &MoveData,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let third_hp = user_pokemon.max_hp / 3;
        
        let mut instruction_list = vec![
            // Damage user for 1/3 their max HP
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: third_hp,
                previous_hp: Some(user_pokemon.hp),
            }),
            // Boost all stats by 1 stage each
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: user_position,
                stat_changes: {
                    let mut boosts = HashMap::new();
                    boosts.insert(Stat::Attack, 1);
                    boosts.insert(Stat::Defense, 1);
                    boosts.insert(Stat::SpecialAttack, 1);
                    boosts.insert(Stat::SpecialDefense, 1);
                    boosts.insert(Stat::Speed, 1);
                    boosts
                },
                previous_boosts: HashMap::new(),
            }),
        ];
        
        instructions.push(BattleInstructions::new(100.0, instruction_list));
    }
    
    instructions
}