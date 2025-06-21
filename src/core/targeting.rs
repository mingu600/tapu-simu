//! # Unified Targeting System
//! 
//! A single, function-based targeting system that replaces the complex 
//! AutoTargetingEngine, FormatMoveTargetResolver, and format_targeting modules.

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::battle_state::BattleState;
use crate::core::move_choice::MoveChoice;
use crate::data::showdown_types::MoveTarget;
use crate::types::BattleError;

/// Resolve targets for a move based on its target type, user position, format, and battle state
pub fn resolve_targets(
    move_target: MoveTarget,
    user_pos: BattlePosition,
    format: &BattleFormat,
    state: &BattleState,
) -> Vec<BattlePosition> {
    let user_side = user_pos.side;
    let user_slot = user_pos.slot;
    let opponent_side = user_side.opposite();
    let active_per_side = format.active_pokemon_count();

    match move_target {
        MoveTarget::Self_ => {
            vec![user_pos]
        }
        
        MoveTarget::Normal | MoveTarget::AdjacentFoe => {
            // In singles, target the opponent
            // In doubles, target the opponent in front (or first available)
            default_opponent_target(opponent_side, user_slot, format, state)
                .map(|pos| vec![pos])
                .unwrap_or_default()
        }
        
        MoveTarget::AllAdjacentFoes => {
            // All active opponents
            all_active_opponents(opponent_side, format, state)
        }
        
        MoveTarget::AllAdjacent => {
            // All adjacent Pokemon (opponents + ally in doubles)
            let mut targets = all_active_opponents(opponent_side, format, state);
            
            // Add ally in doubles
            if active_per_side > 1 {
                if let Some(ally) = ally_position(user_pos, format, state) {
                    targets.push(ally);
                }
            }
            
            targets
        }
        
        MoveTarget::AdjacentAlly => {
            // Only in doubles - target the ally
            if active_per_side > 1 {
                ally_position(user_pos, format, state)
                    .map(|pos| vec![pos])
                    .unwrap_or_default()
            } else {
                vec![]
            }
        }
        
        MoveTarget::AdjacentAllyOrSelf => {
            // Default to self (user can override with explicit target)
            vec![user_pos]
        }
        
        MoveTarget::Any => {
            // Long-range move - default to first opponent
            any_opponent_target(opponent_side, format, state)
                .map(|pos| vec![pos])
                .unwrap_or_default()
        }
        
        MoveTarget::RandomNormal => {
            // Random opponent - select random target from available opponents
            random_opponent_target(opponent_side, format, state)
                .map(|pos| vec![pos])
                .unwrap_or_default()
        }
        
        MoveTarget::Allies => {
            // All active allies (not including user)
            let mut targets = vec![];
            if active_per_side > 1 {
                if let Some(ally) = ally_position(user_pos, format, state) {
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
            // Would need to track last attacker
            vec![]
        }
    }
}

/// Validate that targets are appropriate for the given move target type
pub fn validate_targets(
    move_target: MoveTarget,
    user_pos: BattlePosition,
    targets: &[BattlePosition],
    state: &BattleState,
) -> Result<(), BattleError> {
    // Field effects don't have position targets
    if is_field_target(move_target) && !targets.is_empty() {
        return Err(BattleError::InvalidMoveChoice {
            reason: "Field effect moves cannot have position targets".to_string(),
        });
    }

    // Validate each target is appropriate for the move
    for &target in targets {
        // Check if position is active
        if !state.is_position_active(target) {
            return Err(BattleError::InvalidMoveChoice {
                reason: format!("Target position {:?} has no active Pokemon", target),
            });
        }

        // Check targeting restrictions
        match move_target {
            MoveTarget::Self_ => {
                if target != user_pos {
                    return Err(BattleError::InvalidMoveChoice {
                        reason: "Self-targeting moves can only target the user".to_string(),
                    });
                }
            }
            MoveTarget::AdjacentAlly | MoveTarget::Allies => {
                if target.side != user_pos.side || target == user_pos {
                    return Err(BattleError::InvalidMoveChoice {
                        reason: "Ally-targeting moves can only target allies".to_string(),
                    });
                }
            }
            MoveTarget::Normal | MoveTarget::AdjacentFoe => {
                if target.side == user_pos.side {
                    return Err(BattleError::InvalidMoveChoice {
                        reason: "Opponent-targeting moves cannot target allies".to_string(),
                    });
                }
            }
            MoveTarget::AdjacentAllyOrSelf => {
                if target.side != user_pos.side {
                    return Err(BattleError::InvalidMoveChoice {
                        reason: "This move can only target user or allies".to_string(),
                    });
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

/// Check if a move target type affects the field rather than specific positions
fn is_field_target(move_target: MoveTarget) -> bool {
    matches!(
        move_target,
        MoveTarget::All | MoveTarget::AllySide | MoveTarget::FoeSide | MoveTarget::AllyTeam
    )
}

// Helper functions (simplified from the complex targeting engines)

fn default_opponent_target(
    opponent_side: SideReference,
    user_slot: usize,
    format: &BattleFormat,
    state: &BattleState,
) -> Option<BattlePosition> {
    // In singles, just get the active opponent
    if format.active_pokemon_count() == 1 {
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
    for slot in 0..format.active_pokemon_count() {
        let position = BattlePosition::new(opponent_side, slot);
        if state.is_position_active(position) {
            return Some(position);
        }
    }
    
    None
}

fn any_opponent_target(
    opponent_side: SideReference,
    format: &BattleFormat,
    state: &BattleState,
) -> Option<BattlePosition> {
    for slot in 0..format.active_pokemon_count() {
        let position = BattlePosition::new(opponent_side, slot);
        if state.is_position_active(position) {
            return Some(position);
        }
    }
    None
}

fn all_active_opponents(
    opponent_side: SideReference,
    format: &BattleFormat,
    state: &BattleState,
) -> Vec<BattlePosition> {
    (0..format.active_pokemon_count())
        .map(|slot| BattlePosition::new(opponent_side, slot))
        .filter(|&pos| state.is_position_active(pos))
        .collect()
}

fn ally_position(
    user_pos: BattlePosition,
    format: &BattleFormat,
    state: &BattleState,
) -> Option<BattlePosition> {
    if format.active_pokemon_count() <= 1 {
        return None;
    }
    
    let ally_slot = 1 - user_pos.slot;
    let ally_position = BattlePosition::new(user_pos.side, ally_slot);
    
    if state.is_position_active(ally_position) {
        Some(ally_position)
    } else {
        None
    }
}

fn random_opponent_target(
    opponent_side: SideReference,
    format: &BattleFormat,
    state: &BattleState,
) -> Option<BattlePosition> {
    let active_opponents = all_active_opponents(opponent_side, format, state);
    
    if active_opponents.is_empty() {
        return None;
    }
    
    // Use proper randomization
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    active_opponents.choose(&mut rng).copied()
}

/// Auto-resolve targets for a move choice if they haven't been explicitly set
/// This replaces the functionality from PSAutoTargetingEngine::auto_resolve_targets
pub fn auto_resolve_targets(
    user_side: SideReference,
    user_slot: usize,
    move_choice: &mut MoveChoice,
    format: &BattleFormat,
    state: &BattleState,
) -> Result<(), BattleError> {
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
    let side_index = match user_side {
        SideReference::SideOne => 0,
        SideReference::SideTwo => 1,
    };
    let side = state.get_side(side_index);
    let pokemon = side.and_then(|s| s.get_active_pokemon_at_slot(user_slot))
        .ok_or_else(|| BattleError::InvalidMoveChoice {
            reason: "No active Pokemon at specified slot".to_string(),
        })?;
    
    let move_data = pokemon.get_move(move_index)
        .ok_or_else(|| BattleError::InvalidMoveChoice {
            reason: "Move not found on Pokemon".to_string(),
        })?;
    
    let move_target = move_data.target;

    // Resolve targets using unified targeting system
    let targets = resolve_targets(move_target, user_position, format, state);
    
    // Validate the resolved targets
    validate_targets(move_target, user_position, &targets, state)?;
    
    // Update the move choice with resolved targets
    move_choice.set_target_positions(targets);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::Pokemon;
    use crate::generation::Generation;
    use crate::core::battle_format::FormatType;

    fn create_test_state_doubles() -> BattleState {
        let mut state = BattleState::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        
        // Set up active Pokemon on both sides
        state.side_one.active_pokemon_indices = vec![Some(0), Some(1)];
        state.side_two.active_pokemon_indices = vec![Some(0), Some(1)];
        
        // Add some Pokemon to teams
        for _ in 0..2 {
            let mut pokemon_one = Pokemon::new("Test".to_string());
            pokemon_one.ability = "Test".to_string();
            pokemon_one.hp = 100;
            pokemon_one.max_hp = 100;
            
            let mut pokemon_two = Pokemon::new("Test".to_string());
            pokemon_two.ability = "Test".to_string();
            pokemon_two.hp = 100;
            pokemon_two.max_hp = 100;
            
            state.side_one.pokemon.push(pokemon_one);
            state.side_two.pokemon.push(pokemon_two);
        }
        
        state
    }

    #[test]
    fn test_self_targeting() {
        let state = create_test_state_doubles();
        let format = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = resolve_targets(MoveTarget::Self_, user_pos, &format, &state);
        assert_eq!(targets, vec![user_pos]);
    }

    #[test]
    fn test_spread_move_targeting() {
        let state = create_test_state_doubles();
        let format = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = resolve_targets(MoveTarget::AllAdjacentFoes, user_pos, &format, &state);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&BattlePosition::new(SideReference::SideTwo, 0)));
        assert!(targets.contains(&BattlePosition::new(SideReference::SideTwo, 1)));
    }

    #[test]
    fn test_ally_targeting() {
        let state = create_test_state_doubles();
        let format = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let targets = resolve_targets(MoveTarget::AdjacentAlly, user_pos, &format, &state);
        assert_eq!(targets, vec![BattlePosition::new(SideReference::SideOne, 1)]);
    }

    #[test]
    fn test_validation() {
        let state = create_test_state_doubles();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Valid self-targeting
        let targets = vec![user_pos];
        assert!(validate_targets(MoveTarget::Self_, user_pos, &targets, &state).is_ok());
        
        // Invalid self-targeting (targeting someone else)
        let targets = vec![BattlePosition::new(SideReference::SideTwo, 0)];
        assert!(validate_targets(MoveTarget::Self_, user_pos, &targets, &state).is_err());
    }

    #[test]
    fn test_auto_resolve_targets() {
        use crate::core::move_choice::{MoveChoice, MoveIndex};
        use crate::core::battle_state::Move;
        use crate::data::showdown_types::MoveTarget;
        
        let mut state = create_test_state_doubles();
        let format = BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles);
        
        // Add a move to the first Pokemon
        let tackle = Move {
            name: "Tackle".to_string(),
            base_power: 40,
            accuracy: 100,
            move_type: "Normal".to_string(),
            pp: 35,
            max_pp: 35,
            target: MoveTarget::Normal,
            category: crate::core::battle_state::MoveCategory::Physical,
            priority: 0,
        };
        
        if let Some(pokemon) = state.side_one.pokemon.get_mut(0) {
            pokemon.moves.insert(MoveIndex::M0, tackle);
        }
        
        // Create a move choice without explicit targets
        let mut move_choice = MoveChoice::new_move(MoveIndex::M0, vec![]);
        
        // Auto-resolve targets should work
        let result = auto_resolve_targets(
            SideReference::SideOne,
            0,
            &mut move_choice,
            &format,
            &state,
        );
        
        assert!(result.is_ok(), "Auto-resolve should succeed");
        
        // Should have resolved to targeting an opponent
        let targets = move_choice.target_positions().unwrap();
        assert!(!targets.is_empty(), "Should have resolved targets");
        assert_eq!(targets[0].side, SideReference::SideTwo, "Should target opponent");
    }
}