//! # Simplified Turn Resolution
//! 
//! This module replaces the complex generator hierarchy with simple functions
//! for turn resolution, following the modernization plan.

use std::collections::HashMap;

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction, Weather};
use crate::core::move_choice::MoveChoice;
use crate::core::targeting::resolve_targets;
use crate::data::showdown_types::MoveTarget;
use crate::engine::combat::moves::{MoveContext, OpponentMoveInfo};
use crate::types::{BattleError, BattleResult};

// Note: parse_move_target function removed - now using type-safe MoveTarget enum throughout

// Compatibility for end_of_turn module  
pub mod end_of_turn {
    use crate::core::instructions::{PokemonStatus, Weather};
    use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
    use crate::core::battle_state::BattleState;
    use crate::core::battle_format::BattlePosition;


    /// Generate end-of-turn effects for a given state using comprehensive pipeline
    pub fn process_end_of_turn_effects(state: &BattleState) -> Vec<BattleInstructions> {
        // Use the new comprehensive end-of-turn processing pipeline
        crate::engine::combat::core::end_of_turn::generate_end_of_turn_instructions(state)
    }

}

/// Generate instructions for a complete turn with two move choices
pub fn generate_instructions(
    state: &BattleState,
    move_choices: (&MoveChoice, &MoveChoice),
    branch_on_damage: bool,
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
    
    // Create comprehensive move contexts with opponent information
    let first_context = create_move_context_with_opponents(
        &first_choice,
        &second_choice,
        first_side,
        second_side,
        state,
        true, // This move goes first
    );
    
    let second_context = create_move_context_with_opponents(
        &second_choice,
        &first_choice,
        second_side,
        first_side,
        state,
        false, // This move goes second
    );
    
    // Generate instructions for first move (with context indicating it goes first)
    let first_instructions = generate_move_instructions_with_enhanced_context(
        &first_choice, 
        first_side, 
        0, 
        &state.format, 
        state,
        &first_context,
        branch_on_damage,
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
        
        // Generate second move instructions using the updated state (goes second)
        generate_move_instructions_with_enhanced_context(
            &second_choice, 
            second_side, 
            0, 
            &state.format, 
            &temp_state,
            &second_context,
            branch_on_damage,
        )?
    } else {
        // Either both are switches, first is not a switch, or second is a switch
        // In these cases, use the original state (goes second)
        generate_move_instructions_with_enhanced_context(
            &second_choice, 
            second_side, 
            0, 
            &state.format, 
            state,
            &second_context,
            branch_on_damage,
        )?
    };
    
    // Combine instruction sets from both moves with move cancellation logic
    let combined_instructions = combine_move_instructions_with_cancellation(
        first_instructions, 
        second_instructions, 
        state,
        &second_choice,
        second_side,
    )?;
    
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
    // Default to no specific turn order context
    generate_move_instructions_with_context(choice, user_side, user_slot, format, state, false)
}

/// Generate instructions for a single move choice with turn order context
pub fn generate_move_instructions_with_context(
    choice: &MoveChoice,
    user_side: SideReference,
    user_slot: usize,
    format: &BattleFormat,
    state: &BattleState,
    going_first: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    let user_pos = BattlePosition::new(user_side, user_slot);
    
    match choice {
        MoveChoice::Switch(pokemon_index) => {
            generate_switch_instructions(pokemon_index.to_index(), user_pos, state)
        }
        MoveChoice::Move { move_index, target_positions } => {
            generate_attack_instructions_with_context(*move_index, target_positions, user_pos, format, state, going_first)
        }
        MoveChoice::MoveTera { move_index, target_positions, .. } => {
            // Generate Terastallization instruction first
            let mut all_instructions = Vec::new();
            
            // Check if the Pokemon can Terastallize
            if let Some(user_pokemon) = state.get_pokemon_at_position(user_pos) {
                if !user_pokemon.is_terastallized && user_pokemon.tera_type.is_some() {
                    // Apply Terastallization
                    let tera_instruction = BattleInstructions::new(
                        100.0,
                        vec![BattleInstruction::Pokemon(
                            crate::core::instructions::PokemonInstruction::ToggleTerastallized {
                                target: user_pos,
                                terastallized: true,
                                tera_type: user_pokemon.tera_type,
                                previous_state: user_pokemon.is_terastallized,
                            }
                        )]
                    );
                    all_instructions.push(tera_instruction);
                }
            }
            
            // Then generate the move instructions with enhanced power
            let move_instructions = generate_attack_instructions_with_context(*move_index, target_positions, user_pos, format, state, going_first)?;
            all_instructions.extend(move_instructions);
            
            Ok(all_instructions)
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
    // Default to no specific turn order context
    generate_attack_instructions_with_context(move_index, explicit_targets, user_pos, format, state, false)
}

/// Generate instructions for an attack move with turn order context
fn generate_attack_instructions_with_context(
    move_index: crate::core::move_choice::MoveIndex,
    explicit_targets: &[BattlePosition],
    user_pos: BattlePosition,
    format: &BattleFormat,
    state: &BattleState,
    going_first: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    use crate::engine::combat::core::move_prevention::{cannot_use_move, generate_prevention_instructions};
    
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
    
    // Check for move prevention before proceeding
    let move_choice = crate::core::move_choice::MoveChoice::Move {
        move_index,
        target_positions: explicit_targets.to_vec(),
    };
    
    if let Some(prevention) = cannot_use_move(pokemon, &move_choice, None, state, user_pos) {
        return Ok(generate_prevention_instructions(prevention, user_pos, pokemon));
    }
    
    // Resolve targets if not explicitly provided
    let targets = if explicit_targets.is_empty() {
        resolve_targets(move_data.target, user_pos, format, state)
    } else {
        explicit_targets.to_vec()
    };
    
    // Check move accuracy
    let accuracy_percentage = calculate_move_accuracy(move_data, user_pos, &targets, state, going_first);
    
    let mut instruction_sets = Vec::new();
    
    // If move can miss, create miss instruction set
    if accuracy_percentage < 100.0 {
        let miss_percentage = 100.0 - accuracy_percentage;
        instruction_sets.push(BattleInstructions::new(
            miss_percentage,
            vec![], // Move misses - no damage/effects
        ));
    }
    
    // Only generate hit instructions if move can hit
    if accuracy_percentage > 0.0 {
        // Determine if we should branch on damage (following poke-engine logic)
        // Branch on damage for moves with variable damage ranges or critical hits
        let move_name = move_data.name.as_str().to_lowercase();
        let branch_on_damage = move_data.base_power > 0 && 
            (move_name.contains("variable") || 
             move_name.contains("random"));
        
        // Generate hit instructions with secondary effects
        let hit_instructions = generate_hit_instructions_with_secondary_effects(
            move_data,
            &targets,
            user_pos,
            state,
            accuracy_percentage,
            branch_on_damage,
        )?;
        instruction_sets.extend(hit_instructions);
    }
    
    Ok(instruction_sets)
}

/// Generate damage instructions for a move with poke-engine style damage calculation
fn generate_damage_instructions(
    move_data: &crate::core::battle_state::Move,
    targets: &[BattlePosition],
    user_pos: BattlePosition,
    state: &BattleState,
    is_critical: bool,
) -> BattleResult<Vec<BattleInstruction>> {
    return generate_damage_instructions_with_rolls(
        move_data, targets, user_pos, state, is_critical, false
    );
}

/// Generate damage instructions with poke-engine damage roll logic
fn generate_damage_instructions_with_rolls(
    move_data: &crate::core::battle_state::Move,
    targets: &[BattlePosition],
    user_pos: BattlePosition,
    state: &BattleState,
    is_critical: bool,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstruction>> {
    use crate::core::instructions::{BattleInstruction, PokemonInstruction};
    use crate::engine::combat::damage::{calculate_damage_with_positions, DamageRolls, compare_health_with_damage_multiples};
    
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
    
    for &target in targets {
        // Get defender pokemon
        let defender = state.get_side(target.side.to_index())
            .and_then(|side| side.get_active_pokemon_at_slot(target.slot))
            .ok_or_else(|| BattleError::InvalidState {
                reason: "No defender pokemon found".to_string(),
            })?;
        
        // Convert move data for damage calculation
        let move_data_modern = crate::data::showdown_types::MoveData {
            name: move_data.name,
            base_power: move_data.base_power as u16,
            move_type: move_data.move_type.clone(),
            category: move_data.category,
            accuracy: 100, // Accuracy already handled in calling function
            priority: move_data.priority,
            pp: move_data.pp,
            target: crate::data::showdown_types::MoveTarget::Normal,
            ..Default::default()
        };
        
        // Calculate damage using modern damage system
        let damage = if branch_on_damage {
            // Check if we should use 16-roll branching logic
            let max_damage = calculate_damage_with_positions(
                state, attacker, defender, &move_data_modern,
                is_critical, DamageRolls::Max, targets.len(),
                user_pos, target
            );
            let min_damage = calculate_damage_with_positions(
                state, attacker, defender, &move_data_modern,
                is_critical, DamageRolls::Min, targets.len(),
                user_pos, target
            );
            
            // Check if damage range could result in different outcomes
            if max_damage >= defender.hp && min_damage < defender.hp {
                // Use 16-roll branching logic
                let (average_non_kill_damage, _num_kill_rolls) = 
                    compare_health_with_damage_multiples(max_damage, defender.hp);
                average_non_kill_damage
            } else {
                // No branching needed, use average
                calculate_damage_with_positions(
                    state, attacker, defender, &move_data_modern,
                    is_critical, DamageRolls::Average, targets.len(),
                    user_pos, target
                )
            }
        } else {
            // Use average damage for deterministic calculation
            calculate_damage_with_positions(
                state, attacker, defender, &move_data_modern,
                is_critical, DamageRolls::Average, targets.len(),
                user_pos, target
            )
        };
        
        instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target,
            amount: damage,
            previous_hp: None, // Will be filled in during execution
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
                return move_data.name == crate::types::Moves::PURSUIT;
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

/// Combine instruction sets with move cancellation logic
fn combine_move_instructions_with_cancellation(
    first_instructions: Vec<BattleInstructions>,
    second_instructions: Vec<BattleInstructions>,
    initial_state: &BattleState,
    second_choice: &MoveChoice,
    second_side: SideReference,
) -> BattleResult<Vec<BattleInstructions>> {
    if first_instructions.is_empty() && second_instructions.is_empty() {
        return Ok(vec![BattleInstructions::new_with_positions(100.0, vec![], vec![])]);
    } else if first_instructions.is_empty() {
        return Ok(second_instructions);
    } else if second_instructions.is_empty() {
        return Ok(first_instructions);
    }

    let mut combined = Vec::new();
    
    // For each first instruction, check if second move should be cancelled
    for first_instr in &first_instructions {
        // Apply first move to a temporary state
        let mut temp_state = initial_state.clone();
        temp_state.apply_instructions(&first_instr.instruction_list);
        
        // Check if second move should be cancelled
        if should_cancel_move(&temp_state, second_choice, second_side) {
            // Second move is cancelled - only include first move's instructions
            combined.push(BattleInstructions::new_with_positions(
                first_instr.percentage,
                first_instr.instruction_list.clone(),
                first_instr.affected_positions.clone(),
            ));
        } else {
            // Second move can proceed - combine both instruction sets
            for second_instr in &second_instructions {
                let mut combined_instruction_list = first_instr.instruction_list.clone();
                combined_instruction_list.extend(second_instr.instruction_list.clone());
                
                // Calculate combined probability
                let combined_percentage = (first_instr.percentage * second_instr.percentage) / 100.0;
                
                // Combine affected positions from both instruction sets
                let mut combined_affected_positions = first_instr.affected_positions.clone();
                combined_affected_positions.extend(second_instr.affected_positions.clone());
                combined_affected_positions.sort();
                combined_affected_positions.dedup();
                
                combined.push(BattleInstructions::new_with_positions(
                    combined_percentage,
                    combined_instruction_list,
                    combined_affected_positions,
                ));
            }
        }
    }
    
    Ok(combined)
}

/// Check if a move should be cancelled due to target fainting or other conditions
fn should_cancel_move(
    state: &BattleState,
    choice: &MoveChoice,
    user_side: SideReference,
) -> bool {
    use crate::core::instructions::VolatileStatus;
    
    // Check if the attacker has fainted
    let user_pos = BattlePosition::new(user_side, 0);
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_pos) {
        if user_pokemon.hp == 0 {
            return true; // Cancel move if attacker has fainted
        }
        
        // Check if the attacker is flinched (flinch prevents move execution)
        if user_pokemon.volatile_statuses.contains(VolatileStatus::Flinch) {
            return true; // Cancel move if attacker is flinched
        }
    }
    
    // Only check for attack moves, not switches or status moves
    if let MoveChoice::Move { move_index, target_positions } = choice {
        // Get the move data to check if it's a status move
        if let Some(user_pokemon) = state.get_pokemon_at_position(user_pos) {
            if let Some(move_data) = user_pokemon.get_move(*move_index) {
                // Status moves can still be used even if target has fainted
                if move_data.base_power == 0 {
                    return false;
                }
                
                // For damage-dealing moves, check if any target has fainted
                for &target_pos in target_positions {
                    if let Some(target_pokemon) = state.get_pokemon_at_position(target_pos) {
                        if target_pokemon.hp == 0 {
                            return true; // Cancel move if target has fainted
                        }
                    } else {
                        return true; // Cancel move if target doesn't exist
                    }
                }
            }
        }
    }
    
    false
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
        let position = BattlePosition {
            side,
            slot: 0,
        };
        pokemon.get_effective_speed(state, position) as i16
    } else {
        0
    }
}

/// Calculate move accuracy including weather, ability, and item modifiers
fn calculate_move_accuracy(
    move_data: &crate::core::battle_state::Move,
    user_pos: BattlePosition,
    _targets: &[BattlePosition],
    state: &BattleState,
    going_first: bool,
) -> f32 {
    // Start with base move accuracy (0 means always hit, like status moves)
    let base_accuracy = if move_data.accuracy == 0 {
        100.0 // Always hit moves (status moves, etc.)
    } else {
        move_data.accuracy as f32 // Convert u8 to f32
    };
    
    let mut final_accuracy = base_accuracy;
    
    // Apply weather-based accuracy modifications
    final_accuracy = apply_weather_accuracy_modifiers(
        final_accuracy,
        move_data.name.as_str(),
        state.weather(),
    );
    
    // Apply ability modifiers (e.g., Compound Eyes, No Guard)
    final_accuracy = apply_ability_accuracy_modifiers(
        final_accuracy,
        user_pos,
        _targets,
        state,
    );
    
    // Apply item modifiers (e.g., Wide Lens, Zoom Lens)
    final_accuracy = apply_item_accuracy_modifiers(
        final_accuracy,
        user_pos,
        _targets,
        state,
        going_first,
    );
    
    // Apply stat stage modifiers (accuracy/evasion)
    final_accuracy = apply_stat_stage_accuracy_modifiers(
        final_accuracy,
        user_pos,
        _targets,
        state,
    );
    
    // Clamp to valid range
    final_accuracy.max(0.0).min(100.0)
}

/// Apply weather-specific accuracy modifications for certain moves
fn apply_weather_accuracy_modifiers(
    base_accuracy: f32,
    move_name: &str,
    weather: Weather,
) -> f32 {
    match move_name {
        "Blizzard" => {
            match weather {
                Weather::Hail | Weather::Snow => 100.0, // Perfect accuracy in hail/snow
                _ => base_accuracy,
            }
        }
        "Hurricane" => {
            match weather {
                Weather::Rain | Weather::HeavyRain => 100.0, // Perfect accuracy in rain
                Weather::Sun | Weather::HarshSun | Weather::HarshSunlight => 50.0, // Reduced accuracy in sun
                _ => base_accuracy,
            }
        }
        "Thunder" => {
            match weather {
                Weather::Rain | Weather::HeavyRain => 100.0, // Perfect accuracy in rain
                Weather::Sun | Weather::HarshSun | Weather::HarshSunlight => 50.0, // Reduced accuracy in sun
                _ => base_accuracy,
            }
        }
        _ => {
            // No weather modification for other moves
            base_accuracy
        }
    }
}

/// Apply ability-based accuracy modifiers
fn apply_ability_accuracy_modifiers(
    base_accuracy: f32,
    user_pos: BattlePosition,
    _targets: &[BattlePosition],
    state: &BattleState,
) -> f32 {
    if let Some(user) = state.get_pokemon_at_position(user_pos) {
        let ability_id = user.ability;
        match ability_id {
            crate::types::Abilities::COMPOUNDEYES => {
                // Compound Eyes increases accuracy by 30% (1.3x multiplier)
                base_accuracy * 1.3
            }
            crate::types::Abilities::NOGUARD => {
                // No Guard makes all moves hit regardless of accuracy
                100.0
            }
            _ => {
                // No ability-based accuracy modification
                base_accuracy
            }
        }
    } else {
        base_accuracy
    }
}

/// Apply item-based accuracy modifiers
fn apply_item_accuracy_modifiers(
    base_accuracy: f32,
    user_pos: BattlePosition,
    _targets: &[BattlePosition],
    state: &BattleState,
    going_first: bool,
) -> f32 {
    if let Some(user) = state.get_pokemon_at_position(user_pos) {
        if let Some(ref item) = user.item {
            match item {
                crate::types::Items::WIDELENS => {
                    // Wide Lens increases accuracy by 10% (1.1x multiplier)
                    base_accuracy * 1.1
                }
                crate::types::Items::ZOOMLENS => {
                    // Zoom Lens increases accuracy by 20% when moving after target
                    let moves_after_target = check_moves_after_target(user_pos, _targets, state, going_first);
                    if moves_after_target {
                        base_accuracy * 1.2 // 20% boost when moving after target
                    } else {
                        base_accuracy
                    }
                }
                _ => base_accuracy,
            }
        } else {
            base_accuracy
        }
    } else {
        base_accuracy
    }
}

/// Apply stat stage accuracy/evasion modifiers
fn apply_stat_stage_accuracy_modifiers(
    base_accuracy: f32,
    user_pos: BattlePosition,
    targets: &[BattlePosition],
    state: &BattleState,
) -> f32 {
    let mut final_accuracy = base_accuracy;
    
    // Apply user's accuracy stage
    if let Some(user) = state.get_pokemon_at_position(user_pos) {
        let accuracy_stage = user.stat_boosts.get_direct(crate::core::instructions::Stat::Accuracy);
        if accuracy_stage != 0 {
            let accuracy_multiplier = match accuracy_stage {
                -6 => 3.0 / 9.0,   // 33%
                -5 => 3.0 / 8.0,   // 37.5%
                -4 => 3.0 / 7.0,   // 43%
                -3 => 3.0 / 6.0,   // 50%
                -2 => 3.0 / 5.0,   // 60%
                -1 => 3.0 / 4.0,   // 75%
                0 => 1.0,          // 100%
                1 => 4.0 / 3.0,    // 133%
                2 => 5.0 / 3.0,    // 167%
                3 => 6.0 / 3.0,    // 200%
                4 => 7.0 / 3.0,    // 233%
                5 => 8.0 / 3.0,    // 267%
                6 => 9.0 / 3.0,    // 300%
                _ => 1.0,
            };
            final_accuracy *= accuracy_multiplier;
        }
    }
    
    // Apply target's evasion stage (if single target)
    if targets.len() == 1 {
        if let Some(target) = state.get_pokemon_at_position(targets[0]) {
            let evasion_stage = target.stat_boosts.get_direct(crate::core::instructions::Stat::Evasion);
            if evasion_stage != 0 {
                let evasion_multiplier = match evasion_stage {
                    -6 => 9.0 / 3.0,   // 300% (easier to hit)
                    -5 => 8.0 / 3.0,   // 267%
                    -4 => 7.0 / 3.0,   // 233%
                    -3 => 6.0 / 3.0,   // 200%
                    -2 => 5.0 / 3.0,   // 167%
                    -1 => 4.0 / 3.0,   // 133%
                    0 => 1.0,          // 100%
                    1 => 3.0 / 4.0,    // 75% (harder to hit)
                    2 => 3.0 / 5.0,    // 60%
                    3 => 3.0 / 6.0,    // 50%
                    4 => 3.0 / 7.0,    // 43%
                    5 => 3.0 / 8.0,    // 37.5%
                    6 => 3.0 / 9.0,    // 33%
                    _ => 1.0,
                };
                final_accuracy *= evasion_multiplier;
            }
        }
    }
    
    final_accuracy
}

/// Check if the user moves after the target (for items like Zoom Lens)
fn check_moves_after_target(
    user_pos: BattlePosition,
    targets: &[BattlePosition],
    state: &BattleState,
    going_first: bool,
) -> bool {
    // If we're going first in turn order, we don't move after target
    if going_first {
        return false;
    }
    
    // If no targets or single-target move, check if we move after the target
    if targets.len() == 1 {
        let target_pos = targets[0];
        
        // Get user and target Pokemon
        let user = state.get_pokemon_at_position(user_pos);
        let target = state.get_pokemon_at_position(target_pos);
        
        if let (Some(user_pokemon), Some(target_pokemon)) = (user, target) {
            // Compare effective speeds to determine turn order
            let user_speed = user_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed);
            let target_speed = target_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed);
            
            // User moves after target if target is faster
            return target_speed > user_speed;
        }
    }
    
    // For multi-target moves or unclear situations, assume no boost
    false
}

/// Create a MoveContext with opponent move information
fn create_move_context_with_opponents(
    own_choice: &MoveChoice,
    opponent_choice: &MoveChoice,
    own_side: SideReference,
    opponent_side: SideReference,
    state: &BattleState,
    going_first: bool,
) -> MoveContext {
    let mut context = MoveContext::new();
    context.going_first = going_first;
    
    // Create opponent position (assuming single slot for now, can be enhanced for doubles)
    let opponent_position = BattlePosition::new(opponent_side, 0);
    
    // Build opponent move info if it's an attack move
    if let MoveChoice::Move { move_index, target_positions } = opponent_choice {
        if let Some(opponent_pokemon) = state.get_pokemon_at_position(opponent_position) {
            if let Some(move_data) = opponent_pokemon.get_move(*move_index) {
                let opponent_info = OpponentMoveInfo {
                    move_name: move_data.name.as_str().to_string(),
                    move_category: move_data.category,
                    is_switching: false,
                    priority: move_data.priority,
                    targets: target_positions.clone(),
                };
                context.opponent_moves.insert(opponent_position, opponent_info);
            }
        }
    } else if matches!(opponent_choice, MoveChoice::Switch(_)) {
        // Opponent is switching
        let opponent_info = OpponentMoveInfo {
            move_name: "Switch".to_string(),
            move_category: crate::core::battle_state::MoveCategory::Status, // Switches are treated as status for Sucker Punch purposes
            is_switching: true,
            priority: 6, // Switches have highest priority
            targets: vec![],
        };
        context.opponent_moves.insert(opponent_position, opponent_info);
    }
    
    // Build turn order information
    context.turn_order = vec![
        (BattlePosition::new(own_side, 0), own_choice.clone()),
        (opponent_position, opponent_choice.clone()),
    ];
    
    context
}

/// Generate move instructions with enhanced context containing opponent information
fn generate_move_instructions_with_enhanced_context(
    choice: &MoveChoice,
    user_side: SideReference,
    user_slot: usize,
    format: &BattleFormat,
    state: &BattleState,
    context: &MoveContext,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    let user_pos = BattlePosition::new(user_side, user_slot);
    
    match choice {
        MoveChoice::Switch(pokemon_index) => {
            generate_switch_instructions(pokemon_index.to_index(), user_pos, state)
        }
        MoveChoice::Move { move_index, target_positions } => {
            generate_attack_instructions_with_enhanced_context(*move_index, target_positions, user_pos, format, state, context, branch_on_damage)
        }
        MoveChoice::MoveTera { move_index, target_positions, .. } => {
            // For now, treat Tera moves the same as regular moves (simplified)
            generate_attack_instructions_with_enhanced_context(*move_index, target_positions, user_pos, format, state, context, branch_on_damage)
        }
        MoveChoice::None => {
            Ok(vec![BattleInstructions::new(100.0, vec![])])
        }
    }
}

/// Generate attack instructions with enhanced context
fn generate_attack_instructions_with_enhanced_context(
    move_index: crate::core::move_choice::MoveIndex,
    explicit_targets: &[BattlePosition],
    user_pos: BattlePosition,
    format: &BattleFormat,
    state: &BattleState,
    context: &MoveContext,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    use crate::engine::combat::moves::apply_move_effects;
    use crate::engine::combat::core::move_prevention::{cannot_use_move, generate_prevention_instructions};
    use crate::generation::GenerationMechanics;
    
    // Get user Pokemon and move data
    let user_pokemon = state.get_pokemon_at_position(user_pos)
        .ok_or_else(|| BattleError::InvalidState { 
            reason: "No Pokemon at user position".to_string() 
        })?;
    
    let move_data_raw = user_pokemon.get_move(move_index)
        .ok_or_else(|| BattleError::InvalidMoveChoice { 
            reason: format!("Move index {:?} not found", move_index) 
        })?;
    
    // Convert Move to MoveData via generation-specific repository lookup
    let move_data = if let Some(gen_move_data) = state.generation_repo.find_move_by_name_for_generation(&move_data_raw.name.as_str(), state.format.generation.number()) {
        // Use generation-specific move data directly (already in showdown_types::MoveData format)
        gen_move_data.clone()
    } else {
        // Fallback to standard repository for moves not found in generation-specific data
        if let Some(repo_move_data) = state.game_data_repo.moves.find_by_name(&move_data_raw.name.as_str()) {
            // Convert repository::MoveData to showdown_types::MoveData
            crate::data::showdown_types::MoveData {
                name: repo_move_data.name,
                base_power: repo_move_data.base_power as u16,
                accuracy: repo_move_data.accuracy as u16,
                pp: repo_move_data.pp,
                max_pp: repo_move_data.max_pp,
                move_type: repo_move_data.move_type,
                category: repo_move_data.category.clone(),
                priority: repo_move_data.priority,
                target: repo_move_data.target.clone(),
                flags: repo_move_data.flags.clone(),
                drain: repo_move_data.drain,
                recoil: repo_move_data.recoil,
                ..crate::data::showdown_types::MoveData::default()
            }
        } else {
        // Fallback: create a basic MoveData from the Move
        crate::data::showdown_types::MoveData {
            name: move_data_raw.name.clone(),
            base_power: move_data_raw.base_power as u16,
            accuracy: move_data_raw.accuracy as u16,
            pp: move_data_raw.pp,
            max_pp: move_data_raw.max_pp,
            move_type: move_data_raw.move_type.clone(),
            category: move_data_raw.category,
            priority: move_data_raw.priority,
            target: move_data_raw.target,
            ..crate::data::showdown_types::MoveData::default()
        }
        }
    };
    
    // 1. Pre-move checks (status prevention)
    let move_choice = crate::core::move_choice::MoveChoice::Move {
        move_index,
        target_positions: explicit_targets.to_vec(),
    };
    
    if let Some(prevention) = cannot_use_move(user_pokemon, &move_choice, Some(&move_data), state, user_pos) {
        return Ok(generate_prevention_instructions(prevention, user_pos, user_pokemon));
    }
    
    // Determine targets using the same logic as before
    let targets = if explicit_targets.is_empty() {
        resolve_targets(move_data.target, user_pos, format, state)
    } else {
        explicit_targets.to_vec()
    };
    
    // 2. Check move accuracy (CRITICAL: this was missing!)
    let accuracy_percentage = calculate_move_accuracy(move_data_raw, user_pos, &targets, state, context.going_first);
    
    let mut instruction_sets = Vec::new();
    
    // 3. If move can miss, create miss instruction set
    if accuracy_percentage < 100.0 {
        let miss_percentage = 100.0 - accuracy_percentage;
        instruction_sets.push(BattleInstructions::new(
            miss_percentage,
            vec![], // Move misses - no damage/effects
        ));
    }
    
    // 4. Only generate hit instructions if move can hit
    if accuracy_percentage > 0.0 {
        // Get generation mechanics
        let generation = state.get_generation_mechanics();
        
        // Apply move effects with enhanced context - use generation-specific repository
        let repository = create_generation_repository(&generation)?;
        let hit_instructions = apply_move_effects(
            state,
            &move_data,
            user_pos,
            &targets,
            &generation,
            context,
            &repository,
            branch_on_damage,
        )?;
        
        // Scale hit instruction probabilities by accuracy
        for mut hit_instruction in hit_instructions {
            hit_instruction.percentage = (hit_instruction.percentage * accuracy_percentage) / 100.0;
            instruction_sets.push(hit_instruction);
        }
    }
    
    Ok(instruction_sets)
}

/// Create a generation-specific repository for move effects
fn create_generation_repository(generation: &crate::generation::GenerationMechanics) -> crate::types::BattleResult<crate::data::GameDataRepository> {
    use crate::types::BattleError;
    
    // For now, fall back to standard repository since creating a generation-specific Repository
    // would require significant changes to the Repository structure. The main fix for generation
    // awareness is in the move data loading, which we've already implemented.
    // 
    // TODO: Implement a proper generation-aware Repository that loads generation-specific
    // item data and move data for moves like Me First and Fling
    let repository = crate::data::GameDataRepository::from_path("data/ps-extracted")
        .map_err(|e| BattleError::DataLoad(e))?;
    
    Ok(repository)
}

/// Generate hit instructions with secondary effects integrated
/// Following poke-engine pattern: accuracy -> damage -> secondary effects
fn generate_hit_instructions_with_secondary_effects(
    move_data: &crate::core::battle_state::Move,
    targets: &[BattlePosition],
    user_pos: BattlePosition,
    state: &BattleState,
    accuracy_percentage: f32,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    use crate::data::showdown_types::MoveData;
    use crate::generation::GenerationMechanics;
    use crate::engine::combat::moves::MoveContext;
    
    let mut instruction_sets = Vec::new();
    
    // Convert move data format
    let move_data_modern = MoveData {
        name: move_data.name,
        base_power: move_data.base_power as u16,
        move_type: move_data.move_type.clone(),
        category: move_data.category,
        accuracy: 100, // Accuracy already handled
        priority: move_data.priority,
        pp: move_data.pp,
        target: crate::data::showdown_types::MoveTarget::Normal,
        ..Default::default()
    };
    
    let generation = GenerationMechanics::new(crate::generation::Generation::Gen9);
    let context = MoveContext::new();
    let repository = create_generation_repository(&generation)?;
    
    // Check if this move has secondary effects by trying to get move effects
    let secondary_effects_result = crate::engine::combat::moves::apply_move_effects(
        state,
        &move_data_modern,
        user_pos,
        targets,
        &generation,
        &context,
        &repository,
        branch_on_damage,
    );
    
    match secondary_effects_result {
        Ok(secondary_instruction_sets) => {
            // If move has secondary effects, integrate them with accuracy
            for secondary_set in secondary_instruction_sets {
                // Scale the secondary effect probability by the hit chance
                let final_percentage = accuracy_percentage * secondary_set.percentage / 100.0;
                
                if final_percentage > 0.0 {
                    instruction_sets.push(BattleInstructions::new(
                        final_percentage,
                        secondary_set.instruction_list,
                    ));
                }
            }
        },
        Err(_) => {
            // Fallback to basic damage calculation for moves without special effects
            // Generate different instruction sets based on critical hit probability
            // Non-critical hit (93.75% chance for most moves)
            let normal_instructions = generate_damage_instructions_with_rolls(
                move_data, 
                targets, 
                user_pos, 
                state, 
                false,
                branch_on_damage
            )?;
            instruction_sets.push(BattleInstructions::new(
                accuracy_percentage * 0.9375, // 93.75% of hit chance
                normal_instructions,
            ));
            
            // Critical hit (6.25% chance for most moves)
            let crit_instructions = generate_damage_instructions_with_rolls(
                move_data, 
                targets, 
                user_pos, 
                state, 
                true,
                branch_on_damage
            )?;
            instruction_sets.push(BattleInstructions::new(
                accuracy_percentage * 0.0625, // 6.25% of hit chance
                crit_instructions,
            ));
        }
    }
    
    Ok(instruction_sets)
}

