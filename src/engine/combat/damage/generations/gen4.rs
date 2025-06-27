//! Generation 4 damage calculation implementation
//!
//! This module implements the specific damage calculation mechanics for
//! Generation 4 Pokemon games, including the physical/special split,
//! abilities like Filter and Expert Belt items.

use crate::engine::combat::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::damage::DamageRolls;
use crate::constants::moves::MIN_DAMAGE_PERCENT;

/// Calculate final damage with Gen 4 specific system
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
    let mut damage = (base_damage * (MIN_DAMAGE_PERCENT as f32 + roll_index as f32) / 100.0).floor();
    
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

/// Gen 4 specific damage calculation with generation-specific mechanics
pub fn calculate_damage_gen4(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
        && context.attacker.pokemon.ability != crate::types::Abilities::GUTS
    {
        base_damage = (base_damage * 0.5).floor();
    }

    // Get type effectiveness data
    let type_chart = TypeChart::get_cached(4); // Gen 4 type chart
    let move_type = context.move_info.move_type;

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

    // STAB calculation
    let attacker_type1 = context.attacker.pokemon.types[0];
    let attacker_type2 = if context.attacker.pokemon.types.len() > 1 {
        context.attacker.pokemon.types[1]
    } else {
        attacker_type1
    };

    let stab_mod = if move_type == attacker_type1 || move_type == attacker_type2 {
        if context.attacker.pokemon.ability == crate::types::Abilities::ADAPTABILITY {
            2.0
        } else {
            1.5
        }
    } else {
        1.0
    };

    // Gen 4 specific modifiers (following damage-calc gen4.ts exactly)
    let filter_mod = if (context.defender.pokemon.ability == crate::types::Abilities::FILTER || context.defender.pokemon.ability == crate::types::Abilities::SOLIDROCK) 
        && (type1_effectiveness * type2_effectiveness) > 1.0 {
        0.75
    } else {
        1.0
    };

    let expert_belt_mod = if let Some(ref item) = context.attacker.pokemon.item {
        if *item == crate::types::Items::EXPERTBELT && (type1_effectiveness * type2_effectiveness) > 1.0 {
            1.2
        } else {
            1.0
        }
    } else {
        1.0
    };

    let tinted_lens_mod = if context.attacker.pokemon.ability == crate::types::Abilities::TINTEDLENS 
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