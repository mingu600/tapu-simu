//! Generation 5-6 damage calculation implementation
//!
//! This module implements the specific damage calculation mechanics for
//! Generation 5 and 6 Pokemon games, including the critical hit nerf
//! in Gen 6 and modern damage formula refinements.

use crate::engine::combat::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};
use crate::engine::combat::damage::DamageRolls;
use crate::constants::moves::{CRITICAL_HIT_MULTIPLIER, MIN_DAMAGE_PERCENT};

/// Calculate final damage with Gen 5-6 specific system (no pokeRound)
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
    let mut damage_amount = (base_amount * (MIN_DAMAGE_PERCENT as f32 + roll_index as f32) / 100.0).floor();
    
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

/// Gen 5-6 specific damage calculation with generation-specific mechanics
pub fn calculate_damage_gen56(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen5)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen5)
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
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen5)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, crate::generation::Generation::Gen5)
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

    // Add +2 and apply critical hit
    base_damage = base_damage + 2.0;
    if context.move_info.is_critical {
        // Gen 6 changed critical hit from 2x to 1.5x
        base_damage *= CRITICAL_HIT_MULTIPLIER; // Assume Gen 6 mechanics
        effects.push(DamageEffect::Critical);
    }

    // Get type effectiveness data
    let type_chart = TypeChart::new(6); // Gen 6 type chart (includes Fairy type)
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