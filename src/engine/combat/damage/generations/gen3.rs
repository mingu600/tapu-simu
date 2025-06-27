//! Generation 3 damage calculation implementation
//!
//! This module implements the specific damage calculation mechanics for
//! Generation 3 Pokemon games, including the modern damage formula
//! structure but with Gen 3-specific modifier order.

use crate::engine::combat::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::damage::DamageRolls;
use crate::core::instructions::Weather;
use crate::constants::moves::CRITICAL_HIT_MULTIPLIER_LEGACY;

/// Calculate final damage with Gen 3 specific damage roll system
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

/// Gen 3 specific damage calculation with generation-specific mechanics
pub fn calculate_damage_gen3(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
        CRITICAL_HIT_MULTIPLIER_LEGACY
    } else {
        1.0
    };

    // Weather effects
    let mut weather_multiplier = 1.0;
    if let Weather::Sun = context.field.weather.condition {
        match context.move_info.move_type {
            PokemonType::Fire => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            PokemonType::Water => {
                weather_multiplier = 0.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            _ => {}
        }
    } else if let Weather::Rain = context.field.weather.condition {
        match context.move_info.move_type {
            PokemonType::Water => {
                weather_multiplier = 1.5;
                effects.push(DamageEffect::WeatherEffect {
                    weather: context.field.weather.condition,
                });
            }
            PokemonType::Fire => {
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
    let type_chart = TypeChart::get_cached(3); // Gen 3 type chart
    let move_type = context.move_info.move_type;

    let attacker_type1 = context.attacker.pokemon.types[0];
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        context.attacker.pokemon.types[1]
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
    let defender_type1 = context.defender.pokemon.types[0];
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        context.defender.pokemon.types[1]
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