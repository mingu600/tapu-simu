//! # Simplified Turn Resolution
//! 
//! This module replaces the complex generator hierarchy with simple functions
//! for turn resolution, following the modernization plan.

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::move_choice::MoveChoice;
use crate::core::battle_state::BattleState;
use crate::core::targeting::resolve_targets;
use crate::core::instructions::Weather;
use crate::engine::combat::moves::{MoveContext, OpponentMoveInfo};
use crate::types::{BattleError, BattleResult};
use std::collections::HashMap;
use crate::data::showdown_types::MoveTarget;

/// Parse move target string to MoveTarget enum
fn parse_move_target(target_str: &str) -> MoveTarget {
    match target_str.to_lowercase().as_str() {
        "normal" => MoveTarget::Normal,
        "self" => MoveTarget::Self_,
        "adjacentally" => MoveTarget::AdjacentAlly,
        "adjacentallyorself" => MoveTarget::AdjacentAllyOrSelf,
        "adjacentfoe" => MoveTarget::AdjacentFoe,
        "alladjacentfoes" => MoveTarget::AllAdjacentFoes,
        "alladjacent" => MoveTarget::AllAdjacent,
        "all" => MoveTarget::All,
        "allyside" => MoveTarget::AllySide,
        "foeside" => MoveTarget::FoeSide,
        "allyteam" => MoveTarget::AllyTeam,
        "any" => MoveTarget::Any,
        _ => MoveTarget::Normal, // Default fallback
    }
}

// Compatibility for end_of_turn module  
pub mod end_of_turn {
    use crate::core::instructions::{PokemonStatus, Weather};
    use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
    use crate::core::battle_state::BattleState;
    use crate::core::battle_format::BattlePosition;


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
        
        match state.weather() {
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
            // For now, treat Tera moves the same as regular moves (simplified)
            generate_attack_instructions_with_context(*move_index, target_positions, user_pos, format, state, going_first)
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
        // For now, we'll use false for simplicity - can be enhanced later
        let branch_on_damage = false;
        
        // Generate different instruction sets based on critical hit probability
        // Non-critical hit (93.75% chance for most moves)
        let normal_instructions = generate_damage_instructions_with_rolls(
            move_data, 
            &targets, 
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
            &targets, 
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
    use crate::engine::combat::damage_calc::{calculate_damage_with_positions, DamageRolls, compare_health_with_damage_multiples};
    
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
        
        // Convert legacy move data to modern MoveData for damage calculation
        let move_data_modern = crate::data::showdown_types::MoveData {
            name: move_data.name.clone(),
            base_power: move_data.base_power as u16,
            move_type: move_data.move_type.clone(),
            category: format!("{:?}", move_data.category),
            accuracy: 100, // Accuracy already handled in calling function
            priority: move_data.priority,
            pp: move_data.pp,
            target: "normal".to_string(),
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
        pokemon.get_effective_stat(crate::core::instructions::Stat::Speed) as i16
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
        &move_data.name,
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
        _ => base_accuracy, // No weather modification for other moves
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
        match user.ability.to_lowercase().as_str() {
            "compoundeyes" | "compound-eyes" => {
                // Compound Eyes increases accuracy by 30% (1.3x multiplier)
                base_accuracy * 1.3
            }
            "noguard" | "no-guard" => {
                // No Guard makes all moves hit regardless of accuracy
                100.0
            }
            _ => base_accuracy,
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
            match item.to_lowercase().replace(" ", "").replace("-", "").as_str() {
                "widelens" => {
                    // Wide Lens increases accuracy by 10% (1.1x multiplier)
                    base_accuracy * 1.1
                }
                "zoomlens" => {
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
        if let Some(&accuracy_stage) = user.stat_boosts.get(&crate::core::instructions::Stat::Accuracy) {
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
            if let Some(&evasion_stage) = target.stat_boosts.get(&crate::core::instructions::Stat::Evasion) {
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
                    move_name: move_data.name.clone(),
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
) -> BattleResult<Vec<BattleInstructions>> {
    let user_pos = BattlePosition::new(user_side, user_slot);
    
    match choice {
        MoveChoice::Switch(pokemon_index) => {
            generate_switch_instructions(pokemon_index.to_index(), user_pos, state)
        }
        MoveChoice::Move { move_index, target_positions } => {
            generate_attack_instructions_with_enhanced_context(*move_index, target_positions, user_pos, format, state, context)
        }
        MoveChoice::MoveTera { move_index, target_positions, .. } => {
            // For now, treat Tera moves the same as regular moves (simplified)
            generate_attack_instructions_with_enhanced_context(*move_index, target_positions, user_pos, format, state, context)
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
) -> BattleResult<Vec<BattleInstructions>> {
    use crate::engine::combat::moves::apply_move_effects;
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
    
    // Convert Move to MoveData via repository lookup
    let repository = crate::data::ps::repository::Repository::from_path("data/ps-extracted")
        .expect("Failed to load Pokemon data from data/ps-extracted");
    let move_data = if let Some(repo_move_data) = repository.find_move_by_name(&move_data_raw.name) {
        // Convert repository::MoveData to showdown_types::MoveData
        crate::data::showdown_types::MoveData {
            name: repo_move_data.name.clone(),
            base_power: repo_move_data.base_power as u16,
            accuracy: repo_move_data.accuracy as u16,
            pp: repo_move_data.pp,
            max_pp: repo_move_data.max_pp,
            move_type: repo_move_data.move_type.to_string(),
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
            category: format!("{:?}", move_data_raw.category),
            priority: move_data_raw.priority,
            target: format!("{:?}", move_data_raw.target),
            ..crate::data::showdown_types::MoveData::default()
        }
    };
    
    // Determine targets using the same logic as before
    let targets = if explicit_targets.is_empty() {
        resolve_targets(parse_move_target(&move_data.target), user_pos, format, state)
    } else {
        explicit_targets.to_vec()
    };
    
    // Get generation mechanics
    let generation = state.get_generation_mechanics();
    
    // Apply move effects with enhanced context
    let repository = crate::data::ps::repository::Repository::from_path("data/ps-extracted")
        .expect("Failed to load Pokemon data from data/ps-extracted");
    let instructions = apply_move_effects(
        state,
        &move_data,
        user_pos,
        &targets,
        &generation,
        context,
        &repository,
    )?;
    
    Ok(instructions)
}

