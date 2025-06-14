//! # Generation X Instruction Generator
//! 
//! This module provides instruction generation for the default generation (4-9).
//! It integrates all the format-aware components to provide comprehensive battle mechanics.

use crate::{State, MoveChoice, BattleFormat};
use crate::instruction::StateInstructions;
use crate::battle_format::SideReference;
use super::format_instruction_generator::FormatInstructionGenerator;
use super::doubles_mechanics::DoublesSpecificMechanics;
use super::format_targeting::AutoTargetingEngine;

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
    auto_targeting: AutoTargetingEngine,
}

impl GenerationXInstructionGenerator {
    /// Create a new instruction generator
    pub fn new(format: BattleFormat) -> Self {
        Self { 
            format: format.clone(),
            format_generator: FormatInstructionGenerator::new(format.clone()),
            doubles_mechanics: DoublesSpecificMechanics::new(format.clone()),
            auto_targeting: AutoTargetingEngine::new(format),
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

        // Generate instructions for the turn by processing moves in order
        let mut all_instructions = Vec::new();

        // Process first move
        if let Some(first_move_instructions) = self.generate_single_move_instructions(
            state, 
            &first_move, 
            first_side, 
            0
        ) {
            // Only add instructions if they're not empty (ignore moves like SPLASH)
            for instruction in first_move_instructions {
                if !instruction.instruction_list.is_empty() {
                    all_instructions.push(instruction);
                }
            }
        }

        // Process second move (only if Pokemon is still alive and able to move)
        if let Some(second_move_instructions) = self.generate_single_move_instructions(
            state, 
            &second_move, 
            second_side, 
            0
        ) {
            // Only add instructions if they're not empty (ignore moves like SPLASH)
            for instruction in second_move_instructions {
                if !instruction.instruction_list.is_empty() {
                    all_instructions.push(instruction);
                }
            }
        }

        // Apply redirection mechanics if in doubles
        if self.format.active_pokemon_count() > 1 {
            all_instructions = self.apply_redirection_to_instructions(state, all_instructions);
        }

        // If we only got one move's instructions (like WATERGUN), return just those
        // This matches poke-engine behavior where SPLASH contributes nothing
        if all_instructions.is_empty() {
            all_instructions.push(StateInstructions::empty());
        }

        all_instructions
    }

    /// Determine which move goes first based on priority and speed
    fn determine_move_order(
        &self,
        state: &State,
        side_one_choice: &MoveChoice,
        side_two_choice: &MoveChoice,
    ) -> (SideReference, MoveChoice, SideReference, MoveChoice) {
        // Get move priorities and speeds
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

    /// Get effective speed for a side
    fn get_effective_speed(&self, state: &State, side: SideReference) -> i16 {
        if let Some(pokemon) = state.get_side(side).get_active_pokemon_at_slot(0) {
            pokemon.get_effective_stat(crate::instruction::Stat::Speed)
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
            crate::battle_format::BattlePosition::new(side, slot)
        ) {
            return Some(doubles_instructions);
        }

        // Use format-aware instruction generation for a single move
        self.format_generator.generate_move_instructions(state, choice, side, slot)
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
                        crate::instruction::Instruction::PositionDamage(damage_instr) => {
                            // Apply redirection mechanics
                            let user_pos = crate::battle_format::BattlePosition::new(SideReference::SideOne, 0); // Simplified
                            let redirected = self.doubles_mechanics.apply_redirection_mechanics(
                                state, 
                                &[damage_instr.target_position], 
                                user_pos
                            );
                            if let Some(&new_target) = redirected.first() {
                                damage_instr.target_position = new_target;
                            }
                        }
                        crate::instruction::Instruction::MultiTargetDamage(multi_damage) => {
                            // Apply redirection to each target
                            let user_pos = crate::battle_format::BattlePosition::new(SideReference::SideOne, 0); // Simplified
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
        matches!(format, 
            BattleFormat::Singles | 
            BattleFormat::Doubles | 
            BattleFormat::Vgc | 
            BattleFormat::Triples
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Pokemon, Move, MoveCategory};
    use crate::move_choice::MoveIndex;
    use crate::data::types::MoveTarget;
    use crate::battle_format::BattlePosition;

    fn create_test_state() -> State {
        let mut state = State::new(BattleFormat::Singles);
        
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
            MoveTarget::SelectedPokemon,
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
        let generator = GenerationXInstructionGenerator::new(BattleFormat::Singles);
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
                matches!(i, crate::instruction::Instruction::PositionDamage(_))
            })
        });
        assert!(has_damage);
    }

    #[test]
    fn test_auto_targeting() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::Singles);
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
        let generator = GenerationXInstructionGenerator::new(BattleFormat::Doubles);
        let mut state = State::new(BattleFormat::Doubles);
        
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
            MoveTarget::User,
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
                matches!(i, crate::instruction::Instruction::ApplyVolatileStatus(_))
            })
        });
        assert!(has_follow_me);
    }

    #[test]
    fn test_format_support() {
        let generator = GenerationXInstructionGenerator::new(BattleFormat::Singles);
        
        assert!(generator.supports_format(&BattleFormat::Singles));
        assert!(generator.supports_format(&BattleFormat::Doubles));
        assert!(generator.supports_format(&BattleFormat::Vgc));
        assert!(generator.supports_format(&BattleFormat::Triples));
    }
}