//! Generation 2 damage calculation implementation
//!
//! This module implements the specific damage calculation mechanics for
//! Generation 2 Pokemon games, including the fixed critical hit stages,
//! item modifiers, and Special Attack/Defense split.

use crate::core::battle_state::Pokemon;
use crate::data::showdown_types::MoveData;
use crate::utils::normalize_name;
use crate::engine::combat::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::damage::DamageRolls;
use crate::constants::moves::{CRITICAL_HIT_MULTIPLIER_LEGACY, GEN2_BASE_CRIT_RATE, GEN2_HIGH_CRIT_RATE, GEN2_HIGH_CRIT_MOVES};
use crate::engine::combat::damage::modifiers::items::get_gen2_item_modifier;

/// Calculate Gen 2 critical hit probability using fixed stages
/// Formula: Uses fixed stages - base 17/256 (~6.64%), high crit moves use +1 stage (1/8 = 12.5%)
pub fn critical_hit_probability_gen2(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Normalize move name for comparison
    let move_name = move_data.name.as_str();
    
    // Gen 2 uses fixed stages, not multipliers
    if GEN2_HIGH_CRIT_MOVES.contains(&move_name) {
        // High crit rate: +1 stage = 1/8 = 12.5%
        GEN2_HIGH_CRIT_RATE
    } else {
        // Normal crit rate: +0 stage = 17/256
        GEN2_BASE_CRIT_RATE
    }
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
            (base_damage * roll_index as f32 / 255.0).floor().max(1.0) as i16
        }
    }
}

/// Gen 2 specific damage calculation with generation-specific mechanics
pub fn calculate_damage_gen2(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
                effects,
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
        base_damage *= CRITICAL_HIT_MULTIPLIER_LEGACY;
        effects.push(DamageEffect::Critical);
    }
    
    // Add +2 to base damage
    base_damage = base_damage + 2.0;

    // Type effectiveness calculation (using Gen 2 type chart)
    let type_chart = TypeChart::get_cached(2); // Gen 2 type chart
    let move_type = context.move_info.move_type;

    let defender_type1 = context.defender.pokemon.types[0];
    let defender_type2 = if context.defender.pokemon.types.len() > 1 {
        context.defender.pokemon.types[1]
    } else {
        defender_type1
    };

    // Calculate combined type effectiveness
    let mut type_effectiveness = type_chart.get_effectiveness(move_type, defender_type1);
    if defender_type2 != defender_type1 {
        type_effectiveness *= type_chart.get_effectiveness(move_type, defender_type2);
    }

    // STAB calculation
    let attacker_type1 = context.attacker.pokemon.types[0];
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        context.attacker.pokemon.types[1]
    } else {
        attacker_type1
    };

    let has_stab = move_type == attacker_type1 || move_type == attacker_type2;

    // Apply weather effects (Gen 2 introduced weather)
    if let crate::core::instructions::Weather::Sun = context.field.weather.condition {
        if context.move_info.move_type == PokemonType::Fire {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type == PokemonType::Water {
            base_damage = (base_damage / 2.0).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        }
    } else if let crate::core::instructions::Weather::Rain = context.field.weather.condition {
        if context.move_info.move_type == PokemonType::Water {
            base_damage = (base_damage * 1.5).floor();
            effects.push(DamageEffect::WeatherEffect { weather: context.field.weather.condition });
        } else if context.move_info.move_type == PokemonType::Fire {
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