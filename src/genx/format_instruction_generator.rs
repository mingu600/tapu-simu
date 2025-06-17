//! # Format-Aware Instruction Generator
//! 
//! This module provides format-aware instruction generation that handles:
//! - Multi-target moves in doubles/VGC formats
//! - Position-based targeting for spread moves
//! - Format-specific damage calculations (spread move reduction)
//! - Integration with the format targeting system

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction, 
    MultiTargetDamageInstruction, PositionHealInstruction
};
use crate::move_choice::MoveChoice;
use crate::state::{State, Move};
use super::format_targeting::FormatMoveTargetResolver;
use super::damage_calc::calculate_damage;
use super::move_effects;

/// Format-aware instruction generator for enhanced battle mechanics
pub struct FormatInstructionGenerator {
    format: BattleFormat,
    target_resolver: FormatMoveTargetResolver,
}

impl FormatInstructionGenerator {
    /// Create a new format instruction generator
    pub fn new(format: BattleFormat) -> Self {
        Self {
            format: format.clone(),
            target_resolver: FormatMoveTargetResolver::new(format),
        }
    }

    /// Generate format-aware instructions from move choices
    pub fn generate_instructions(
        &self,
        state: &State,
        side_one_choice: &MoveChoice,
        side_two_choice: &MoveChoice,
    ) -> Vec<StateInstructions> {
        let mut instructions = Vec::new();

        // Generate instructions for side one's move
        if let Some(side_one_instructions) = self.generate_move_instructions(
            state, 
            side_one_choice, 
            SideReference::SideOne, 
            0
        ) {
            instructions.extend(side_one_instructions);
        }

        // Generate instructions for side two's move
        if let Some(side_two_instructions) = self.generate_move_instructions(
            state, 
            side_two_choice, 
            SideReference::SideTwo, 
            0
        ) {
            instructions.extend(side_two_instructions);
        }

        if instructions.is_empty() {
            instructions.push(StateInstructions::empty());
        }

        instructions
    }

    /// Generate instructions for a single move choice
    pub fn generate_move_instructions(
        &self,
        state: &State,
        move_choice: &MoveChoice,
        user_side: SideReference,
        user_slot: usize,
    ) -> Option<Vec<StateInstructions>> {
        let move_index = move_choice.move_index()?;
        let user_position = BattlePosition::new(user_side, user_slot);

        // Get the move data
        let side = state.get_side(user_side);
        let pokemon = side.get_active_pokemon_at_slot(user_slot)?;
        let move_data = pokemon.get_move(move_index)?;

        // Resolve targets
        let targets = self.resolve_targets(move_choice, user_side, user_slot, state)?;

        // Generate instructions based on move type and targets
        if move_data.is_damaging() {
            Some(self.generate_damage_instructions(state, move_data, user_position, &targets))
        } else {
            Some(self.generate_status_instructions(state, move_data, user_position, &targets))
        }
    }

    /// Resolve targets for a move choice
    fn resolve_targets(
        &self,
        move_choice: &MoveChoice,
        user_side: SideReference,
        user_slot: usize,
        state: &State,
    ) -> Option<Vec<BattlePosition>> {
        // If move choice has explicit targets, use those
        // But treat empty target lists as needing auto-resolution
        if let Some(targets) = move_choice.target_positions() {
            if !targets.is_empty() {
                return Some(targets.clone());
            }
        }

        // Otherwise, resolve targets automatically
        self.target_resolver
            .resolve_move_targets(user_side, user_slot, move_choice, state)
            .ok()
    }

    /// Generate damage instructions with format-aware modifications
    fn generate_damage_instructions(
        &self,
        state: &State,
        move_data: &Move,
        user_position: BattlePosition,
        targets: &[BattlePosition],
    ) -> Vec<StateInstructions> {
        let mut instructions = Vec::new();

        // Calculate spread move damage multiplier
        let damage_multiplier = if self.is_spread_move_by_target_count(targets) {
            self.format.spread_damage_multiplier()
        } else {
            1.0
        };

        // Generate damage for each target
        if targets.len() == 1 {
            // Single target damage
            let base_damage = self.calculate_base_damage(state, move_data, user_position, targets[0]);
            
            // If immune (0 damage), don't generate damage instructions
            if base_damage == 0 {
                return vec![StateInstructions::empty()];
            }
            
            let final_damage = (base_damage as f32 * damage_multiplier) as i16;

            let damage_instruction = Instruction::PositionDamage(PositionDamageInstruction {
                target_position: targets[0],
                damage_amount: final_damage,
            });

            instructions.push(StateInstructions::new(100.0, vec![damage_instruction]));
        } else if targets.len() > 1 {
            // Multi-target damage
            let mut target_damages = Vec::new();
            let mut has_any_damage = false;

            for &target in targets {
                let base_damage = self.calculate_base_damage(state, move_data, user_position, target);
                let final_damage = (base_damage as f32 * damage_multiplier) as i16;
                target_damages.push((target, final_damage));
                
                if final_damage > 0 {
                    has_any_damage = true;
                }
            }

            // If no target takes damage (all immune), don't generate damage instructions
            if !has_any_damage {
                return vec![StateInstructions::empty()];
            }

            let multi_damage_instruction = Instruction::MultiTargetDamage(MultiTargetDamageInstruction {
                target_damages,
            });

            instructions.push(StateInstructions::new(100.0, vec![multi_damage_instruction]));
        }

        // Add critical hit branching for damaging moves
        self.add_critical_hit_branching(&mut instructions, state, move_data, user_position, targets, damage_multiplier);
        
        instructions
    }

    /// Generate status move instructions
    fn generate_status_instructions(
        &self,
        state: &State,
        move_data: &Move,
        user_position: BattlePosition,
        targets: &[BattlePosition],
    ) -> Vec<StateInstructions> {
        // Convert Move to EngineMoveData for the move effects system
        let engine_move_data = crate::data::types::EngineMoveData {
            id: 1, // Placeholder ID
            name: move_data.name.clone(),
            base_power: Some(move_data.base_power as i16),
            accuracy: Some(move_data.accuracy as i16),
            pp: move_data.pp as i16,
            move_type: move_data.move_type.clone(),
            category: move_data.category,
            priority: move_data.priority,
            target: move_data.target,
            effect_chance: None,
            effect_description: String::new(),
            flags: Vec::new(),
        };

        // Use the comprehensive move effects system
        move_effects::apply_move_effects(state, &engine_move_data, user_position, targets)
    }
    
    /// Check if a move targets the user (self-targeting)
    fn is_self_targeting_move(&self, move_data: &Move) -> bool {
        // Check move target - moves that target the user are not blocked by opponent's Substitute
        matches!(move_data.target, 
            crate::data::ps_types::PSMoveTarget::Self_ | 
            crate::data::ps_types::PSMoveTarget::AllySide |
            crate::data::ps_types::PSMoveTarget::AllyTeam |
            crate::data::ps_types::PSMoveTarget::AdjacentAllyOrSelf
        )
    }

    /// Calculate base damage for a move against a target
    fn calculate_base_damage(
        &self,
        state: &State,
        move_data: &Move,
        user_position: BattlePosition,
        target_position: BattlePosition,
    ) -> i16 {
        // Get attacking Pokemon
        let attacker = state
            .get_pokemon_at_position(user_position)
            .expect("Attacker position should be valid");

        // Get defending Pokemon
        let defender = state
            .get_pokemon_at_position(target_position)
            .expect("Target position should be valid");

        // Create a simple EngineMoveData for the damage calculator
        let engine_move_data = crate::data::types::EngineMoveData {
            id: 1, // Placeholder ID
            name: move_data.name.clone(),
            base_power: Some(move_data.base_power as i16),
            accuracy: Some(move_data.accuracy as i16),
            pp: move_data.pp as i16,
            move_type: move_data.move_type.clone(),
            category: move_data.category, // Use the correct category from move data
            priority: move_data.priority,
            target: move_data.target,
            effect_chance: None,
            effect_description: String::new(),
            flags: Vec::new(),
        };

        // Check for type immunities first
        if self.is_immune_to_move_type(&move_data.move_type, defender) {
            return 0;
        }

        // Check for ability immunities
        if self.is_immune_due_to_ability(&engine_move_data, defender) {
            return 0;
        }

        // Calculate damage using the damage calculator
        calculate_damage(
            state,
            attacker,
            defender,
            &engine_move_data,
            false, // Not a critical hit for base damage
            1.0,   // Full damage roll
        )
    }

    /// Check if a move is a spread move based on target count
    fn is_spread_move_by_target_count(&self, targets: &[BattlePosition]) -> bool {
        // In multi-Pokemon formats, moves hitting multiple targets get spread reduction
        self.format.supports_spread_moves() && targets.len() > 1
    }

    /// Add critical hit branching to damage instructions
    fn add_critical_hit_branching(
        &self,
        instructions: &mut Vec<StateInstructions>,
        _state: &State,
        _move_data: &Move,
        _user_position: BattlePosition,
        targets: &[BattlePosition],
        damage_multiplier: f32,
    ) {
        // For moves that can critically hit, create branching instructions
        if targets.is_empty() {
            return;
        }

        // Constants from poke-engine
        const BASE_CRIT_CHANCE: f32 = 1.0 / 24.0;
        const CRIT_MULTIPLIER: f32 = 1.5;

        // Replace existing instructions with critical hit branches
        let original_instructions = instructions.clone();
        instructions.clear();

        for state_instruction in original_instructions {
            if state_instruction.instruction_list.is_empty() {
                instructions.push(state_instruction);
                continue;
            }

            // Create normal damage branch
            let normal_percentage = 100.0 * (1.0 - BASE_CRIT_CHANCE);
            instructions.push(StateInstructions::new(
                normal_percentage,
                state_instruction.instruction_list.clone(),
            ));

            // Create critical hit branch
            let crit_percentage = 100.0 * BASE_CRIT_CHANCE;
            let crit_instructions = self.apply_critical_hit_multiplier(
                &state_instruction.instruction_list,
                CRIT_MULTIPLIER,
            );
            instructions.push(StateInstructions::new(
                crit_percentage,
                crit_instructions,
            ));
        }
    }

    /// Apply critical hit multiplier to damage instructions
    fn apply_critical_hit_multiplier(
        &self,
        instructions: &[Instruction],
        crit_multiplier: f32,
    ) -> Vec<Instruction> {
        instructions
            .iter()
            .map(|instruction| match instruction {
                Instruction::PositionDamage(damage_instr) => {
                    let crit_damage = (damage_instr.damage_amount as f32 * crit_multiplier).floor() as i16;
                    Instruction::PositionDamage(PositionDamageInstruction {
                        target_position: damage_instr.target_position,
                        damage_amount: crit_damage,
                    })
                }
                Instruction::MultiTargetDamage(multi_damage) => {
                    let crit_target_damages = multi_damage
                        .target_damages
                        .iter()
                        .map(|(pos, damage)| {
                            let crit_damage = (*damage as f32 * crit_multiplier).floor() as i16;
                            (*pos, crit_damage)
                        })
                        .collect();
                    
                    Instruction::MultiTargetDamage(MultiTargetDamageInstruction {
                        target_damages: crit_target_damages,
                    })
                }
                other => other.clone(),
            })
            .collect()
    }

    /// Check if a Pokemon is immune to a move type (e.g., Ghost immune to Normal/Fighting)
    fn is_immune_to_move_type(&self, move_type: &str, defender: &crate::state::Pokemon) -> bool {
        use super::type_effectiveness::{PokemonType, TypeChart};

        let type_chart = TypeChart::new(self.format.generation.number());
        let attacking_type = PokemonType::from_str(move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if defender.types.len() > 1 {
            PokemonType::from_str(&defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            attacking_type,
            (defender_type1, defender_type2),
            None,
            None,
        );

        // If type effectiveness is 0, the Pokemon is immune
        type_effectiveness == 0.0
    }

    /// Check if a Pokemon is immune due to ability (e.g., Levitate vs Ground)
    fn is_immune_due_to_ability(&self, move_data: &crate::data::types::EngineMoveData, defender: &crate::state::Pokemon) -> bool {
        use super::abilities::get_ability_by_name;
        
        if let Some(ability) = get_ability_by_name(&defender.ability) {
            ability.provides_immunity(&move_data.move_type)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Pokemon, MoveCategory};
    use crate::move_choice::MoveIndex;
    use crate::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    fn create_test_state() -> State {
        let mut state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        
        // Add Pokemon to both sides
        let mut pokemon1 = Pokemon::new("Attacker".to_string());
        let mut pokemon2 = Pokemon::new("Defender".to_string());
        
        // Add a basic attack move
        let tackle = Move::new_with_details(
            "Tackle".to_string(),
            40,
            100,
            "Normal".to_string(),
            35,
            crate::data::ps_types::PSMoveTarget::Normal,
            MoveCategory::Physical,
            0,
        );
        
        pokemon1.add_move(MoveIndex::M0, tackle);
        pokemon2.hp = 100;
        pokemon2.max_hp = 100;
        
        state.side_one.add_pokemon(pokemon1);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(pokemon2);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));
        
        state
    }

    #[test]
    fn test_single_target_damage_generation() {
        let generator = FormatInstructionGenerator::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let state = create_test_state();
        
        let move_choice = MoveChoice::new_move(
            MoveIndex::M0,
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&state, &move_choice, &no_move);
        
        // Should have critical hit branching (2 instructions)
        assert_eq!(instructions.len(), 2);
        
        // Verify one instruction has damage
        let has_damage = instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| matches!(i, Instruction::PositionDamage(_)))
        });
        assert!(has_damage);
    }

    #[test]
    fn test_spread_move_damage_reduction() {
        let generator = FormatInstructionGenerator::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let mut state = State::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        // Add Pokemon to all positions
        for side in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..2 {
                let pokemon = Pokemon::new(format!("Pokemon-{:?}-{}", side, slot));
                state.get_side_mut(side).add_pokemon(pokemon);
                state.get_side_mut(side).set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        // Add Earthquake to the attacker
        let earthquake = Move::new_with_details(
            "Earthquake".to_string(),
            100,
            100,
            "Ground".to_string(),
            10,
            crate::data::ps_types::PSMoveTarget::AllAdjacent,
            MoveCategory::Physical,
            0,
        );
        
        if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
            pokemon.add_move(MoveIndex::M0, earthquake);
        }
        
        let spread_move = MoveChoice::new_move(MoveIndex::M0, vec![]);
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&state, &spread_move, &no_move);
        
        // Should generate instructions with spread damage reduction
        assert!(!instructions.is_empty());
        
        // Verify that multi-target damage is used
        let has_multi_target = instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| matches!(i, Instruction::MultiTargetDamage(_)))
        });
        assert!(has_multi_target);
    }

    #[test]
    fn test_critical_hit_branching() {
        let generator = FormatInstructionGenerator::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let state = create_test_state();
        
        let move_choice = MoveChoice::new_move(
            MoveIndex::M0,
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );
        let no_move = MoveChoice::None;
        
        let instructions = generator.generate_instructions(&state, &move_choice, &no_move);
        
        // Should have exactly 2 instructions (normal + crit)
        assert_eq!(instructions.len(), 2);
        
        // Verify percentages
        let total_percentage: f32 = instructions.iter().map(|i| i.percentage).sum();
        assert!((total_percentage - 100.0).abs() < 0.001);
        
        // Verify that crit branch has higher damage
        if let (Some(first), Some(second)) = (instructions.get(0), instructions.get(1)) {
            if let (
                Some(Instruction::PositionDamage(damage1)),
                Some(Instruction::PositionDamage(damage2))
            ) = (
                first.instruction_list.first(),
                second.instruction_list.first()
            ) {
                // One should have higher damage than the other (crit vs normal)
                assert_ne!(damage1.damage_amount, damage2.damage_amount);
            }
        }
    }
}