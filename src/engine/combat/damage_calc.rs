//! # Generation-Aware Damage Calculation
//!
//! This module provides damage calculation for Pokemon moves with full
//! generation-specific mechanics support.

use crate::engine::mechanics::abilities::calculate_ability_modifiers;
use crate::engine::mechanics::items::{apply_expert_belt_boost, calculate_item_modifiers, get_item_by_name};
use super::type_effectiveness::{PokemonType, TypeChart};
use super::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::data::types::EngineMoveData;
use crate::generation::{GenerationBattleMechanics, GenerationMechanics};
use crate::core::battle_state::Pokemon;
use crate::core::battle_state::BattleState;

/// Calculate damage with generation-aware mechanics
pub fn calculate_damage(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
) -> i16 {
    // Use placeholder positions for legacy compatibility
    use crate::core::battle_format::{BattlePosition, SideReference};
    let placeholder_attacker_pos = BattlePosition { side: SideReference::SideOne, slot: 0 };
    let placeholder_defender_pos = BattlePosition { side: SideReference::SideTwo, slot: 0 };
    
    calculate_damage_with_positions(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_roll,
        1,
        placeholder_attacker_pos,
        placeholder_defender_pos,
    )
}

/// Calculate damage with generation-aware mechanics and target count for spread moves
pub fn calculate_damage_with_targets(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
) -> i16 {
    // Use placeholder positions for legacy compatibility
    use crate::core::battle_format::{BattlePosition, SideReference};
    let placeholder_attacker_pos = BattlePosition { side: SideReference::SideOne, slot: 0 };
    let placeholder_defender_pos = BattlePosition { side: SideReference::SideTwo, slot: 0 };
    
    calculate_damage_with_positions(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_roll,
        target_count,
        placeholder_attacker_pos,
        placeholder_defender_pos,
    )
}

/// Calculate damage with explicit battle positions 
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> i16 {
    // Try to use modern DamageContext when possible
    if let Ok(modern_result) = calculate_damage_with_modern_context(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_roll,
        target_count,
        attacker_position,
        defender_position,
    ) {
        return modern_result;
    }

    // Fallback to legacy system
    let generation_mechanics = state.get_generation_mechanics();
    let type_chart = TypeChart::new(state.get_generation().number());

    calculate_damage_with_generation(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_roll,
        target_count,
        &type_chart,
        &generation_mechanics,
        attacker_position,
        defender_position,
    )
}

/// Attempt to calculate damage using the modern DamageContext system
fn calculate_damage_with_modern_context(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> Result<i16, &'static str> {
    use super::damage_context::{DamageContext, AttackerContext, DefenderContext, MoveContext, FieldContext, FormatContext, EffectiveStats, AbilityState, ItemEffects};

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
        base_power: move_data.base_power.unwrap_or(0) as u8,
        is_critical,
        is_contact: move_data.flags.contains(&"contact".to_string()),
        move_type: move_data.move_type.clone(),
        category: move_data.category,
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
    let result = calculate_damage_modern(&damage_context, damage_roll);
    Ok(result.damage)
}

/// Calculate damage with full generation-specific mechanics
pub fn calculate_damage_with_generation(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
    type_chart: &TypeChart,
    generation_mechanics: &GenerationMechanics,
    attacker_position: crate::core::battle_format::BattlePosition,
    defender_position: crate::core::battle_format::BattlePosition,
) -> i16 {
    // Basic damage calculation formula
    let level = attacker.level as f32;
    let mut base_power = move_data.base_power.unwrap_or(0) as f32;

    if base_power == 0.0 {
        return 0; // Status moves don't deal damage
    }

    // Create modern damage context for ability calculations
    let damage_context = DamageContext::from_battle_state(
        attacker,
        attacker_position,
        defender,
        defender_position,
        move_data,
        &state.field,
        &state.format,
        1, // target_count
        is_critical,
    );

    // Apply base power modifications FIRST, like poke-engine does
    base_power = apply_base_power_modifications(base_power, &damage_context, generation_mechanics);

    if base_power == 0.0 {
        return 0; // Move blocked or nullified
    }

    // Use modern ability modifier calculation
    let ability_modifier = crate::engine::mechanics::abilities::calculate_ability_modifiers(&damage_context, state, generation_mechanics);
    
    // Calculate type effectiveness early for item modifiers that need it
    let effective_move_type = ability_modifier.changed_move_type
        .as_ref()
        .unwrap_or(&move_data.move_type);
    let move_type = PokemonType::from_str(effective_move_type).unwrap_or(PokemonType::Normal);
    
    let defender_type1 = PokemonType::from_str(&defender.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    // Type effectiveness (with generation-specific overrides and Tera type support)
    let mut type_effectiveness = type_chart.calculate_damage_multiplier(
        move_type,
        (defender_type1, defender_type2),
        get_tera_type(defender), // Tera type support
        Some(&move_data.name.to_lowercase()),
    );

    // Apply generation-specific type effectiveness overrides
    if let Some(override_mult) = generation_mechanics
        .get_type_effectiveness_override(&move_data.move_type, &defender.types[0])
    {
        type_effectiveness = override_mult;
    }
    
    let item_modifier = calculate_item_modifiers(&damage_context, type_effectiveness, generation_mechanics);

    // Check for complete immunity early
    if ability_modifier.blocks_move || item_modifier.blocks_move {
        return 0;
    }

    let attack_stat = match move_data.category {
        crate::core::battle_state::MoveCategory::Physical => {
            attacker.get_effective_stat(crate::core::instruction::Stat::Attack) as f32
                * ability_modifier.attack_multiplier
                * item_modifier.attack_multiplier
        }
        crate::core::battle_state::MoveCategory::Special => {
            attacker.get_effective_stat(crate::core::instruction::Stat::SpecialAttack) as f32
                * ability_modifier.special_attack_multiplier
                * item_modifier.special_attack_multiplier
        }
        crate::core::battle_state::MoveCategory::Status => return 0,
    };

    let base_defense_stat = match move_data.category {
        crate::core::battle_state::MoveCategory::Physical => {
            defender.get_effective_stat(crate::core::instruction::Stat::Defense)
        }
        crate::core::battle_state::MoveCategory::Special => {
            defender.get_effective_stat(crate::core::instruction::Stat::SpecialDefense)
        }
        crate::core::battle_state::MoveCategory::Status => return 0,
    } as f32;

    // Apply weather-based stat boosts
    let weather_defense_multiplier = get_weather_stat_multiplier(
        &state,
        &state.weather,
        defender,
        match move_data.category {
            crate::core::battle_state::MoveCategory::Physical => crate::core::instruction::Stat::Defense,
            crate::core::battle_state::MoveCategory::Special => crate::core::instruction::Stat::SpecialDefense,
            crate::core::battle_state::MoveCategory::Status => return 0,
        },
    );

    let defense_stat = base_defense_stat
        * weather_defense_multiplier
        * match move_data.category {
            crate::core::battle_state::MoveCategory::Physical => ability_modifier.defense_multiplier,
            crate::core::battle_state::MoveCategory::Special => ability_modifier.special_defense_multiplier,
            crate::core::battle_state::MoveCategory::Status => 1.0,
        }
        * match move_data.category {
            crate::core::battle_state::MoveCategory::Physical => item_modifier.defense_multiplier,
            crate::core::battle_state::MoveCategory::Special => item_modifier.special_defense_multiplier,
            crate::core::battle_state::MoveCategory::Status => 1.0,
        };

    // Apply ability power multipliers (items already applied to base_power)
    let power = base_power * ability_modifier.power_multiplier;

    // Base damage calculation with proper flooring like poke-engine
    let mut damage = 2.0 * level;
    damage = damage.floor() / 5.0;
    damage = damage.floor() + 2.0;
    damage = damage.floor() * power;
    damage = damage * attack_stat / defense_stat;
    damage = damage.floor() / 50.0;
    let base_damage = damage.floor() + 2.0;

    // Apply modifiers
    let mut damage = base_damage;

    // Critical hit multiplier (generation-specific)
    if is_critical {
        damage *= generation_mechanics.get_critical_multiplier();
    }

    // Type effectiveness already calculated earlier for item modifiers
    damage *= type_effectiveness;

    // Apply Expert Belt boost for super effective moves
    damage *= apply_expert_belt_boost(&damage_context, type_effectiveness, generation_mechanics.generation.number());

    // STAB (Same Type Attack Bonus)
    let attacker_type1 = PokemonType::from_str(&attacker.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if attacker.types.len() > 1 {
        PokemonType::from_str(&attacker.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };

    let stab_multiplier = type_chart.calculate_stab_multiplier(
        move_type,
        (attacker_type1, attacker_type2),
        get_tera_type(attacker), // Tera type support
        has_adaptability_ability(attacker),
    );
    
    damage *= stab_multiplier;

    // Apply burn modifier (generation-specific)
    if attacker.status == crate::core::instruction::PokemonStatus::Burn
        && move_data.category == crate::core::battle_state::MoveCategory::Physical
    {
        damage *= generation_mechanics.get_burn_reduction();
    }

    // Apply weather modifier
    damage *= get_weather_damage_modifier(
        &state,
        &state.weather,
        &move_data.move_type,
        &generation_mechanics,
    );

    // Apply screen effects (Reflect, Light Screen, Aurora Veil)
    damage *= get_screen_damage_modifier(
        &state,
        attacker,
        defender,
        &move_data.category,
        &generation_mechanics,
    );

    // Apply terrain effects
    damage *= get_terrain_damage_modifier(
        &state.terrain,
        &move_data.move_type,
        attacker,
        defender,
        &generation_mechanics,
    );

    // Apply spread move damage reduction
    damage *= get_spread_move_modifier(&state.format, target_count);

    // Apply ability damage modifiers (already calculated above)
    damage *= ability_modifier.damage_multiplier;

    // Apply damage roll with floor() before multiplication like poke-engine
    damage = damage.floor() * damage_roll;

    damage as i16
}

/// Generate a random damage roll
pub fn random_damage_roll() -> f32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0.85..=1.0)
}

/// Calculate critical hit probability with move, ability, and item modifiers
pub fn critical_hit_probability(attacker: &Pokemon, move_data: &EngineMoveData) -> f32 {
    // Base critical hit rate is 1/24 (about 4.17%)
    let mut crit_stage = 0;

    // High critical hit ratio moves
    let high_crit_moves = [
        "slash", "razorleaf", "crabhammer", "karatechop", "aerialace", "airslash",
        "attackorder", "crosschop", "leafblade", "nightslash", "psychocut",
        "shadowclaw", "spacialrend", "stoneedge", "frostbreath", "stormthrow"
    ];
    
    if high_crit_moves.contains(&move_data.name.to_lowercase().as_str()) {
        crit_stage += 1;
    }
    
    // Super high critical hit ratio moves
    let super_high_crit_moves = ["frostbreath", "stormthrow"];
    if super_high_crit_moves.contains(&move_data.name.to_lowercase().as_str()) {
        crit_stage += 1; // These moves always crit
    }

    // Ability modifiers
    match attacker.ability.to_lowercase().as_str() {
        "superluck" => {
            // Super Luck increases critical hit ratio by 1 stage
            crit_stage += 1;
        }
        "sniper" => {
            // Sniper increases critical hit damage but not rate (handled in damage calc)
        }
        _ => {}
    }

    // Item modifiers
    if let Some(item) = &attacker.item {
        match item.to_lowercase().as_str() {
            "scopelens" => {
                // Scope Lens increases critical hit ratio by 1 stage
                crit_stage += 1;
            }
            "razorclaw" => {
                // Razor Claw increases critical hit ratio by 1 stage
                crit_stage += 1;
            }
            "luckypunch" => {
                // Lucky Punch increases Chansey's critical hit ratio by 2 stages
                if attacker.species.to_lowercase() == "chansey" {
                    crit_stage += 2;
                }
            }
            "leek" | "stick" => {
                // Leek/Stick increases Farfetch'd's critical hit ratio by 2 stages
                if attacker.species.to_lowercase() == "farfetchd" || attacker.species.to_lowercase() == "sirfetchd" {
                    crit_stage += 2;
                }
            }
            _ => {}
        }
    }

    // Calculate critical hit rate based on stage
    let crit_rate: f32 = match crit_stage {
        0 => 1.0 / 24.0,    // ~4.17%
        1 => 1.0 / 8.0,     // 12.5%
        2 => 1.0 / 2.0,     // 50%
        _ => 1.0 / 2.0,     // Cap at 50%
    };

    // Cap at 50% (1/2) - some moves like Frost Breath always crit
    if super_high_crit_moves.contains(&move_data.name.to_lowercase().as_str()) {
        1.0 // Always critical hit
    } else {
        crit_rate.min(0.5)
    }
}

/// Check if weather effects should be negated by any active Pokemon abilities
pub fn is_weather_negated(state: &BattleState) -> bool {
    use crate::engine::mechanics::abilities::get_ability_by_name;

    // Check all active Pokemon for weather negation abilities
    for side in [&state.side_one, &state.side_two] {
        for pokemon in &side.pokemon {
            if let Some(ability) = get_ability_by_name(pokemon.ability.as_str()) {
                if ability.negates_weather() {
                    return true;
                }
            }
        }
    }
    false
}

/// Calculate weather-based stat multipliers (Sandstorm SpDef for Rock, Snow Def for Ice)
pub fn get_weather_stat_multiplier(
    state: &BattleState,
    weather: &crate::core::instruction::Weather,
    pokemon: &Pokemon,
    stat: crate::core::instruction::Stat,
) -> f32 {
    use crate::core::instruction::Weather;

    // Check if weather is negated by Cloud Nine or Air Lock
    if is_weather_negated(state) {
        return 1.0;
    }

    match weather {
        Weather::Sand => {
            // Sandstorm boosts Special Defense of Rock types by 1.5x
            if stat == crate::core::instruction::Stat::SpecialDefense
                && pokemon.types.iter().any(|t| t.to_lowercase() == "rock")
            {
                1.5
            } else {
                1.0
            }
        }
        Weather::Snow => {
            // Snow boosts Defense of Ice types by 1.5x
            if stat == crate::core::instruction::Stat::Defense
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
    weather: &crate::core::instruction::Weather,
    move_type: &str,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::core::instruction::Weather;

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
        Weather::Sand | Weather::Sandstorm | Weather::Hail | Weather::Snow | Weather::None | 
        Weather::HarshSunlight | Weather::StrongWinds => 1.0,
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
    use crate::engine::mechanics::abilities::get_ability_by_name;
    use crate::core::instruction::SideCondition;
    use crate::core::battle_state::MoveCategory;

    // Check if attacker has Infiltrator ability to bypass screens
    if let Some(ability) = get_ability_by_name(attacker.ability.as_str()) {
        if ability.bypasses_screens() {
            return 1.0;
        }
    }

    // Determine defending side by finding which side contains the defender
    let defending_side = if state
        .side_one
        .pokemon
        .iter()
        .any(|p| std::ptr::eq(p, defender))
    {
        &state.side_one
    } else {
        &state.side_two
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
    terrain: &crate::core::instruction::Terrain,
    move_type: &str,
    attacker: &Pokemon,
    defender: &Pokemon,
    generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::core::instruction::Terrain;

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
    pokemon.ability.to_lowercase() == "adaptability"
}

/// Check if a Pokemon is grounded (affected by terrain)
pub fn is_grounded(pokemon: &Pokemon) -> bool {
    use crate::engine::mechanics::abilities::get_ability_by_name;

    // Check for Flying type
    if pokemon.types.iter().any(|t| t.to_lowercase() == "flying") {
        return false;
    }

    // Check for Levitate ability
    if let Some(ability) = get_ability_by_name(pokemon.ability.as_str()) {
        if ability.name().to_lowercase() == "levitate" {
            return false;
        }
    }

    // Check for items that affect grounding
    if let Some(ref item) = pokemon.item {
        match item.to_lowercase().as_str() {
            "airballoon" | "air balloon" => return false, // Air Balloon makes Pokemon ungrounded
            _ => {}
        }
    }

    // Check for volatile statuses that affect grounding
    if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::MagnetRise) {
        return false; // Magnet Rise makes Pokemon ungrounded
    }
    if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::Telekinesis) {
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

/// Apply base power modifications from items, like poke-engine does
/// This function modifies base power before damage calculation begins
fn apply_base_power_modifications(
    mut base_power: f32,
    context: &DamageContext,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    // Apply attacker's item modifications to base power
    if let Some(ref item_name) = context.attacker.pokemon.item {
        if let Some(item) = get_item_by_name(item_name) {
            // Only apply if this is actually an attacker item
            if item.is_attacker_item() {
                let item_modifier = item.modify_damage(context);
                base_power *= item_modifier.power_multiplier;
            }
        }
    }

    // Apply defender's item modifications to base power (for berries and defensive items)
    if let Some(ref item_name) = context.defender.pokemon.item {
        if let Some(item) = get_item_by_name(item_name) {
            // Only apply if this is actually a defender item
            if item.is_defender_item() {
                let item_modifier = item.modify_damage(context);
                // Berries and defensive items can reduce base power
                base_power *= item_modifier.power_multiplier;
                if item_modifier.damage_multiplier != 1.0 {
                    base_power *= item_modifier.damage_multiplier;
                }
            }
        }
    }

    base_power
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

/// Modern damage calculation using focused DamageContext
/// This replaces the legacy calculate_damage function that requires the entire State
pub fn calculate_damage_modern(
    context: &DamageContext,
    damage_roll: f32,
) -> DamageResult {
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
    let mut base_power = context.move_info.base_power as f32;

    // Early immunity checks would go here
    // (These would be extracted from ability/item logic)

    // Basic damage formula: ((2 * level / 5 + 2) * base_power * attack / defense / 50 + 2) * modifiers
    let level = context.attacker.pokemon.level as f32;
    
    // Get effective attack stat
    let attack_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => {
            context.attacker.effective_stats.get_effective_stat(crate::core::instruction::Stat::Attack) as f32
        }
        crate::core::battle_state::MoveCategory::Special => {
            context.attacker.effective_stats.get_effective_stat(crate::core::instruction::Stat::SpecialAttack) as f32
        }
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

    // Get effective defense stat
    let defense_stat = match context.move_info.category {
        crate::core::battle_state::MoveCategory::Physical => {
            context.defender.effective_stats.get_effective_stat(crate::core::instruction::Stat::Defense) as f32
        }
        crate::core::battle_state::MoveCategory::Special => {
            context.defender.effective_stats.get_effective_stat(crate::core::instruction::Stat::SpecialDefense) as f32
        }
        crate::core::battle_state::MoveCategory::Status => return DamageResult {
            damage: 0,
            blocked: false,
            was_critical: false,
            type_effectiveness: 1.0,
            effects,
        },
    };

    // Calculate base damage
    let base_damage = ((2.0 * level / 5.0 + 2.0) * base_power * attack_stat / defense_stat / 50.0 + 2.0);
    
    // Apply critical hit modifier
    let critical_modifier = if context.move_info.is_critical { 1.5 } else { 1.0 };
    let mut damage = base_damage * critical_modifier;
    
    // Apply random damage roll
    damage *= damage_roll;
    
    // Type effectiveness calculation
    let type_chart = TypeChart::new(9); // TODO: Get generation from context
    let move_type = PokemonType::from_str(&context.move_info.move_type).unwrap_or(PokemonType::Normal);
    
    let defender_type1 = PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
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
    let attacker_type1 = PokemonType::from_str(&context.attacker.pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        PokemonType::from_str(&context.attacker.pokemon.types[1]).unwrap_or(attacker_type1)
    } else {
        attacker_type1
    };
    
    let stab_multiplier = type_chart.calculate_stab_multiplier(
        move_type,
        (attacker_type1, attacker_type2),
        None, // Tera type support would go here
        false, // Adaptability check would go here
    );
    damage *= stab_multiplier;
    
    // Weather effects
    if let crate::core::instruction::Weather::Sun = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "fire" => {
                damage *= 1.5;
                effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
            }
            "water" => {
                damage *= 0.5;
                effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
            }
            _ => {}
        }
    } else if let crate::core::instruction::Weather::Rain = context.field.weather.condition {
        match context.move_info.move_type.to_lowercase().as_str() {
            "water" => {
                damage *= 1.5;
                effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
            }
            "fire" => {
                damage *= 0.5;
                effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
            }
            _ => {}
        }
    }
    
    // Status effects
    if context.attacker.pokemon.status == crate::core::instruction::PokemonStatus::Burn
        && context.move_info.category == crate::core::battle_state::MoveCategory::Physical
    {
        damage *= 0.5; // Burn halves physical attack
    }
    
    // Multi-target reduction
    if context.format.target_count > 1 {
        damage *= 0.75; // 25% reduction for spread moves
    }
    
    DamageResult {
        damage: damage.max(1.0) as i16, // Minimum 1 damage
        blocked: false,
        was_critical: context.move_info.is_critical,
        type_effectiveness,
        effects,
    }
}

// All damage calculation tests have been moved to tests/ directory
// using the TestFramework for realistic testing with PS data
