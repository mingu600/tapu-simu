//! Centralized status effect application system
//!
//! This module provides a unified interface for applying status effects that consolidates
//! all the logic previously duplicated across move implementations. It handles immunity
//! checks, existing status interactions, duration management, and cure conditions.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, PokemonStatus, VolatileStatus, StatusInstruction, Stat, StatsInstruction};
use crate::data::showdown_types::MoveData;
use std::collections::HashMap;

/// Configuration for applying a status effect
#[derive(Debug, Clone)]
pub struct StatusApplication {
    /// The status to apply
    pub status: PokemonStatus,
    /// Target position
    pub target: BattlePosition,
    /// Chance of application (0.0 to 100.0)
    pub chance: f32,
    /// Duration in turns (None for permanent)
    pub duration: Option<u8>,
}

/// Configuration for applying a volatile status effect
#[derive(Debug, Clone)]
pub struct VolatileStatusApplication {
    /// The volatile status to apply
    pub status: VolatileStatus,
    /// Target position
    pub target: BattlePosition,
    /// Chance of application (0.0 to 100.0)
    pub chance: f32,
    /// Duration in turns (None for permanent)
    pub duration: Option<u8>,
}

/// Result of status application attempt
#[derive(Debug, Clone)]
pub struct StatusResult {
    /// Whether the status was successfully applied
    pub applied: bool,
    /// The instruction generated (if any)
    pub instruction: Option<BattleInstruction>,
    /// Reason for failure (if any)
    pub failure_reason: Option<StatusFailureReason>,
}

/// Reasons why status application might fail
#[derive(Debug, Clone)]
pub enum StatusFailureReason {
    /// Target is immune due to type
    TypeImmunity,
    /// Target is immune due to ability
    AbilityImmunity,
    /// Target is immune due to item
    ItemImmunity,
    /// Target already has this status
    AlreadyStatused,
    /// Target already has a different status
    ConflictingStatus,
    /// Chance roll failed
    ChanceFailed,
    /// Safeguard is active
    Safeguard,
    /// Misty Terrain prevents status
    MistyTerrain,
}

/// Apply a single status effect with comprehensive immunity checks
///
/// This centralized function handles:
/// - Immunity checks (type, ability, item)
/// - Existing status interactions
/// - Duration management
/// - Cure conditions
pub fn apply_status_effect(
    state: &BattleState,
    application: StatusApplication,
) -> StatusResult {
    let target = state.get_pokemon_at_position(application.target);

    // Check if target exists
    let target = match target {
        Some(target) => target,
        None => return StatusResult {
            applied: false,
            instruction: None,
            failure_reason: Some(StatusFailureReason::TypeImmunity), // No better option for "target doesn't exist"
        },
    };

    // Check chance first
    if application.chance < 100.0 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..100.0) >= application.chance {
            return StatusResult {
                applied: false,
                instruction: None,
                failure_reason: Some(StatusFailureReason::ChanceFailed),
            };
        }
    }

    // Check if target already has a status
    if target.status != PokemonStatus::None {
        if target.status == application.status {
            return StatusResult {
                applied: false,
                instruction: None,
                failure_reason: Some(StatusFailureReason::AlreadyStatused),
            };
        } else {
            return StatusResult {
                applied: false,
                instruction: None,
                failure_reason: Some(StatusFailureReason::ConflictingStatus),
            };
        }
    }

    // Check immunity based on status type
    if let Some(reason) = check_status_immunity(state, target, &application.status) {
        return StatusResult {
            applied: false,
            instruction: None,
            failure_reason: Some(reason),
        };
    }

    // Create the status instruction
    let instruction = BattleInstruction::Status(StatusInstruction::Apply {
        target: application.target,
        status: application.status,
        duration: application.duration,
        previous_status: Some(target.status),
        previous_duration: target.status_duration,
    });

    StatusResult {
        applied: true,
        instruction: Some(instruction),
        failure_reason: None,
    }
}

/// Apply multiple status effects with proper ordering
///
/// For moves with multiple possible status effects, this ensures they're
/// applied in the correct order and handles interactions properly
pub fn apply_multiple_status_effects(
    state: &BattleState,
    applications: Vec<StatusApplication>,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let mut current_state = state.clone();

    for application in applications {
        let result = apply_status_effect(&current_state, application);
        
        if let Some(instruction) = result.instruction {
            instructions.push(instruction.clone());
            // Update state for next application
            current_state.apply_instruction(&instruction);
        }
    }

    instructions
}

/// Check if a Pokemon is immune to a specific status
fn check_status_immunity(
    state: &BattleState,
    target: &crate::core::battle_state::Pokemon,
    status: &PokemonStatus,
) -> Option<StatusFailureReason> {
    // Check type immunity
    if has_type_immunity(target, status) {
        return Some(StatusFailureReason::TypeImmunity);
    }

    // Check ability immunity
    if has_ability_immunity(target, status) {
        return Some(StatusFailureReason::AbilityImmunity);
    }

    // Check item immunity
    if has_item_immunity(target, status) {
        return Some(StatusFailureReason::ItemImmunity);
    }

    // Check field effects
    if has_field_immunity(state, target, status) {
        return Some(StatusFailureReason::MistyTerrain);
    }

    None
}

/// Check if a Pokemon has type-based immunity to a status
fn has_type_immunity(target: &crate::core::battle_state::Pokemon, status: &PokemonStatus) -> bool {
    match status {
        PokemonStatus::Burn => {
            // Fire types are immune to burn
            target.types.iter().any(|t| t.to_lowercase() == "fire")
        }
        PokemonStatus::Freeze => {
            // Ice types are immune to freeze
            target.types.iter().any(|t| t.to_lowercase() == "ice")
        }
        PokemonStatus::Paralysis => {
            // Electric types are immune to paralysis (Gen 6+)
            // For now, always apply this rule
            target.types.iter().any(|t| t.to_lowercase() == "electric")
        }
        PokemonStatus::Poison | PokemonStatus::BadlyPoisoned => {
            // Poison and Steel types are immune to poison
            target.types.iter().any(|t| {
                let t_lower = t.to_lowercase();
                t_lower == "poison" || t_lower == "steel"
            })
        }
        _ => false,
    }
}

/// Check if a Pokemon has ability-based immunity to a status
fn has_ability_immunity(target: &crate::core::battle_state::Pokemon, status: &PokemonStatus) -> bool {
    let ability = target.ability.to_lowercase();
    
    match status {
        PokemonStatus::Burn => {
            matches!(ability.as_str(), "waterveil" | "waterbubble")
        }
        PokemonStatus::Freeze => {
            matches!(ability.as_str(), "magmaarmor")
        }
        PokemonStatus::Paralysis => {
            matches!(ability.as_str(), "limber")
        }
        PokemonStatus::Poison | PokemonStatus::BadlyPoisoned => {
            matches!(ability.as_str(), "immunity" | "poisonheal")
        }
        PokemonStatus::Sleep => {
            matches!(ability.as_str(), "insomnia" | "vitalspirit" | "sweetveil")
        }
        _ => false,
    }
}

/// Check if a Pokemon has item-based immunity to a status
fn has_item_immunity(target: &crate::core::battle_state::Pokemon, status: &PokemonStatus) -> bool {
    if let Some(ref item) = target.item {
        let item_lower = item.to_lowercase();
        
        match status {
            PokemonStatus::Burn => {
                item_lower == "rawstberry"
            }
            PokemonStatus::Freeze => {
                item_lower == "aspearberry"
            }
            PokemonStatus::Paralysis => {
                item_lower == "cheriberry"
            }
            PokemonStatus::Poison | PokemonStatus::BadlyPoisoned => {
                item_lower == "pechaberry"
            }
            PokemonStatus::Sleep => {
                item_lower == "chestoberry"
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Check if field effects prevent status application
fn has_field_immunity(
    state: &BattleState,
    target: &crate::core::battle_state::Pokemon,
    status: &PokemonStatus,
) -> bool {
    use crate::core::instructions::{SideCondition, Terrain};
    use crate::engine::combat::damage::is_grounded;

    // Check for Safeguard
    let target_side = if state.sides[0].pokemon.iter().any(|p| std::ptr::eq(p, target)) {
        &state.sides[0]
    } else {
        &state.sides[1]
    };

    if target_side.side_conditions.contains_key(&SideCondition::Safeguard) {
        return true;
    }

    // Check for Misty Terrain (prevents status for grounded Pokemon)
    if matches!(
        state.field.terrain.condition,
        Terrain::Misty | Terrain::MistyTerrain
    ) && is_grounded(target) {
        return true;
    }

    false
}

/// Status move implementation using the core system
///
/// This replaces complex status move implementations with a simple function call
pub fn status_move_with_stats(
    state: &BattleState,
    status_effects: Vec<StatusApplication>,
    stat_changes: Option<HashMap<Stat, i8>>,
    target_positions: &[BattlePosition],
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    // Apply status effects
    for status_app in status_effects {
        let result = apply_status_effect(state, status_app);
        if let Some(instruction) = result.instruction {
            instructions.push(instruction);
        }
    }

    // Apply stat changes if provided
    if let Some(stat_changes) = stat_changes {
        for &target_position in target_positions {
            for (stat, change) in &stat_changes {
                if *change != 0 {
                    let mut stat_changes = HashMap::new();
                    stat_changes.insert(*stat, *change);
                    let previous_boosts = HashMap::new(); // TODO: Get actual previous boosts
                    instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                        target: target_position,
                        stat_changes,
                        previous_boosts,
                    }));
                }
            }
        }
    }

    instructions
}

/// Simple status move that only applies a status
pub fn simple_status_move(
    state: &BattleState,
    status: PokemonStatus,
    target_positions: &[BattlePosition],
    chance: f32,
) -> Vec<BattleInstruction> {
    let status_effects = target_positions
        .iter()
        .map(|&position| StatusApplication {
            status: status.clone(),
            target: position,
            chance,
            duration: None,
        })
        .collect();

    status_move_with_stats(state, status_effects, None, target_positions)
}

/// Apply status with chance (for secondary effects)
pub fn apply_secondary_status(
    state: &BattleState,
    target_position: BattlePosition,
    status: PokemonStatus,
    chance: f32,
) -> Vec<BattleInstruction> {
    let application = StatusApplication {
        status,
        target: target_position,
        chance,
        duration: None,
    };

    let result = apply_status_effect(state, application);
    if let Some(instruction) = result.instruction {
        vec![instruction]
    } else {
        vec![]
    }
}

/// Apply a single volatile status effect with comprehensive checks
///
/// This centralized function handles:
/// - Immunity checks (ability, item, field effects)
/// - Existing volatile status interactions
/// - Duration management
/// - Chance-based application
pub fn apply_volatile_status_effect(
    state: &BattleState,
    application: VolatileStatusApplication,
) -> StatusResult {
    let target = state.get_pokemon_at_position(application.target);

    // Check if target exists
    let target = match target {
        Some(target) => target,
        None => return StatusResult {
            applied: false,
            instruction: None,
            failure_reason: Some(StatusFailureReason::TypeImmunity), // No better option for "target doesn't exist"
        },
    };

    // Check chance first
    if application.chance < 100.0 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..100.0) >= application.chance {
            return StatusResult {
                applied: false,
                instruction: None,
                failure_reason: Some(StatusFailureReason::ChanceFailed),
            };
        }
    }

    // Check if target already has this volatile status
    if target.volatile_statuses.contains(&application.status) {
        return StatusResult {
            applied: false,
            instruction: None,
            failure_reason: Some(StatusFailureReason::AlreadyStatused),
        };
    }

    // Check immunity based on volatile status type
    if let Some(reason) = check_volatile_status_immunity(state, target, &application.status) {
        return StatusResult {
            applied: false,
            instruction: None,
            failure_reason: Some(reason),
        };
    }

    // Get previous state for the instruction
    let previous_had_status = target.volatile_statuses.contains(&application.status);
    let previous_duration = target.volatile_status_durations.get(&application.status).copied();

    // Create the volatile status instruction
    let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: application.target,
        status: application.status,
        duration: application.duration,
        previous_had_status,
        previous_duration,
    });

    StatusResult {
        applied: true,
        instruction: Some(instruction),
        failure_reason: None,
    }
}

/// Apply multiple volatile status effects with proper ordering
pub fn apply_multiple_volatile_status_effects(
    state: &BattleState,
    applications: Vec<VolatileStatusApplication>,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let mut current_state = state.clone();

    for application in applications {
        let result = apply_volatile_status_effect(&current_state, application);
        
        if let Some(instruction) = result.instruction {
            instructions.push(instruction.clone());
            // Update state for next application
            current_state.apply_instruction(&instruction);
        }
    }

    instructions
}

/// Check if a Pokemon is immune to a specific volatile status
fn check_volatile_status_immunity(
    state: &BattleState,
    target: &crate::core::battle_state::Pokemon,
    volatile_status: &VolatileStatus,
) -> Option<StatusFailureReason> {
    // Check ability immunity
    if has_volatile_ability_immunity(target, volatile_status) {
        return Some(StatusFailureReason::AbilityImmunity);
    }

    // Check item immunity
    if has_volatile_item_immunity(target, volatile_status) {
        return Some(StatusFailureReason::ItemImmunity);
    }

    // Check field effects
    if has_volatile_field_immunity(state, target, volatile_status) {
        return Some(StatusFailureReason::Safeguard);
    }

    // Check substitute protection
    if target.volatile_statuses.contains(&VolatileStatus::Substitute) && target.substitute_health > 0 {
        // Substitute blocks most volatile statuses except for certain ones
        match volatile_status {
            // These can bypass substitute
            VolatileStatus::Attract | VolatileStatus::Torment | VolatileStatus::Disable => {},
            // Most others are blocked
            _ => return Some(StatusFailureReason::ItemImmunity), // Using ItemImmunity as a generic "blocked" reason
        }
    }

    None
}

/// Check if a Pokemon has ability-based immunity to a volatile status
fn has_volatile_ability_immunity(target: &crate::core::battle_state::Pokemon, volatile_status: &VolatileStatus) -> bool {
    let ability = target.ability.to_lowercase();
    
    match volatile_status {
        VolatileStatus::Attract => {
            matches!(ability.as_str(), "oblivious")
        }
        VolatileStatus::Taunt => {
            matches!(ability.as_str(), "oblivious" | "mentalherb")
        }
        VolatileStatus::Confusion => {
            matches!(ability.as_str(), "owntempo")
        }
        VolatileStatus::Flinch => {
            matches!(ability.as_str(), "innerfocus")
        }
        _ => false,
    }
}

/// Check if a Pokemon has item-based immunity to a volatile status
fn has_volatile_item_immunity(target: &crate::core::battle_state::Pokemon, volatile_status: &VolatileStatus) -> bool {
    if let Some(ref item) = target.item {
        let item_lower = item.to_lowercase();
        
        match volatile_status {
            VolatileStatus::Attract => {
                item_lower == "mentalherb"
            }
            VolatileStatus::Taunt => {
                item_lower == "mentalherb"
            }
            VolatileStatus::Confusion => {
                matches!(item_lower.as_str(), "persimberry" | "mentalherb")
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Check if field effects prevent volatile status application
fn has_volatile_field_immunity(
    state: &BattleState,
    target: &crate::core::battle_state::Pokemon,
    volatile_status: &VolatileStatus,
) -> bool {
    use crate::core::instructions::SideCondition;

    // Check for Safeguard (prevents most volatile statuses)
    let target_side = if state.sides[0].pokemon.iter().any(|p| std::ptr::eq(p, target)) {
        &state.sides[0]
    } else {
        &state.sides[1]
    };

    if target_side.side_conditions.contains_key(&SideCondition::Safeguard) {
        match volatile_status {
            // Safeguard blocks these
            VolatileStatus::Attract | VolatileStatus::Confusion | VolatileStatus::Taunt => return true,
            // Others pass through
            _ => {}
        }
    }

    false
}