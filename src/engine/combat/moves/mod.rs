//! # Move Effects Module

//! 
//! This module contains all move effect implementations, organized by category.
//! This is the modularized version of the original move_effects.rs file.

use crate::core::battle_state::BattleState;
use crate::core::battle_format::BattlePosition;
use crate::core::move_choice::MoveChoice;
use crate::core::battle_state::MoveCategory;
use crate::generation::GenerationMechanics;
use crate::core::instructions::{BattleInstructions, BattleInstruction, PokemonInstruction};
use crate::types::BattleResult;
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

/// Information about an opponent's move choice for context-aware moves
#[derive(Debug, Clone)]
pub struct OpponentMoveInfo {
    pub move_name: String,
    pub move_category: MoveCategory,
    pub is_switching: bool,
    pub priority: i8,
    pub targets: Vec<BattlePosition>,
}

/// Move context for tracking complex move state and opponent information
#[derive(Debug, Clone, Default)]
pub struct MoveContext {
    // Move execution state
    pub is_first_turn: bool,
    pub is_charging: bool,
    pub charge_turn: u8,
    pub consecutive_uses: u8,
    pub last_move_used: Option<String>,
    pub damage_dealt: i16,
    pub hit_count: u8,
    pub crit_hit: bool,
    pub missed: bool,
    pub flinched: bool,
    
    // Turn order and opponent information
    pub going_first: bool,
    pub opponent_priority: i8,
    pub target_switching: bool,
    
    // Opponent move choices for this turn (key: position, value: move info if known)
    pub opponent_moves: HashMap<BattlePosition, OpponentMoveInfo>,
    
    // All move choices for this turn in execution order
    pub turn_order: Vec<(BattlePosition, MoveChoice)>,
}

impl MoveContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if a Pokemon at the given position is using a status move
    pub fn is_opponent_using_status_move(&self, position: BattlePosition) -> bool {
        if let Some(opponent_info) = self.opponent_moves.get(&position) {
            !opponent_info.is_switching && opponent_info.move_category == MoveCategory::Status
        } else {
            false
        }
    }
    
    /// Check if a Pokemon at the given position is switching out
    pub fn is_opponent_switching(&self, position: BattlePosition) -> bool {
        if let Some(opponent_info) = self.opponent_moves.get(&position) {
            opponent_info.is_switching
        } else {
            false
        }
    }
    
    /// Get the priority of an opponent's move at the given position
    pub fn get_opponent_priority(&self, position: BattlePosition) -> Option<i8> {
        self.opponent_moves.get(&position).map(|info| info.priority)
    }
    
    /// Check if an opponent is targeting a specific position
    pub fn is_opponent_targeting(&self, opponent_position: BattlePosition, target_position: BattlePosition) -> bool {
        if let Some(opponent_info) = self.opponent_moves.get(&opponent_position) {
            opponent_info.targets.contains(&target_position)
        } else {
            false
        }
    }
    
    pub fn with_charge_turn(mut self, turn: u8) -> Self {
        self.charge_turn = turn;
        self.is_charging = turn > 0;
        self
    }
    
    pub fn with_consecutive_uses(mut self, uses: u8) -> Self {
        self.consecutive_uses = uses;
        self
    }
}

// Re-export all move effect functions from their respective modules

// Status effect moves
pub mod status;
pub use status::status_effects::*;
pub use status::stat_modifying::*;
pub use status::healing::*;

// Field effect moves
pub mod field;
pub use field::weather::*;
pub use field::screens::*;
pub use field::hazards::*;

// Damage moves
pub mod damage;
pub use damage::multi_hit::*;
pub use damage::variable_power::*;

// Special moves
pub mod special;
pub use special::protection::*;

// Import remaining modules following the directory structure
pub use special::complex::*;
pub use special::substitute::*;
pub use special::priority::*;
pub use special::counter::*;
pub use special::two_turn::*;
pub use special::type_changing::*;
pub use special::type_removal::*;
pub use special::utility::*;
pub use damage::fixed_damage::*;
pub use damage::self_targeting::*;
pub use field::field_manipulation::*;
pub use field::terrain_dependent::*;
pub use field::weather_accuracy::*;
pub use field::advanced_hazards::*;
pub use status::item_interaction::*;
pub use special::form_dependent::*;

// Import simple and special_combat directly
pub mod simple;
pub use simple::*;
pub mod special_combat;
pub use special_combat::*;
pub mod secondary_effects;

// Move registry system
pub mod registry;
pub use secondary_effects::*;

// Re-export common types and structs

// =============================================================================
// MAIN MOVE EFFECTS ENTRY POINT
// =============================================================================

/// Main move effect dispatcher - handles all move effects through the registry system
pub fn apply_move_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
    repository: &crate::data::GameDataRepository,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    // Use the global registry system for all move dispatching
    let registry = registry::get_move_registry();
    registry.apply_move_effects(
        state,
        move_data,
        user_position,
        target_positions,
        generation,
        context,
        repository,
        branch_on_damage,
    ).or_else(|_| {
        // Fallback for moves not in registry
        if move_data.base_power > 0 {
            // This is a damaging move without special effects - use generic damage with configurable crit branching
            Ok(apply_generic_damage_effects(state, move_data, user_position, target_positions, generation, branch_on_damage))
        } else {
            // This is a status move or zero-power move without implementation
            Ok(apply_generic_secondary_effects(state, move_data, user_position, target_positions, generation))
        }
    })
}

/// Helper function for moves that don't need context
/// 
/// This function provides a simplified interface for move application when no special context
/// is needed. It properly handles errors by returning them as Results rather than swallowing them.
/// Callers should handle the Result appropriately for their use case.
pub fn apply_move_effects_simple(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    repository: &crate::data::GameDataRepository,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    let context = MoveContext::new();
    apply_move_effects(state, move_data, user_position, target_positions, generation, &context, repository, branch_on_damage)
}

// =============================================================================
// SHARED GENERIC EFFECTS UTILITIES
// =============================================================================

/// Apply generic move effects - handles both damage calculation and secondary effects
/// This is the unified implementation used by all move modules
pub fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if this is a damaging move
    if move_data.base_power > 0 {
        // Use damage calculation for attacking moves
        apply_generic_damage_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    } else {
        // Use secondary effects for status moves
        apply_generic_secondary_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply generic damage effects with critical hits and damage calculation
/// Note: Accuracy is handled by the turn engine, not here
fn apply_generic_damage_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::damage::{calculate_damage_with_positions, critical_hit_probability, DamageRolls};
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    let mut instruction_sets = Vec::new();
    
    // Note: Accuracy is handled by the turn engine, so we assume this is a hit
    // and generate damage/effect instructions
    
    if branch_on_damage {
        // Use advanced probability branching that combines identical outcomes
        instruction_sets.extend(generate_advanced_damage_branching(
            state, move_data, user_position, target_positions, generation, 100.0
        ));
    } else {
        // No branching - single hit with average/expected damage
        // But check for guaranteed critical hit moves
        let crit_probability = if target_positions.is_empty() {
            0.0
        } else {
            // For multi-target moves, check if any target lacks critical hit immunity
            target_positions.iter()
                .map(|&target_pos| {
                    if let Some(target) = state.get_pokemon_at_position(target_pos) {
                        critical_hit_probability(user, target, move_data, generation.generation)
                    } else {
                        0.0
                    }
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0)
        };
        let is_guaranteed_crit = crit_probability >= 1.0;
        
        let damage_instructions = generate_damage_instructions(
            state, move_data, user_position, target_positions, is_guaranteed_crit, generation
        );
        let affected_positions = target_positions.to_vec();
        instruction_sets.push(BattleInstructions::new_with_positions(100.0, damage_instructions, affected_positions));
    }
    
    if instruction_sets.is_empty() {
        vec![BattleInstructions::new(100.0, vec![])]
    } else {
        instruction_sets
    }
}

/// Generate advanced damage branching that combines identical outcomes
fn generate_advanced_damage_branching(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    accuracy: f32,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::damage::{calculate_damage_with_positions, critical_hit_probability, DamageRolls};
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![],
    };
    
    // For simplicity, handle single target first (most common case)
    if target_positions.len() != 1 {
        // Fall back to simple critical hit branching for multi-target moves
        return generate_simple_crit_branching(state, move_data, user_position, target_positions, generation, accuracy);
    }
    
    let target_position = target_positions[0];
    let target = match state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return vec![],
    };
    
    let crit_chance = critical_hit_probability(user, target, move_data, generation.generation);
    
    // Calculate base damage for normal hits (use max damage as base for variance calculation)
    let normal_min = calculate_damage_with_positions(
        state, user, target, move_data, false, DamageRolls::Min, 1, user_position, target_position
    );
    let normal_max = calculate_damage_with_positions(
        state, user, target, move_data, false, DamageRolls::Max, 1, user_position, target_position
    );
    let crit_min = calculate_damage_with_positions(
        state, user, target, move_data, true, DamageRolls::Min, 1, user_position, target_position
    );
    let crit_max = calculate_damage_with_positions(
        state, user, target, move_data, true, DamageRolls::Max, 1, user_position, target_position
    );
    
    // Check if we actually need advanced branching (some rolls kill, others don't)
    // If minimum damage from normal hits guarantees a kill, no branching needed
    let min_possible_damage = normal_min.min(crit_min);
    if min_possible_damage >= target.hp {
        // All possible damage rolls kill - just deal remaining HP without branching
        let damage = target.hp;
        let instructions = vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target_position,
            amount: damage,
            previous_hp: Some(target.hp),
        })];
        let affected_positions = vec![target_position];
        return vec![BattleInstructions::new_with_positions(accuracy, instructions, affected_positions)];
    }
    
    // Check for mixed scenarios where some rolls kill and others don't
    let normal_range_kills = normal_max >= target.hp;
    let crit_range_kills = crit_max >= target.hp;
    let has_killing_scenario = normal_range_kills || crit_range_kills;
    let has_non_killing_scenario = normal_min < target.hp || crit_min < target.hp;
    let needs_advanced_branching = has_killing_scenario && has_non_killing_scenario;
    
    if !needs_advanced_branching {
        // Fall back to simple critical hit branching
        return generate_simple_crit_branching(state, move_data, user_position, target_positions, generation, accuracy);
    }
    
    
    // Track outcomes by kill vs non-kill, not by exact damage
    let mut non_kill_probability = 0.0;
    let mut kill_probability = 0.0;
    let mut total_non_kill_damage = 0i32; // Sum of all non-killing damage
    let mut non_kill_count = 0i32; // Count of non-killing rolls
    let kill_damage = target.hp; // Killing damage is always capped at target HP
    
    // Calculate probabilities for all possible outcomes
    // Pokemon damage variance: 85%-100% in 16 equal steps
    
    // Process normal (non-critical) hits
    let normal_hit_prob = (1.0 - crit_chance);
    let mut normal_non_kill_rolls = 0;
    let mut normal_kill_rolls = 0;
    for roll in 0..16 {
        let damage_multiplier = 0.85 + (roll as f32 * 0.01); // 85% to 100% in 1% increments
        let damage = ((normal_max as f32) * damage_multiplier) as i16;
        
        let roll_probability = 1.0 / 16.0;
        if damage >= target.hp {
            // This roll kills the target
            normal_kill_rolls += 1;
            kill_probability += normal_hit_prob * roll_probability;
        } else {
            // This roll doesn't kill the target
            normal_non_kill_rolls += 1;
            non_kill_probability += normal_hit_prob * roll_probability;
            total_non_kill_damage += damage as i32;
            non_kill_count += 1;
        }
        
    }
    
    // Process critical hits (these almost always kill)
    let crit_hit_prob = crit_chance;
    for roll in 0..16 {
        let damage_multiplier = 0.85 + (roll as f32 * 0.01); // 85% to 100% in 1% increments
        let damage = ((crit_max as f32) * damage_multiplier) as i16;
        
        let roll_probability = 1.0 / 16.0;
        if damage >= target.hp {
            // This crit kills the target (almost always)
            kill_probability += crit_hit_prob * roll_probability;
        } else {
            // This crit doesn't kill (very rare)
            non_kill_probability += crit_hit_prob * roll_probability;
            total_non_kill_damage += damage as i32;
            non_kill_count += 1;
        }
    }
    
    // Calculate average non-kill damage
    let non_kill_damage = if non_kill_count > 0 {
        (total_non_kill_damage / non_kill_count) as i16
    } else {
        0
    };
    
    
    // Convert outcomes to BattleInstructions
    let mut instruction_sets = Vec::new();
    
    // Add non-kill outcome if it has meaningful probability
    if non_kill_probability > 0.001 {
        let percentage = non_kill_probability * accuracy; // accuracy is already in percentage (100.0)
        let instructions = vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target_position,
            amount: non_kill_damage,
            previous_hp: Some(target.hp),
        })];
        instruction_sets.push(BattleInstructions::new_with_positions(percentage, instructions, vec![target_position]));
    }
    
    // Add kill outcome if it has meaningful probability
    if kill_probability > 0.001 {
        let percentage = kill_probability * accuracy; // accuracy is already in percentage (100.0)
        let instructions = vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target_position,
            amount: kill_damage,
            previous_hp: Some(target.hp),
        })];
        instruction_sets.push(BattleInstructions::new_with_positions(percentage, instructions, vec![target_position]));
    }
    
    // Sort by damage amount for consistent ordering
    instruction_sets.sort_by(|a, b| {
        if let (Some(BattleInstruction::Pokemon(PokemonInstruction::Damage { amount: a_dmg, .. })),
                Some(BattleInstruction::Pokemon(PokemonInstruction::Damage { amount: b_dmg, .. }))) = 
            (a.instruction_list.first(), b.instruction_list.first()) {
            a_dmg.cmp(b_dmg)
        } else {
            std::cmp::Ordering::Equal
        }
    });
    
    instruction_sets
}

/// Generate simple critical hit branching (fallback for complex cases)
fn generate_simple_crit_branching(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    accuracy: f32,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::damage::critical_hit_probability;
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![],
    };
    
    let mut instruction_sets = Vec::new();
    let crit_probability = if target_positions.is_empty() {
        0.0
    } else {
        // For multi-target moves, check if any target lacks critical hit immunity
        target_positions.iter()
            .map(|&target_pos| {
                if let Some(target) = state.get_pokemon_at_position(target_pos) {
                    critical_hit_probability(user, target, move_data, generation.generation)
                } else {
                    0.0
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    };
    let normal_hit_chance = accuracy * (1.0 - crit_probability);
    let crit_hit_chance = accuracy * crit_probability;
    
    // Normal hit
    if normal_hit_chance > 0.0 {
        let damage_instructions = generate_damage_instructions(
            state, move_data, user_position, target_positions, false, generation
        );
        let affected_positions = target_positions.to_vec();
        instruction_sets.push(BattleInstructions::new_with_positions(normal_hit_chance, damage_instructions, affected_positions));
    }
    
    // Critical hit
    if crit_hit_chance > 0.0 {
        let damage_instructions = generate_damage_instructions(
            state, move_data, user_position, target_positions, true, generation
        );
        let affected_positions = target_positions.to_vec();
        instruction_sets.push(BattleInstructions::new_with_positions(crit_hit_chance, damage_instructions, affected_positions));
    }
    
    instruction_sets
}

/// Apply generic secondary effects for status moves
fn apply_generic_secondary_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // For moves without specific implementations, check for secondary effects
    if let Some(secondary) = &move_data.secondary {
        if secondary.chance > 0 {
            return apply_secondary_effects(state, move_data, user_position, target_positions, generation, secondary);
        }
    }
    
    // Return empty instructions for moves with no secondary effects
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply secondary effects from move data
fn apply_secondary_effects(
    state: &BattleState,
    _move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    secondary: &crate::data::showdown_types::SecondaryEffect,
) -> Vec<BattleInstructions> {
    use crate::core::instructions::{BattleInstruction, PokemonInstruction, StatsInstruction, StatusInstruction, Stat, PokemonStatus};
    use std::collections::HashMap;

    let mut instructions = Vec::new();
    let probability = secondary.chance as f32;
    
    // Miss chance
    if probability < 100.0 {
        instructions.push(BattleInstructions::new(100.0 - probability, vec![]));
    }
    
    // Success chance with effects
    let mut effect_instructions = Vec::new();
    
    // Handle status effects
    if let Some(status_name) = &secondary.status {
        for &target in target_positions {
            let status = match status_name {
                PokemonStatus::Paralysis => PokemonStatus::Paralysis,
                PokemonStatus::Sleep => PokemonStatus::Sleep,
                PokemonStatus::Freeze => PokemonStatus::Freeze,
                PokemonStatus::Burn => PokemonStatus::Burn,
                PokemonStatus::Poison => PokemonStatus::Poison,
                PokemonStatus::BadlyPoisoned => PokemonStatus::BadlyPoisoned,
                _ => continue,
            };
            
            effect_instructions.push(BattleInstruction::Status(StatusInstruction::Apply {
                target,
                status,
                duration: Some(match status {
                    PokemonStatus::Sleep => 2,
                    PokemonStatus::Freeze => 0, // Indefinite until thawed
                    _ => 0,
                }),
                previous_status: None,
                previous_duration: None,
            }));
        }
    }
    
    // Handle stat boosts/drops
    if let Some(boosts) = &secondary.boosts {
        for &target in target_positions {
            let mut stat_changes = HashMap::new();
            
            for (stat_name, change) in boosts {
                let stat = match stat_name.as_str() {
                    "atk" => Stat::Attack,
                    "def" => Stat::Defense,
                    "spa" => Stat::SpecialAttack,
                    "spd" => Stat::SpecialDefense,
                    "spe" => Stat::Speed,
                    "accuracy" => Stat::Accuracy,
                    "evasion" => Stat::Evasion,
                    _ => continue,
                };
                stat_changes.insert(stat, *change);
            }
            
            if !stat_changes.is_empty() {
                effect_instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target,
                    stat_changes,
                    previous_boosts: HashMap::new(),
                }));
            }
        }
    }
    
    if probability > 0.0 && !effect_instructions.is_empty() {
        let affected_positions = target_positions.to_vec();
        instructions.push(BattleInstructions::new_with_positions(probability, effect_instructions, affected_positions));
    }
    
    instructions
}

/// Generate damage instructions for a move hit
fn generate_damage_instructions(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    is_critical: bool,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    use crate::engine::combat::damage::{calculate_damage_with_positions, DamageRolls};
    
    let mut instructions = Vec::new();
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    for &target_position in target_positions {
        let target = match state.get_pokemon_at_position(target_position) {
            Some(pokemon) => pokemon,
            None => continue,
        };
        
        let damage = calculate_damage_with_positions(
            state,
            user,
            target,
            move_data,
            is_critical,
            DamageRolls::Average, // Use average damage for generic moves
            target_positions.len(),
            user_position,
            target_position,
        );
        
        if damage > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage,
                previous_hp: Some(target.hp),
            }));
        }
    }
    
    instructions
}