//! # Stat Boosting Items
//!
//! Items that boost offensive/defensive stats or provide conditional stat boosts.
//! This includes power items, defensive items, reactive items, gems, and seeds.

use std::collections::HashMap;

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, Stat,
    StatusInstruction, StatsInstruction,
};
use crate::data::repositories::{GameDataRepository, PokemonRepository};
use crate::engine::combat::damage_context::DamageContext;
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::generation::{Generation, GenerationBattleMechanics};
use crate::types::{Items, Moves};

use super::{ItemModifier, StatBoosts};

/// Get stat boosting item effect if the item is a stat booster
pub fn get_stat_boosting_item_effect(
    item_id: &Items,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    defender: Option<&Pokemon>,
    move_id: &Moves,
    move_type_id: &PokemonType,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    let move_type = move_type_id;
    
    match item_id {
        // Power Items
        Items::LIFEORB => Some(life_orb_effect()),
        Items::EXPERTBELT => Some(expert_belt_effect(context)),
        Items::MUSCLEBAND => Some(muscle_band_effect(move_category)),
        Items::WISEGLASSES => Some(wise_glasses_effect(move_category)),
        
        // Defensive Items
        Items::EVIOLITE => Some(eviolite_effect(attacker)),
        Items::ASSAULTVEST => Some(assault_vest_effect(move_category)),
        Items::AIRBALLOON => Some(air_balloon_effect()),
        Items::HEAVYDUTYBOOTS => Some(heavy_duty_boots_effect()),
        Items::ROCKYHELMET => Some(rocky_helmet_effect(context)),
        
        // Reactive Stat Items
        Items::WEAKNESSPOLICY => Some(weakness_policy_effect(context)),
        Items::FOCUSSASH => Some(focus_sash_effect(defender)),
        Items::ABSORBBULB => Some(absorb_bulb_effect(move_type)),
        Items::CELLBATTERY => Some(cell_battery_effect(move_type)),
        Items::SHELLBELL => Some(shell_bell_effect()),
        Items::METALPOWDER => Some(metal_powder_effect(defender)),
        Items::PUNCHINGGLOVE => Some(punching_glove_effect(context)),
        Items::BOOSTERENERGY => Some(booster_energy_effect()),
        
        // Generation-Aware Gems
        Items::NORMALGEM => Some(gem_effect(&PokemonType::Normal, move_type, generation)),
        Items::FIGHTINGGEM => Some(gem_effect(&PokemonType::Fighting, move_type, generation)),
        Items::FLYINGGEM => Some(gem_effect(&PokemonType::Flying, move_type, generation)),
        Items::POISONGEM => Some(gem_effect(&PokemonType::Poison, move_type, generation)),
        Items::GROUNDGEM => Some(gem_effect(&PokemonType::Ground, move_type, generation)),
        Items::ROCKGEM => Some(gem_effect(&PokemonType::Rock, move_type, generation)),
        Items::BUGGEM => Some(gem_effect(&PokemonType::Bug, move_type, generation)),
        Items::GHOSTGEM => Some(gem_effect(&PokemonType::Ghost, move_type, generation)),
        Items::STEELGEM => Some(gem_effect(&PokemonType::Steel, move_type, generation)),
        Items::FIREGEM => Some(gem_effect(&PokemonType::Fire, move_type, generation)),
        Items::WATERGEM => Some(gem_effect(&PokemonType::Water, move_type, generation)),
        Items::GRASSGEM => Some(gem_effect(&PokemonType::Grass, move_type, generation)),
        Items::ELECTRICGEM => Some(gem_effect(&PokemonType::Electric, move_type, generation)),
        Items::PSYCHICGEM => Some(gem_effect(&PokemonType::Psychic, move_type, generation)),
        Items::ICEGEM => Some(gem_effect(&PokemonType::Ice, move_type, generation)),
        Items::DRAGONGEM => Some(gem_effect(&PokemonType::Dragon, move_type, generation)),
        Items::DARKGEM => Some(gem_effect(&PokemonType::Dark, move_type, generation)),
        Items::FAIRYGEM => Some(gem_effect(&PokemonType::Fairy, move_type, generation)),
        
        // Seeds
        Items::ELECTRICSEED => Some(electric_seed_effect()),
        Items::GRASSYSEED => Some(grassy_seed_effect()),
        Items::MISTYSEED => Some(misty_seed_effect()),
        Items::PSYCHICSEED => Some(psychic_seed_effect()),
        
        _ => None,
    }
}

/// Check for item effects that trigger on switch-in
pub fn get_item_on_switch_in_effects(
    item_id: &Items,
    pokemon: &Pokemon,
    position: BattlePosition,
    generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    match item_id {
        Items::BOOSTERENERGY => Some(booster_energy_switch_in_effect(pokemon, position)),
        _ => None,
    }
}

// =============================================================================
// POWER ITEMS (4 items)
// =============================================================================

/// Life Orb - Boosts all moves by 1.3x but causes 10% recoil
fn life_orb_effect() -> ItemModifier {
    ItemModifier::new()
        .with_power_multiplier(1.3)
        .with_recoil_percentage(0.1) // 10% recoil
}

/// Expert Belt - Boosts super effective moves by 1.2x
fn expert_belt_effect(context: &DamageContext) -> ItemModifier {
    // Get type chart for the current generation
    let generation = match context.get_generation() {
        Generation::Gen1 => 1,
        Generation::Gen2 => 2,
        Generation::Gen3 => 3,
        Generation::Gen4 => 4,
        Generation::Gen5 => 5,
        Generation::Gen6 => 6,
        Generation::Gen7 => 7,
        Generation::Gen8 => 8,
        Generation::Gen9 => 9,
    };
    
    let type_chart = TypeChart::get_cached(generation);
    
    // Get move type directly
    let move_type = context.move_info.move_type;
    
    let defender_type1 = context.defender.pokemon.types.get(0).copied().unwrap_or(PokemonType::Normal);
    
    let defender_type2 = context.defender.pokemon.types.get(1).copied().unwrap_or(defender_type1);
    
    // Calculate type effectiveness including Tera type if applicable
    let type_effectiveness = type_chart.calculate_damage_multiplier(
        move_type,
        (defender_type1, defender_type2),
        context.defender.pokemon.tera_type,
        Some(context.move_info.name.as_str()),
    );
    
    if type_effectiveness > 1.0 {
        ItemModifier::new().with_power_multiplier(1.2)
    } else {
        ItemModifier::default()
    }
}

/// Muscle Band - Boosts physical moves by 1.1x
fn muscle_band_effect(move_category: MoveCategory) -> ItemModifier {
    if move_category == MoveCategory::Physical {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Wise Glasses - Boosts special moves by 1.1x
fn wise_glasses_effect(move_category: MoveCategory) -> ItemModifier {
    if move_category == MoveCategory::Special {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// DEFENSIVE ITEMS (5 items)
// =============================================================================

/// Eviolite - Boosts Defense and Special Defense by 1.5x for not fully evolved Pokemon
fn eviolite_effect(pokemon: &Pokemon) -> ItemModifier {
    // Check if Pokemon can still evolve (simplified - would need evolution data)
    let can_evolve = !is_fully_evolved(&pokemon.species);
    
    if can_evolve {
        ItemModifier::new()
            .with_defense_multiplier(crate::constants::EVIOLITE_DEF_MULTIPLIER)
            .with_special_defense_multiplier(crate::constants::EVIOLITE_SPDEF_MULTIPLIER)
    } else {
        ItemModifier::default()
    }
}

/// Helper function to check if Pokemon is fully evolved using data repository
fn is_fully_evolved(species: &crate::types::PokemonName) -> bool {
    // Use the global data repository to check evolution status
    if let Ok(repo) = GameDataRepository::global("data/ps-extracted") {
        repo.pokemon.find_by_name(species.as_str())
            .map(|data| {
                // Pokemon is fully evolved if it has no further evolutions
                data.evos.as_ref().map_or(true, |evos| evos.is_empty())
            })
            .unwrap_or(false) // Default to false if Pokemon not found
    } else {
        // Fallback to false if repository unavailable
        false
    }
}

/// Assault Vest - Boosts Special Defense by 1.5x but prevents status moves
fn assault_vest_effect(move_category: MoveCategory) -> ItemModifier {
    // Blocks status moves (handled elsewhere)
    // Provides 1.5x Special Defense multiplier
    ItemModifier::new().with_special_defense_multiplier(1.5)
}

/// Air Balloon - Provides Ground immunity until hit by damaging move
fn air_balloon_effect() -> ItemModifier {
    ItemModifier::new().with_ground_immunity()
}

/// Heavy Duty Boots - Provides hazard immunity
fn heavy_duty_boots_effect() -> ItemModifier {
    ItemModifier::new().with_hazard_immunity()
}

/// Rocky Helmet - Contact moves cause 1/6 max HP recoil to attacker
fn rocky_helmet_effect(context: &DamageContext) -> ItemModifier {
    // Check if move makes contact
    if context.move_info.is_contact {
        ItemModifier::new().with_contact_recoil(1.0 / 6.0) // 1/6 max HP
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// REACTIVE STAT ITEMS (8 items)
// =============================================================================

/// Weakness Policy - +2 Attack/Special Attack when hit by super effective moves
fn weakness_policy_effect(context: &DamageContext) -> ItemModifier {
    // Get type chart for the current generation
    let generation = match context.get_generation() {
        Generation::Gen1 => 1,
        Generation::Gen2 => 2,
        Generation::Gen3 => 3,
        Generation::Gen4 => 4,
        Generation::Gen5 => 5,
        Generation::Gen6 => 6,
        Generation::Gen7 => 7,
        Generation::Gen8 => 8,
        Generation::Gen9 => 9,
    };
    
    let type_chart = TypeChart::get_cached(generation);
    
    // Get move type directly
    let move_type = context.move_info.move_type;
    
    let defender_type1 = context.defender.pokemon.types.get(0).copied().unwrap_or(PokemonType::Normal);
    
    let defender_type2 = context.defender.pokemon.types.get(1).copied().unwrap_or(defender_type1);
    
    // Calculate type effectiveness including Tera type if applicable
    let type_effectiveness = type_chart.calculate_damage_multiplier(
        move_type,
        (defender_type1, defender_type2),
        context.defender.pokemon.tera_type,
        Some(context.move_info.name.as_str()),
    );
    
    if type_effectiveness > 1.0 {
        ItemModifier::new()
            .with_stat_boosts(StatBoosts::attack_and_special_attack(2, 2))
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Focus Sash - Survive any attack that would KO at full HP
fn focus_sash_effect(defender: Option<&Pokemon>) -> ItemModifier {
    if let Some(pokemon) = defender {
        if pokemon.hp == pokemon.max_hp {
            ItemModifier::new()
                .with_ko_prevention_at_full_hp()
                .with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Absorb Bulb - +1 Special Attack when hit by Water moves
fn absorb_bulb_effect(move_type: &PokemonType) -> ItemModifier {
    if *move_type == PokemonType::Water {
        ItemModifier::new()
            .with_stat_boosts(StatBoosts::special_attack(1))
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Cell Battery - +1 Attack when hit by Electric moves
fn cell_battery_effect(move_type: &PokemonType) -> ItemModifier {
    if *move_type == PokemonType::Electric {
        ItemModifier::new()
            .with_stat_boosts(StatBoosts::attack(1))
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Shell Bell - Restore 1/8 of damage dealt as HP
fn shell_bell_effect() -> ItemModifier {
    ItemModifier::new().with_drain_percentage(0.125) // 1/8 = 12.5%
}

/// Metal Powder - Reduce damage by 50% when held by Ditto
fn metal_powder_effect(defender: Option<&Pokemon>) -> ItemModifier {
    if let Some(pokemon) = defender {
        if pokemon.species == crate::types::PokemonName::DITTO {
            ItemModifier::new().with_damage_multiplier(0.5)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Punching Glove - 1.1x punch moves, removes contact, no Iron Fist boost
fn punching_glove_effect(context: &DamageContext) -> ItemModifier {
    // Check if move has punch flag
    if context.move_info.is_punch {
        ItemModifier::new()
            .with_power_multiplier(1.1)
            .with_contact_removal()
    } else {
        ItemModifier::default()
    }
}

/// Booster Energy - Activates Protosynthesis/Quark Drive abilities
fn booster_energy_effect() -> ItemModifier {
    // The actual ability boost is handled by the ability system
    // This just marks the item for consumption when appropriate
    ItemModifier::default()
}

/// Booster Energy switch-in effect
fn booster_energy_switch_in_effect(pokemon: &Pokemon, position: BattlePosition) -> BattleInstructions {
    // Check if Pokemon has Protosynthesis or Quark Drive
    let ability_id = pokemon.ability;
    if ability_id == crate::types::Abilities::PROTOSYNTHESIS || ability_id == crate::types::Abilities::QUARKDRIVE {
        // Consume the item and trigger the ability
        let consume_instruction = BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some(crate::types::Items::BOOSTERENERGY),
        });
        
        BattleInstructions::new(100.0, vec![consume_instruction])
    } else {
        BattleInstructions::new(100.0, vec![])
    }
}

// =============================================================================
// GENERATION-AWARE GEMS (18 items)
// =============================================================================

/// Gem effect with generation-aware multipliers
fn gem_effect(gem_type: &PokemonType, move_type: &PokemonType, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if move_type == gem_type {
        // Generation-aware multipliers:
        // Gen 5: 1.5x multiplier
        // Gen 6+: 1.3x multiplier
        let multiplier = match generation.generation() {
            Generation::Gen5 => 1.5,
            _ => 1.3, // Gen 6 and later
        };
        
        ItemModifier::new()
            .with_power_multiplier(multiplier)
            .with_consumed() // Gems are consumed after use
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// SEEDS (4 items)
// =============================================================================

/// Electric Seed - +1 Defense when Electric Terrain is active
fn electric_seed_effect() -> ItemModifier {
    // The terrain check and stat boost would be handled in terrain mechanics
    // This just marks the item for consumption when appropriate
    ItemModifier::new()
        .with_stat_boosts(StatBoosts::defense(1))
        .with_consumed()
}

/// Grassy Seed - +1 Defense when Grassy Terrain is active
fn grassy_seed_effect() -> ItemModifier {
    ItemModifier::new()
        .with_stat_boosts(StatBoosts::defense(1))
        .with_consumed()
}

/// Misty Seed - +1 Special Defense when Misty Terrain is active
fn misty_seed_effect() -> ItemModifier {
    ItemModifier::new()
        .with_stat_boosts(StatBoosts::special_defense(1))
        .with_consumed()
}

/// Psychic Seed - +1 Special Defense when Psychic Terrain is active
fn psychic_seed_effect() -> ItemModifier {
    ItemModifier::new()
        .with_stat_boosts(StatBoosts::special_defense(1))
        .with_consumed()
}

