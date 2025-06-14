//! # Pokemon Showdown Compatible Targeting System
//! 
//! This module provides a targeting system that uses Pokemon Showdown's
//! move target conventions directly, enabling seamless PS data integration.

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::data::ps_types::PSMoveTarget;
use crate::state::State;

/// PS-compatible auto-targeting engine
pub struct PSAutoTargetingEngine {
    format: BattleFormat,
}

impl PSAutoTargetingEngine {
    pub fn new(format: BattleFormat) -> Self {
        Self { format }
    }

    /// Resolve targets for a PS move target in the current format
    pub fn resolve_targets(
        &self,
        ps_target: PSMoveTarget,
        user_position: BattlePosition,
        state: &State,
    ) -> Vec<BattlePosition> {
        let user_side = user_position.side;
        let user_slot = user_position.slot;
        let opponent_side = user_side.opposite();
        let active_per_side = self.format.active_pokemon_count();

        match ps_target {
            PSMoveTarget::Self_ => {
                vec![user_position]
            }
            
            PSMoveTarget::Normal | PSMoveTarget::AdjacentFoe => {
                // In singles, target the opponent
                // In doubles, target the opponent in front (or first available)
                self.get_default_opponent_target(opponent_side, user_slot, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            PSMoveTarget::AllAdjacentFoes => {
                // All active opponents
                self.get_all_active_opponents(opponent_side, state)
            }
            
            PSMoveTarget::AllAdjacent => {
                // All adjacent Pokemon (opponents + ally in doubles)
                let mut targets = self.get_all_active_opponents(opponent_side, state);
                
                // Add ally in doubles
                if active_per_side > 1 {
                    if let Some(ally) = self.get_ally_position(user_position, state) {
                        targets.push(ally);
                    }
                }
                
                targets
            }
            
            PSMoveTarget::AdjacentAlly => {
                // Only in doubles - target the ally
                if active_per_side > 1 {
                    self.get_ally_position(user_position, state)
                        .map(|pos| vec![pos])
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            }
            
            PSMoveTarget::AdjacentAllyOrSelf => {
                // Default to self (user can override with explicit target)
                vec![user_position]
            }
            
            PSMoveTarget::Any => {
                // Long-range move - default to first opponent
                // In full implementation, this would allow targeting any Pokemon
                self.get_any_opponent_target(opponent_side, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            PSMoveTarget::RandomNormal => {
                // Random opponent - for now just use first available
                self.get_any_opponent_target(opponent_side, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            PSMoveTarget::Allies => {
                // All active allies (not including user)
                let mut targets = vec![];
                if active_per_side > 1 {
                    if let Some(ally) = self.get_ally_position(user_position, state) {
                        targets.push(ally);
                    }
                }
                targets
            }
            
            // Field/side targets don't have position targets
            PSMoveTarget::All | PSMoveTarget::AllySide | PSMoveTarget::FoeSide => {
                vec![]
            }
            
            // Team targets affect all team members (not position-based)
            PSMoveTarget::AllyTeam => {
                vec![]
            }
            
            // Scripted moves need special handling (Counter, Mirror Coat)
            PSMoveTarget::Scripted => {
                // Would need to track last attacker
                vec![]
            }
        }
    }

    /// Get the default opponent target (prefers adjacent in doubles)
    fn get_default_opponent_target(
        &self,
        opponent_side: SideReference,
        user_slot: usize,
        state: &State,
    ) -> Option<BattlePosition> {
        // In singles, just get the active opponent
        if self.format.active_pokemon_count() == 1 {
            let position = BattlePosition::new(opponent_side, 0);
            if state.is_position_active(position) {
                return Some(position);
            }
        }
        
        // In doubles, prefer the opponent "in front"
        let preferred_slot = user_slot; // Same slot on opposite side
        let position = BattlePosition::new(opponent_side, preferred_slot);
        if state.is_position_active(position) {
            return Some(position);
        }
        
        // Otherwise, get any active opponent
        for slot in 0..self.format.active_pokemon_count() {
            let position = BattlePosition::new(opponent_side, slot);
            if state.is_position_active(position) {
                return Some(position);
            }
        }
        
        None
    }

    /// Get any active opponent (for long-range moves)
    fn get_any_opponent_target(
        &self,
        opponent_side: SideReference,
        state: &State,
    ) -> Option<BattlePosition> {
        for slot in 0..self.format.active_pokemon_count() {
            let position = BattlePosition::new(opponent_side, slot);
            if state.is_position_active(position) {
                return Some(position);
            }
        }
        None
    }

    /// Get all active opponents
    fn get_all_active_opponents(
        &self,
        opponent_side: SideReference,
        state: &State,
    ) -> Vec<BattlePosition> {
        (0..self.format.active_pokemon_count())
            .map(|slot| BattlePosition::new(opponent_side, slot))
            .filter(|&pos| state.is_position_active(pos))
            .collect()
    }

    /// Get ally position in doubles
    fn get_ally_position(
        &self,
        user_position: BattlePosition,
        state: &State,
    ) -> Option<BattlePosition> {
        if self.format.active_pokemon_count() <= 1 {
            return None;
        }
        
        let ally_slot = 1 - user_position.slot;
        let ally_position = BattlePosition::new(user_position.side, ally_slot);
        
        if state.is_position_active(ally_position) {
            Some(ally_position)
        } else {
            None
        }
    }

    /// Check if explicit targets are valid for the given PS target type
    pub fn validate_targets(
        &self,
        ps_target: PSMoveTarget,
        user_position: BattlePosition,
        explicit_targets: &[BattlePosition],
        state: &State,
    ) -> Result<(), String> {
        // Field effects don't have position targets
        if ps_target.is_field_target() && !explicit_targets.is_empty() {
            return Err("Field effect moves cannot have position targets".to_string());
        }

        // Validate each target is appropriate for the move
        for &target in explicit_targets {
            // Check if position is active
            if !state.is_position_active(target) {
                return Err(format!("Target position {:?} has no active Pokemon", target));
            }

            // Check targeting restrictions
            match ps_target {
                PSMoveTarget::Self_ => {
                    if target != user_position {
                        return Err("Self-targeting moves can only target the user".to_string());
                    }
                }
                PSMoveTarget::AdjacentAlly | PSMoveTarget::Allies => {
                    if target.side != user_position.side || target == user_position {
                        return Err("Ally-targeting moves can only target allies".to_string());
                    }
                }
                PSMoveTarget::Normal | PSMoveTarget::AdjacentFoe => {
                    if target.side == user_position.side {
                        return Err("Opponent-targeting moves cannot target allies".to_string());
                    }
                }
                PSMoveTarget::AdjacentAllyOrSelf => {
                    if target.side != user_position.side {
                        return Err("This move can only target user or allies".to_string());
                    }
                }
                // Any allows any target
                PSMoveTarget::Any => {}
                // Spread moves are validated differently
                _ => {}
            }
        }

        Ok(())
    }

    /// Auto-resolve targets for a move choice, compatible with existing code
    pub fn auto_resolve_targets(
        &self,
        user_side: SideReference,
        user_slot: usize,
        move_choice: &mut crate::move_choice::MoveChoice,
        state: &State,
    ) -> Result<(), String> {
        use crate::battle_format::BattlePosition;

        // If move choice already has targets, don't modify them
        if move_choice.target_positions().is_some() {
            return Ok(());
        }

        // Get the move index from the choice
        let move_index = match move_choice.move_index() {
            Some(index) => index,
            None => return Ok(()), // No targets for switches or no-moves
        };

        let user_position = BattlePosition::new(user_side, user_slot);
        
        // Get the move and its PS target type
        let side = state.get_side(user_side);
        let pokemon = side.get_active_pokemon_at_slot(user_slot)
            .ok_or("No active Pokemon at specified slot")?;
        
        let move_data = pokemon.get_move(move_index)
            .ok_or("Move not found on Pokemon")?;
        
        let ps_target = move_data.target;

        // Resolve targets using PS targeting system
        let targets = self.resolve_targets(ps_target, user_position, state);
        
        // Validate the resolved targets
        self.validate_targets(ps_target, user_position, &targets, state)?;
        
        // Update the move choice with resolved targets
        move_choice.set_target_positions(targets);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{State, Pokemon, PokemonStatus, StatBoosts};
    use crate::move_choice::MoveIndex;
    use std::collections::HashMap;

    fn create_test_state_doubles() -> State {
        let mut state = State::new(BattleFormat::Doubles);
        
        // Set up active Pokemon on both sides
        state.side_one.active_pokemon = vec![Some(0), Some(1)];
        state.side_two.active_pokemon = vec![Some(0), Some(1)];
        
        // Add some Pokemon to teams
        for _ in 0..2 {
            state.side_one.pokemon.push(Pokemon {
                species: "Test".to_string(),
                level: 50,
                hp: 100,
                max_hp: 100,
                status: PokemonStatus::None,
                status_duration: 0,
                stats: StatBoosts::default(),
                stat_boosts: StatBoosts::default(),
                moves: HashMap::new(),
                types: vec!["Normal".to_string()],
                volatile_statuses: HashMap::new(),
                substitute_health: None,
                ability: "Test".to_string(),
            });
            state.side_two.pokemon.push(Pokemon {
                species: "Test".to_string(),
                level: 50,
                hp: 100,
                max_hp: 100,
                status: PokemonStatus::None,
                status_duration: 0,
                stats: StatBoosts::default(),
                stat_boosts: StatBoosts::default(),
                moves: HashMap::new(),
                types: vec!["Normal".to_string()],
                volatile_statuses: HashMap::new(),
                substitute_health: None,
                ability: "Test".to_string(),
            });
        }
        
        state
    }

    #[test]
    fn test_ps_self_targeting() {
        let state = create_test_state_doubles();
        let engine = PSAutoTargetingEngine::new(BattleFormat::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = engine.resolve_targets(PSMoveTarget::Self_, user_pos, &state);
        assert_eq!(targets, vec![user_pos]);
    }

    #[test]
    fn test_ps_spread_move_targeting() {
        let state = create_test_state_doubles();
        let engine = PSAutoTargetingEngine::new(BattleFormat::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = engine.resolve_targets(PSMoveTarget::AllAdjacentFoes, user_pos, &state);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&BattlePosition::new(SideReference::SideTwo, 0)));
        assert!(targets.contains(&BattlePosition::new(SideReference::SideTwo, 1)));
    }

    #[test]
    fn test_ps_ally_targeting() {
        let state = create_test_state_doubles();
        let engine = PSAutoTargetingEngine::new(BattleFormat::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = engine.resolve_targets(PSMoveTarget::AdjacentAlly, user_pos, &state);
        assert_eq!(targets, vec![BattlePosition::new(SideReference::SideOne, 1)]);
    }
}