//! Generation 1 damage calculation implementation
//!
//! This module implements the specific damage calculation mechanics for
//! Generation 1 Pokemon games, including the unique critical hit system,
//! Special stat mechanics, and damage formula variations.

use crate::core::battle_state::Pokemon;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::damage::DamageRolls;
use crate::engine::combat::damage_context::{DamageContext, DamageEffect, DamageResult};
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::utils::normalize_name;
use crate::constants::moves::{GEN1_HIGH_CRIT_MOVES, GEN1_CRIT_SPEED_DIVISOR, GEN1_CRIT_RATE_DIVISOR, GEN1_HIGH_CRIT_MULTIPLIER};

/// Calculate Gen 1 critical hit probability based on base Speed
/// Formula: floor(base_speed / 2) / 256 for normal moves
/// High crit moves: min(8 * floor(base_speed / 2), 255) / 256
pub fn critical_hit_probability_gen1(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Get the base Speed stat for critical hit calculation
    // In Gen 1, we use the base stat, not the effective stat
    let base_speed = attacker.base_stats.speed;

    // Normalize move name for comparison
    let move_name = normalize_name(&move_data.name);

    // Calculate critical hit rate using the correct Gen 1 formula
    let crit_rate = if GEN1_HIGH_CRIT_MOVES.contains(&move_name.as_str()) {
        // High crit moves: min(8 * floor(base_speed / 2), 255)
        (GEN1_HIGH_CRIT_MULTIPLIER * (base_speed / GEN1_CRIT_SPEED_DIVISOR)).min(255)
    } else {
        // Normal moves: floor(base_speed / 2)
        (base_speed / GEN1_CRIT_SPEED_DIVISOR).min(255)
    };

    let final_rate = crit_rate as f32 / GEN1_CRIT_RATE_DIVISOR;

    final_rate
}

/// Calculate final damage with Gen 1/2 specific damage roll system
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

/// Gen 1 specific damage calculation with generation-specific mechanics
pub fn calculate_damage_gen1(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
            .get_effective_stat_with_crit_gen(
                crate::core::instructions::Stat::Attack,
                context.move_info.is_critical,
                true,
                crate::generation::Generation::Gen1,
            ) as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(
                crate::core::instructions::Stat::SpecialAttack,
                context.move_info.is_critical,
                true,
                crate::generation::Generation::Gen1,
            ) as f32,
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
            .get_effective_stat_with_crit_gen(
                crate::core::instructions::Stat::Defense,
                context.move_info.is_critical,
                false,
                crate::generation::Generation::Gen1,
            ) as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(
                crate::core::instructions::Stat::SpecialAttack,
                context.move_info.is_critical,
                false,
                crate::generation::Generation::Gen1,
            ) as f32,
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
    base_damage =
        (base_damage * attack_stat.max(1.0) * base_power / defense_stat.max(1.0)).floor() / 50.0;
    base_damage = base_damage.floor();

    // Track critical hit effect
    if context.move_info.is_critical {
        effects.push(DamageEffect::Critical);
    }

    // Apply additional modifiers before +2

    // Type effectiveness calculation (using Gen 1 type chart)
    let type_chart = TypeChart::get_cached(1); // Gen 1 type chart
    let move_type =
        PokemonType::from_normalized_str(context.move_info.move_type.as_str()).unwrap_or(PokemonType::Normal);

    let defender_type1 = context.defender.pokemon.types[0];
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        context.defender.pokemon.types[1]
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
    let attacker_type1 = context.attacker.pokemon.types[0];
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        context.attacker.pokemon.types[1]
    } else {
        attacker_type1
    };

    let has_stab = move_type == attacker_type1 || move_type == attacker_type2;

    // Add +2 to base damage (capped at 997)
    base_damage = (base_damage.min(997.0) + 2.0);

    // Apply weather effects (Gen 1 has weather but limited)
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        if context.move_info.move_type.as_str() == "fire" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect {
                weather: context.field.weather.condition,
            });
        } else if context.move_info.move_type.as_str() == "water" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect {
                weather: context.field.weather.condition,
            });
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        if context.move_info.move_type.as_str() == "water" {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect {
                weather: context.field.weather.condition,
            });
        } else if context.move_info.move_type.as_str() == "fire" {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect {
                weather: context.field.weather.condition,
            });
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
    let final_damage = calculate_final_damage_gen12(
        base_damage,
        damage_rolls,
        crate::generation::Generation::Gen1,
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
