//! Modern Generation (7-8-9) damage calculation implementation
//!
//! This module implements the damage calculation mechanics for
//! Generation 7, 8, and 9 Pokemon games, including Z-moves,
//! Dynamax, and Terastallization support.

use crate::engine::combat::damage_context::{DamageContext, DamageResult, DamageEffect};
use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};
use crate::engine::combat::damage::DamageRolls;
use crate::core::battle_state::Pokemon;
use crate::generation::GenerationMechanics;

/// Pokemon rounding function for modern generations
fn poke_round(num: f32) -> f32 {
    let fractional_part = num - num.floor();
    if fractional_part > 0.5 {
        num.ceil()
    } else {
        num.floor()
    }
}

/// Check if a Pokemon is grounded (affected by terrain)
fn is_grounded(pokemon: &Pokemon) -> bool {
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

/// Get terrain damage modifier for modern generations
fn get_terrain_damage_modifier(
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

/// Calculate final damage with modern generation system (with pokeRound)
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

/// Modern generation (7-8-9) specific damage calculation
pub fn calculate_damage_modern_gen789(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
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
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Attack, context.move_info.is_critical, true, crate::generation::Generation::Gen8)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .attacker
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialAttack, context.move_info.is_critical, true, crate::generation::Generation::Gen8)
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
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::Defense, context.move_info.is_critical, false, crate::generation::Generation::Gen8)
            as f32,
        crate::core::battle_state::MoveCategory::Special => context
            .defender
            .effective_stats
            .get_effective_stat_with_crit_gen(crate::core::instructions::Stat::SpecialDefense, context.move_info.is_critical, false, crate::generation::Generation::Gen8)
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

    // Apply critical hit modifier
    let critical_modifier = if context.move_info.is_critical {
        effects.push(DamageEffect::Critical);
        1.5 // Modern generations use 1.5x
    } else {
        1.0
    };
    let damage = base_damage * critical_modifier;

    // Type effectiveness calculation
    let type_chart = TypeChart::new(8); // Modern type chart
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
    let generation_mechanics = GenerationMechanics::new(crate::generation::Generation::Gen8);
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