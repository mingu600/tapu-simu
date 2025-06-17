//! # Doubles-Specific Mechanics
//! 
//! This module handles mechanics that are unique to doubles formats:
//! - Spread moves hitting allies (Earthquake, Surf, etc.)
//! - Doubles-specific move interactions
//! - Position-based ability interactions
//! - Format-specific move behavior differences

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::data::ps_types::PSMoveTarget;
use crate::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction, 
    ApplyVolatileStatusInstruction, VolatileStatus
};
use crate::state::State;
use crate::move_choice::MoveChoice;

/// Doubles-specific mechanics handler
pub struct DoublesSpecificMechanics {
    format: BattleFormat,
}

impl DoublesSpecificMechanics {
    /// Create a new doubles mechanics handler
    pub fn new(format: BattleFormat) -> Self {
        Self { format }
    }

    /// Check if a move should hit the user's ally in doubles
    pub fn move_hits_ally(&self, move_target: &PSMoveTarget, _user_position: BattlePosition) -> bool {
        // Only relevant in multi-Pokemon formats
        if self.format.active_pokemon_count() <= 1 {
            return false;
        }

        match move_target {
            PSMoveTarget::AllAdjacent => {
                // Moves like Earthquake, Surf hit all adjacent Pokemon including allies
                true
            }
            PSMoveTarget::All => {
                // Moves like Self-Destruct affect the entire field
                true
            }
            _ => false,
        }
    }

    /// Get ally position for a given user position
    pub fn get_ally_position(&self, user_position: BattlePosition) -> Option<BattlePosition> {
        if self.format.active_pokemon_count() <= 1 {
            return None;
        }

        // In doubles, slots 0 and 1 are allies
        let ally_slot = match user_position.slot {
            0 => 1,
            1 => 0,
            // For triples, more complex logic would be needed
            _ => return None,
        };

        Some(BattlePosition::new(user_position.side, ally_slot))
    }

    /// Handle Follow Me / Rage Powder redirection mechanics
    pub fn apply_redirection_mechanics(
        &self,
        state: &State,
        original_targets: &[BattlePosition],
        user_position: BattlePosition,
    ) -> Vec<BattlePosition> {
        if self.format.active_pokemon_count() <= 1 {
            return original_targets.to_vec();
        }

        let mut redirected_targets = Vec::new();

        for &target in original_targets {
            // Check if there's a Pokemon using Follow Me/Rage Powder on the target's side
            if let Some(redirector) = self.find_redirector(state, target.side) {
                // Only redirect single-target moves
                if original_targets.len() == 1 && target.side != user_position.side {
                    redirected_targets.push(redirector);
                } else {
                    redirected_targets.push(target);
                }
            } else {
                redirected_targets.push(target);
            }
        }

        redirected_targets
    }

    /// Find a Pokemon using Follow Me or Rage Powder on the specified side
    fn find_redirector(&self, state: &State, side: SideReference) -> Option<BattlePosition> {
        let battle_side = state.get_side(side);
        
        for slot in 0..self.format.active_pokemon_count() {
            if let Some(pokemon) = battle_side.get_active_pokemon_at_slot(slot) {
                if pokemon.volatile_statuses.contains(&VolatileStatus::FollowMe) {
                    return Some(BattlePosition::new(side, slot));
                }
            }
        }
        
        None
    }

    /// Generate Follow Me/Rage Powder effect instructions
    pub fn generate_follow_me_instructions(
        &self,
        user_position: BattlePosition,
    ) -> Vec<StateInstructions> {
        let follow_me_instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: user_position,
            volatile_status: VolatileStatus::FollowMe,
            duration: Some(1), // Follow Me lasts for 1 turn
        });

        vec![StateInstructions::new(100.0, vec![follow_me_instruction])]
    }

    /// Generate Helping Hand effect instructions
    pub fn generate_helping_hand_instructions(
        &self,
        user_position: BattlePosition,
        ally_position: Option<BattlePosition>,
    ) -> Vec<StateInstructions> {
        if let Some(ally_pos) = ally_position {
            let helping_hand_instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                target_position: ally_pos,
                volatile_status: VolatileStatus::HelpingHand,
                duration: Some(1), // Helping Hand lasts for 1 turn
            });

            vec![StateInstructions::new(100.0, vec![helping_hand_instruction])]
        } else {
            vec![StateInstructions::empty()]
        }
    }

    /// Handle Wide Guard mechanics
    pub fn generate_wide_guard_instructions(
        &self,
        user_position: BattlePosition,
    ) -> Vec<StateInstructions> {
        // Wide Guard protects the user's side from spread moves
        // In a full implementation, this would set a side condition
        vec![StateInstructions::new(100.0, vec![])]
    }

    /// Handle Quick Guard mechanics  
    pub fn generate_quick_guard_instructions(
        &self,
        user_position: BattlePosition,
    ) -> Vec<StateInstructions> {
        // Quick Guard protects the user's side from priority moves
        // In a full implementation, this would set a side condition
        vec![StateInstructions::new(100.0, vec![])]
    }

    /// Check if a move is blocked by Wide Guard
    pub fn is_blocked_by_wide_guard(
        &self,
        state: &State,
        move_target: &PSMoveTarget,
        target_side: SideReference,
    ) -> bool {
        // Check if the target side has Wide Guard active
        let side = state.get_side(target_side);
        
        // In a full implementation, this would check for WideGuard side condition
        // For now, just check the move target type
        matches!(move_target, 
            PSMoveTarget::AllAdjacentFoes | 
            PSMoveTarget::AllAdjacent | 
            PSMoveTarget::All
        )
    }

    /// Check if a move is blocked by Quick Guard
    pub fn is_blocked_by_quick_guard(
        &self,
        state: &State,
        move_priority: i8,
        target_side: SideReference,
    ) -> bool {
        if move_priority <= 0 {
            return false;
        }

        // Check if the target side has Quick Guard active
        let _side = state.get_side(target_side);
        
        // In a full implementation, this would check for QuickGuard side condition
        false
    }

    /// Calculate ally damage for spread moves
    pub fn calculate_ally_damage(
        &self,
        base_damage: i16,
        move_target: &PSMoveTarget,
        user_position: BattlePosition,
        ally_position: BattlePosition,
        state: &State,
    ) -> Option<i16> {
        if !self.move_hits_ally(move_target, user_position) {
            return None;
        }

        // Check if ally is present and active
        if !state.is_position_active(ally_position) {
            return None;
        }

        // Apply spread move damage reduction
        let damage_multiplier = if self.format.supports_spread_moves() {
            0.75
        } else {
            1.0
        };

        Some((base_damage as f32 * damage_multiplier) as i16)
    }

    /// Handle Protect-style moves interaction in doubles
    pub fn handle_protect_interaction(
        &self,
        state: &State,
        target_position: BattlePosition,
        move_target: &PSMoveTarget,
    ) -> bool {
        let pokemon = match state.get_pokemon_at_position(target_position) {
            Some(p) => p,
            None => return false,
        };

        // Check if Pokemon is protected
        if pokemon.volatile_statuses.contains(&VolatileStatus::Protect) {
            // Protect blocks most moves, but not status moves targeting allies
            match move_target {
                PSMoveTarget::AdjacentAlly | PSMoveTarget::AllyTeam => false,
                _ => true,
            }
        } else {
            false
        }
    }

    /// Generate instructions for doubles-specific move interactions
    pub fn generate_doubles_move_instructions(
        &self,
        state: &State,
        move_choice: &MoveChoice,
        user_position: BattlePosition,
    ) -> Option<Vec<StateInstructions>> {
        let move_index = move_choice.move_index()?;
        
        // Get move data
        let side = state.get_side(user_position.side);
        let pokemon = side.get_active_pokemon_at_slot(user_position.slot)?;
        let move_data = pokemon.get_move(move_index)?;

        // Handle specific doubles moves
        match move_data.name.as_str() {
            "Follow Me" | "Rage Powder" => {
                Some(self.generate_follow_me_instructions(user_position))
            }
            "Helping Hand" => {
                let ally_pos = self.get_ally_position(user_position);
                Some(self.generate_helping_hand_instructions(user_position, ally_pos))
            }
            "Wide Guard" => {
                Some(self.generate_wide_guard_instructions(user_position))
            }
            "Quick Guard" => {
                Some(self.generate_quick_guard_instructions(user_position))
            }
            _ => None,
        }
    }

    /// Check if position is adjacent in doubles (for moves like Beat Up, etc.)
    pub fn is_adjacent_position(&self, pos1: BattlePosition, pos2: BattlePosition) -> bool {
        if self.format.active_pokemon_count() <= 1 {
            return false;
        }

        // In doubles, positions are adjacent if they're on the same side
        // or if they're across from each other
        pos1.side == pos2.side || 
        (pos1.side != pos2.side && pos1.slot == pos2.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Pokemon;
    use crate::move_choice::MoveIndex;
    use crate::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    fn create_doubles_state() -> State {
        let mut state = State::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        // Add Pokemon to all positions
        for side in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..2 {
                let pokemon = Pokemon::new(format!("Pokemon-{:?}-{}", side, slot));
                state.get_side_mut(side).add_pokemon(pokemon);
                state.get_side_mut(side).set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        state
    }

    #[test]
    fn test_ally_position_calculation() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        let pos_0_0 = BattlePosition::new(SideReference::SideOne, 0);
        let pos_0_1 = BattlePosition::new(SideReference::SideOne, 1);
        
        assert_eq!(mechanics.get_ally_position(pos_0_0), Some(pos_0_1));
        assert_eq!(mechanics.get_ally_position(pos_0_1), Some(pos_0_0));
    }

    #[test]
    fn test_move_hits_ally() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Earthquake should hit ally
        assert!(mechanics.move_hits_ally(&PSMoveTarget::AllAdjacent, user_pos));
        
        // Single target move should not hit ally
        assert!(!mechanics.move_hits_ally(&PSMoveTarget::Normal, user_pos));
        
        // Self-Destruct should hit ally
        assert!(mechanics.move_hits_ally(&PSMoveTarget::All, user_pos));
    }

    #[test]
    fn test_adjacency_check() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        let pos_1_0 = BattlePosition::new(SideReference::SideOne, 0);
        let pos_1_1 = BattlePosition::new(SideReference::SideOne, 1);
        let pos_2_0 = BattlePosition::new(SideReference::SideTwo, 0);
        let pos_2_1 = BattlePosition::new(SideReference::SideTwo, 1);
        
        // Same side positions are adjacent
        assert!(mechanics.is_adjacent_position(pos_1_0, pos_1_1));
        
        // Opposite slots are adjacent
        assert!(mechanics.is_adjacent_position(pos_1_0, pos_2_0));
        assert!(mechanics.is_adjacent_position(pos_1_1, pos_2_1));
        
        // Diagonal positions are not adjacent in standard doubles
        assert!(!mechanics.is_adjacent_position(pos_1_0, pos_2_1));
        assert!(!mechanics.is_adjacent_position(pos_1_1, pos_2_0));
    }

    #[test]
    fn test_redirection_mechanics() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let mut state = create_doubles_state();
        
        // Add Follow Me status to a Pokemon
        if let Some(pokemon) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            pokemon.volatile_statuses.insert(VolatileStatus::FollowMe);
        }
        
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let original_target = BattlePosition::new(SideReference::SideTwo, 1);
        let expected_redirect = BattlePosition::new(SideReference::SideTwo, 0);
        
        let redirected = mechanics.apply_redirection_mechanics(
            &state, 
            &[original_target], 
            user_pos
        );
        
        assert_eq!(redirected, vec![expected_redirect]);
    }

    #[test]
    fn test_protect_interaction() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let mut state = create_doubles_state();
        
        // Add Protect status to a Pokemon
        let protected_pos = BattlePosition::new(SideReference::SideTwo, 0);
        if let Some(pokemon) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            pokemon.volatile_statuses.insert(VolatileStatus::Protect);
        }
        
        // Damaging move should be blocked
        assert!(mechanics.handle_protect_interaction(&state, protected_pos, &PSMoveTarget::Normal));
        
        // Ally move should not be blocked
        assert!(!mechanics.handle_protect_interaction(&state, protected_pos, &PSMoveTarget::AdjacentAlly));
    }

    #[test]
    fn test_follow_me_instruction_generation() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = mechanics.generate_follow_me_instructions(user_pos);
        
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0].percentage, 100.0);
        assert_eq!(instructions[0].instruction_list.len(), 1);
        
        if let Instruction::ApplyVolatileStatus(status_instr) = &instructions[0].instruction_list[0] {
            assert_eq!(status_instr.target_position, user_pos);
            assert_eq!(status_instr.volatile_status, VolatileStatus::FollowMe);
        } else {
            panic!("Expected ApplyVolatileStatus instruction");
        }
    }

    #[test]
    fn test_helping_hand_instruction_generation() {
        let mechanics = DoublesSpecificMechanics::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let ally_pos = BattlePosition::new(SideReference::SideOne, 1);
        
        let instructions = mechanics.generate_helping_hand_instructions(user_pos, Some(ally_pos));
        
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0].instruction_list.len(), 1);
        
        if let Instruction::ApplyVolatileStatus(status_instr) = &instructions[0].instruction_list[0] {
            assert_eq!(status_instr.target_position, ally_pos);
            assert_eq!(status_instr.volatile_status, VolatileStatus::HelpingHand);
        } else {
            panic!("Expected ApplyVolatileStatus instruction");
        }
    }
}