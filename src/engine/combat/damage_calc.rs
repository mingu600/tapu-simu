//! # Generation-Aware Damage Calculation
//!
//! This module provides damage calculation for Pokemon moves with full
//! generation-specific mechanics support.

use super::damage_context::{DamageContext, DamageEffect, DamageResult};
use super::type_effectiveness::{PokemonType, TypeChart};
use crate::constants::damage::*;
use crate::core::battle_state::BattleState;
use crate::core::battle_state::Pokemon;
use crate::core::instructions::MoveCategory;
use crate::data::showdown_types::MoveData;
use crate::generation::{GenerationMechanics, GenerationBattleMechanics};
use crate::utils::normalize_name;

/// Get item damage modifier for Generation 2
fn get_gen2_item_modifier(item: &str, move_type: &str) -> f32 {
    match item.to_lowercase().replace("-", "").replace(" ", "").as_str() {
        // Type-boosting items from Gen 2
        "blackbelt" if move_type == "Fighting" => 1.1,
        "blackglasses" if move_type == "Dark" => 1.1,
        "charcoal" if move_type == "Fire" => 1.1,
        "dragonfang" if move_type == "Dragon" => 1.1,
        "hardstone" if move_type == "Rock" => 1.1,
        "magnet" if move_type == "Electric" => 1.1,
        "metalcoat" if move_type == "Steel" => 1.1,
        "miracleseed" if move_type == "Grass" => 1.1,
        "mysticwater" if move_type == "Water" => 1.1,
        "nevermeltice" if move_type == "Ice" => 1.1,
        "pinkbow" | "polkadotbow" if move_type == "Normal" => 1.1,
        "poisonbarb" if move_type == "Poison" => 1.1,
        "sharpbeak" if move_type == "Flying" => 1.1,
        "silverpowder" if move_type == "Bug" => 1.1,
        "softsand" if move_type == "Ground" => 1.1,
        "spelltag" if move_type == "Ghost" => 1.1,
        "twistedspoon" if move_type == "Psychic" => 1.1,
        
        // Light Ball for Pikachu (doubles attack)
        "lightball" => 2.0, // Applied to Pikachu's attack stat specifically
        
        // Thick Club for Cubone/Marowak (doubles attack)
        "thickclub" => 2.0, // Applied to Cubone/Marowak's attack stat specifically
        
        _ => 1.0,
    }
}

/// Calculate damage between two Pokemon with explicit battle positions.
///
/// This is the primary damage calculation function that implements Pokemon's
/// damage formula with full generation support and format awareness. The
/// calculation includes all standard damage modifiers including STAB, type
/// effectiveness, critical hits, abilities, items, and field conditions.
///
/// ## Algorithm Overview
///
/// The damage calculation follows Pokemon's standard damage formula:
/// Damage = ((((2 * Level / 5 + 2) * Power * A / D) / 50) + 2) * Modifiers
///
/// Where modifiers include:
/// - Critical hit multiplier (1.5x for Gen 6+, 2.0x for earlier generations)
/// - Same Type Attack Bonus (STAB) - typically 1.5x
/// - Type effectiveness (0x, 0.25x, 0.5x, 1x, 2x, or 4x)
/// - Random damage roll (85%-100% in 16 discrete steps)
/// - Weather conditions (e.g., rain boosting Water moves)
/// - Abilities (e.g., Adaptability changing STAB to 2x)
/// - Items (e.g., Life Orb adding 30% damage)
/// - Multi-target spread move penalty
/// - Generation-specific mechanics
///
/// ## Parameters
///
/// - `state`: The current battle state containing field conditions
/// - `attacker`: The Pokemon using the move
/// - `defender`: The Pokemon receiving the damage
/// - `move_data`: Complete move information including base power and type
/// - `is_critical`: Whether this is a critical hit
/// - `damage_rolls`: Which damage roll variant to use (min/max/average/all)
/// - `target_count`: Number of targets (affects spread move damage)
/// - `attacker_position`: Battle position of the attacking Pokemon
/// - `defender_position`: Battle position of the defending Pokemon
///
/// ## Returns
///
/// The calculated damage as an i16. Returns 0 for moves that deal no damage
/// (e.g., status moves, immune type matchups).
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: usize,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> i16 {
    // Use modern DamageContext system
    calculate_damage_with_modern_context(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_rolls,
        target_count,
        attacker_position,
        defender_position,
    )
}

/// Calculate damage using the modern DamageContext system
fn calculate_damage_with_modern_context(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: usize,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> i16 {
    use super::damage_context::{
        AbilityState, AttackerContext, DamageContext, DefenderContext, EffectiveStats,
        FieldContext, FormatContext, ItemEffects, MoveContext,
    };

    // Build modern DamageContext
    let attacker_context = AttackerContext {
        pokemon: attacker.clone(),
        position: attacker_position,
        effective_stats: EffectiveStats::from_pokemon(attacker),
        ability_state: AbilityState::from_pokemon(attacker),
        item_effects: ItemEffects::from_pokemon(attacker),
    };

    let defender_context = DefenderContext {
        pokemon: defender.clone(),
        position: defender_position,
        effective_stats: EffectiveStats::from_pokemon(defender),
        ability_state: AbilityState::from_pokemon(defender),
        item_effects: ItemEffects::from_pokemon(defender),
    };

    let move_context = MoveContext {
        name: move_data.name.clone(),
        base_power: move_data.base_power as u8,
        is_critical,
        is_contact: move_data.flags.contains_key("contact"),
        move_type: move_data.move_type.clone(),
        category: MoveCategory::from_str(&move_data.category),
        data: move_data.clone(),
    };

    let field_context = FieldContext {
        weather: state.field.weather.clone(),
        terrain: state.field.terrain.clone(),
        global_effects: state.field.global_effects.clone(),
    };

    let format_context = FormatContext {
        format: state.format.clone(),
        target_count,
    };

    let damage_context = DamageContext::new(
        attacker_context,
        defender_context,
        move_context,
        field_context,
        format_context,
    );

    // Use the modern damage calculation
    let result = calculate_damage_modern(&damage_context, damage_rolls);
    result.damage
}

/// DamageRolls enum for consistent damage calculation
/// Matches Pokemon's actual 16-roll system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageRolls {
    Average, // Uses the average of the 8th and 9th damage values (true median)
    Min,     // Uses the minimum damage roll (85%)
    Max,     // Uses the maximum damage roll (100%)
    All,     // Returns all 16 possible damage values
}

impl DamageRolls {
    /// Convert DamageRolls enum to damage multiplier (legacy)
    pub fn as_multiplier(self) -> f32 {
        match self {
            DamageRolls::Average => 0.925, // Keep for backwards compatibility
            DamageRolls::Min => 0.85,
            DamageRolls::Max => 1.0,
            DamageRolls::All => 0.925, // Default to average
        }
    }
}

/// Calculate all 16 possible damage rolls for a base damage value
/// This matches Pokemon's actual damage calculation: random number 0-15 added to 85%
pub fn calculate_all_damage_rolls(base_damage_no_roll: f32) -> Vec<i16> {
    let mut damage_values = Vec::with_capacity(DAMAGE_ROLL_COUNT);

    // Generate all 16 possible damage rolls (85% + 0% through 85% + 15%)
    for roll in 0..DAMAGE_ROLL_COUNT {
        let multiplier = (MIN_DAMAGE_PERCENT + roll as u8) as f32 / 100.0;
        let damage_float = base_damage_no_roll * multiplier;
        // Pokemon uses floor for damage rolls, not rounding
        let damage = damage_float.floor() as i16;
        damage_values.push(damage.max(MIN_DAMAGE)); // Minimum 1 damage
    }

    damage_values
}

/// Get the specific damage value for a given DamageRolls variant
pub fn get_damage_for_roll(base_damage_no_roll: f32, roll_type: DamageRolls) -> i16 {
    let all_rolls = calculate_all_damage_rolls(base_damage_no_roll);

    match roll_type {
        DamageRolls::Min => all_rolls[0],  // 85% roll
        DamageRolls::Max => all_rolls[MAX_DAMAGE_ROLL_INDEX], // 100% roll
        DamageRolls::Average => {
            // Pokemon uses the 8th damage value (0-indexed 7) as the "average"
            all_rolls[AVERAGE_DAMAGE_ROLL_INDEX]
        }
        DamageRolls::All => {
            // For All, just return the average as default
            ((all_rolls[AVERAGE_DAMAGE_ROLL_INDEX] as f32 + all_rolls[AVERAGE_DAMAGE_ROLL_INDEX + 1] as f32) / 2.0).round() as i16
        }
    }
}

/// Generate a random damage roll (deprecated - use DamageRolls enum instead)
pub fn random_damage_roll() -> f32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0.85..=1.0)
}

/// Compare health with damage multiples to determine kill/non-kill scenarios
/// This implements the poke-engine 16-roll damage calculation logic
pub fn compare_health_with_damage_multiples(max_damage: i16, health: i16) -> (i16, i16) {
    let max_damage_f32 = max_damage as f32;
    let health_f32 = health as f32;
    let increment = max_damage_f32 * DAMAGE_ROLL_INCREMENT; // 1% increments

    let mut damage = max_damage_f32 * DAMAGE_ROLL_START; // Start at 85%
    let mut total_less_than = 0i16;
    let mut num_less_than = 0i16;
    let mut num_greater_than = 0i16;

    // Calculate 16 discrete damage rolls from 85% to 100%
    for _ in 0..DAMAGE_ROLL_COUNT {
        if damage < health_f32 {
            total_less_than += damage as i16;
            num_less_than += 1;
        } else {
            num_greater_than += 1;
        }
        damage += increment;
    }

    // Return (average_non_kill_damage, num_kill_rolls)
    let average_non_kill_damage = if num_less_than > 0 {
        total_less_than / num_less_than
    } else {
        0
    };

    (average_non_kill_damage, num_greater_than)
}

/// Get base speed stat for a Pokemon species
/// This is a temporary solution until we store base stats in the Pokemon struct
fn get_base_speed_for_pokemon(species: &str) -> i32 {
    // Gen 1 base speeds (some are the same as modern, some different)
    match species.to_lowercase().as_str() {
        "dugtrio" => 120,  // Same in Gen 1 and modern
        "persian" => 115,  // Same in Gen 1 and modern
        "tauros" => 110,   // Same in Gen 1 and modern
        "pikachu" => 90,   // Same in Gen 1 and modern
        "charmander" => 65, // Same in Gen 1 and modern
        "squirtle" => 43,  // Same in Gen 1 and modern
        "bulbasaur" => 45, // Same in Gen 1 and modern
        "farfetchd" => 60, // Same in Gen 1 and modern
        "throh" => 45,     // Gen 5+ Pokemon, use modern stats
        "urshifu" => 97,   // Gen 8+ Pokemon, use modern stats
        "clefable" => 60,  // Same in Gen 1 and modern
        "lapras" => 60,    // Same in Gen 1 and modern
        _ => 80, // Default fallback
    }
}

/// Calculate Gen 1 critical hit probability based on base Speed
/// Formula: floor(base_speed / 2) / 256 for normal moves
/// High crit moves: min(8 * floor(base_speed / 2), 255) / 256
fn critical_hit_probability_gen1(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Get the base Speed stat for critical hit calculation
    // In Gen 1, we need the base stat, not the effective stat
    // For now, we need to calculate the base stat from the species name
    // TODO: Store base stats separately in Pokemon struct for proper Gen 1 support
    let base_speed = get_base_speed_for_pokemon(&attacker.species);
    
    // Normalize move name for comparison
    let move_name = normalize_name(&move_data.name);
    
    // High critical hit ratio moves in Gen 1
    let high_crit_moves = [
        "slash",
        "razorleaf", 
        "crabhammer",
        "karatechop",
    ];
    
    // Calculate critical hit rate using the correct Gen 1 formula
    let crit_rate = if high_crit_moves.contains(&move_name.as_str()) {
        // High crit moves: min(8 * floor(base_speed / 2), 256)
        (8 * (base_speed as i32 / 2)).min(256)
    } else {
        // Normal moves: floor(base_speed / 2)
        (base_speed as i32 / 2).min(256)
    };
    
    let final_rate = crit_rate as f32 / 256.0;
    
    final_rate
}

/// Calculate Gen 2 critical hit probability 
/// Formula: Uses fixed stages - base 17/256 (~6.64%), high crit moves use +1 stage (1/8 = 12.5%)
fn critical_hit_probability_gen2(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Gen 2 base critical hit rate: 17/256 ≈ 6.64%
    const GEN2_BASE_CRIT_RATE: f32 = 17.0 / 256.0;
    // Gen 2 +1 stage critical hit rate: 1/8 = 12.5%
    const GEN2_HIGH_CRIT_RATE: f32 = 1.0 / 8.0;
    
    // Normalize move name for comparison
    let move_name = normalize_name(&move_data.name);
    
    // High critical hit ratio moves in Gen 2
    let high_crit_moves = [
        "slash",
        "razorleaf", 
        "crabhammer",
        "karatechop",
        "aerialace", // Added in Gen 3 but should work in Gen 2 fallback
    ];
    
    // Gen 2 uses fixed stages, not multipliers
    if high_crit_moves.contains(&move_name.as_str()) {
        // High crit rate: +1 stage = 1/8 = 12.5%
        GEN2_HIGH_CRIT_RATE
    } else {
        // Normal crit rate: +0 stage = 17/256
        GEN2_BASE_CRIT_RATE
    }
}

/// Calculate critical hit probability with generation-specific stage system
/// Uses the official critical hit stage table for accurate calculation
pub fn critical_hit_probability(attacker: &Pokemon, defender: &Pokemon, move_data: &MoveData, generation: crate::generation::Generation) -> f32 {
    // Check for abilities that prevent critical hits (Gen 3+)
    if matches!(generation, crate::generation::Generation::Gen3 | crate::generation::Generation::Gen4 | 
                           crate::generation::Generation::Gen5 | crate::generation::Generation::Gen6 |
                           crate::generation::Generation::Gen7 | crate::generation::Generation::Gen8 |
                           crate::generation::Generation::Gen9) {
        let defender_ability = &defender.ability;
        if defender_ability == "shellarmor" || defender_ability == "battlearmor" {
            return 0.0; // No critical hit possible
        }
    }
    
    // Check for guaranteed critical hit moves first (applies to certain generations)
    let normalized_move_name = normalize_name(&move_data.name);
    let guaranteed_crit_moves = [
        "frostbreath",
        "stormthrow", 
        "wickedblow",
        "surgingstrikes",
        "flowertrick",
    ];
    if guaranteed_crit_moves.contains(&normalized_move_name.as_str()) {
        return 1.0; // Always critical hit
    }
    
    // Generation-specific critical hit calculation
    match generation {
        crate::generation::Generation::Gen1 => {
            return critical_hit_probability_gen1(attacker, move_data);
        }
        crate::generation::Generation::Gen2 => {
            return critical_hit_probability_gen2(attacker, move_data);
        }
        _ => {
            // Gen 3+ uses stage-based system
        }
    }
    
    // Calculate critical hit stage for Gen 3+
    let mut crit_stage = 0;

    // High critical hit ratio moves increase stage by 1
    let high_crit_moves = [
        "slash",
        "razorleaf",
        "crabhammer", 
        "karatechop",
        "aerialace",
        "airslash",
        "attackorder",
        "crosschop",
        "leafblade",
        "nightslash",
        "psychocut",
        "shadowclaw",
        "spacialrend",
        "stoneedge",
    ];

    if high_crit_moves.contains(&normalized_move_name.as_str()) {
        crit_stage += 1;
    }

    // Ability modifiers (Gen 3+)
    match attacker.ability.as_str() {
        "superluck" => {
            crit_stage += 1;
        }
        _ => {}
    }

    // Item modifiers
    if let Some(item) = &attacker.item {
        match item.to_lowercase().as_str() {
            "scopelens" => {
                crit_stage += 1;
            }
            "razorclaw" => {
                crit_stage += 1;
            }
            "luckypunch" => {
                if attacker.species.to_lowercase() == "chansey" {
                    crit_stage += 2;
                }
            }
            "leek" | "stick" => {
                if attacker.species.to_lowercase() == "farfetchd"
                    || attacker.species.to_lowercase() == "sirfetchd"
                {
                    crit_stage += 2;
                }
            }
            _ => {}
        }
    }

    // Convert stage to probability using generation-specific table
    calculate_crit_rate_from_stage(crit_stage, generation)
}

/// Convert critical hit stage to probability using generation-specific table
/// Based on official Pokemon critical hit probability table
fn calculate_crit_rate_from_stage(stage: i32, generation: crate::generation::Generation) -> f32 {
    match generation {
        crate::generation::Generation::Gen2 => {
            // Gen 2 uses different formula - handled separately
            match stage {
                0 => 17.0 / 256.0,  // ~6.64%
                1 => 1.0 / 8.0,     // 12.5%
                2 => 1.0 / 4.0,     // 25%
                3 => 85.0 / 256.0,  // ~33.2%
                _ => 1.0 / 2.0,     // 50% (cap)
            }
        }
        crate::generation::Generation::Gen3 | crate::generation::Generation::Gen4 | crate::generation::Generation::Gen5 => {
            // Gen 3-5
            match stage {
                0 => 1.0 / 16.0,    // 6.25%
                1 => 1.0 / 8.0,     // 12.5%
                2 => 1.0 / 4.0,     // 25%
                3 => 1.0 / 3.0,     // ~33.33%
                _ => 1.0 / 2.0,     // 50% (cap)
            }
        }
        crate::generation::Generation::Gen6 => {
            // Gen 6
            match stage {
                0 => 1.0 / 16.0,    // 6.25%
                1 => 1.0 / 8.0,     // 12.5%
                2 => 1.0 / 2.0,     // 50%
                _ => 1.0,           // 100% (always crit)
            }
        }
        crate::generation::Generation::Gen7 | crate::generation::Generation::Gen8 | crate::generation::Generation::Gen9 => {
            // Gen 7-9
            match stage {
                0 => 1.0 / 24.0,    // ~4.17%
                1 => 1.0 / 8.0,     // 12.5%
                2 => 1.0 / 2.0,     // 50%
                _ => 1.0,           // 100% (always crit)
            }
        }
        _ => {
            // Fallback - shouldn't reach here for Gen 1 or Gen 2
            1.0 / 24.0
        }
    }
}

/// Check if weather effects should be negated by any active Pokemon abilities
pub fn is_weather_negated(state: &BattleState) -> bool {
    use crate::engine::mechanics::abilities::ability_negates_weather;

    // Check all active Pokemon for weather negation abilities
    for side in [&state.sides[0], &state.sides[1]] {
        for pokemon in &side.pokemon {
            if ability_negates_weather(pokemon.ability.as_str()) {
                return true;
            }
        }
    }
    false
}

/// Calculate weather-based stat multipliers (Sandstorm SpDef for Rock, Snow Def for Ice)
pub fn get_weather_stat_multiplier(
    state: &BattleState,
    weather: &crate::core::instructions::Weather,
    pokemon: &Pokemon,
    stat: crate::core::instructions::Stat,
) -> f32 {
    use crate::core::instructions::Weather;

    // Check if weather is negated by Cloud Nine or Air Lock
    if is_weather_negated(state) {
        return 1.0;
    }

    match weather {
        Weather::Sand => {
            // Sandstorm boosts Special Defense of Rock types by 1.5x
            if stat == crate::core::instructions::Stat::SpecialDefense
                && pokemon.types.iter().any(|t| t.to_lowercase() == "rock")
            {
                1.5
            } else {
                1.0
            }
        }
        Weather::Snow => {
            // Snow boosts Defense of Ice types by 1.5x
            if stat == crate::core::instructions::Stat::Defense
                && pokemon.types.iter().any(|t| t.to_lowercase() == "ice")
            {
                1.5
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// Pokemon's rounding function: rounds down at exactly 0.5
/// This matches the damage-calc implementation: num % 1 > 0.5 ? Math.ceil(num) : Math.floor(num)
fn poke_round(num: f32) -> f32 {
    let fractional_part = num - num.floor();
    if fractional_part > 0.5 {
        num.ceil()
    } else {
        num.floor()
    }
}

/// Calculate final damage roll following the exact sequence from damage-calc getFinalDamage
/// This matches: getFinalDamage(baseAmount, i, effectiveness, isBurned, stabMod, finalMod, protect)
fn calculate_final_damage_roll(
    base_amount: f32,
    damage_rolls: DamageRolls,
    effectiveness: f32,
    is_burned: bool,
    stab_mod: u32,
    final_mod: u32,
) -> i16 {
    // Get the specific damage roll we want (0-15)
    let roll_index = match damage_rolls {
        DamageRolls::Min => 0,     // 85% roll
        DamageRolls::Max => 15,    // 100% roll 
        DamageRolls::Average => 7, // ~92% roll (index 7)
        DamageRolls::All => 7,     // Default to average
    };
    
    // Step 1: Apply damage roll (85 + i) / 100
    let mut damage_amount = (base_amount * (85.0 + roll_index as f32) / 100.0).floor();
    
    // Step 2: Apply STAB (if not 4096 to avoid unnecessary calculation)
    if stab_mod != 4096 {
        damage_amount = damage_amount * stab_mod as f32 / 4096.0;
    }
    
    // Step 3: Apply type effectiveness with pokeRound then floor
    damage_amount = (poke_round(damage_amount) * effectiveness).floor();
    
    // Step 4: Apply burn (floor division by 2)
    if is_burned {
        damage_amount = (damage_amount / 2.0).floor();
    }
    
    // Step 5: Apply final modifiers with pokeRound
    let final_damage = poke_round((damage_amount * final_mod as f32 / 4096.0).max(1.0));
    
    final_damage as i16
}

/// Calculate Gen 1/2 damage using the old 217-255 random range
fn calculate_final_damage_gen12(
    base_damage: f32,
    damage_rolls: DamageRolls,
    generation: crate::generation::Generation,
) -> i16 {
    // Gen 1/2 use range 217-255 instead of 85-100
    let roll_index = match damage_rolls {
        DamageRolls::Min => 217,     // Min roll
        DamageRolls::Max => 255,     // Max roll 
        DamageRolls::Average => 236, // Average roll ~(217+255)/2
        DamageRolls::All => 236,     // Default to average
    };
    
    if generation == crate::generation::Generation::Gen2 {
        // Gen 2: damage is always rounded up to 1
        (base_damage * roll_index as f32 / 255.0).floor().max(1.0) as i16
    } else {
        // Gen 1: random factor multiplication is skipped if damage = 1
        if base_damage == 1.0 {
            1
        } else {
            (base_damage * roll_index as f32 / 255.0).floor() as i16
        }
    }
}

/// Calculate Gen 3 damage using 85-100 range with floor at each step
fn calculate_final_damage_gen3(
    base_damage: f32,
    damage_rolls: DamageRolls,
    type_effectiveness: f32,
) -> i16 {
    // Gen 3 uses 85-100 range
    let roll_index = match damage_rolls {
        DamageRolls::Min => 85,     // 85% roll
        DamageRolls::Max => 100,    // 100% roll 
        DamageRolls::Average => 92, // Average roll
        DamageRolls::All => 92,     // Default to average
    };
    
    // Apply damage roll and type effectiveness with floor
    let damage = (base_damage * roll_index as f32 / 100.0).floor() * type_effectiveness;
    damage.floor().max(1.0) as i16
}

/// Calculate Gen 4 damage using 16 rolls with floor at each multiplier
fn calculate_final_damage_gen4(
    base_damage: f32,
    damage_rolls: DamageRolls,
    stab_mod: f32,
    type1_effectiveness: f32,
    type2_effectiveness: f32,
    filter_mod: f32,
    expert_belt_mod: f32,
    tinted_lens_mod: f32,
    berry_mod: f32,
) -> i16 {
    // Gen 4 uses 0-15 range (85%-100%)
    let roll_index = match damage_rolls {
        DamageRolls::Min => 0,     // 85% roll
        DamageRolls::Max => 15,    // 100% roll 
        DamageRolls::Average => 7, // ~92% roll (index 7)
        DamageRolls::All => 7,     // Default to average
    };
    
    // Apply damage roll
    let mut damage = (base_damage * (85.0 + roll_index as f32) / 100.0).floor();
    
    // Apply each modifier with floor at each step (Gen 4 specific)
    damage = (damage * stab_mod).floor();
    damage = (damage * type1_effectiveness).floor();
    damage = (damage * type2_effectiveness).floor();
    damage = (damage * filter_mod).floor();
    damage = (damage * expert_belt_mod).floor();
    damage = (damage * tinted_lens_mod).floor();
    damage = (damage * berry_mod).floor();
    
    damage.max(1.0) as i16
}

/// Calculate Gen 5-6 damage using getFinalDamage but without pokeRound
fn calculate_final_damage_gen56(
    base_amount: f32,
    damage_rolls: DamageRolls,
    effectiveness: f32,
    is_burned: bool,
    stab_mod: u32,
    final_mod: u32,
) -> i16 {
    // Get the specific damage roll we want (0-15)
    let roll_index = match damage_rolls {
        DamageRolls::Min => 0,     // 85% roll
        DamageRolls::Max => 15,    // 100% roll 
        DamageRolls::Average => 7, // ~92% roll (index 7)
        DamageRolls::All => 7,     // Default to average
    };
    
    // Step 1: Apply damage roll (85 + i) / 100
    let mut damage_amount = (base_amount * (85.0 + roll_index as f32) / 100.0).floor();
    
    // Step 2: Apply STAB (if not 4096 to avoid unnecessary calculation)
    if stab_mod != 4096 {
        damage_amount = damage_amount * stab_mod as f32 / 4096.0;
    }
    
    // Step 3: Apply type effectiveness with floor (no pokeRound in Gen 5-6)
    damage_amount = (damage_amount * effectiveness).floor();
    
    // Step 4: Apply burn (floor division by 2)
    if is_burned {
        damage_amount = (damage_amount / 2.0).floor();
    }
    
    // Step 5: Apply final modifiers with floor (no pokeRound in Gen 5-6)
    let final_damage = (damage_amount * final_mod as f32 / 4096.0).max(1.0).floor();
    
    final_damage as i16
}

/// Calculate weather damage modifier
pub fn get_weather_damage_modifier(
    state: &BattleState,
    weather: &crate::core::instructions::Weather,
    move_type: &str,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::core::instructions::Weather;

    // Check if weather is negated by Cloud Nine or Air Lock
    if is_weather_negated(state) {
        return 1.0;
    }

    match weather {
        Weather::Sun => match move_type.to_lowercase().as_str() {
            "fire" => 1.5,
            "water" => 0.5,
            _ => 1.0,
        },
        Weather::Rain => match move_type.to_lowercase().as_str() {
            "water" => 1.5,
            "fire" => 0.5,
            _ => 1.0,
        },
        Weather::HarshSun => {
            match move_type.to_lowercase().as_str() {
                "fire" => 1.5,
                "water" => 0.0, // Water moves fail in harsh sun
                _ => 1.0,
            }
        }
        Weather::HeavyRain => {
            match move_type.to_lowercase().as_str() {
                "water" => 1.5,
                "fire" => 0.0, // Fire moves fail in heavy rain
                _ => 1.0,
            }
        }
        Weather::Sand
        | Weather::Sandstorm
        | Weather::Hail
        | Weather::Snow
        | Weather::None
        | Weather::HarshSunlight
        | Weather::StrongWinds => 1.0,
    }
}

/// Calculate screen damage modifier (Reflect, Light Screen, Aurora Veil)
pub fn get_screen_damage_modifier(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_category: &crate::core::battle_state::MoveCategory,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::core::battle_state::MoveCategory;
    use crate::core::instructions::SideCondition;
    use crate::engine::mechanics::abilities::ability_bypasses_screens;

    // Check if attacker has Infiltrator ability to bypass screens
    if ability_bypasses_screens(attacker.ability.as_str()) {
        return 1.0;
    }

    // Determine defending side by finding which side contains the defender
    let defending_side = if state.sides[0]
        .pokemon
        .iter()
        .any(|p| std::ptr::eq(p, defender))
    {
        &state.sides[0]
    } else {
        &state.sides[1]
    };

    // Check for Aurora Veil (affects both physical and special moves)
    if defending_side
        .side_conditions
        .contains_key(&SideCondition::AuroraVeil)
    {
        // Aurora Veil: 0.5x in singles, 0.66x in doubles
        return if state.format.supports_spread_moves() {
            2.0 / 3.0 // 0.66x
        } else {
            0.5
        };
    }

    // Check for specific screens based on move category
    match move_category {
        MoveCategory::Physical => {
            if defending_side
                .side_conditions
                .contains_key(&SideCondition::Reflect)
            {
                // Reflect: 0.5x in singles, 0.66x in doubles
                if state.format.supports_spread_moves() {
                    2.0 / 3.0 // 0.66x
                } else {
                    0.5
                }
            } else {
                1.0
            }
        }
        MoveCategory::Special => {
            if defending_side
                .side_conditions
                .contains_key(&SideCondition::LightScreen)
            {
                // Light Screen: 0.5x in singles, 0.66x in doubles
                if state.format.supports_spread_moves() {
                    2.0 / 3.0 // 0.66x
                } else {
                    0.5
                }
            } else {
                1.0
            }
        }
        MoveCategory::Status => 1.0, // Status moves aren't affected by screens
    }
}

/// Calculate terrain damage modifier
pub fn get_terrain_damage_modifier(
    terrain: &crate::core::instructions::Terrain,
    move_type: &str,
    attacker: &Pokemon,
    defender: &Pokemon,
    generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::core::instructions::Terrain;

    match terrain {
        Terrain::Electric | Terrain::ElectricTerrain => {
            if move_type.to_lowercase() == "electric" && is_grounded(attacker) {
                // Electric Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                if generation_mechanics.generation.number() >= 8 {
                    1.3
                } else {
                    1.5
                }
            } else {
                1.0
            }
        }
        Terrain::Grassy | Terrain::GrassyTerrain => {
            if move_type.to_lowercase() == "grass" && is_grounded(attacker) {
                // Grassy Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                if generation_mechanics.generation.number() >= 8 {
                    1.3
                } else {
                    1.5
                }
            } else if move_type.to_lowercase() == "ground" && is_grounded(defender) {
                // Grassy Terrain reduces Earthquake and other ground moves by 0.5x
                0.5
            } else {
                1.0
            }
        }
        Terrain::Psychic | Terrain::PsychicTerrain => {
            if move_type.to_lowercase() == "psychic" && is_grounded(attacker) {
                // Psychic Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                if generation_mechanics.generation.number() >= 8 {
                    1.3
                } else {
                    1.5
                }
            } else {
                1.0
            }
        }
        Terrain::Misty | Terrain::MistyTerrain => {
            // Misty Terrain reduces Dragon moves by 0.5x when target is grounded
            if move_type.to_lowercase() == "dragon" && is_grounded(defender) {
                0.5
            } else {
                1.0
            }
        }
        Terrain::None => 1.0,
    }
}

/// Check if a Pokemon has the Adaptability ability
fn has_adaptability_ability(pokemon: &Pokemon) -> bool {
    pokemon.ability == "adaptability"
}

/// Check if a Pokemon is grounded (affected by terrain)
pub fn is_grounded(pokemon: &Pokemon) -> bool {
    // Check for Flying type
    if pokemon.types.iter().any(|t| t.to_lowercase() == "flying") {
        return false;
    }

    // Check for Levitate ability
    if pokemon.ability == "levitate" {
        return false;
    }

    // Check for items that affect grounding
    if let Some(ref item) = pokemon.item {
        match item.to_lowercase().as_str() {
            "airballoon" | "air balloon" => return false, // Air Balloon makes Pokemon ungrounded
            _ => {}
        }
    }

    // Check for volatile statuses that affect grounding
    if pokemon
        .volatile_statuses
        .contains(&crate::core::instructions::VolatileStatus::MagnetRise)
    {
        return false; // Magnet Rise makes Pokemon ungrounded
    }
    if pokemon
        .volatile_statuses
        .contains(&crate::core::instructions::VolatileStatus::Telekinesis)
    {
        return false; // Telekinesis makes Pokemon ungrounded
    }

    true
}

/// Calculate spread move damage modifier
pub fn get_spread_move_modifier(
    format: &crate::core::battle_format::BattleFormat,
    target_count: usize,
) -> f32 {
    // Spread moves only have damage reduction in multi-Pokemon formats
    // and only when actually hitting multiple targets
    if format.supports_spread_moves() && target_count > 1 {
        0.75 // 25% damage reduction for spread moves hitting multiple targets
    } else {
        1.0
    }
}

/// Get the Tera type of a Pokemon if it's Terastallized (Gen 9+ only)
fn get_tera_type(pokemon: &Pokemon) -> Option<super::type_effectiveness::PokemonType> {
    if pokemon.is_terastallized {
        pokemon.tera_type.map(|tera_type| {
            // Convert from move_choice::PokemonType to type_effectiveness::PokemonType
            use super::type_effectiveness::PokemonType;
            match tera_type {
                crate::core::move_choice::PokemonType::Normal => PokemonType::Normal,
                crate::core::move_choice::PokemonType::Fire => PokemonType::Fire,
                crate::core::move_choice::PokemonType::Water => PokemonType::Water,
                crate::core::move_choice::PokemonType::Electric => PokemonType::Electric,
                crate::core::move_choice::PokemonType::Grass => PokemonType::Grass,
                crate::core::move_choice::PokemonType::Ice => PokemonType::Ice,
                crate::core::move_choice::PokemonType::Fighting => PokemonType::Fighting,
                crate::core::move_choice::PokemonType::Poison => PokemonType::Poison,
                crate::core::move_choice::PokemonType::Ground => PokemonType::Ground,
                crate::core::move_choice::PokemonType::Flying => PokemonType::Flying,
                crate::core::move_choice::PokemonType::Psychic => PokemonType::Psychic,
                crate::core::move_choice::PokemonType::Bug => PokemonType::Bug,
                crate::core::move_choice::PokemonType::Rock => PokemonType::Rock,
                crate::core::move_choice::PokemonType::Ghost => PokemonType::Ghost,
                crate::core::move_choice::PokemonType::Dragon => PokemonType::Dragon,
                crate::core::move_choice::PokemonType::Dark => PokemonType::Dark,
                crate::core::move_choice::PokemonType::Steel => PokemonType::Steel,
                crate::core::move_choice::PokemonType::Fairy => PokemonType::Fairy,
                crate::core::move_choice::PokemonType::Unknown => PokemonType::Normal, // Fallback to Normal
            }
        })
    } else {
        None
    }
}

/// Gen 1 specific damage calculation with generation-specific mechanics
fn calculate_damage_gen1(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    
    // Gen 1 critical hits double level before damage calculation
    let level = if context.move_info.is_critical {
        (context.attacker.pokemon.level as f32) * 2.0
    } else {
        context.attacker.pokemon.level as f32
    };

    // Get effective stats with Gen 1 critical hit considerations (raw stats if critical)
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen1)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen1)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects,
            };
        }
    };

    // Gen 1 Special mechanics: uses Special Attack for both offense and defense for special moves
    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen1)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, false, crate::generation::Generation::Gen1)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects,
            }
        }
    };

    // Gen 1 damage formula following official damage-calc exactly
    // Base damage calculation: floor(floor((floor((2*Level)/5+2) * max(1,Attack) * BP) / max(1,Defense)) / 50)
    let mut base_damage = (2.0 * level / 5.0).floor() + 2.0;
    base_damage = (base_damage * attack_stat.max(1.0) * base_power / defense_stat.max(1.0)).floor() / 50.0;
    base_damage = base_damage.floor();

    // Track critical hit effect
    if context.move_info.is_critical {
        effects.push(DamageEffect::Critical);
    }

    // Apply additional modifiers before +2
    
    // Type effectiveness calculation (using Gen 1 type chart)
    let type_chart = TypeChart::new(1); // Gen 1 type chart
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    // Calculate type effectiveness against primary type  
    let type1_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    let type2_effectiveness = if defender_type2 != defender_type1 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let has_stab = move_type == attacker_type1 || move_type == attacker_type2;

    // Add +2 to base damage (capped at 997)
    base_damage = (base_damage.min(997.0) + 2.0);

    // Apply weather effects (Gen 1 has weather but limited)
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        if context.move_info.move_type.to_lowercase() == "fire" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type.to_lowercase() == "water" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        if context.move_info.move_type.to_lowercase() == "water" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type.to_lowercase() == "fire" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        }
    }

    // Apply STAB
    if has_stab {
        base_damage = (base_damage * 1.5).floor();
    }

    // Calculate combined type effectiveness
    let mut type_effectiveness = type1_effectiveness;
    if type2_effectiveness != type1_effectiveness {
        type_effectiveness *= type2_effectiveness;
    }

    // Apply type effectiveness (Gen 1 applies each type separately with floor)
    base_damage = (base_damage * type1_effectiveness).floor();
    base_damage = (base_damage * type2_effectiveness).floor();

    // Apply damage roll using Gen 1/2 specific system (217-255 range)
    let final_damage = calculate_final_damage_gen12(base_damage, damage_rolls, crate::generation::Generation::Gen1);

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Gen 2 damage calculation using focused DamageContext
/// Gen 2 uses modern formula but with 2.0x critical hit multiplier
fn calculate_damage_gen2(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    let level = context.attacker.pokemon.level as f32;

    // Get effective stats with Gen 2 critical hit considerations (2.0x multiplier)
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen2)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen2)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects: vec![],
            };
        }
    };

    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen2)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, crate::generation::Generation::Gen2)
            as f32,
        crate::core::battle_state::MoveCategory::Status => 1.0,
    };


    // Gen 2 damage formula following official damage-calc exactly
    // Base damage calculation: floor(floor((floor((2*Level)/5+2) * max(1,Attack) * BP) / max(1,Defense)) / 50)
    let mut base_damage = (2.0 * level / 5.0).floor() + 2.0;
    base_damage = (base_damage * attack_stat.max(1.0) * base_power / defense_stat.max(1.0)).floor() / 50.0;
    base_damage = base_damage.floor();

    // Apply item modifier if any (type-boosting items)
    if let Some(item) = context.attacker.pokemon.item.as_ref() {
        let item_multiplier = get_gen2_item_modifier(item, &context.move_info.move_type);
        if item_multiplier != 1.0 {
            base_damage *= item_multiplier;
            // Note: ItemBoost effect would need to be added to DamageEffect enum
        }
    }

    // Gen 2 critical hit multiplier is 2.0x and applied BEFORE the +2
    if context.move_info.is_critical {
        base_damage *= 2.0;
        effects.push(DamageEffect::Critical);
    }
    
    // Add +2 to base damage
    base_damage = base_damage + 2.0;

    // Type effectiveness calculation (using Gen 2 type chart)
    use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
    use std::str::FromStr;
    
    let type_chart = TypeChart::new(2); // Gen 2 type chart
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    // Calculate combined type effectiveness
    let mut type_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    if defender_type2 != defender_type1 {
        type_effectiveness *= type_chart.get_effectiveness(move_type, defender_type2);
    }

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let has_stab = move_type == attacker_type1 || move_type == attacker_type2;

    // Apply weather effects (Gen 2 introduced weather)
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        if context.move_info.move_type.to_lowercase() == "fire" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type.to_lowercase() == "water" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        if context.move_info.move_type.to_lowercase() == "water" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type.to_lowercase() == "fire" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        }
    }

    // Apply STAB
    if has_stab {
        base_damage = (base_damage * 1.5).floor();
    }

    // Apply type effectiveness (Gen 2 applies combined effectiveness)
    base_damage = (base_damage * type_effectiveness).floor();

    // Status effects (Gen 2 has burn)
    if context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
    {
        base_damage = (base_damage * 0.5).floor(); // Burn halves physical attack
    }

    // Apply damage roll using Gen 1/2 specific system (217-255 range)
    let final_damage = calculate_final_damage_gen12(base_damage, damage_rolls, crate::generation::Generation::Gen2);

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Gen 3 damage calculation using focused DamageContext
/// Gen 3 uses 85-100% damage rolls with separate type effectiveness applications
fn calculate_damage_gen3(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    let level = context.attacker.pokemon.level as f32;

    // Get effective stats with Gen 3 critical hit considerations
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen3)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen3)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects: vec![],
            };
        }
    };

    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen3)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, crate::generation::Generation::Gen3)
            as f32,
        crate::core::battle_state::MoveCategory::Status => 1.0,
    };

    // Gen 3 base damage formula: floor(floor((floor((2*Level)/5+2) * Attack * BP) / Defense) / 50)
    let mut base_damage = (2.0 * level / 5.0).floor() + 2.0;
    base_damage = ((base_damage * attack_stat * base_power).floor() / defense_stat).floor() / 50.0;
    base_damage = base_damage.floor();

    // Apply final modifiers (similar to modern but different order)
    
    // Critical hit (Gen 3+ uses 2x multiplier applied at end)
    let critical_modifier = if context.move_info.is_critical {
        effects.push(DamageEffect::Critical);
        2.0
    } else {
        1.0
    };

    // Weather effects
    let mut weather_multiplier = 1.0;
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "fire" => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "water" => {
                weather_multiplier = 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "water" => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "fire" => {
                weather_multiplier = 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    }

    // Apply all modifiers before +2
    base_damage = base_damage * weather_multiplier;

    // Flash Fire boost
    // TODO: Add Flash Fire detection
    
    // Add +2
    base_damage = base_damage + 2.0;

    // Apply critical hit
    base_damage = base_damage * critical_modifier;

    // Apply STAB
    let type_chart = TypeChart::new(context.format.format.generation.number());
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let stab_multiplier = type_chart.calculate_stab_multiplier(
        move_type,
        (attacker_type1, attacker_type2),
        None,  // No Tera type in Gen 3
        false, // Adaptability check would go here
    );

    base_damage = (base_damage * stab_multiplier).floor();

    // Type effectiveness (calculated and applied separately in Gen 3)
    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    let type1_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    let type2_effectiveness = if defender_type2 != defender_type1 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };

    // Apply type effectiveness separately with floor (Gen 3 specific)
    base_damage = (base_damage * type1_effectiveness).floor();
    base_damage = (base_damage * type2_effectiveness).floor();

    let total_type_effectiveness = type1_effectiveness * type2_effectiveness;

    // Apply damage roll using Gen 3 specific system (85-100 range)
    let final_damage = calculate_final_damage_gen3(base_damage, damage_rolls, 1.0); // Type effectiveness already applied

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness: total_type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Gen 4 damage calculation using focused DamageContext
/// Gen 4 uses 16 damage rolls with floor at each modifier step
fn calculate_damage_gen4(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    let level = context.attacker.pokemon.level as f32;

    // Get effective stats with Gen 4 critical hit considerations
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen4)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen4)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects: vec![],
            };
        }
    };

    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen4)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, crate::generation::Generation::Gen4)
            as f32,
        crate::core::battle_state::MoveCategory::Status => 1.0,
    };

    // Gen 4 base damage formula following official damage-calc exactly
    let mut base_damage = ((2.0 * level / 5.0).floor() + 2.0) * base_power * attack_stat / 50.0;
    base_damage = (base_damage / defense_stat).floor();

    // Apply burn status (before other modifiers)
    if context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
        && context.attacker.pokemon.ability != "guts"
    {
        base_damage = (base_damage * 0.5).floor();
    }

    // Apply final modifiers (critical hit, weather, etc.)
    
    // TODO: Add other final modifiers like weather, Flash Fire, etc.
    
    // Get type effectiveness data
    let type_chart = TypeChart::new(context.format.format.generation.number());
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    let type1_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    let type2_effectiveness = if defender_type2 != defender_type1 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let stab_mod = if move_type == attacker_type1 || move_type == attacker_type2 {
        if context.attacker.pokemon.ability == "adaptability" {
            2.0
        } else {
            1.5
        }
    } else {
        1.0
    };

    // Gen 4 specific modifiers (following damage-calc gen4.ts exactly)
    let filter_mod = if (context.defender.pokemon.ability == "filter" || context.defender.pokemon.ability == "solidrock") 
        && (type1_effectiveness * type2_effectiveness) > 1.0 {
        0.75
    } else {
        1.0
    };

    let expert_belt_mod = if let Some(ref item) = context.attacker.pokemon.item {
        if item.to_lowercase() == "expertbelt" && (type1_effectiveness * type2_effectiveness) > 1.0 {
            1.2
        } else {
            1.0
        }
    } else {
        1.0
    };

    let tinted_lens_mod = if context.attacker.pokemon.ability == "tintedlens" 
        && (type1_effectiveness * type2_effectiveness) < 1.0 {
        2.0
    } else {
        1.0
    };

    // Berry resistance (simplified)
    let berry_mod = 1.0; // TODO: Implement berry resistance

    let total_type_effectiveness = type1_effectiveness * type2_effectiveness;

    // Apply damage roll using Gen 4 specific system (16 rolls with floor at each step)
    let final_damage = calculate_final_damage_gen4(
        base_damage,
        damage_rolls,
        stab_mod,
        type1_effectiveness,
        type2_effectiveness,
        filter_mod,
        expert_belt_mod,
        tinted_lens_mod,
        berry_mod,
    );

    if context.move_info.is_critical {
        effects.push(DamageEffect::Critical);
    }

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness: total_type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Gen 5-6 damage calculation using focused DamageContext
/// Gen 5-6 uses getFinalDamage but without pokeRound
fn calculate_damage_gen56(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    let level = context.attacker.pokemon.level as f32;

    // Get effective stats with critical hit considerations
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects: vec![],
            };
        }
    };

    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Status => 1.0,
    };

    // Gen 5-6 base damage formula following getBaseDamage exactly
    let mut base_damage = (2.0 * level / 5.0).floor() + 2.0;
    base_damage = (base_damage * base_power * attack_stat / defense_stat).floor() / 50.0;
    base_damage = base_damage.floor();

    // Apply burn status
    let is_burned = context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
        && context.attacker.pokemon.ability != "guts";

    // Apply final modifiers before getFinalDamage
    // TODO: Add weather, Flash Fire, etc.

    // Add +2 and apply critical hit
    base_damage = base_damage + 2.0;
    if context.move_info.is_critical {
        base_damage *= if context.format.format.generation.number() >= 6 { 1.5 } else { 2.0 };
        effects.push(DamageEffect::Critical);
    }

    // Apply Life Orb and other final modifiers
    // TODO: Add Life Orb check

    // Get type effectiveness data
    let type_chart = TypeChart::new(context.format.format.generation.number());
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    let type1_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    let type2_effectiveness = if defender_type2 != defender_type1 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };
    let total_type_effectiveness = type1_effectiveness * type2_effectiveness;

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let stab_multiplier = type_chart.calculate_stab_multiplier(
        move_type,
        (attacker_type1, attacker_type2),
        None,  // No Tera type in Gen 5-6
        context.attacker.pokemon.ability == "adaptability",
    );

    // Convert to 4096-based multiplier for getFinalDamage
    let stab_mod_4096 = (stab_multiplier * 4096.0) as u32;

    // TODO: Calculate final_mod (1.3x for Life Orb, etc.)
    let final_mod = 4096; // No modifiers for now

    // Apply damage roll using Gen 5-6 specific system (getFinalDamage without pokeRound)
    let final_damage = calculate_final_damage_gen56(
        base_damage,
        damage_rolls,
        total_type_effectiveness,
        is_burned,
        stab_mod_4096,
        final_mod,
    );

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness: total_type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Gen 7-9 damage calculation using modern getFinalDamage with pokeRound
/// This is the current working implementation
fn calculate_damage_modern_gen789(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;

    // Early immunity checks 
    
    // Check for Levitate immunity to Ground-type moves
    if context.move_info.move_type.to_lowercase() == "ground" && 
       context.defender.pokemon.ability.to_lowercase() == "levitate" {
        return DamageResult {
            damage: 0,
            blocked: true,
            was_critical: false,
            type_effectiveness: 0.0,
            hit_substitute: false,
            effects: vec![],
        };
    }
    
    // Check for Flash Fire immunity to Fire-type moves (absorbs and boosts)
    if context.move_info.move_type.to_lowercase() == "fire" && 
       context.defender.pokemon.ability.to_lowercase() == "flashfire" {
        return DamageResult {
            damage: 0,
            blocked: true,
            was_critical: false,
            type_effectiveness: 0.0,
            hit_substitute: false,
            effects: vec![],
        };
    }
    
    // Check for Water Absorb immunity to Water-type moves
    if context.move_info.move_type.to_lowercase() == "water" && 
       context.defender.pokemon.ability.to_lowercase() == "waterabsorb" {
        return DamageResult {
            damage: 0,
            blocked: true,
            was_critical: false,
            type_effectiveness: 0.0,
            hit_substitute: false,
            effects: vec![],
        };
    }
    
    // Check for Volt Absorb immunity to Electric-type moves  
    if context.move_info.move_type.to_lowercase() == "electric" && 
       context.defender.pokemon.ability.to_lowercase() == "voltabsorb" {
        return DamageResult {
            damage: 0,
            blocked: true,
            was_critical: false,
            type_effectiveness: 0.0,
            hit_substitute: false,
            effects: vec![],
        };
    }

    // Exact poke-engine damage formula with floor operations at each step
    let level = context.attacker.pokemon.level as f32;

    // Get effective attack stat (with critical hit consideration)
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects,
            };
        }
    };

    // Apply ability modifiers to attack stat
    let mut modified_attack_stat = attack_stat;
    
    // Apply Guts ability: 1.5x attack when statused
    if context.attacker.pokemon.ability.to_lowercase() == "guts" 
        && context.attacker.pokemon.status != crate::core::instructions::PokemonStatus::None {
        modified_attack_stat *= 1.5;
    }

    // Get effective defense stat (with critical hit consideration)
    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, context.format.format.generation)
            as f32,
        crate::core::battle_state::MoveCategory::Status => {
            return DamageResult {
                damage: 0,
                blocked: false,
                was_critical: false,
                type_effectiveness: 1.0,
                hit_substitute: false,
                effects,
            }
        }
    };

    // Calculate base damage using exact poke-engine formula with floor operations

    let mut damage = 2.0 * level;
    damage = damage.floor() / 5.0;
    damage = damage.floor() + 2.0;
    damage = damage.floor() * base_power;
    damage = damage * modified_attack_stat / defense_stat;
    damage = damage.floor() / 50.0;
    damage = damage.floor() + 2.0;
    let base_damage = damage;

    // Apply critical hit modifier (Gen 2+ only, Gen 1 handled separately)
    let critical_modifier = if context.move_info.is_critical {
        // Use generation-specific critical hit multiplier for Gen 2+
        let generation_mechanics = crate::generation::GenerationMechanics::new(context.format.format.generation);
        generation_mechanics.get_critical_multiplier()
    } else {
        1.0
    };
    let damage = base_damage * critical_modifier;

    // Type effectiveness calculation
    let type_chart = TypeChart::new(context.format.format.generation.number());
    let move_type =
        PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);

    let defender_type1 =
        PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    // Calculate type effectiveness against primary type
    let mut type_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    // Calculate type effectiveness against secondary type if it exists
    if defender_type2 != defender_type1 {
        type_effectiveness *= type_chart.get_effectiveness(move_type, defender_type2);
    }
    
    // Handle Mind's Eye ability: allows Normal and Fighting moves to hit Ghost types
    if context.attacker.pokemon.ability.as_str() == "Mind's Eye" || context.attacker.pokemon.ability.as_str() == "mindseye" {
        if (move_type == PokemonType::Normal || move_type == PokemonType::Fighting) {
            // Check if any of the defender's types is Ghost
            if defender_type1 == PokemonType::Ghost || defender_type2 == PokemonType::Ghost {
                // If the move would normally be ineffective due to Ghost immunity, make it neutral (1.0x)
                if type_effectiveness == 0.0 {
                    type_effectiveness = 1.0;
                }
            }
        }
    }

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let stab_multiplier = type_chart.calculate_stab_multiplier(
        move_type,
        (attacker_type1, attacker_type2),
        None,  // Tera type support would go here
        false, // Adaptability check would go here
    );

    // Weather effects
    let mut weather_multiplier = 1.0;
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "fire" => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "water" => {
                weather_multiplier = 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "water" => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "fire" => {
                weather_multiplier = 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    }

    // Terrain effects (Gen 7+)
    let generation_mechanics = crate::generation::GenerationMechanics::new(context.format.format.generation);
    let terrain_multiplier = get_terrain_damage_modifier(
        &context.field.terrain.condition,
        &context.move_info.move_type,
        &context.attacker.pokemon,
        &context.defender.pokemon,
        &generation_mechanics,
    );

    // Burn status effect (Guts ability prevents burn's attack reduction)
    let is_burned = context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
        && context.attacker.pokemon.ability.to_lowercase() != "guts";

    // Multi-target reduction
    let spread_multiplier = if context.format.target_count > 1 {
        0.75 // 25% reduction for spread moves
    } else {
        1.0
    };

    // Final damage multiplier (combining all remaining modifiers except damage roll)
    let final_multiplier = spread_multiplier * weather_multiplier * terrain_multiplier;

    // Apply final damage roll using Pokemon's actual damage calculation sequence
    // This follows the exact sequence from damage-calc getFinalDamage function
    // Convert STAB multiplier to 4096-based value for damage calc compatibility
    let stab_mod_4096 = (stab_multiplier * 4096.0) as u32;
    let final_mod_4096 = (final_multiplier * 4096.0) as u32;
    
    let final_damage = calculate_final_damage_roll(
        damage,
        damage_rolls,
        type_effectiveness,
        is_burned,
        stab_mod_4096,
        final_mod_4096,
    );

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
        hit_substitute: false,
        effects,
    }
}

/// Modern damage calculation using focused DamageContext
/// This replaces the legacy calculate_damage function that requires the entire State
pub fn calculate_damage_modern(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Use generation-specific damage calculation
    match context.format.format.generation {
        crate::generation::Generation::Gen1 => {
            return calculate_damage_gen1(context, damage_rolls);
        }
        crate::generation::Generation::Gen2 => {
            return calculate_damage_gen2(context, damage_rolls);
        }
        crate::generation::Generation::Gen3 => {
            return calculate_damage_gen3(context, damage_rolls);
        }
        crate::generation::Generation::Gen4 => {
            return calculate_damage_gen4(context, damage_rolls);
        }
        crate::generation::Generation::Gen5 | crate::generation::Generation::Gen6 => {
            return calculate_damage_gen56(context, damage_rolls);
        }
        _ => {
            // Gen 7-9 calculation (modern getFinalDamage with pokeRound)
            return calculate_damage_modern_gen789(context, damage_rolls);
        }
    }
}

// All damage calculation tests have been moved to tests/ directory
// using the TestFramework for realistic testing with PS data
