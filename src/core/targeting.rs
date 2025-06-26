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
            // In singles, target the opposing Pokemon
            // In doubles, target the opposing Pokemon in front (or first available)
            default_opponent_target(opponent_side, user_slot, format, state)
                .map(|pos| vec![pos])
                .unwrap_or_default()
        }
        
        MoveTarget::AllAdjacentFoes => {
            // All active opposing Pokemon
            all_active_opponents(opponent_side, format, state)
        }
        
        MoveTarget::AllAdjacent => {
            // All adjacent Pokemon (opposing Pokemon + ally in doubles)
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
            // Target the last Pokemon that damaged this Pokemon with a direct attack
            if let Some(damage_info) = state.turn_info.damaged_this_turn.get(&user_pos) {
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
    
    // Use the built-in ally_position method that handles format-specific logic
    if let Some(ally_pos) = user_pos.ally_position(format) {
        if state.is_position_active(ally_pos) {
            Some(ally_pos)
        } else {
            None
        }
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

