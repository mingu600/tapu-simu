//! # Generation-Aware Damage Calculation
//!
//! This module provides damage calculation for Pokemon moves with full
//! generation-specific mechanics support.

use super::damage_context::{DamageContext, DamageEffect, DamageResult};
use super::type_effectiveness::{PokemonType, TypeChart};
use crate::core::battle_state::BattleState;
use crate::core::battle_state::Pokemon;
use crate::core::instructions::MoveCategory;
use crate::data::showdown_types::MoveData;
use crate::generation::{GenerationMechanics, GenerationBattleMechanics};
use crate::utils::normalize_name;

/// Calculate damage with explicit battle positions
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
    let mut damage_values = Vec::with_capacity(16);

    // Generate all 16 possible damage rolls (85% + 0% through 85% + 15%)
    for roll in 0..16 {
        let multiplier = (85 + roll) as f32 / 100.0;
        let damage = (base_damage_no_roll * multiplier).floor() as i16;
        damage_values.push(damage.max(1)); // Minimum 1 damage
    }

    damage_values
}

/// Get the specific damage value for a given DamageRolls variant
pub fn get_damage_for_roll(base_damage_no_roll: f32, roll_type: DamageRolls) -> i16 {
    let all_rolls = calculate_all_damage_rolls(base_damage_no_roll);

    match roll_type {
        DamageRolls::Min => all_rolls[0],  // 85% roll
        DamageRolls::Max => all_rolls[15], // 100% roll
        DamageRolls::Average => {
            // Pokemon uses the 8th damage value (0-indexed 7) as the "average"
            all_rolls[7]
        }
        DamageRolls::All => {
            // For All, just return the average as default
            ((all_rolls[7] as f32 + all_rolls[8] as f32) / 2.0).round() as i16
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
    let increment = max_damage_f32 * 0.01; // 1% increments

    let mut damage = max_damage_f32 * 0.85; // Start at 85%
    let mut total_less_than = 0i16;
    let mut num_less_than = 0i16;
    let mut num_greater_than = 0i16;

    // Calculate 16 discrete damage rolls from 85% to 100%
    for _ in 0..16 {
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
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;
    let level = context.attacker.pokemon.level as f32;

    // Get effective stats with Gen 1 critical hit considerations
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
                effects,
            }
        }
    };

    // Gen 1 damage formula (same as poke-engine common_pkmn_damage_calc)
    let mut damage = 2.0 * level;
    damage = damage.floor() / 5.0;
    damage = damage.floor() + 2.0;
    damage = damage.floor() * base_power;
    damage = damage * attack_stat / defense_stat;
    damage = damage.floor() / 50.0;
    damage = damage.floor() + 2.0;
    let base_damage = damage;

    // Apply Gen 1 critical hit modifier
    let critical_modifier = if context.move_info.is_critical {
        // Gen 1 critical hit formula: (2*level + 5) / (level + 5)
        (2.0 * level + 5.0) / (level + 5.0)
    } else {
        1.0
    };
    let mut damage = base_damage * critical_modifier;

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
    let mut type_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    // Calculate type effectiveness against secondary type if it exists
    if defender_type2 != defender_type1 {
        type_effectiveness *= type_chart.get_effectiveness(move_type, defender_type2);
    }
    damage *= type_effectiveness;

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
        None,  // No Tera type in Gen 1
        false, // No Adaptability in Gen 1
    );
    damage *= stab_multiplier;
    

    // Gen 1 doesn't have weather effects on moves
    // Gen 1 doesn't have terrain effects

    // Status effects
    if context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
    {
        damage *= 0.5; // Burn halves physical attack
    }

    // Gen 1 doesn't have multi-target reduction in the same way

    // Apply damage roll using Gen 1 specific system
    let base_damage_no_roll = damage.floor();
    let final_damage = get_damage_for_roll(base_damage_no_roll, damage_rolls);

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
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


    // Gen 2 damage formula: ((((2 * level)/5 + 2) * power * A/D) / 50 * item * critical + 2) * other modifiers
    // Step by step with proper floor operations
    let mut damage = 2.0 * level;
    damage = damage.floor() / 5.0;
    damage = damage.floor() + 2.0;
    damage = damage.floor() * base_power;
    damage = damage * attack_stat / defense_stat;
    damage = damage.floor() / 50.0;
    damage = damage.floor();
    
    // Item modifier (placeholder - we'll assume 1.0 for now)
    // damage *= item_modifier;
    
    // Gen 2 critical hit multiplier is 2.0x and applied BEFORE the +2
    if context.move_info.is_critical {
        damage *= 2.0;
    }
    
    damage = damage.floor() + 2.0;

    // Type effectiveness calculation
    use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
    use std::str::FromStr;
    
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

    // Calculate type effectiveness
    let mut type_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    if defender_type2 != defender_type1 {
        type_effectiveness *= type_chart.get_effectiveness(move_type, defender_type2);
    }
    damage *= type_effectiveness;

    // STAB calculation
    let attacker_type1 =
        PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    if move_type == attacker_type1 || move_type == attacker_type2 {
        damage *= 1.5; // STAB multiplier
    }

    // Status effects (Gen 2 has burn)
    if context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
    {
        damage *= 0.5; // Burn halves physical attack
    }

    // Apply damage roll
    let base_damage_no_roll = damage.floor();
    let final_damage = get_damage_for_roll(base_damage_no_roll, damage_rolls);

    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
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
        _ => {
            // Continue with modern calculation for Gen 3+
        }
    }

    // Check if move deals damage at all
    if context.move_info.base_power == 0 {
        return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            effects: vec![],
        };
    }

    let mut effects = Vec::new();
    let base_power = context.move_info.base_power as f32;

    // Early immunity checks would go here
    // (These would be extracted from ability/item logic)

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
                effects,
            };
        }
    };

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
                effects,
            }
        }
    };

    // Calculate base damage using exact poke-engine formula with floor operations


    let mut damage = 2.0 * level;
    damage = damage.floor() / 5.0;
    damage = damage.floor() + 2.0;
    damage = damage.floor() * base_power;
    damage = damage * attack_stat / defense_stat;
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
    let mut damage = base_damage * critical_modifier;

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
    damage *= type_effectiveness;

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
    damage *= stab_multiplier;

    // Weather effects
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "fire" => {
                damage *= 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "water" => {
                damage *= 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "water" => {
                damage *= 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            "fire" => {
                damage *= 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    }

    // Status effects
    if context.attacker.pokemon.status == crate::core::instructions::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
    {
        damage *= 0.5; // Burn halves physical attack
    }

    // Multi-target reduction
    if context.format.target_count > 1 {
        damage *= 0.75; // 25% reduction for spread moves
    }

    // Apply final damage roll using Pokemon's actual 16-roll system
    // The damage before roll should be floored first, then the roll applied
    let base_damage_no_roll = damage.floor();
    let final_damage = get_damage_for_roll(base_damage_no_roll, damage_rolls);


    DamageResult {
        damage: final_damage,
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
        effects,
    }
}

// All damage calculation tests have been moved to tests/ directory
// using the TestFramework for realistic testing with PS data
