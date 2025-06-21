//! # Simplified Turn Resolution
//! 
//! This module replaces the complex generator hierarchy with simple functions
//! for turn resolution, following the modernization plan.

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::move_choice::MoveChoice;
use crate::core::battle_state::BattleState;
use crate::core::targeting::resolve_targets;
use crate::types::{BattleError, BattleResult};

// Compatibility layer for existing code
pub mod instruction_generator {
    use super::*;
    
    /// Compatibility wrapper for the old GenerationXInstructionGenerator
    pub struct GenerationXInstructionGenerator {
        format: BattleFormat,
    }
    
    impl GenerationXInstructionGenerator {
        pub fn new(format: BattleFormat) -> Self {
            Self { format }
        }
        
        pub fn generate_instructions(
            &self,
            state: &mut BattleState,
            side_one_choice: &MoveChoice,
            side_two_choice: &MoveChoice,
        ) -> Vec<BattleInstructions> {
            // Use the new simplified function
            super::generate_instructions(state, (side_one_choice, side_two_choice))
                .unwrap_or_else(|_| vec![BattleInstructions::new(100.0, vec![])])
        }
        
        pub fn generate_instructions_from_move_pair(
            &self,
            state: &mut BattleState,
            side_one_choice: &MoveChoice,
            side_two_choice: &MoveChoice,
        ) -> Vec<BattleInstructions> {
            // Alias for compatibility
            self.generate_instructions(state, side_one_choice, side_two_choice)
        }
        
        /// Generate modern battle instructions
        pub fn generate_modern_instructions(
            &self,
            state: &mut BattleState,
            side_one_choice: &MoveChoice,
            side_two_choice: &MoveChoice,
        ) -> Vec<BattleInstructions> {
            // Generate legacy instructions first, then convert
            let legacy_instructions = self.generate_instructions(state, side_one_choice, side_two_choice);
            
            // Convert to modern instructions
            legacy_instructions.into_iter().map(|legacy| {
                let modern_instructions: Vec<BattleInstruction> = legacy.instruction_list
                    .into_iter()
                    .map(|instr| instr.into())
                    .collect();
                
                BattleInstructions::new(legacy.percentage, modern_instructions)
            }).collect()
        }
        
        pub fn get_format(&self) -> &BattleFormat {
            &self.format
        }
        
        pub fn supports_format(&self, format: &BattleFormat) -> bool {
            use crate::core::battle_format::FormatType;
            matches!(format.format_type, 
                FormatType::Singles | 
                FormatType::Doubles | 
                FormatType::Vgc | 
                FormatType::Triples
            )
        }
    }
}

// Compatibility for end_of_turn module  
pub mod end_of_turn {
    use crate::core::instruction::{PokemonStatus, Weather};
    use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
    use crate::core::battle_state::BattleState;
    use crate::core::battle_format::BattlePosition;

    pub fn generate_end_of_turn_instructions() -> Vec<BattleInstructions> {
        // Simplified - just return empty for now
        vec![BattleInstructions::new(100.0, vec![])]
    }

    /// Generate end-of-turn effects for a given state (simplified implementation)
    pub fn process_end_of_turn_effects(state: &BattleState) -> Vec<BattleInstructions> {
        let mut instructions = Vec::new();
        
        // 1. Status damage (burn, poison, toxic)
        instructions.extend(process_status_damage(state));
        
        // 2. Weather effects (sandstorm, hail damage)
        instructions.extend(process_weather_damage(state));
        
        // If no effects, return empty instruction set
        if instructions.is_empty() {
            vec![BattleInstructions::new(100.0, vec![])]
        } else {
            instructions
        }
    }

    /// Process status damage for all active Pokemon
    fn process_status_damage(state: &BattleState) -> Vec<BattleInstructions> {
        let mut status_instructions = Vec::new();
        
        // Check side one
        for slot in 0..state.format.active_pokemon_count() {
            let position = BattlePosition::new(crate::core::battle_format::SideReference::SideOne, slot);
            if let Some(pokemon) = state.get_pokemon_at_position(position) {
                if let Some(damage_instruction) = calculate_status_damage(pokemon, position) {
                    status_instructions.push(BattleInstructions::new(
                        100.0,
                        vec![damage_instruction],
                    ));
                }
            }
        }
        
        // Check side two
        for slot in 0..state.format.active_pokemon_count() {
            let position = BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, slot);
            if let Some(pokemon) = state.get_pokemon_at_position(position) {
                if let Some(damage_instruction) = calculate_status_damage(pokemon, position) {
                    status_instructions.push(BattleInstructions::new(
                        100.0,
                        vec![damage_instruction],
                    ));
                }
            }
        }
        
        status_instructions
    }

    /// Calculate status damage for a Pokemon
    fn calculate_status_damage(
        pokemon: &crate::core::battle_state::Pokemon, 
        position: BattlePosition
    ) -> Option<BattleInstruction> {
        match pokemon.status {
            PokemonStatus::Burn => {
                let damage = pokemon.max_hp / 16; // 1/16 of max HP for burn
                Some(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: position,
                    amount: damage as i16,
                    previous_hp: Some(pokemon.hp),
                }))
            }
            PokemonStatus::Poison => {
                let damage = pokemon.max_hp / 8; // 1/8 of max HP for poison
                Some(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: position,
                    amount: damage as i16,
                    previous_hp: Some(pokemon.hp),
                }))
            }
            PokemonStatus::Toxic => {
                // Toxic damage increases each turn (1/16 * turn counter)
                let base_damage = pokemon.max_hp / 16;
                let turn_multiplier = 1; // Simplified - would need to track toxic counter
                let damage = base_damage * turn_multiplier;
                Some(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: position,
                    amount: damage as i16,
                    previous_hp: Some(pokemon.hp),
                }))
            }
            _ => None, // No damage for other status conditions
        }
    }

    /// Process weather damage effects
    fn process_weather_damage(state: &BattleState) -> Vec<BattleInstructions> {
        let mut weather_instructions = Vec::new();
        
        match state.weather {
            Weather::Sand => {
                // Sandstorm damages non-Ground/Rock/Steel types
                for slot in 0..state.format.active_pokemon_count() {
                    for side in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
                        let position = BattlePosition::new(side, slot);
                        if let Some(pokemon) = state.get_pokemon_at_position(position) {
                            // Check if Pokemon is immune to sandstorm
                            let is_immune = pokemon.types.iter().any(|t| {
                                matches!(t.as_str(), "Ground" | "Rock" | "Steel")
                            });
                            
                            if !is_immune {
                                let damage = pokemon.max_hp / 16; // 1/16 of max HP
                                weather_instructions.push(BattleInstructions::new(
                                    100.0,
                                    vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                                        target: position,
                                        amount: damage as i16,
                                        previous_hp: Some(pokemon.hp),
                                    })],
                                ));
                            }
                        }
                    }
                }
            }
            Weather::Hail => {
                // Hail damages non-Ice types
                for slot in 0..state.format.active_pokemon_count() {
                    for side in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
                        let position = BattlePosition::new(side, slot);
                        if let Some(pokemon) = state.get_pokemon_at_position(position) {
                            // Check if Pokemon is immune to hail
                            let is_immune = pokemon.types.iter().any(|t| t.as_str() == "Ice");
                            
                            if !is_immune {
                                let damage = pokemon.max_hp / 16; // 1/16 of max HP
                                weather_instructions.push(BattleInstructions::new(
                                    100.0,
                                    vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                                        target: position,
                                        amount: damage as i16,
                                        previous_hp: Some(pokemon.hp),
                                    })],
                                ));
                            }
                        }
                    }
                }
            }
            _ => {} // No damage for other weather conditions
        }
        
        weather_instructions
    }
}

/// Generate instructions for a complete turn with two move choices
pub fn generate_instructions(
    state: &BattleState,
    move_choices: (&MoveChoice, &MoveChoice),
) -> BattleResult<Vec<BattleInstructions>> {
    let (choice1, choice2) = move_choices;
    
    // Clone choices so we can modify them for auto-targeting
    let mut side_one_choice = choice1.clone();
    let mut side_two_choice = choice2.clone();

    // Auto-resolve targets using unified targeting system
    crate::core::targeting::auto_resolve_targets(
        SideReference::SideOne, 
        0, 
        &mut side_one_choice, 
        &state.format,
        state
    )?;

    crate::core::targeting::auto_resolve_targets(
        SideReference::SideTwo, 
        0, 
        &mut side_two_choice, 
        &state.format,
        state
    )?;
    
    // Determine move order based on priority and speed (including special switch/pursuit rules)
    let (first_side, first_choice, second_side, second_choice) = 
        determine_move_order_advanced(state, &side_one_choice, &side_two_choice);
    
    // Generate instructions for first move
    let first_instructions = generate_move_instructions(
        &first_choice, 
        first_side, 
        0, 
        &state.format, 
        state
    )?;
    
    // Handle switch-attack interactions
    let second_instructions = if first_choice.is_switch() && !second_choice.is_switch() {
        // First move is a switch, second is an attack - apply the switch first
        let mut temp_state = state.clone();
        
        // Apply switch instructions to get the final switched state
        if !first_instructions.is_empty() {
            // Use the first instruction set (switches are deterministic)
            temp_state.apply_instructions(&first_instructions[0].instruction_list);
        }
        
        // Generate second move instructions using the updated state
        generate_move_instructions(
            &second_choice, 
            second_side, 
            0, 
            &state.format, 
            &temp_state
        )?
    } else {
        // Either both are switches, first is not a switch, or second is a switch
        // In these cases, use the original state
        generate_move_instructions(
            &second_choice, 
            second_side, 
            0, 
            &state.format, 
            state
        )?
    };
    
    // Combine instruction sets from both moves to create all possible combinations
    let combined_instructions = combine_move_instructions(first_instructions, second_instructions);
    
    if combined_instructions.is_empty() {
        Ok(vec![BattleInstructions::new(100.0, vec![])])
    } else {
        Ok(combined_instructions)
    }
}

/// Generate instructions for a single move choice
pub fn generate_move_instructions(
    choice: &MoveChoice,
    user_side: SideReference,
    user_slot: usize,
    format: &BattleFormat,
    state: &BattleState,
) -> BattleResult<Vec<BattleInstructions>> {
    let user_pos = BattlePosition::new(user_side, user_slot);
    
    match choice {
        MoveChoice::Switch(pokemon_index) => {
            generate_switch_instructions(pokemon_index.to_index(), user_pos, state)
        }
        MoveChoice::Move { move_index, target_positions } => {
            generate_attack_instructions(*move_index, target_positions, user_pos, format, state)
        }
        MoveChoice::MoveTera { move_index, target_positions, .. } => {
            // For now, treat Tera moves the same as regular moves (simplified)
            generate_attack_instructions(*move_index, target_positions, user_pos, format, state)
        }
        MoveChoice::None => {
            Ok(vec![BattleInstructions::new(100.0, vec![])])
        }
    }
}

/// Generate instructions for a switch move
fn generate_switch_instructions(
    pokemon_index: usize,
    user_pos: BattlePosition,
    state: &BattleState,
) -> BattleResult<Vec<BattleInstructions>> {
    
    // Get current active pokemon index
    let current_index = state.get_side(user_pos.side.to_index())
        .and_then(|side| side.active_pokemon_indices.get(user_pos.slot))
        .and_then(|&idx| idx)
        .unwrap_or(0);
    
    // Simple switch instruction - no complex generator needed
    let switch_instruction = BattleInstruction::Pokemon(PokemonInstruction::Switch {
        position: user_pos,
        new_pokemon: pokemon_index,
        previous_pokemon: Some(current_index),
    });
    
    Ok(vec![BattleInstructions::new(
        100.0,
        vec![switch_instruction],
    )])
}

/// Generate instructions for an attack move
fn generate_attack_instructions(
    move_index: crate::core::move_choice::MoveIndex,
    explicit_targets: &[BattlePosition],
    user_pos: BattlePosition,
    format: &BattleFormat,
    state: &BattleState,
) -> BattleResult<Vec<BattleInstructions>> {
    // Get the move data
    let pokemon = state.get_side(user_pos.side.to_index())
        .and_then(|side| side.get_active_pokemon_at_slot(user_pos.slot))
        .ok_or_else(|| BattleError::InvalidState {
            reason: "No active Pokemon at user position".to_string(),
        })?;
    
    let move_data = pokemon.get_move(move_index)
        .ok_or_else(|| BattleError::InvalidState {
            reason: "Move not found on Pokemon".to_string(),
        })?;
    
    // Resolve targets if not explicitly provided
    let targets = if explicit_targets.is_empty() {
        resolve_targets(move_data.target, user_pos, format, state)
    } else {
        explicit_targets.to_vec()
    };
    
    // Generate different instruction sets based on critical hit probability
    let mut instruction_sets = Vec::new();
    
    // Non-critical hit (93.75% chance for most moves)
    let normal_instructions = generate_damage_instructions(
        move_data, 
        &targets, 
        user_pos, 
        state, 
        false
    )?;
    instruction_sets.push(BattleInstructions::new(
        93.75,
        normal_instructions,
    ));
    
    // Critical hit (6.25% chance for most moves)
    let crit_instructions = generate_damage_instructions(
        move_data, 
        &targets, 
        user_pos, 
        state, 
        true
    )?;
    instruction_sets.push(BattleInstructions::new(
        6.25,
        crit_instructions,
    ));
    
    Ok(instruction_sets)
}

/// Generate damage instructions for a move
fn generate_damage_instructions(
    move_data: &crate::core::battle_state::Move,
    targets: &[BattlePosition],
    user_pos: BattlePosition,
    state: &BattleState,
    is_critical: bool,
) -> BattleResult<Vec<BattleInstruction>> {
    use crate::core::instructions::{BattleInstruction, PokemonInstruction};
    
    let mut instructions = Vec::new();
    
    // Status moves don't deal damage
    if move_data.base_power == 0 {
        return Ok(instructions);
    }
    
    // Get attacker pokemon
    let attacker = state.get_side(user_pos.side.to_index())
        .and_then(|side| side.get_active_pokemon_at_slot(user_pos.slot))
        .ok_or_else(|| BattleError::InvalidState {
            reason: "No attacker pokemon found".to_string(),
        })?;
    
    if targets.len() == 1 {
        // Single target move
        let target = targets[0];
        
        // Get defender pokemon
        let defender = state.get_side(target.side.to_index())
            .and_then(|side| side.get_active_pokemon_at_slot(target.slot))
            .ok_or_else(|| BattleError::InvalidState {
                reason: "No defender pokemon found".to_string(),
            })?;
        
        // Simple damage calculation (simplified for modernization)
        let base_damage = if move_data.base_power > 0 {
            let attack_stat = if move_data.category == crate::core::battle_state::MoveCategory::Physical {
                attacker.stats.attack
            } else {
                attacker.stats.special_attack
            };
            
            let defense_stat = if move_data.category == crate::core::battle_state::MoveCategory::Physical {
                defender.stats.defense
            } else {
                defender.stats.special_defense
            };
            
            let level_factor = 2.0 * attacker.level as f32 / 5.0 + 2.0;
            let damage = level_factor * move_data.base_power as f32 * attack_stat as f32 / defense_stat as f32 / 50.0 + 2.0;
            
            if is_critical {
                (damage * 1.5) as i16
            } else {
                damage as i16
            }
        } else {
            0
        };
        
        instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target,
            amount: base_damage,
            previous_hp: None, // Will be filled in during execution
        }));
    } else if targets.len() > 1 {
        // Multi-target move
        let mut target_damages = Vec::new();
        
        for &target in targets {
            // Get defender pokemon
            let defender = state.get_side(target.side.to_index())
                .and_then(|side| side.get_active_pokemon_at_slot(target.slot));
            
            if let Some(defender) = defender {
                // Simple damage calculation
                let base_damage = if move_data.base_power > 0 {
                    let attack_stat = if move_data.category == crate::core::battle_state::MoveCategory::Physical {
                        attacker.stats.attack
                    } else {
                        attacker.stats.special_attack
                    };
                    
                    let defense_stat = if move_data.category == crate::core::battle_state::MoveCategory::Physical {
                        defender.stats.defense
                    } else {
                        defender.stats.special_defense
                    };
                    
                    let level_factor = 2.0 * attacker.level as f32 / 5.0 + 2.0;
                    let damage = level_factor * move_data.base_power as f32 * attack_stat as f32 / defense_stat as f32 / 50.0 + 2.0;
                    
                    let mut final_damage = if is_critical {
                        (damage * 1.5) as i16
                    } else {
                        damage as i16
                    };
                    
                    // Apply spread move damage reduction in doubles/VGC
                    if targets.len() > 1 && state.format.active_pokemon_count() > 1 {
                        final_damage = (final_damage as f32 * 0.75) as i16;  // 25% reduction for spread moves
                    }
                    
                    final_damage
                } else {
                    0
                };
                
                target_damages.push((target, base_damage));
            }
        }
        
        instructions.push(BattleInstruction::Pokemon(PokemonInstruction::MultiTargetDamage {
            target_damages,
            previous_hps: vec![], // Will be filled in during execution
        }));
    }
    
    Ok(instructions)
}

/// Determine which move goes first based on priority and speed (advanced version with Pursuit handling)
fn determine_move_order_advanced(
    state: &BattleState,
    side_one_choice: &MoveChoice,
    side_two_choice: &MoveChoice,
) -> (SideReference, MoveChoice, SideReference, MoveChoice) {
    // Special handling for switches (following poke-engine logic)
    if side_one_choice.is_switch() && side_two_choice.is_switch() {
        // Both switches - use speed to determine order
        let side_one_speed = get_effective_speed(state, SideReference::SideOne);
        let side_two_speed = get_effective_speed(state, SideReference::SideTwo);
        
        if side_one_speed > side_two_speed {
            return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
        } else if side_one_speed == side_two_speed {
            // Speed tie - side one wins for now (could implement random choice)
            return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
        } else {
            return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
        }
    } else if side_one_choice.is_switch() {
        // Side one switching - switch goes first unless opponent uses Pursuit
        if is_pursuit(state, side_two_choice, SideReference::SideTwo) {
            // Pursuit hits the switching Pokemon first
            return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
        } else {
            // Switch goes first
            return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
        }
    } else if side_two_choice.is_switch() {
        // Side two switching - switch goes first unless opponent uses Pursuit
        if is_pursuit(state, side_one_choice, SideReference::SideOne) {
            // Pursuit hits the switching Pokemon first
            return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
        } else {
            // Switch goes first
            return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
        }
    }

    // Neither choice is a switch - use normal priority/speed rules
    let side_one_priority = get_move_priority(state, side_one_choice, SideReference::SideOne);
    let side_two_priority = get_move_priority(state, side_two_choice, SideReference::SideTwo);
    
    // Higher priority goes first
    if side_one_priority > side_two_priority {
        return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
    } else if side_two_priority > side_one_priority {
        return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
    }

    // Same priority - compare speed
    let side_one_speed = get_effective_speed(state, SideReference::SideOne);
    let side_two_speed = get_effective_speed(state, SideReference::SideTwo);
    
    if side_one_speed > side_two_speed {
        (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone())
    } else if side_one_speed == side_two_speed {
        // Speed tie - side one wins for now (could implement random choice)
        (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone())
    } else {
        (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone())
    }
}

/// Check if a move choice is Pursuit
fn is_pursuit(state: &BattleState, choice: &MoveChoice, side: SideReference) -> bool {
    if let Some(move_index) = choice.move_index() {
        let pokemon = state.get_side(side.to_index()).and_then(|s| s.get_active_pokemon_at_slot(0));
        if let Some(pokemon) = pokemon {
            if let Some(move_data) = pokemon.get_move(move_index) {
                return move_data.name.to_lowercase() == "pursuit";
            }
        }
    }
    false
}

/// Combine instruction sets from two moves to create all possible combinations
fn combine_move_instructions(
    first_instructions: Vec<BattleInstructions>,
    second_instructions: Vec<BattleInstructions>,
) -> Vec<BattleInstructions> {
    if first_instructions.is_empty() && second_instructions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    } else if first_instructions.is_empty() {
        return second_instructions;
    } else if second_instructions.is_empty() {
        return first_instructions;
    }

    let mut combined = Vec::new();
    
    // Create combinations by combining each first instruction with each second instruction
    for first_instr in &first_instructions {
        for second_instr in &second_instructions {
            let mut combined_instruction_list = first_instr.instruction_list.clone();
            combined_instruction_list.extend(second_instr.instruction_list.clone());
            
            // Calculate combined probability (multiply percentages and normalize to 100)
            let combined_percentage = (first_instr.percentage * second_instr.percentage) / 100.0;
            
            combined.push(BattleInstructions::new(
                combined_percentage,
                combined_instruction_list,
            ));
        }
    }
    
    combined
}

/// Determine which move goes first based on priority and speed (simple version)
fn determine_move_order<'a>(
    state: &BattleState,
    choice1: &'a MoveChoice,
    choice2: &'a MoveChoice,
) -> (SideReference, &'a MoveChoice, SideReference, &'a MoveChoice) {
    // Switches generally go first (simplified rule)
    if choice1.is_switch() && !choice2.is_switch() {
        return (SideReference::SideOne, choice1, SideReference::SideTwo, choice2);
    } else if !choice1.is_switch() && choice2.is_switch() {
        return (SideReference::SideTwo, choice2, SideReference::SideOne, choice1);
    }
    
    // Both switches or both moves - compare priority then speed
    let priority1 = get_move_priority(state, choice1, SideReference::SideOne);
    let priority2 = get_move_priority(state, choice2, SideReference::SideTwo);
    
    if priority1 > priority2 {
        (SideReference::SideOne, choice1, SideReference::SideTwo, choice2)
    } else if priority2 > priority1 {
        (SideReference::SideTwo, choice2, SideReference::SideOne, choice1)
    } else {
        // Same priority - compare speed
        let speed1 = get_effective_speed(state, SideReference::SideOne);
        let speed2 = get_effective_speed(state, SideReference::SideTwo);
        
        if speed1 >= speed2 {
            (SideReference::SideOne, choice1, SideReference::SideTwo, choice2)
        } else {
            (SideReference::SideTwo, choice2, SideReference::SideOne, choice1)
        }
    }
}

/// Get move priority for a choice
fn get_move_priority(state: &BattleState, choice: &MoveChoice, side: SideReference) -> i8 {
    if let Some(move_index) = choice.move_index() {
        let pokemon = state.get_side(side.to_index()).and_then(|s| s.get_active_pokemon_at_slot(0));
        if let Some(pokemon) = pokemon {
            if let Some(move_data) = pokemon.get_move(move_index) {
                return move_data.priority;
            }
        }
    }
    0 // Default priority
}

/// Get effective speed for a side
fn get_effective_speed(state: &BattleState, side: SideReference) -> i16 {
    if let Some(pokemon) = state.get_side(side.to_index()).and_then(|s| s.get_active_pokemon_at_slot(0)) {
        pokemon.get_effective_stat(crate::core::instruction::Stat::Speed) as i16
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::{Pokemon, Move, MoveCategory};
    use crate::core::move_choice::MoveIndex;
    use crate::core::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;
    use crate::data::showdown_types::MoveTarget;

    fn create_test_state() -> BattleState {
        let mut state = BattleState::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        
        // Add Pokemon to both sides
        let mut pokemon1 = Pokemon::new("Attacker".to_string());
        let pokemon2 = Pokemon::new("Defender".to_string());
        
        // Add a basic attack move
        let tackle = Move::new_with_details(
            "Tackle".to_string(),
            40,
            100,
            "Normal".to_string(),
            35,
            MoveTarget::Normal,
            MoveCategory::Physical,
            0,
        );
        
        pokemon1.add_move(MoveIndex::M0, tackle);
        
        state.side_one.add_pokemon(pokemon1);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(pokemon2);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));
        
        state
    }

    #[test]
    fn test_simple_turn_generation() {
        let state = create_test_state();
        
        let move_choice = MoveChoice::new_move(
            MoveIndex::M0,
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );
        let no_move = MoveChoice::None;
        
        let instructions = generate_instructions(&state, (&move_choice, &no_move));
        
        assert!(instructions.is_ok());
        let instructions = instructions.unwrap();
        assert!(!instructions.is_empty());
        
        // Should have critical hit branching
        assert!(instructions.len() >= 2);
    }

    #[test]
    fn test_switch_generation() {
        let mut state = create_test_state();
        
        // Add a second Pokemon to switch to
        let pokemon3 = Pokemon::new("Switcher".to_string());
        state.side_one.add_pokemon(pokemon3);
        
        let switch_choice = MoveChoice::Switch(crate::core::move_choice::PokemonIndex::P1);
        let no_move = MoveChoice::None;
        
        let instructions = generate_instructions(&state, (&switch_choice, &no_move));
        
        assert!(instructions.is_ok());
        let instructions = instructions.unwrap();
        assert!(!instructions.is_empty());
        
        // Should contain switch instruction
        let has_switch = instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, BattleInstruction::Pokemon(PokemonInstruction::Switch { .. }))
            })
        });
        assert!(has_switch);
    }

    #[test]
    fn test_move_order_priority() {
        let state = create_test_state();
        
        let high_priority = MoveChoice::new_move(MoveIndex::M0, vec![]);
        let low_priority = MoveChoice::new_move(MoveIndex::M0, vec![]);
        
        let (first_side, _, second_side, _) = determine_move_order(
            &state, 
            &high_priority, 
            &low_priority
        );
        
        // Should maintain order when priorities are equal
        assert_eq!(first_side, SideReference::SideOne);
        assert_eq!(second_side, SideReference::SideTwo);
    }
}