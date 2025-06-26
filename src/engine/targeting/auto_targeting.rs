//! # Pokemon Showdown Compatible Targeting System
//! 
//! This module provides a targeting system that uses Pokemon Showdown's
//! move target conventions directly, enabling seamless PS data integration.

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::data::showdown_types::MoveTarget;
use crate::core::battle_state::BattleState;

/// PS-compatible auto-targeting engine
pub struct AutoTargetingEngine {
    format: BattleFormat,
}

impl AutoTargetingEngine {
    pub fn new(format: BattleFormat) -> Self {
        Self { format }
    }

    /// Resolve targets for a PS move target in the current format
    pub fn resolve_targets(
        &self,
        target: MoveTarget,
        user_position: BattlePosition,
        state: &BattleState,
    ) -> Vec<BattlePosition> {
        let user_side = user_position.side;
        let user_slot = user_position.slot;
        let opponent_side = user_side.opposite();
        let active_per_side = self.format.active_pokemon_count();

        match target {
            MoveTarget::Self_ => {
                vec![user_position]
            }
            
            MoveTarget::Normal | MoveTarget::AdjacentFoe => {
                // In singles, target the opposing Pokemon
                // In doubles, target the opposing Pokemon in front (or first available)
                self.get_default_opponent_target(opponent_side, user_slot, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            MoveTarget::AllAdjacentFoes => {
                // All active opposing Pokemon
                self.get_all_active_opponents(opponent_side, state)
            }
            
            MoveTarget::AllAdjacent => {
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
            
            MoveTarget::AdjacentAlly => {
                // Only in doubles - target the ally
                if active_per_side > 1 {
                    self.get_ally_position(user_position, state)
                        .map(|pos| vec![pos])
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            }
            
            MoveTarget::AdjacentAllyOrSelf => {
                // Default to self (user can override with explicit target)
                vec![user_position]
            }
            
            MoveTarget::Any => {
                // Long-range move - default to first opponent
                // In full implementation, this would allow targeting any Pokemon
                self.get_any_opponent_target(opponent_side, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            MoveTarget::RandomNormal => {
                // Random opponent - select random target from available opponents
                self.get_random_opponent_target(opponent_side, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            }
            
            MoveTarget::Allies => {
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
            MoveTarget::All | MoveTarget::AllySide | MoveTarget::FoeSide => {
                vec![]
            }
            
            // Team targets affect all team members (not position-based)
            MoveTarget::AllyTeam => {
                vec![]
            }
            
            // Scripted moves need special handling (Counter, Mirror Coat)
            MoveTarget::Scripted => {
                // Target the last Pokemon that damaged this Pokemon with a direct attack
                if let Some(damage_info) = state.turn_info.damaged_this_turn.get(&user_position) {
                    if damage_info.is_direct_damage {
                        vec![damage_info.attacker_position]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
        }
    }

    /// Get the default opponent target (prefers adjacent in doubles)
    fn get_default_opponent_target(
        &self,
        opponent_side: SideReference,
        user_slot: usize,
        state: &BattleState,
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
        state: &BattleState,
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
        state: &BattleState,
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
        state: &BattleState,
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

    /// Get a random active opponent (for RandomNormal targeting)
    fn get_random_opponent_target(
        &self,
        opponent_side: SideReference,
        state: &BattleState,
    ) -> Option<BattlePosition> {
        let active_opponents = self.get_all_active_opponents(opponent_side, state);
        
        if active_opponents.is_empty() {
            return None;
        }
        
        // Use proper randomization
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        active_opponents.choose(&mut rng).copied()
    }

    /// Check if explicit targets are valid for the given PS target type
    pub fn validate_targets(
        &self,
        target: MoveTarget,
        user_position: BattlePosition,
        explicit_targets: &[BattlePosition],
        state: &BattleState,
    ) -> Result<(), String> {
        // Field effects don't have position targets
        if target.is_field_target() && !explicit_targets.is_empty() {
            return Err("Field effect moves cannot have position targets".to_string());
        }

        // Validate each target is appropriate for the move
        for &target_position in explicit_targets {
            // Check if position is active
            if !state.is_position_active(target_position) {
                return Err(format!("Target position {:?} has no active Pokemon", target_position));
            }

            // Check targeting restrictions
            match target {
                MoveTarget::Self_ => {
                    if target_position != user_position {
                        return Err("Self-targeting moves can only target the user".to_string());
                    }
                }
                MoveTarget::AdjacentAlly | MoveTarget::Allies => {
                    if target_position.side != user_position.side || target_position == user_position {
                        return Err("Ally-targeting moves can only target allies".to_string());
                    }
                }
                MoveTarget::Normal | MoveTarget::AdjacentFoe => {
                    if target_position.side == user_position.side {
                        return Err("Opponent-targeting moves cannot target allies".to_string());
                    }
                }
                MoveTarget::AdjacentAllyOrSelf => {
                    if target_position.side != user_position.side {
                        return Err("This move can only target user or allies".to_string());
                    }
                }
                // Any allows any target
                MoveTarget::Any => {}
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
        move_choice: &mut crate::core::move_choice::MoveChoice,
        state: &BattleState,
    ) -> Result<(), String> {
        use crate::core::battle_format::BattlePosition;

        // If move choice already has targets, don't modify them
        // But treat empty target lists as needing auto-resolution
        if let Some(targets) = move_choice.target_positions() {
            if !targets.is_empty() {
                return Ok(());
            }
        }

        // Get the move index from the choice
        let move_index = match move_choice.move_index() {
            Some(index) => index,
            None => return Ok(()), // No targets for switches or no-moves
        };

        let user_position = BattlePosition::new(user_side, user_slot);
        
        // Get the move and its PS target type
        let side = state.get_side(user_side.to_index());
        let pokemon = side.and_then(|s| s.get_active_pokemon_at_slot(user_slot))
            .ok_or("No active Pokemon at specified slot")?;
        
        let move_data = pokemon.get_move(move_index)
            .ok_or("Move not found on Pokemon")?;
        
        let move_target = move_data.target;

        // Resolve targets using PS targeting system
        let targets = self.resolve_targets(move_target, user_position, state);
        
        // Validate the resolved targets
        self.validate_targets(move_target, user_position, &targets, state)?;
        
        // Update the move choice with resolved targets
        move_choice.set_target_positions(targets);
        
        Ok(())
    }
}

