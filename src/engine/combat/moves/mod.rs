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
pub mod status_effects;
pub use status_effects::*;

// Stat modification moves  
pub mod stat_modifying;
pub use stat_modifying::*;

// Healing moves
pub mod healing;
pub use healing::*;

// Recoil moves
pub mod recoil;
pub use recoil::*;

// Drain moves
pub mod drain;
pub use drain::*;

// Protection moves
pub mod protection;
pub use protection::*;

// Weather moves
pub mod weather;
pub use weather::*;

// Screen moves
pub mod screens;
pub use screens::*;

// Multi-hit moves
pub mod multi_hit;
pub use multi_hit::*;

// Variable power moves
pub mod variable_power;
pub use variable_power::*;

// Hazard moves
pub mod hazards;
pub use hazards::*;

// Hazard removal moves
pub mod hazard_removal;
pub use hazard_removal::*;

// Complex moves
pub mod complex;
pub use complex::*;

// Substitute moves
pub mod substitute;
pub use substitute::*;

// Simple moves
pub mod simple;
pub use simple::*;

// Utility and field effect moves
pub mod utility;
pub use utility::*;

// Priority moves
pub mod priority;
pub use priority::*;

// Fixed damage moves
pub mod fixed_damage;
pub use fixed_damage::*;

// Counter moves
pub mod counter;
pub use counter::*;

// Item interaction moves
pub mod item_interaction;
pub use item_interaction::*;

// Field manipulation moves
pub mod field_manipulation;
pub use field_manipulation::*;

// Terrain-dependent moves
pub mod terrain_dependent;
pub use terrain_dependent::*;

// Two-turn/charge moves
pub mod two_turn;
pub use two_turn::*;

// Type-changing moves
pub mod type_changing;
pub use type_changing::*;

// Type removal moves
pub mod type_removal;
pub use type_removal::*;

// Weather-dependent accuracy moves
pub mod weather_accuracy;
pub use weather_accuracy::*;

// Self-destruct moves
pub mod self_destruct;
pub use self_destruct::*;

// Self-damage moves
pub mod self_damage;
pub use self_damage::*;

// Form-dependent moves
pub mod form_dependent;
pub use form_dependent::*;

// Special combat mechanics
pub mod special_combat;
pub use special_combat::*;

// Advanced hazard manipulation
pub mod advanced_hazards;
pub use advanced_hazards::*;

// Secondary effects moves  
pub mod secondary_effects;
pub use secondary_effects::*;

// Re-export common types and structs
pub use recoil::{DamageBasedEffectType, DamageBasedEffect, create_damage_based_effect, apply_damage_based_secondary_effects};

// =============================================================================
// MAIN MOVE EFFECTS ENTRY POINT
// =============================================================================

/// Main move effect dispatcher - handles all move effects through the modular system
pub fn apply_move_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
    repository: &crate::data::Repository,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    let move_name = move_data.name.to_lowercase();
    
    // Handle moves by name - delegate to appropriate module function
    match move_name.as_str() {
        // Status effects moves
        "thunderwave" | "thunder wave" => Ok(apply_thunder_wave(state, user_position, target_positions, generation)),
        "sleeppowder" | "sleep powder" => Ok(apply_sleep_powder(state, user_position, target_positions, generation)),
        "toxic" => Ok(apply_toxic(state, user_position, target_positions, generation)),
        "willowisp" | "will-o-wisp" => Ok(apply_will_o_wisp(state, user_position, target_positions, generation)),
        "stunspore" | "stun spore" => Ok(apply_stun_spore(state, user_position, target_positions, generation)),
        "poisonpowder" | "poison powder" => Ok(apply_poison_powder(state, user_position, target_positions, generation)),
        "glare" => Ok(apply_glare(state, user_position, target_positions, generation)),
        "spore" => Ok(apply_spore(state, user_position, target_positions, generation)),
        
        // Stat modifying moves
        "swordsdance" | "swords dance" => Ok(apply_swords_dance(state, user_position, target_positions, generation)),
        "dragondance" | "dragon dance" => Ok(apply_dragon_dance(state, user_position, target_positions, generation)),
        "nastyplot" | "nasty plot" => Ok(apply_nasty_plot(state, user_position, target_positions, generation)),
        "agility" => Ok(apply_agility(state, user_position, target_positions, generation)),
        "growl" => Ok(apply_growl(state, user_position, target_positions, generation)),
        "leer" => Ok(apply_leer(state, user_position, target_positions, generation)),
        "tailwhip" | "tail whip" => Ok(apply_tail_whip(state, user_position, target_positions, generation)),
        "stringshot" | "string shot" => Ok(apply_string_shot(state, user_position, target_positions, generation)),
        "acid" => Ok(apply_acid(state, move_data, user_position, target_positions, generation)),
        "charm" => Ok(apply_charm(state, user_position, target_positions, generation)),
        "growth" => Ok(apply_growth(state, user_position, target_positions, generation)),
        "filletaway" | "fillet away" => Ok(apply_fillet_away(state, move_data, user_position, target_positions, generation)),
        "clangoroussoul" | "clangorous soul" => Ok(apply_clangorous_soul(state, move_data, user_position, target_positions, generation)),
        
        // Healing moves
        "recover" => Ok(apply_recover(state, user_position, target_positions, generation)),
        "roost" => Ok(apply_roost(state, user_position, target_positions, generation)),
        "moonlight" => Ok(apply_moonlight(state, user_position, target_positions, generation)),
        "synthesis" => Ok(apply_synthesis(state, user_position, target_positions, generation)),
        "morningsun" | "morning sun" => Ok(apply_morning_sun(state, user_position, target_positions, generation)),
        "softboiled" | "soft-boiled" => Ok(apply_soft_boiled(state, user_position, target_positions, generation)),
        "milkdrink" | "milk drink" => Ok(apply_milk_drink(state, user_position, target_positions, generation)),
        "slackoff" | "slack off" => Ok(apply_slack_off(state, user_position, target_positions, generation)),
        "aquaring" | "aqua ring" => Ok(apply_aqua_ring(state, user_position, target_positions, generation)),
        "shoreup" | "shore up" => Ok(apply_shore_up(state, user_position, target_positions, generation)),
        
        // Recoil moves
        "doubleedge" | "double-edge" => Ok(apply_double_edge(state, user_position, target_positions, generation)),
        "takedown" | "take down" => Ok(apply_take_down(state, user_position, target_positions, generation)),
        "submission" => Ok(apply_submission(state, user_position, target_positions, generation)),
        "volttackle" | "volt tackle" => Ok(apply_volt_tackle(state, user_position, target_positions, generation)),
        "flareblitz" | "flare blitz" => Ok(apply_flare_blitz(state, user_position, target_positions, generation)),
        "bravebird" | "brave bird" => Ok(apply_brave_bird(state, user_position, target_positions, generation)),
        "wildcharge" | "wild charge" => Ok(apply_wild_charge(state, user_position, target_positions, generation)),
        "headsmash" | "head smash" => Ok(apply_head_smash(state, user_position, target_positions, generation)),
        
        // Form-dependent moves
        "aurawheel" | "aura wheel" => Ok(apply_aura_wheel(state, move_data, user_position, target_positions, generation)),
        "ragingbull" | "raging bull" => Ok(apply_raging_bull(state, move_data, user_position, target_positions, generation)),
        
        // Special combat moves
        "photongeyser" | "photon geyser" => Ok(apply_photon_geyser(state, move_data, user_position, target_positions, generation)),
        "skydrop" | "sky drop" => Ok(apply_sky_drop(state, move_data, user_position, target_positions, generation)),
        
        // Advanced hazards
        "mortalspin" | "mortal spin" => Ok(apply_mortal_spin(state, move_data, user_position, target_positions, generation)),
        
        // Secondary effects moves
        "flamethrower" => Ok(apply_flamethrower(state, move_data, user_position, target_positions, generation)),
        "fireblast" | "fire blast" => Ok(apply_fire_blast(state, move_data, user_position, target_positions, generation)),
        "thunderbolt" => Ok(apply_thunderbolt(state, move_data, user_position, target_positions, generation)),
        "icebeam" | "ice beam" => Ok(apply_ice_beam(state, move_data, user_position, target_positions, generation)),
        "sludgebomb" | "sludge bomb" => Ok(apply_sludge_bomb(state, move_data, user_position, target_positions, generation)),
        "airslash" | "air slash" => Ok(apply_air_slash(state, move_data, user_position, target_positions, generation)),
        "ironhead" | "iron head" => Ok(apply_iron_head(state, move_data, user_position, target_positions, generation)),
        "rockslide" | "rock slide" => Ok(apply_rock_slide(state, move_data, user_position, target_positions, generation)),
        
        // Variable power moves (context-aware)
        "boltbeak" | "bolt beak" => Ok(apply_boltbeak(state, move_data, user_position, target_positions, generation, context)),
        "fishiousrend" | "fishious rend" => Ok(apply_fishious_rend(state, move_data, user_position, target_positions, generation, context)),
        
        // Priority moves that depend on opponent's move choice
        "suckerpunch" | "sucker punch" => Ok(variable_power::apply_sucker_punch(state, move_data, user_position, target_positions, generation, context)),
        "thunderclap" | "thunder clap" => Ok(variable_power::apply_thunder_clap(state, move_data, user_position, target_positions, generation, context)),
        "pursuit" => Ok(variable_power::apply_pursuit(state, move_data, user_position, target_positions, generation, context)),
        "upperhand" | "upper hand" => Ok(variable_power::apply_upper_hand(state, move_data, user_position, target_positions, generation, context)),
        "mefirst" | "me first" => Ok(variable_power::apply_me_first(state, move_data, user_position, target_positions, generation, context, repository)),
        
        // Counter moves (damage-based)
        "counter" => Ok(counter::apply_counter(state, user_position, target_positions, generation)),
        "mirrorcoat" | "mirror coat" => Ok(counter::apply_mirror_coat(state, user_position, target_positions, generation)),
        "comeuppance" => Ok(counter::apply_comeuppance(state, user_position, target_positions, generation)),
        "metalburst" | "metal burst" => Ok(counter::apply_metal_burst(state, user_position, target_positions, generation)),
        
        // Default case - fallback to basic damage for moves without special effects
        _ => {
            // For moves that don't have special implementations, just do basic damage
            if move_data.base_power > 0 {
                // This is a damaging move without special effects - use generic damage with configurable crit branching
                Ok(apply_generic_damage_effects(state, move_data, user_position, target_positions, generation, branch_on_damage))
            } else {
                // This is a status move or zero-power move without implementation
                Ok(apply_generic_secondary_effects(state, move_data, user_position, target_positions, generation))
            }
        }
    }
}

/// Helper function for moves that don't need context
pub fn apply_move_effects_simple(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    repository: &crate::data::Repository,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let context = MoveContext::new();
    apply_move_effects(state, move_data, user_position, target_positions, generation, &context, repository, branch_on_damage)
        .unwrap_or_else(|_| vec![BattleInstructions::new(100.0, vec![])])
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

/// Apply generic damage effects with accuracy, critical hits, and damage calculation
fn apply_generic_damage_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::damage_calc::{calculate_damage_with_positions, critical_hit_probability, DamageRolls};
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    let mut instruction_sets = Vec::new();
    
    // Generate instruction sets for hits and misses
    let accuracy = move_data.accuracy as f32;
    
    // Miss chance
    if accuracy < 100.0 {
        let miss_chance = 100.0 - accuracy;
        instruction_sets.push(BattleInstructions::new(miss_chance, vec![]));
    }
    
    if accuracy > 0.0 {
        if branch_on_damage {
            // Use advanced probability branching that combines identical outcomes
            instruction_sets.extend(generate_advanced_damage_branching(
                state, move_data, user_position, target_positions, generation, accuracy
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
            instruction_sets.push(BattleInstructions::new(accuracy, damage_instructions));
        }
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
    use crate::engine::combat::damage_calc::{calculate_damage_with_positions, critical_hit_probability, DamageRolls};
    
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
        return vec![BattleInstructions::new(accuracy, instructions)];
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
        instruction_sets.push(BattleInstructions::new(percentage, instructions));
    }
    
    // Add kill outcome if it has meaningful probability
    if kill_probability > 0.001 {
        let percentage = kill_probability * accuracy; // accuracy is already in percentage (100.0)
        let instructions = vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: target_position,
            amount: kill_damage,
            previous_hp: Some(target.hp),
        })];
        instruction_sets.push(BattleInstructions::new(percentage, instructions));
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
    use crate::engine::combat::damage_calc::critical_hit_probability;
    
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
        instruction_sets.push(BattleInstructions::new(normal_hit_chance, damage_instructions));
    }
    
    // Critical hit
    if crit_hit_chance > 0.0 {
        let damage_instructions = generate_damage_instructions(
            state, move_data, user_position, target_positions, true, generation
        );
        instruction_sets.push(BattleInstructions::new(crit_hit_chance, damage_instructions));
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
            // Use the secondary effects system from stat_modifying module
            return crate::engine::combat::moves::stat_modifying::apply_probability_based_secondary_effects(
                state, 
                move_data, 
                user_position, 
                target_positions, 
                generation, 
                secondary.chance as i16
            );
        }
    }
    
    // Return empty instructions for moves with no secondary effects
    vec![BattleInstructions::new(100.0, vec![])]
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
    use crate::engine::combat::damage_calc::{calculate_damage_with_positions, DamageRolls};
    
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