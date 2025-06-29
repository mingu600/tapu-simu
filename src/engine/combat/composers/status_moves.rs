//! Status move composers for common patterns
//!
//! This module provides composer functions for common status move patterns,
//! building on the core status system to create reusable move implementations.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, PokemonStatus, PokemonInstruction, Stat, StatsInstruction, StatusInstruction};
use crate::data::showdown_types::MoveData;
use super::super::core::status_system::{
    StatusApplication, apply_multiple_status_effects, simple_status_move,
};
use std::collections::HashMap;

/// Status move with optional stat changes
///
/// This is the most common pattern for status moves: apply a status effect
/// and potentially modify stats at the same time.
pub fn status_move_with_stats(
    state: &BattleState,
    status_effects: Vec<StatusApplication>,
    stat_changes: Option<HashMap<Stat, i8>>,
    target_positions: &[BattlePosition],
) -> Vec<BattleInstruction> {
    super::super::core::status_system::status_move_with_stats(
        state,
        status_effects,
        stat_changes,
        target_positions,
    )
}

/// Simple status move that applies one status to targets
pub fn single_status_move(
    state: &BattleState,
    status: PokemonStatus,
    target_positions: &[BattlePosition],
    chance: f32,
) -> Vec<BattleInstruction> {
    simple_status_move(state, status, target_positions, chance)
}

/// Stat-modifying move composer for moves that change Pokemon stats
///
/// Use this for moves like Swords Dance (+2 Attack), Growl (-1 Attack), 
/// Calm Mind (+1 SpAtk/SpDef), Scary Face (-2 Speed), etc.
///
/// ## Handles automatically:
/// - Stat boost/reduction application
/// - Stat stage clamping (-6 to +6)
/// - Multi-target stat changes
/// - Position tracking for affected Pokemon
/// - Failure handling (when stats are already at limits)
///
/// ## When to use:
/// - Pure stat-changing moves (no damage or status)
/// - Self-targeting stat boosts
/// - Enemy-targeting stat reductions
/// - Moves that change multiple stats simultaneously
///
/// ## When NOT to use:
/// - Moves that deal damage AND change stats (use damage composers with secondary effects)
/// - Moves that apply status conditions AND change stats (use `status_plus_stat_move`)
///
/// ## Example usage:
/// ```rust
/// // Swords Dance: +2 Attack to self
/// let mut changes = HashMap::new();
/// changes.insert(Stat::Attack, 2);
/// stat_modification_move(state, &[user_position], &changes, Some(user_position))
///
/// // Growl: -1 Attack to all enemies
/// let mut changes = HashMap::new();
/// changes.insert(Stat::Attack, -1);
/// stat_modification_move(state, target_positions, &changes, Some(user_position))
/// ```
pub fn stat_modification_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    stat_changes: &HashMap<Stat, i8>,
    source: Option<BattlePosition>,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Convert single stat changes to HashMap for BoostStats
        let mut boost_map = HashMap::new();
        for (&stat, &change) in stat_changes {
            if change != 0 {
                boost_map.insert(stat, change);
            }
        }
        
        if !boost_map.is_empty() {
            instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: target_position,
                stat_changes: boost_map,
                previous_boosts: if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
                    pokemon.stat_boosts.to_hashmap()
                } else {
                    HashMap::new()
                },
            }));
        }
    }

    instructions
}

/// Self-targeting stat boost (like Swords Dance, Calm Mind)
pub fn self_stat_boost_move(
    state: &BattleState,
    user_position: BattlePosition,
    stat_changes: &HashMap<Stat, i8>,
) -> Vec<BattleInstruction> {
    stat_modification_move(state, &[user_position], stat_changes, Some(user_position))
}

/// Enemy stat reduction move (like Growl, Leer)
pub fn enemy_stat_reduction_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    stat_changes: &HashMap<Stat, i8>,
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    stat_modification_move(state, target_positions, stat_changes, Some(source))
}

/// Healing move (like Recover, Roost)
pub fn healing_move(
    state: &BattleState,
    target_position: BattlePosition,
    heal_fraction: f32,
    source: Option<BattlePosition>,
) -> Vec<BattleInstruction> {
    // Calculate proper healing based on max HP
    let heal_amount = if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        (pokemon.max_hp as f32 * heal_fraction) as i16
    } else {
        50 // Fallback amount if Pokemon not found
    };
    
    vec![BattleInstruction::Pokemon(PokemonInstruction::Heal {
        target: target_position,
        amount: heal_amount,
        previous_hp: None, // TODO: Get actual previous HP
    })]
}

/// Status cure move (like Aromatherapy, Heal Bell)
pub fn status_cure_move(
    target_positions: &[BattlePosition],
    source: Option<BattlePosition>,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        instructions.push(BattleInstruction::Status(StatusInstruction::Remove {
            target: target_position,
            status: PokemonStatus::None, // TODO: Should specify which status to cure
            previous_duration: None, // TODO: Get actual previous duration
        }));
    }

    instructions
}

/// Sleep-inducing move (like Sleep Powder, Hypnosis)
pub fn sleep_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    accuracy: f32,
) -> Vec<BattleInstruction> {
    single_status_move(state, PokemonStatus::Sleep, target_positions, accuracy)
}

/// Paralysis-inducing move (like Thunder Wave, Stun Spore)
pub fn paralysis_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    accuracy: f32,
) -> Vec<BattleInstruction> {
    single_status_move(state, PokemonStatus::Paralysis, target_positions, accuracy)
}

/// Poison-inducing move (like Poison Powder, Toxic)
pub fn poison_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    is_badly_poisoned: bool,
    accuracy: f32,
) -> Vec<BattleInstruction> {
    let status = if is_badly_poisoned {
        PokemonStatus::BadlyPoisoned
    } else {
        PokemonStatus::Poison
    };
    
    single_status_move(state, status, target_positions, accuracy)
}

/// Burn-inducing move (like Will-O-Wisp)
pub fn burn_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    accuracy: f32,
) -> Vec<BattleInstruction> {
    single_status_move(state, PokemonStatus::Burn, target_positions, accuracy)
}

/// Freeze-inducing move (rare, but included for completeness)
pub fn freeze_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    accuracy: f32,
) -> Vec<BattleInstruction> {
    single_status_move(state, PokemonStatus::Freeze, target_positions, accuracy)
}

/// Multi-status move (like Tri Attack)
pub fn multi_status_move(
    state: &BattleState,
    target_positions: &[BattlePosition],
    possible_statuses: Vec<(PokemonStatus, f32)>, // (status, chance)
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // For multi-status moves, typically only one status is applied
        // This could be expanded to handle moves that apply multiple statuses
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let total_chance: f32 = possible_statuses.iter().map(|(_, chance)| chance).sum();
        
        if rng.gen_range(0.0..100.0) < total_chance {
            // Select which status to apply based on weighted probability
            let mut accumulated = 0.0;
            let roll = rng.gen_range(0.0..total_chance);
            
            for (status, chance) in &possible_statuses {
                accumulated += chance;
                if roll <= accumulated {
                    let status_app = StatusApplication {
                        status: status.clone(),
                        target: target_position,
                        chance: 100.0, // Already passed the chance check
                        duration: None,
                    };
                    
                    let result = super::super::core::status_system::apply_status_effect(state, status_app);
                    if let Some(instruction) = result.instruction {
                        instructions.push(instruction);
                    }
                    break;
                }
            }
        }
    }

    instructions
}

/// Combo status + stat move (like Swagger - confuse and raise attack)
pub fn status_plus_stat_move(
    state: &BattleState,
    status: PokemonStatus,
    status_chance: f32,
    stat_changes: HashMap<Stat, i8>,
    target_positions: &[BattlePosition],
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Apply status effect
        let status_app = StatusApplication {
            status: status.clone(),
            target: target_position,
            chance: status_chance,
            duration: None,
        };
        
        let result = super::super::core::status_system::apply_status_effect(state, status_app);
        if let Some(instruction) = result.instruction {
            instructions.push(instruction);
        }

        // Apply stat changes
        let mut boost_map = HashMap::new();
        for (&stat, &change) in &stat_changes {
            if change != 0 {
                boost_map.insert(stat, change);
            }
        }
        
        if !boost_map.is_empty() {
            instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: target_position,
                stat_changes: boost_map,
                previous_boosts: if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
                    pokemon.stat_boosts.to_hashmap()
                } else {
                    HashMap::new()
                },
            }));
        }
    }

    instructions
}

/// Create stat change map for common patterns
pub fn create_stat_changes(changes: &[(Stat, i8)]) -> HashMap<Stat, i8> {
    changes.iter().cloned().collect()
}

/// Common stat boost patterns
pub mod stat_patterns {
    use super::*;

    pub fn swords_dance() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::Attack, 2)])
    }

    pub fn dragon_dance() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::Attack, 1), (Stat::Speed, 1)])
    }

    pub fn calm_mind() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::SpecialAttack, 1), (Stat::SpecialDefense, 1)])
    }

    pub fn nasty_plot() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::SpecialAttack, 2)])
    }

    pub fn growl() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::Attack, -1)])
    }

    pub fn leer() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::Defense, -1)])
    }

    pub fn tail_whip() -> HashMap<Stat, i8> {
        create_stat_changes(&[(Stat::Defense, -1)])
    }
}