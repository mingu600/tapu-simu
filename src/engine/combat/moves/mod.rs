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
pub use field::hazard_removal::*;

// Damage moves
pub mod damage;
pub use damage::multi_hit::*;
pub use damage::variable_power::*;
pub use damage::recoil::*;
pub use damage::drain::*;

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
pub use damage::self_destruct::*;
pub use damage::self_damage::*;
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
pub use secondary_effects::*;

// Re-export common types and structs
pub use damage::recoil::{DamageBasedEffectType, DamageBasedEffect, create_damage_based_effect, apply_damage_based_secondary_effects};

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
    repository: &crate::data::GameDataRepository,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    use crate::utils::normalize_name;
    let move_name = normalize_name(&move_data.name);
    
    // Handle moves by name - delegate to appropriate module function
    match move_name.as_str() {
        // Status effects moves
        "thunderwave" => Ok(apply_thunder_wave(state, user_position, target_positions, generation)),
        "sleeppowder" => Ok(apply_sleep_powder(state, user_position, target_positions, generation)),
        "toxic" => Ok(apply_toxic(state, user_position, target_positions, generation)),
        "willowisp" => Ok(apply_will_o_wisp(state, user_position, target_positions, generation)),
        "stunspore" => Ok(apply_stun_spore(state, user_position, target_positions, generation)),
        "poisonpowder" => Ok(apply_poison_powder(state, user_position, target_positions, generation)),
        "glare" => Ok(apply_glare(state, user_position, target_positions, generation)),
        "spore" => Ok(apply_spore(state, user_position, target_positions, generation)),
        
        // Stat modifying moves
        "swordsdance" => Ok(apply_swords_dance(state, user_position, target_positions, generation)),
        "dragondance" => Ok(apply_dragon_dance(state, user_position, target_positions, generation)),
        "nastyplot" => Ok(apply_nasty_plot(state, user_position, target_positions, generation)),
        "agility" => Ok(apply_agility(state, user_position, target_positions, generation)),
        "growl" => Ok(apply_growl(state, user_position, target_positions, generation)),
        "leer" => Ok(apply_leer(state, user_position, target_positions, generation)),
        "tailwhip" => Ok(apply_tail_whip(state, user_position, target_positions, generation)),
        "stringshot" => Ok(apply_string_shot(state, user_position, target_positions, generation)),
        "acid" => Ok(apply_acid(state, move_data, user_position, target_positions, generation)),
        "charm" => Ok(apply_charm(state, user_position, target_positions, generation)),
        "growth" => Ok(apply_growth(state, user_position, target_positions, generation)),
        "filletaway" => Ok(apply_fillet_away(state, user_position, target_positions, generation)),
        "clangoroussoul" => Ok(apply_clangorous_soul(state, user_position, target_positions, generation)),
        
        // Healing moves
        "recover" => Ok(apply_recover(state, user_position, target_positions, generation)),
        "roost" => Ok(apply_roost(state, user_position, target_positions, generation)),
        "moonlight" => Ok(apply_moonlight(state, user_position, target_positions, generation)),
        "synthesis" => Ok(apply_synthesis(state, user_position, target_positions, generation)),
        "morningsun" => Ok(apply_morning_sun(state, user_position, target_positions, generation)),
        "softboiled" => Ok(apply_soft_boiled(state, user_position, target_positions, generation)),
        "milkdrink" => Ok(apply_milk_drink(state, user_position, target_positions, generation)),
        "slackoff" => Ok(apply_slack_off(state, user_position, target_positions, generation)),
        "aquaring" => Ok(apply_aqua_ring(state, user_position, target_positions, generation)),
        "shoreup" => Ok(apply_shore_up(state, user_position, target_positions, generation)),
        "painsplit" => Ok(status::healing::apply_pain_split(state, user_position, target_positions, generation)),
        
        // Recoil moves
        "doubleedge" => Ok(apply_double_edge(state, user_position, target_positions, generation)),
        "takedown" => Ok(apply_take_down(state, user_position, target_positions, generation)),
        "submission" => Ok(apply_submission(state, user_position, target_positions, generation)),
        "volttackle" => Ok(apply_volt_tackle(state, user_position, target_positions, generation)),
        "flareblitz" => Ok(apply_flare_blitz(state, user_position, target_positions, generation)),
        "bravebird" => Ok(apply_brave_bird(state, user_position, target_positions, generation)),
        "wildcharge" => Ok(apply_wild_charge(state, user_position, target_positions, generation)),
        "headsmash" => Ok(apply_head_smash(state, user_position, target_positions, generation)),
        
        // Variable power moves
        "facade" => Ok(damage::variable_power::apply_facade(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "hex" => Ok(damage::variable_power::apply_hex(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "gyroball" => Ok(damage::variable_power::apply_gyro_ball(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "reversal" => Ok(damage::variable_power::apply_reversal(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "acrobatics" => Ok(damage::variable_power::apply_acrobatics(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "weatherball" => Ok(damage::variable_power::apply_weather_ball(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "avalanche" => Ok(damage::variable_power::apply_avalanche(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "boltbeak" => Ok(damage::variable_power::apply_boltbeak(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "fishiousrend" => Ok(damage::variable_power::apply_fishious_rend(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "electroball" => Ok(damage::variable_power::apply_electroball(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "eruption" => Ok(damage::variable_power::apply_eruption(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "waterspout" => Ok(damage::variable_power::apply_waterspout(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "punishment" => Ok(damage::variable_power::apply_punishment(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "wakeupslap" => Ok(damage::variable_power::apply_wakeup_slap(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "dragonenergy" => Ok(damage::variable_power::apply_dragon_energy(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "grassknot" => Ok(damage::variable_power::apply_grass_knot(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "lowkick" => Ok(damage::variable_power::apply_low_kick(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "heatcrash" => Ok(damage::variable_power::apply_heat_crash(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "heavyslam" => Ok(damage::variable_power::apply_heavy_slam(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "barbbarrage" => Ok(damage::variable_power::apply_barb_barrage(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "collisioncourse" => Ok(damage::variable_power::apply_collision_course(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "electrodrift" => Ok(damage::variable_power::apply_electro_drift(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "freezedry" => Ok(damage::variable_power::apply_freeze_dry(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "hardpress" => Ok(damage::variable_power::apply_hard_press(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "hydrosteam" => Ok(damage::variable_power::apply_hydro_steam(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "lastrespects" => Ok(damage::variable_power::apply_last_respects(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "poltergeist" => Ok(damage::variable_power::apply_poltergeist(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "pursuit" => Ok(damage::variable_power::apply_pursuit(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "storedpower" => Ok(damage::variable_power::apply_stored_power(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "powertrip" => Ok(damage::variable_power::apply_power_trip(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "strengthsap" => Ok(damage::variable_power::apply_strength_sap(state, move_data, user_position, target_positions, generation)),
        "suckerpunch" => Ok(damage::variable_power::apply_sucker_punch(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "thunderclap" => Ok(damage::variable_power::apply_thunder_clap(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "terrainpulse" => Ok(damage::variable_power::apply_terrain_pulse(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "upperhand" => Ok(damage::variable_power::apply_upper_hand(state, move_data, user_position, target_positions, generation, context, branch_on_damage)),
        "mefirst" => Ok(damage::variable_power::apply_me_first(state, move_data, user_position, target_positions, generation, context, repository, branch_on_damage)),
        
        // Fixed damage moves
        "seismictoss" => Ok(damage::fixed_damage::apply_seismic_toss(state, user_position, target_positions, generation)),
        "nightshade" => Ok(damage::fixed_damage::apply_night_shade(state, user_position, target_positions, generation)),
        "endeavor" => Ok(damage::fixed_damage::apply_endeavor(state, user_position, target_positions, generation)),
        "finalgambit" => Ok(damage::fixed_damage::apply_final_gambit(state, user_position, target_positions, generation)),
        "naturesmadness" => Ok(damage::fixed_damage::apply_natures_madness(state, user_position, target_positions, generation)),
        "ruination" => Ok(damage::fixed_damage::apply_ruination(state, user_position, target_positions, generation)),
        "superfang" => Ok(damage::fixed_damage::apply_super_fang(state, user_position, target_positions, generation)),
        
        // Self-destruct moves
        "explosion" => Ok(damage::self_destruct::apply_explosion(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "selfdestruct" => Ok(damage::self_destruct::apply_self_destruct(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        
        // Self-damage moves
        "mindblown" => Ok(damage::self_damage::apply_mind_blown(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        
        // Special combat moves
        "bodypress" => Ok(apply_body_press(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "foulplay" => Ok(apply_foul_play(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "photongeyser" => Ok(apply_photon_geyser(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "skydrop" => Ok(apply_sky_drop(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        
        // Multi-hit moves
        "tailslap" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "rockblast" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "bulletseed" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "iciclespear" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "pinmissile" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "furyattack" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "furyswipes" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "bonerush" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "spikecannon" => Ok(damage::multi_hit::apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "surgingstrikes" => Ok(damage::multi_hit::apply_surging_strikes(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "dragondarts" => Ok(damage::multi_hit::apply_dragon_darts(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "populationbomb" => Ok(damage::multi_hit::apply_population_bomb(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "scaleshot" => Ok(damage::multi_hit::apply_scale_shot(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        
        // Form-dependent moves
        "aurawheel" => Ok(apply_aura_wheel(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        "ragingbull" => Ok(apply_raging_bull(state, move_data, user_position, target_positions, generation, branch_on_damage)),
        
        // Advanced hazards
        "mortalspin" => Ok(apply_mortal_spin(state, move_data, user_position, target_positions, generation)),
        
        // Secondary effects moves
        "flamethrower" => Ok(apply_flamethrower(state, move_data, user_position, target_positions, generation)),
        "fireblast" => Ok(apply_fire_blast(state, move_data, user_position, target_positions, generation)),
        "thunderbolt" => Ok(apply_thunderbolt(state, move_data, user_position, target_positions, generation)),
        "icebeam" => Ok(apply_ice_beam(state, move_data, user_position, target_positions, generation)),
        "sludgebomb" => Ok(apply_sludge_bomb(state, move_data, user_position, target_positions, generation)),
        "airslash" => Ok(apply_air_slash(state, move_data, user_position, target_positions, generation)),
        "ironhead" => Ok(apply_iron_head(state, move_data, user_position, target_positions, generation)),
        "rockslide" => Ok(apply_rock_slide(state, move_data, user_position, target_positions, generation)),
        
        // Counter moves (damage-based)
        "counter" => Ok(special::counter::apply_counter(state, user_position, target_positions, generation)),
        "mirrorcoat" => Ok(special::counter::apply_mirror_coat(state, user_position, target_positions, generation)),
        "comeuppance" => Ok(special::counter::apply_comeuppance(state, user_position, target_positions, generation)),
        "metalburst" => Ok(special::counter::apply_metal_burst(state, user_position, target_positions, generation)),
        
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
    repository: &crate::data::GameDataRepository,
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
            let status = match status_name.as_str() {
                "par" => PokemonStatus::Paralysis,
                "slp" => PokemonStatus::Sleep,
                "frz" => PokemonStatus::Freeze,
                "brn" => PokemonStatus::Burn,
                "psn" => PokemonStatus::Poison,
                "tox" => PokemonStatus::BadlyPoisoned,
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
        instructions.push(BattleInstructions::new(probability, effect_instructions));
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