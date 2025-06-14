//! # Format-Aware Move Targeting System
//! 
//! This module provides format-aware move targeting that integrates rustemon MoveTarget
//! enum with actual battle execution. It resolves move targets based on the current
//! battle format and validates target positions.

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::data::types::MoveTarget;
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
    ) -> Result<MoveTarget, String> {
        let side = state.get_side(user_side);
        let pokemon = side.get_active_pokemon_at_slot(user_slot)
            .ok_or("No active Pokemon at specified slot")?;
        
        let move_data = pokemon.get_move(move_index)
            .ok_or("Move not found on Pokemon")?;
        
        Ok(move_data.target)
    }

    /// Resolve targets based on move target type and battle format
    fn resolve_targets_for_move_target(
        &self,
        move_target: MoveTarget,
        user_position: BattlePosition,
        state: &State,
    ) -> Result<Vec<BattlePosition>, String> {
        let opponent_side = user_position.side.opposite();
        let user_side = user_position.side;
        
        match move_target {
            MoveTarget::SelectedPokemon => {
                // Single target move - target the first available opponent
                self.get_single_opponent_target(opponent_side, state)
            }
            
            MoveTarget::AllOpponents => {
                // Target all active opponents
                self.get_all_opponent_targets(opponent_side, state)
            }
            
            MoveTarget::AllOtherPokemon => {
                // Target all other Pokemon (opponents + ally in doubles)
                let mut targets = self.get_all_opponent_targets(opponent_side, state)?;
                
                // Add ally if in multi-Pokemon format
                if self.format.active_pokemon_count() > 1 {
                    if let Some(ally_pos) = self.get_ally_position(user_position) {
                        if state.is_position_active(ally_pos) {
                            targets.push(ally_pos);
                        }
                    }
                }
                
                Ok(targets)
            }
            
            MoveTarget::User => {
                // Target the user
                Ok(vec![user_position])
            }
            
            MoveTarget::Ally => {
                // Target ally (only in multi-Pokemon formats)
                if self.format.active_pokemon_count() > 1 {
                    if let Some(ally_pos) = self.get_ally_position(user_position) {
                        if state.is_position_active(ally_pos) {
                            return Ok(vec![ally_pos]);
                        }
                    }
                }
                Ok(vec![])
            }
            
            MoveTarget::UserOrAlly => {
                // Can target user or ally - default to user
                Ok(vec![user_position])
            }
            
            MoveTarget::UserAndAllies => {
                // Target user and all allies
                let mut targets = vec![user_position];
                
                if self.format.active_pokemon_count() > 1 {
                    if let Some(ally_pos) = self.get_ally_position(user_position) {
                        if state.is_position_active(ally_pos) {
                            targets.push(ally_pos);
                        }
                    }
                }
                
                Ok(targets)
            }
            
            MoveTarget::AllPokemon => {
                // Target all Pokemon on the field
                let mut targets = vec![user_position];
                
                // Add ally
                if self.format.active_pokemon_count() > 1 {
                    if let Some(ally_pos) = self.get_ally_position(user_position) {
                        if state.is_position_active(ally_pos) {
                            targets.push(ally_pos);
                        }
                    }
                }
                
                // Add all opponents
                targets.extend(self.get_all_opponent_targets(opponent_side, state)?);
                
                Ok(targets)
            }
            
            MoveTarget::AllAllies => {
                // Target all allies (not including user)
                if self.format.active_pokemon_count() > 1 {
                    if let Some(ally_pos) = self.get_ally_position(user_position) {
                        if state.is_position_active(ally_pos) {
                            return Ok(vec![ally_pos]);
                        }
                    }
                }
                Ok(vec![])
            }
            
            MoveTarget::RandomOpponent => {
                // For deterministic resolution, choose the first available opponent
                self.get_single_opponent_target(opponent_side, state)
            }
            
            MoveTarget::UsersField | MoveTarget::OpponentsField | MoveTarget::EntireField => {
                // Field moves don't target specific positions
                Ok(vec![])
            }
            
            MoveTarget::SpecificMove | MoveTarget::SelectedPokemonMeFirst | MoveTarget::FaintingPokemon => {
                // Special targeting that requires additional context
                // For now, default to single opponent
                self.get_single_opponent_target(opponent_side, state)
            }
        }
    }

    /// Get a single opponent target (first available)
    fn get_single_opponent_target(&self, opponent_side: SideReference, state: &State) -> Result<Vec<BattlePosition>, String> {
        for slot in 0..self.format.active_pokemon_count() {
            let position = BattlePosition::new(opponent_side, slot);
            if state.is_position_active(position) {
                return Ok(vec![position]);
            }
        }
        
        Err("No valid opponent targets available".to_string())
    }

    /// Get all active opponent targets
    fn get_all_opponent_targets(&self, opponent_side: SideReference, state: &State) -> Result<Vec<BattlePosition>, String> {
        let mut targets = Vec::new();
        
        for slot in 0..self.format.active_pokemon_count() {
            let position = BattlePosition::new(opponent_side, slot);
            if state.is_position_active(position) {
                targets.push(position);
            }
        }
        
        if targets.is_empty() {
            Err("No active opponent targets available".to_string())
        } else {
            Ok(targets)
        }
    }

    /// Get the ally position for a user position (doubles/triples)
    fn get_ally_position(&self, user_position: BattlePosition) -> Option<BattlePosition> {
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

    /// Validate that all target positions are valid for the current format
    pub fn validate_targets(&self, targets: &[BattlePosition], state: &State) -> Result<(), String> {
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

/// Auto-targeting engine for moves that don't require manual selection
pub struct AutoTargetingEngine {
    resolver: FormatMoveTargetResolver,
}

impl AutoTargetingEngine {
    /// Create a new auto-targeting engine
    pub fn new(format: BattleFormat) -> Self {
        Self {
            resolver: FormatMoveTargetResolver::new(format),
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
        // If move choice already has targets, don't modify them
        if move_choice.target_positions().is_some() {
            return Ok(());
        }

        // Resolve targets automatically
        let targets = self.resolver.resolve_move_targets(user_side, user_slot, move_choice, state)?;
        
        // Validate the resolved targets
        self.resolver.validate_targets(&targets, state)?;
        
        // Update the move choice with resolved targets
        move_choice.set_target_positions(targets);
        
        Ok(())
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