//! Status Effect Functions
//! 
//! This module contains status effect move implementations extracted from move_effects.rs
//! These functions handle major status conditions like paralysis, sleep, poison, and burn.

use crate::core::battle_state::{Pokemon, BattleState};
use crate::core::instructions::PokemonStatus;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

// =============================================================================
// STATUS MOVES THAT INFLICT MAJOR STATUS CONDITIONS
// =============================================================================

/// Apply Thunder Wave - paralyzes the target
/// Generation-aware: Electric types become immune to paralysis in Gen 6+
pub fn apply_thunder_wave(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check if target can be paralyzed
            if target.status == PokemonStatus::None {
                // Check for Electric immunity (Ground types in early gens)
                if !is_immune_to_paralysis(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Paralysis,
                        duration: Some(1), // Default paralysis duration
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    // Move has no effect
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                // Already has a status condition
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Sleep Powder - puts target to sleep
/// Generation-aware: Grass types become immune to powder moves in Gen 6+
pub fn apply_sleep_powder(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Grass immunity or Overcoat/Safety Goggles
                if !is_immune_to_powder(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Sleep,
                        duration: Some(1), // Default sleep duration
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Toxic - badly poisons the target
/// Generation-aware: Steel types become immune to poison in Gen 2+
pub fn apply_toxic(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Poison/Steel immunity
                if !is_immune_to_poison(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Toxic,
                        duration: None,
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Will-O-Wisp - burns the target
/// Generation-aware: Fire types are always immune to burn
pub fn apply_will_o_wisp(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Fire immunity
                if !is_immune_to_burn(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Burn,
                        duration: None,
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Stun Spore - paralyzes the target
/// Generation-aware: Grass types immune to powder moves in Gen 6+, Electric types immune to paralysis in Gen 6+
pub fn apply_stun_spore(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) && !is_immune_to_paralysis(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Paralysis,
                        duration: Some(1), // Default paralysis duration
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Poison Powder - poisons the target
/// Generation-aware: Grass types immune to powder moves in Gen 6+, Poison/Steel types immune to poison
pub fn apply_poison_powder(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) && !is_immune_to_poison(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Poison,
                        duration: None,
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Glare - paralyzes the target
/// Generation-aware: Not affected by Electric immunity like Thunder Wave
pub fn apply_glare(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Glare can paralyze Electric types (unlike Thunder Wave in Gen 6+)
                let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                    target: target_position,
                    status: PokemonStatus::Paralysis,
                    duration: None,
                    previous_status: Some(target.status),
                    previous_duration: target.status_duration,
                });
                instructions.push(BattleInstructions::new(100.0, vec![instruction]));
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Spore - 100% sleep move
/// Generation-aware: Grass types immune to powder moves in Gen 6+
pub fn apply_spore(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) {
                    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                        target: target_position,
                        status: PokemonStatus::Sleep,
                        duration: Some(1), // Default sleep duration
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(BattleInstructions::new(100.0, vec![]));
                }
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS FOR STATUS IMMUNITY CHECKING
// =============================================================================

/// Check if a Pokemon is immune to paralysis
/// Generation-aware: Electric types become immune to paralysis in Gen 6+
fn is_immune_to_paralysis(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    if generation.generation.number() >= 6 {
        // Gen 6+: Electric types are immune to paralysis
        pokemon.types.iter().any(|t| t.to_lowercase() == "electric")
    } else {
        // Earlier gens: no electric immunity to paralysis
        false
    }
}

/// Check if a Pokemon is immune to powder moves
/// Generation-aware: Grass types become immune to powder moves in Gen 6+
/// Also checks for Overcoat ability and Safety Goggles item
fn is_immune_to_powder(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Check for Overcoat ability (immune to powder moves in all generations)
    if pokemon.ability.to_lowercase() == "overcoat" {
        return true;
    }
    
    // Check for Safety Goggles item (immune to powder moves and weather damage)
    if let Some(item) = &pokemon.item {
        if item.to_lowercase() == "safetygoggles" {
            return true;
        }
    }
    
    if generation.generation.number() >= 6 {
        // Gen 6+: Grass types are immune to powder moves
        pokemon.types.iter().any(|t| t.to_lowercase() == "grass")
    } else {
        // Earlier gens: no grass immunity to powder moves
        false
    }
}

/// Check if a Pokemon is immune to poison
/// Generation-aware: Steel types become immune to poison in Gen 2+
fn is_immune_to_poison(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Poison types are always immune to poison
    let is_poison_type = pokemon.types.iter().any(|t| t.to_lowercase() == "poison");
    
    if generation.generation.number() >= 2 {
        // Gen 2+: Steel types are also immune to poison
        let is_steel_type = pokemon.types.iter().any(|t| t.to_lowercase() == "steel");
        is_poison_type || is_steel_type
    } else {
        // Gen 1: Only Poison types are immune
        is_poison_type
    }
}

/// Check if a Pokemon is immune to burn
/// Generation-aware: Fire types are always immune to burn
fn is_immune_to_burn(pokemon: &Pokemon, _generation: &GenerationMechanics) -> bool {
    // Fire types are immune to burn in all generations
    pokemon.types.iter().any(|t| t.to_lowercase() == "fire")
}