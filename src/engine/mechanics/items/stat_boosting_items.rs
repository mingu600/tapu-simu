//! # Stat Boosting Items
//!
//! Items that boost offensive/defensive stats or provide conditional stat boosts.
//! This includes power items, defensive items, reactive items, gems, and seeds.

use super::{ItemModifier, StatBoosts};
use crate::engine::combat::damage_context::DamageContext;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use crate::generation::{GenerationBattleMechanics, Generation};
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{Stat, PokemonStatus};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, StatsInstruction};
use std::collections::HashMap;

/// Get stat boosting item effect if the item is a stat booster
pub fn get_stat_boosting_item_effect(
    item_name: &str,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    defender: Option<&Pokemon>,
    move_name: &str,
    move_type: &str,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        // Power Items
        "lifeorb" => Some(life_orb_effect()),
        "expertbelt" => Some(expert_belt_effect(context)),
        "muscleband" => Some(muscle_band_effect(move_category)),
        "wiseglasses" => Some(wise_glasses_effect(move_category)),
        
        // Defensive Items
        "eviolite" => Some(eviolite_effect(attacker)),
        "assaultvest" => Some(assault_vest_effect(move_category)),
        "airballoon" => Some(air_balloon_effect()),
        "heavydutyboots" => Some(heavy_duty_boots_effect()),
        "rockyhelmet" => Some(rocky_helmet_effect(context)),
        
        // Reactive Stat Items
        "weaknesspolicy" => Some(weakness_policy_effect(context)),
        "focussash" => Some(focus_sash_effect(defender)),
        "absorbbulb" => Some(absorb_bulb_effect(move_type)),
        "cellbattery" => Some(cell_battery_effect(move_type)),
        "shellbell" => Some(shell_bell_effect()),
        "metalpowder" => Some(metal_powder_effect(defender)),
        "punchingglove" => Some(punching_glove_effect(context)),
        "boosterenergy" => Some(booster_energy_effect()),
        
        // Generation-Aware Gems
        "normalgem" => Some(gem_effect("normal", move_type, generation)),
        "fightinggem" => Some(gem_effect("fighting", move_type, generation)),
        "flyinggem" => Some(gem_effect("flying", move_type, generation)),
        "poisongem" => Some(gem_effect("poison", move_type, generation)),
        "groundgem" => Some(gem_effect("ground", move_type, generation)),
        "rockgem" => Some(gem_effect("rock", move_type, generation)),
        "buggem" => Some(gem_effect("bug", move_type, generation)),
        "ghostgem" => Some(gem_effect("ghost", move_type, generation)),
        "steelgem" => Some(gem_effect("steel", move_type, generation)),
        "firegem" => Some(gem_effect("fire", move_type, generation)),
        "watergem" => Some(gem_effect("water", move_type, generation)),
        "grassgem" => Some(gem_effect("grass", move_type, generation)),
        "electricgem" => Some(gem_effect("electric", move_type, generation)),
        "psychicgem" => Some(gem_effect("psychic", move_type, generation)),
        "icegem" => Some(gem_effect("ice", move_type, generation)),
        "dragongem" => Some(gem_effect("dragon", move_type, generation)),
        "darkgem" => Some(gem_effect("dark", move_type, generation)),
        "fairygem" => Some(gem_effect("fairy", move_type, generation)),
        
        // Seeds
        "electricseed" => Some(electric_seed_effect()),
        "grassyseed" => Some(grassy_seed_effect()),
        "mistyseed" => Some(misty_seed_effect()),
        "psychicseed" => Some(psychic_seed_effect()),
        
        _ => None,
    }
}

/// Check for item effects that trigger on switch-in
pub fn get_item_on_switch_in_effects(
    item_name: &str,
    pokemon: &Pokemon,
    position: BattlePosition,
    generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        "boosterenergy" => Some(booster_energy_switch_in_effect(pokemon, position)),
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
    
    let type_chart = TypeChart::new(generation);
    
    // Parse move type and defender types
    let move_type = match PokemonType::from_str(&context.move_info.move_type) {
        Some(t) => t,
        None => return ItemModifier::default(), // Invalid move type
    };
    
    let defender_type1 = match PokemonType::from_str(context.defender.pokemon.types.get(0).unwrap_or(&"Normal".to_string())) {
        Some(t) => t,
        None => return ItemModifier::default(), // Invalid defender type
    };
    
    let defender_type2 = match PokemonType::from_str(context.defender.pokemon.types.get(1).unwrap_or(&defender_type1.to_str().to_string())) {
        Some(t) => t,
        None => defender_type1, // Single type Pokemon
    };
    
    // Calculate type effectiveness including Tera type if applicable
    let type_effectiveness = type_chart.calculate_damage_multiplier(
        move_type,
        (defender_type1, defender_type2),
        context.defender.pokemon.tera_type
            .and_then(|t| PokemonType::from_str(&format!("{:?}", t))),
        Some(&context.move_info.name),
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
            .with_defense_multiplier(1.5)
            .with_special_defense_multiplier(1.5)
    } else {
        ItemModifier::default()
    }
}

/// Helper function to check if Pokemon is fully evolved
fn is_fully_evolved(species: &str) -> bool {
    // Simplified check - in real implementation would use evolution data
    match species.to_lowercase().as_str() {
        // Fully evolved Pokemon (partial list)
        "charizard" | "blastoise" | "venusaur" | "pikachu" | "raichu" |
        "nidoking" | "nidoqueen" | "clefable" | "ninetales" | "jigglypuff" |
        "wigglytuff" | "vileplume" | "parasect" | "venomoth" | "dugtrio" |
        "persian" | "golduck" | "primeape" | "arcanine" | "poliwrath" |
        "alakazam" | "machamp" | "victreebel" | "tentacruel" | "golem" |
        "rapidash" | "slowbro" | "magnezone" | "farfetchd" | "dodrio" |
        "dewgong" | "muk" | "cloyster" | "gengar" | "onix" | "hypno" |
        "kingler" | "electrode" | "exeggutor" | "marowak" | "hitmonlee" |
        "hitmonchan" | "lickitung" | "weezing" | "rhydon" | "chansey" |
        "tangela" | "kangaskhan" | "seaking" | "starmie" | "mrmine" |
        "scyther" | "jynx" | "electabuzz" | "magmar" | "pinsir" |
        "tauros" | "gyarados" | "lapras" | "ditto" | "vaporeon" |
        "jolteon" | "flareon" | "porygon" | "omastar" | "kabutops" |
        "aerodactyl" | "snorlax" | "articuno" | "zapdos" | "moltres" |
        "dragonite" | "mewtwo" | "mew" => true,
        _ => false,
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
    if context.move_info.data.flags.contains_key("contact") {
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
    
    let type_chart = TypeChart::new(generation);
    
    // Parse move type and defender types
    let move_type = match PokemonType::from_str(&context.move_info.move_type) {
        Some(t) => t,
        None => return ItemModifier::default(), // Invalid move type
    };
    
    let defender_type1 = match PokemonType::from_str(context.defender.pokemon.types.get(0).unwrap_or(&"Normal".to_string())) {
        Some(t) => t,
        None => return ItemModifier::default(), // Invalid defender type
    };
    
    let defender_type2 = match PokemonType::from_str(context.defender.pokemon.types.get(1).unwrap_or(&defender_type1.to_str().to_string())) {
        Some(t) => t,
        None => defender_type1, // Single type Pokemon
    };
    
    // Calculate type effectiveness including Tera type if applicable
    let type_effectiveness = type_chart.calculate_damage_multiplier(
        move_type,
        (defender_type1, defender_type2),
        context.defender.pokemon.tera_type
            .and_then(|t| PokemonType::from_str(&format!("{:?}", t))),
        Some(&context.move_info.name),
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
fn absorb_bulb_effect(move_type: &str) -> ItemModifier {
    if move_type.to_lowercase() == "water" {
        ItemModifier::new()
            .with_stat_boosts(StatBoosts::special_attack(1))
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Cell Battery - +1 Attack when hit by Electric moves
fn cell_battery_effect(move_type: &str) -> ItemModifier {
    if move_type.to_lowercase() == "electric" {
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
        if pokemon.species.to_lowercase() == "ditto" {
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
    if context.move_info.data.flags.contains_key("punch") {
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
    let ability_name = pokemon.ability.to_lowercase();
    
    if ability_name == "protosynthesis" || ability_name == "quarkdrive" {
        // Consume the item and trigger the ability
        let consume_instruction = BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some("Booster Energy".to_string()),
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
fn gem_effect(gem_type: &str, move_type: &str, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if move_type.to_lowercase() == gem_type.to_lowercase() {
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

