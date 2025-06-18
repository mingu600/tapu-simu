//! # Generation-Aware Damage Calculation
//!
//! This module provides damage calculation for Pokemon moves with full
//! generation-specific mechanics support.

use crate::engine::mechanics::abilities::{calculate_ability_modifiers, DamageContext};
use crate::engine::mechanics::items::{apply_expert_belt_boost, calculate_item_modifiers, get_item_by_name};
use super::type_effectiveness::{PokemonType, TypeChart};
use crate::data::types::EngineMoveData;
use crate::generation::{GenerationBattleMechanics, GenerationMechanics};
use crate::core::state::{Pokemon, State};

/// Calculate damage with generation-aware mechanics
pub fn calculate_damage(
    state: &State,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
) -> i16 {
    calculate_damage_with_targets(
        state,
        attacker,
        defender,
        move_data,
        is_critical,
        damage_roll,
        1,
    )
}

/// Calculate damage with generation-aware mechanics and target count for spread moves
pub fn calculate_damage_with_targets(
    state: &State,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
) -> i16 {
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
    )
}

/// Calculate damage with full generation-specific mechanics
pub fn calculate_damage_with_generation(
    state: &State,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &EngineMoveData,
    is_critical: bool,
    damage_roll: f32,
    target_count: usize,
    type_chart: &TypeChart,
    generation_mechanics: &GenerationMechanics,
) -> i16 {
    // Basic damage calculation formula
    let level = attacker.level as f32;
    let mut base_power = move_data.base_power.unwrap_or(0) as f32;

    if base_power == 0.0 {
        return 0; // Status moves don't deal damage
    }

    // Create damage context for ability calculations
    let damage_context = DamageContext {
        attacker: attacker.clone(),
        defender: defender.clone(),
        move_data: move_data.clone(),
        base_power: move_data.base_power.unwrap_or(0) as u8,
        is_critical,
        move_type: move_data.move_type.clone(),
        state: state.clone(),
    };

    // Apply base power modifications FIRST, like poke-engine does
    base_power = apply_base_power_modifications(base_power, &damage_context, generation_mechanics);

    if base_power == 0.0 {
        return 0; // Move blocked or nullified
    }

    let ability_modifier = calculate_ability_modifiers(&damage_context, generation_mechanics);
    let item_modifier = calculate_item_modifiers(&damage_context, generation_mechanics);

    // Check for complete immunity early
    if ability_modifier.blocks_move || item_modifier.blocks_move {
        return 0;
    }

    let attack_stat = match move_data.category {
        crate::core::state::MoveCategory::Physical => {
            attacker.get_effective_stat(crate::core::instruction::Stat::Attack) as f32
                * ability_modifier.attack_multiplier
                * item_modifier.attack_multiplier
        }
        crate::core::state::MoveCategory::Special => {
            attacker.get_effective_stat(crate::core::instruction::Stat::SpecialAttack) as f32
                * ability_modifier.special_attack_multiplier
                * item_modifier.special_attack_multiplier
        }
        crate::core::state::MoveCategory::Status => return 0,
    };

    let base_defense_stat = match move_data.category {
        crate::core::state::MoveCategory::Physical => {
            defender.get_effective_stat(crate::core::instruction::Stat::Defense)
        }
        crate::core::state::MoveCategory::Special => {
            defender.get_effective_stat(crate::core::instruction::Stat::SpecialDefense)
        }
        crate::core::state::MoveCategory::Status => return 0,
    } as f32;

    // Apply weather-based stat boosts
    let weather_defense_multiplier = get_weather_stat_multiplier(
        &state,
        &state.weather,
        defender,
        match move_data.category {
            crate::core::state::MoveCategory::Physical => crate::core::instruction::Stat::Defense,
            crate::core::state::MoveCategory::Special => crate::core::instruction::Stat::SpecialDefense,
            crate::core::state::MoveCategory::Status => return 0,
        },
    );

    let defense_stat = base_defense_stat
        * weather_defense_multiplier
        * match move_data.category {
            crate::core::state::MoveCategory::Physical => ability_modifier.defense_multiplier,
            crate::core::state::MoveCategory::Special => ability_modifier.special_defense_multiplier,
            crate::core::state::MoveCategory::Status => 1.0,
        }
        * match move_data.category {
            crate::core::state::MoveCategory::Physical => item_modifier.defense_multiplier,
            crate::core::state::MoveCategory::Special => item_modifier.special_defense_multiplier,
            crate::core::state::MoveCategory::Status => 1.0,
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

    // Type effectiveness
    let move_type = PokemonType::from_str(&move_data.move_type).unwrap_or(PokemonType::Normal);
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

    // Note: berries now modify base power instead of type effectiveness

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
        && move_data.category == crate::core::state::MoveCategory::Physical
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
pub fn is_weather_negated(state: &State) -> bool {
    use crate::engine::mechanics::abilities::get_ability_by_name;

    // Check all active Pokemon for weather negation abilities
    for side in [&state.side_one, &state.side_two] {
        for pokemon in &side.pokemon {
            if let Some(ability) = get_ability_by_name(&pokemon.ability) {
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
    state: &State,
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
    state: &State,
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
        Weather::Sand | Weather::Hail | Weather::Snow | Weather::None => 1.0,
    }
}

/// Calculate screen damage modifier (Reflect, Light Screen, Aurora Veil)
pub fn get_screen_damage_modifier(
    state: &State,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_category: &crate::core::state::MoveCategory,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    use crate::engine::mechanics::abilities::get_ability_by_name;
    use crate::core::instruction::SideCondition;
    use crate::core::state::MoveCategory;

    // Check if attacker has Infiltrator ability to bypass screens
    if let Some(ability) = get_ability_by_name(&attacker.ability) {
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
        Terrain::ElectricTerrain => {
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
        Terrain::GrassyTerrain => {
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
        Terrain::PsychicTerrain => {
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
        Terrain::MISTYTERRAIN => {
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
    if let Some(ability) = get_ability_by_name(&pokemon.ability) {
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
    if let Some(ref item_name) = context.attacker.item {
        if let Some(item) = get_item_by_name(item_name) {
            // Only apply if this is actually an attacker item
            if item.is_attacker_item() {
                let item_modifier = item.modify_damage(context);
                base_power *= item_modifier.power_multiplier;
            }
        }
    }

    // Apply defender's item modifications to base power (for berries and defensive items)
    if let Some(ref item_name) = context.defender.item {
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

/// Get the Tera type of a Pokemon if it's Terastallized
fn get_tera_type(pokemon: &Pokemon) -> Option<super::type_effectiveness::PokemonType> {
    #[cfg(feature = "terastallization")]
    {
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
    
    #[cfg(not(feature = "terastallization"))]
    {
        None
    }
}

// All damage calculation tests have been moved to tests/ directory
// using the TestFramework for realistic testing with PS data
