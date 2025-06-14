//! # Format-Aware Move Targeting System
//! 
//! This module provides format-aware move targeting that integrates Pokemon Showdown
//! PSMoveTarget enum with actual battle execution. It resolves move targets based on the current
//! battle format and validates target positions.

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::move_choice::MoveChoice;
use crate::state::State;

/// Move target resolution service for format-aware battles
pub struct FormatMoveTargetResolver {
    format: BattleFormat,
}

impl FormatMoveTargetResolver {
    /// Create a new format move target resolver
    pub fn new(format: BattleFormat) -> Self {
        Self { format }
    }

    /// Resolve targets for a move choice based on the current battle format
    pub fn resolve_move_targets(
        &self,
        user_side: SideReference,
        user_slot: usize,
        move_choice: &MoveChoice,
        state: &State,
    ) -> Result<Vec<BattlePosition>, String> {
        // If move choice already has explicit targets, use those
        if let Some(targets) = move_choice.target_positions() {
            return Ok(targets.clone());
        }

        // Otherwise, resolve targets based on the move's target type
        let move_index = match move_choice.move_index() {
            Some(index) => index,
            None => return Ok(vec![]), // No targets for switches or no-moves
        };

        let user_position = BattlePosition::new(user_side, user_slot);
        
        // Get the move target type from the move data
        let move_target = self.get_move_target(state, user_side, user_slot, move_index)?;
        
        // Resolve targets based on the move target type and current format
        self.resolve_targets_for_move_target(move_target, user_position, state)
    }

    /// Get the move target type for a specific move
    fn get_move_target(
        &self,
        state: &State,
        user_side: SideReference,
        user_slot: usize,
        move_index: crate::move_choice::MoveIndex,
    ) -> Result<crate::data::ps_types::PSMoveTarget, String> {
        let side = state.get_side(user_side);
        let pokemon = side.get_active_pokemon_at_slot(user_slot)
            .ok_or("No active Pokemon at specified slot")?;
        
        let move_data = pokemon.get_move(move_index)
            .ok_or("Move not found on Pokemon")?;
        
        Ok(move_data.target)
    }

    /// Resolve targets based on move target type and battle format
    /// This is a legacy bridge that delegates to PSAutoTargetingEngine
    fn resolve_targets_for_move_target(
        &self,
        move_target: crate::data::ps_types::PSMoveTarget,
        user_position: BattlePosition,
        state: &State,
    ) -> Result<Vec<BattlePosition>, String> {
        // Delegate to PS auto-targeting engine for accurate resolution
        let ps_engine = super::ps_targeting::PSAutoTargetingEngine::new(self.format.clone());
        let targets = ps_engine.resolve_targets(move_target, user_position, state);
        Ok(targets)
    }


    /// Validate that all target positions are valid for the current format
    pub fn validate_targets(&self, targets: &[BattlePosition], _state: &State) -> Result<(), String> {
        for target in targets {
            if !target.is_valid_for_format(&self.format) {
                return Err(format!("Target position {:?} is not valid for format {:?}", target, self.format));
            }
            
            // Note: We don't check if positions are active here because some moves
            // can target fainted positions or may become active during the turn
        }
        
        Ok(())
    }
}

/// Legacy auto-targeting engine - deprecated in favor of PSAutoTargetingEngine
/// This is kept for compatibility but should be replaced with PSAutoTargetingEngine
pub struct AutoTargetingEngine {
    ps_engine: super::ps_targeting::PSAutoTargetingEngine,
}

impl AutoTargetingEngine {
    /// Create a new auto-targeting engine using PS targeting system
    pub fn new(format: BattleFormat) -> Self {
        Self {
            ps_engine: super::ps_targeting::PSAutoTargetingEngine::new(format),
        }
    }

    /// Automatically resolve targets for a move choice if needed
    pub fn auto_resolve_targets(
        &self,
        user_side: SideReference,
        user_slot: usize,
        move_choice: &mut MoveChoice,
        state: &State,
    ) -> Result<(), String> {
        // Delegate to PS auto-targeting engine
        self.ps_engine.auto_resolve_targets(user_side, user_slot, move_choice, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Pokemon, Move, MoveCategory};
    use crate::move_choice::MoveIndex;

    fn create_test_state_with_pokemon() -> State {
        let mut state = State::new(BattleFormat::Doubles);
        
        // Add Pokemon to both sides
        let pokemon1 = Pokemon::new("TestPokemon1".to_string());
        let pokemon2 = Pokemon::new("TestPokemon2".to_string());
        
        state.side_one.add_pokemon(pokemon1);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(pokemon2);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));
        
        state
    }

    #[test]
    fn test_single_target_resolution() {
        let resolver = FormatMoveTargetResolver::new(BattleFormat::Singles);
        let state = create_test_state_with_pokemon();
        
        let move_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        
        let targets = resolver.resolve_move_targets(
            SideReference::SideOne, 
            0, 
            &move_choice, 
            &state
        );
        
        // Should resolve to the opponent's position
        assert!(targets.is_ok());
        let targets = targets.unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], BattlePosition::new(SideReference::SideTwo, 0));
    }

    #[test]
    fn test_spread_move_resolution() {
        let resolver = FormatMoveTargetResolver::new(BattleFormat::Doubles);
        let mut state = create_test_state_with_pokemon();
        
        // Add an ally and more opponents for doubles
        let ally = Pokemon::new("Ally".to_string());
        let opponent2 = Pokemon::new("Opponent2".to_string());
        
        state.side_one.add_pokemon(ally);
        state.side_one.set_active_pokemon_at_slot(1, Some(1));
        
        state.side_two.add_pokemon(opponent2);
        state.side_two.set_active_pokemon_at_slot(1, Some(1));
        
        // Add a spread move to the Pokemon
        let earthquake = Move::new_with_details(
            "Earthquake".to_string(),
            100,
            100,
            "Ground".to_string(),
            10,
            MoveTarget::AllOtherPokemon,
            MoveCategory::Physical,
            0,
        );
        
        if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
            pokemon.add_move(MoveIndex::M0, earthquake);
        }
        
        let move_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        
        let targets = resolver.resolve_move_targets(
            SideReference::SideOne, 
            0, 
            &move_choice, 
            &state
        );
        
        // Should target all other Pokemon (both opponents + ally)
        assert!(targets.is_ok());
        let targets = targets.unwrap();
        assert_eq!(targets.len(), 3); // 2 opponents + 1 ally
    }

    #[test]
    fn test_target_validation() {
        let resolver = FormatMoveTargetResolver::new(BattleFormat::Singles);
        let state = create_test_state_with_pokemon();
        
        // Valid target for singles
        let valid_targets = vec![BattlePosition::new(SideReference::SideTwo, 0)];
        assert!(resolver.validate_targets(&valid_targets, &state).is_ok());
        
        // Invalid target for singles (slot 1 doesn't exist)
        let invalid_targets = vec![BattlePosition::new(SideReference::SideTwo, 1)];
        assert!(resolver.validate_targets(&invalid_targets, &state).is_err());
    }

    #[test]
    fn test_auto_targeting_engine() {
        let engine = AutoTargetingEngine::new(BattleFormat::Singles);
        let state = create_test_state_with_pokemon();
        
        let mut move_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        
        let result = engine.auto_resolve_targets(
            SideReference::SideOne, 
            0, 
            &mut move_choice, 
            &state
        );
        
        assert!(result.is_ok());
        assert!(move_choice.target_positions().is_some());
        assert_eq!(move_choice.target_positions().unwrap().len(), 1);
    }
}