//! # Generation X Instruction Generator
//! 
//! This module provides instruction generation for the default generation (4-9).
//! It integrates all the format-aware components to provide comprehensive battle mechanics.

use crate::core::state::State;
use crate::core::move_choice::MoveChoice;
use crate::core::battle_format::BattleFormat;
use crate::core::instruction::StateInstructions;
use crate::core::battle_format::SideReference;
use crate::engine::turn::format_instruction_generator::FormatInstructionGenerator;
use crate::engine::turn::doubles_mechanics::DoublesSpecificMechanics;
use crate::engine::targeting::auto_targeting::PSAutoTargetingEngine;

/// Instruction generator for Generation X (4-9)
/// 
/// This is the main instruction generator that coordinates all format-aware mechanics:
/// - Format-aware targeting resolution
/// - Spread move damage calculation  
/// - Doubles-specific mechanics
/// - Critical hit branching
/// - Multi-target instruction generation
pub struct GenerationXInstructionGenerator {
    format: BattleFormat,
    format_generator: FormatInstructionGenerator,
    doubles_mechanics: DoublesSpecificMechanics,
    auto_targeting: PSAutoTargetingEngine,
}

impl GenerationXInstructionGenerator {
    /// Create a new instruction generator
    pub fn new(format: BattleFormat) -> Self {
        Self { 
            format: format.clone(),
            format_generator: FormatInstructionGenerator::new(format.clone()),
            doubles_mechanics: DoublesSpecificMechanics::new(format.clone()),
            auto_targeting: PSAutoTargetingEngine::new(format),
        }
    }

    /// Generate instructions from move choices with full format awareness
    /// This processes both moves together as a single turn, like poke-engine
    pub fn generate_instructions(
        &self,
        state: &mut State,
        side_one_choice: &MoveChoice,
        side_two_choice: &MoveChoice,
    ) -> Vec<StateInstructions> {
        // Use the new turn-based instruction generation
        self.generate_instructions_from_move_pair(state, side_one_choice, side_two_choice)
    }

    /// Generate instructions from a move pair, processing both moves together as a single turn
    /// This is the equivalent of poke-engine's generate_instructions_from_move_pair
    pub fn generate_instructions_from_move_pair(
        &self,
        state: &mut State,
        side_one_choice: &MoveChoice,
        side_two_choice: &MoveChoice,
    ) -> Vec<StateInstructions> {
        // Clone choices so we can modify them for auto-targeting
        let mut side_one_choice = side_one_choice.clone();
        let mut side_two_choice = side_two_choice.clone();

        // Auto-resolve targets if needed
        if let Err(e) = self.auto_targeting.auto_resolve_targets(
            SideReference::SideOne, 
            0, 
            &mut side_one_choice, 
            state
        ) {
            eprintln!("Warning: Failed to auto-resolve targets for side one: {}", e);
        }

        if let Err(e) = self.auto_targeting.auto_resolve_targets(
            SideReference::SideTwo, 
            0, 
            &mut side_two_choice, 
            state
        ) {
            eprintln!("Warning: Failed to auto-resolve targets for side two: {}", e);
        }

        // Determine move order based on priority and speed
        let (first_side, first_move, second_side, second_move) = 
            self.determine_move_order(state, &side_one_choice, &side_two_choice);

        // Generate instructions for both moves, handling switches properly
        let first_move_instructions = self.generate_single_move_instructions(
            state, 
            &first_move, 
            first_side, 
            0
        );

        // Handle switch interactions - switches need to be applied before calculating subsequent moves
        let second_move_instructions = if first_move.is_switch() && !second_move.is_switch() {
            // First move is a switch, second is an attack - apply the switch first
            let mut temp_state = state.clone();
            
            // Apply ALL switch instructions to the temporary state to get the final switched state
            if let Some(instruction_sets) = &first_move_instructions {
                for instruction_set in instruction_sets {
                    temp_state.apply_instructions(&instruction_set.instruction_list);
                }
            }
            
            // Generate second move instructions using the updated state
            self.generate_single_move_instructions(
                &temp_state, 
                &second_move, 
                second_side, 
                0
            )
        } else {
            // Either both are switches, first is not a switch, or second is a switch
            // In these cases, use the original state
            self.generate_single_move_instructions(
                state, 
                &second_move, 
                second_side, 
                0
            )
        };

        // Combine instruction sets from both moves to create all possible combinations
        let all_instructions = self.combine_move_instructions(
            first_move_instructions,
            second_move_instructions
        );

        // Apply redirection mechanics if in doubles
        let final_instructions = if self.format.active_pokemon_count() > 1 {
            self.apply_redirection_to_instructions(state, all_instructions)
        } else {
            all_instructions
        };

        // If we got no instructions, return an empty instruction set
        if final_instructions.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            final_instructions
        }
    }

    /// Determine which move goes first based on priority and speed
    /// Follows poke-engine's move ordering logic with switch priority rules
    fn determine_move_order(
        &self,
        state: &State,
        side_one_choice: &MoveChoice,
        side_two_choice: &MoveChoice,
    ) -> (SideReference, MoveChoice, SideReference, MoveChoice) {
        // Special handling for switches (following poke-engine logic)
        if side_one_choice.is_switch() && side_two_choice.is_switch() {
            // Both switches - use speed to determine order
            let side_one_speed = self.get_effective_speed(state, SideReference::SideOne);
            let side_two_speed = self.get_effective_speed(state, SideReference::SideTwo);
            
            if side_one_speed > side_two_speed {
                return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
            } else if side_one_speed == side_two_speed {
                // Speed tie - could implement random choice here, for now side one wins
                return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
            } else {
                return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
            }
        } else if side_one_choice.is_switch() {
            // Side one switching - switch goes first unless opponent uses Pursuit
            if self.is_pursuit(state, side_two_choice, SideReference::SideTwo) {
                // Pursuit hits the switching Pokemon first
                return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
            } else {
                // Switch goes first
                return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
            }
        } else if side_two_choice.is_switch() {
            // Side two switching - switch goes first unless opponent uses Pursuit
            if self.is_pursuit(state, side_one_choice, SideReference::SideOne) {
                // Pursuit hits the switching Pokemon first
                return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
            } else {
                // Switch goes first
                return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
            }
        }

        // Neither choice is a switch - use normal priority/speed rules
        let side_one_priority = self.get_move_priority(state, side_one_choice, SideReference::SideOne);
        let side_two_priority = self.get_move_priority(state, side_two_choice, SideReference::SideTwo);
        
        // Higher priority goes first
        if side_one_priority > side_two_priority {
            return (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone());
        } else if side_two_priority > side_one_priority {
            return (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone());
        }

        // Same priority - check speed
        let side_one_speed = self.get_effective_speed(state, SideReference::SideOne);
        let side_two_speed = self.get_effective_speed(state, SideReference::SideTwo);

        if side_one_speed > side_two_speed {
            (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone())
        } else if side_one_speed == side_two_speed {
            // Speed tie - for now side one wins, could implement random choice
            (SideReference::SideOne, side_one_choice.clone(), SideReference::SideTwo, side_two_choice.clone())
        } else {
            (SideReference::SideTwo, side_two_choice.clone(), SideReference::SideOne, side_one_choice.clone())
        }
    }

    /// Get move priority for a choice
    fn get_move_priority(&self, state: &State, choice: &MoveChoice, side: SideReference) -> i8 {
        if let Some(move_index) = choice.move_index() {
            let pokemon = state.get_side(side).get_active_pokemon_at_slot(0);
            if let Some(pokemon) = pokemon {
                if let Some(move_data) = pokemon.get_move(move_index) {
                    return move_data.priority;
                }
            }
        }
        0 // Default priority
    }

    /// Check if a move choice is Pursuit
    fn is_pursuit(&self, state: &State, choice: &MoveChoice, side: SideReference) -> bool {
        if let Some(move_index) = choice.move_index() {
            let pokemon = state.get_side(side).get_active_pokemon_at_slot(0);
            if let Some(pokemon) = pokemon {
                if let Some(move_data) = pokemon.get_move(move_index) {
                    // Check if the move name is "Pursuit" (case insensitive)
                    return move_data.name.to_lowercase() == "pursuit";
                }
            }
        }
        false
    }

    /// Get effective speed for a side
    fn get_effective_speed(&self, state: &State, side: SideReference) -> i16 {
        if let Some(pokemon) = state.get_side(side).get_active_pokemon_at_slot(0) {
            pokemon.get_effective_stat(crate::core::instruction::Stat::Speed)
        } else {
            0
        }
    }

    /// Generate instructions for a single move (used in move pair processing)
    fn generate_single_move_instructions(
        &self,
        state: &State,
        choice: &MoveChoice,
        side: SideReference,
        slot: usize,
    ) -> Option<Vec<StateInstructions>> {
        // Check for doubles-specific mechanics first
        if let Some(doubles_instructions) = self.doubles_mechanics.generate_doubles_move_instructions(
            state, 
            choice, 
            crate::core::battle_format::BattlePosition::new(side, slot)
        ) {
            return Some(doubles_instructions);
        }

        // Use format-aware instruction generation for a single move
        self.format_generator.generate_move_instructions(state, choice, side, slot)
    }

    /// Combine instruction sets from two moves to create all possible combinations
    /// This creates the Cartesian product of the two instruction sets with proper probability calculation
    fn combine_move_instructions(
        &self,
        first_move_instructions: Option<Vec<StateInstructions>>,
        second_move_instructions: Option<Vec<StateInstructions>>,
    ) -> Vec<StateInstructions> {
        let first_instructions = first_move_instructions.unwrap_or_else(|| vec![StateInstructions::empty()]);
        let second_instructions = second_move_instructions.unwrap_or_else(|| vec![StateInstructions::empty()]);

        let mut combined_instructions = Vec::new();

        // Special handling for switches: if first move is a switch, we need to sequence
        // all switch effects followed by each possible outcome of the second move
        if self.contains_switch_instruction(&first_instructions) {
            // Combine all switch instructions into a single sequential instruction set
            let mut all_switch_instructions = Vec::new();
            for instruction_set in &first_instructions {
                all_switch_instructions.extend(instruction_set.instruction_list.clone());
            }
            
            // For each possible outcome of the second move, prepend the complete switch sequence
            for second_set in &second_instructions {
                let mut combined_instruction_list = Vec::new();
                combined_instruction_list.extend(all_switch_instructions.clone());
                combined_instruction_list.extend(second_set.instruction_list.clone());

                // Switch has 100% probability, so final probability is just the second move's probability
                combined_instructions.push(StateInstructions::new(
                    second_set.percentage,
                    combined_instruction_list,
                ));
            }
        } else {
            // Create all combinations of first move × second move (normal behavior)
            for first_set in &first_instructions {
                for second_set in &second_instructions {
                    // Skip empty instruction sets (moves like Splash)
                    if first_set.instruction_list.is_empty() && second_set.instruction_list.is_empty() {
                        continue;
                    }

                    // Combine the instruction lists
                    let mut combined_instruction_list = Vec::new();
                    combined_instruction_list.extend(first_set.instruction_list.clone());
                    combined_instruction_list.extend(second_set.instruction_list.clone());

                    // Calculate combined probability (independent events)
                    let combined_percentage = (first_set.percentage * second_set.percentage) / 100.0;

                    combined_instructions.push(StateInstructions::new(
                        combined_percentage,
                        combined_instruction_list,
                    ));
                }
            }
        }

        // If no valid combinations were created, return an empty instruction set
        if combined_instructions.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            combined_instructions
        }
    }

    /// Check if an instruction set contains a switch instruction
    fn contains_switch_instruction(&self, instruction_sets: &[StateInstructions]) -> bool {
        instruction_sets.iter().any(|set| {
            set.instruction_list.iter().any(|instruction| {
                matches!(instruction, crate::core::instruction::Instruction::SwitchPokemon(_))
            })
        })
    }

    /// Apply redirection mechanics to generated instructions
    fn apply_redirection_to_instructions(
        &self,
        state: &State,
        instructions: Vec<StateInstructions>,
    ) -> Vec<StateInstructions> {
        // For each instruction set, apply redirection to damage instructions
        instructions
            .into_iter()
            .map(|mut instruction_set| {
                for instruction in &mut instruction_set.instruction_list {
                    match instruction {
                        crate::core::instruction::Instruction::PositionDamage(damage_instr) => {
                            // Apply redirection mechanics
                            let user_pos = crate::core::battle_format::BattlePosition::new(SideReference::SideOne, 0); // Simplified
                            let redirected = self.doubles_mechanics.apply_redirection_mechanics(
                                state, 
                                &[damage_instr.target_position], 
                                user_pos
                            );
                            if let Some(&new_target) = redirected.first() {
                                damage_instr.target_position = new_target;
                            }
                        }
                        crate::core::instruction::Instruction::MultiTargetDamage(multi_damage) => {
                            // Apply redirection to each target
                            let user_pos = crate::core::battle_format::BattlePosition::new(SideReference::SideOne, 0); // Simplified
                            let original_targets: Vec<_> = multi_damage.target_damages.iter().map(|(pos, _)| *pos).collect();
                            let redirected = self.doubles_mechanics.apply_redirection_mechanics(
                                state, 
                                &original_targets, 
                                user_pos
                            );
                            
                            // Update targets with redirection
                            for (i, (pos, damage)) in multi_damage.target_damages.iter_mut().enumerate() {
                                if let Some(&new_pos) = redirected.get(i) {
                                    *pos = new_pos;
                                }
                            }
                        }
                        _ => {} // Other instructions don't need redirection
                    }
                }
                instruction_set
            })
            .collect()
    }

    /// Get the battle format this generator is configured for
    pub fn get_format(&self) -> &BattleFormat {
        &self.format
    }

    /// Check if this generator supports the given format
    pub fn supports_format(&self, format: &BattleFormat) -> bool {
        // This generation supports all current formats
        use crate::core::battle_format::FormatType;
        matches!(format.format_type, 
            FormatType::Singles | 
            FormatType::Doubles | 
            FormatType::Vgc | 
            FormatType::Triples
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{Pokemon, Move, MoveCategory};
    use crate::core::move_choice::MoveIndex;
    use crate::data::ps_types::PSMoveTarget;
    use crate::core::battle_format::{BattlePosition, BattleFormat, FormatType};
    use crate::generation::Generation;

    fn create_test_state() -> State {
        let mut state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        
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
            PSMoveTarget::Normal,
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
    fn test_basic_instruction_generation() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let mut state = create_test_state();
        
        let move_choice = MoveChoice::new_move(
            MoveIndex::M0,
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&mut state, &move_choice, &no_move);
        
        // Should generate instructions with critical hit branching
        assert!(!instructions.is_empty());
        
        // Verify some instruction contains damage
        let has_damage = instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, crate::core::instruction::Instruction::PositionDamage(_))
            })
        });
        assert!(has_damage);
    }

    #[test]
    fn test_auto_targeting() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let mut state = create_test_state();
        
        // Move choice without explicit targets
        let move_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&mut state, &move_choice, &no_move);
        
        // Should still generate valid instructions due to auto-targeting
        assert!(!instructions.is_empty());
    }

    #[test]
    fn test_doubles_mechanics_integration() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let mut state = State::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        // Add Pokemon to all positions
        for side in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..2 {
                let pokemon = Pokemon::new(format!("Pokemon-{:?}-{}", side, slot));
                state.get_side_mut(side).add_pokemon(pokemon);
                state.get_side_mut(side).set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        // Add Follow Me move
        let follow_me = Move::new_with_details(
            "Follow Me".to_string(),
            0,
            100,
            "Normal".to_string(),
            20,
            PSMoveTarget::Self_,
            MoveCategory::Status,
            2,
        );
        
        if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
            pokemon.add_move(MoveIndex::M0, follow_me);
        }
        
        let follow_me_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&mut state, &follow_me_choice, &no_move);
        
        // Should generate Follow Me status instructions
        assert!(!instructions.is_empty());
        
        let has_follow_me = instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, crate::core::instruction::Instruction::ApplyVolatileStatus(_))
            })
        });
        assert!(has_follow_me);
    }

    #[test]
    fn test_format_support() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        
        assert!(generator.supports_format(&BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles)));
        assert!(generator.supports_format(&BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles)));
        assert!(generator.supports_format(&BattleFormat::vgc2024()));
        assert!(generator.supports_format(&BattleFormat::new("Triples".to_string(), Generation::Gen9, FormatType::Triples)));
    }
}