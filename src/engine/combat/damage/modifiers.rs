//! # Damage Modifiers System
//!
//! This module contains all damage modifier functions including weather, terrain,
//! abilities, items, field effects, and other battle conditions that affect damage calculation.

use crate::core::battle_state::{BattleState};
use crate::core::battle_state::pokemon_state::{Pokemon, Move};
use crate::core::battle_format::{BattleFormat, FormatType};
use crate::core::instructions::{Weather, Terrain, SideCondition};
use crate::data::showdown_types::MoveData;
use crate::engine::combat::type_effectiveness::{get_type_effectiveness, PokemonType};
use crate::generation::GenerationBattleMechanics;
use crate::utils::normalize_name;

// =============================================================================
// Weather Modifiers
// =============================================================================

/// Check if weather effects are negated by abilities
///
/// ## Parameters
/// - `state`: Current battle state
/// - `pokemon`: Pokemon to check for weather negation abilities
///
/// ## Returns
/// True if weather effects are negated
pub fn is_weather_negated(state: &BattleState, pokemon: &Pokemon) -> bool {
    if let Some(ref ability) = pokemon.ability {
        let ability_name = normalize_name(&ability.name);
        matches!(ability_name.as_str(), "cloudnine" | "airlock")
    } else {
        false
    }
}

/// Get weather-based damage modifier
///
/// ## Parameters
/// - `state`: Current battle state
/// - `move_type`: Type of the move being used
/// - `attacker`: The attacking Pokemon
///
/// ## Returns
/// Weather damage multiplier (typically 1.5x boost or 0.5x reduction)
pub fn get_weather_damage_modifier(
    state: &BattleState,
    move_type: &str,
    attacker: &Pokemon,
) -> f64 {
    if is_weather_negated(state, attacker) {
        return 1.0;
    }

    match (&state.field.weather, move_type) {
        (Some(Weather::Sun), "Fire") => 1.5,
        (Some(Weather::Sun), "Water") => 0.5,
        (Some(Weather::Rain), "Water") => 1.5,
        (Some(Weather::Rain), "Fire") => 0.5,
        _ => 1.0,
    }
}

/// Get weather-based stat multiplier
///
/// ## Parameters
/// - `state`: Current battle state
/// - `pokemon`: Pokemon to check for weather stat boosts
/// - `stat`: The stat being modified ("spdef" or "def")
///
/// ## Returns
/// Weather stat multiplier
pub fn get_weather_stat_multiplier(
    state: &BattleState,
    pokemon: &Pokemon,
    stat: &str,
) -> f64 {
    if is_weather_negated(state, pokemon) {
        return 1.0;
    }

    match (&state.field.weather, stat) {
        (Some(Weather::Sandstorm), "spdef") => {
            // Rock types get 1.5x Special Defense in Sandstorm
            if pokemon.types.contains(&PokemonType::Rock) {
                1.5
            } else {
                1.0
            }
        },
        (Some(Weather::Snow), "def") => {
            // Ice types get 1.5x Defense in Snow (Gen 9+)
            if pokemon.types.contains(&PokemonType::Ice) {
                1.5
            } else {
                1.0
            }
        },
        _ => 1.0,
    }
}

// =============================================================================
// Terrain Modifiers
// =============================================================================

/// Check if a Pokemon is grounded (affected by terrain)
///
/// ## Parameters
/// - `pokemon`: Pokemon to check
///
/// ## Returns
/// True if the Pokemon is grounded and affected by terrain
pub fn is_grounded(pokemon: &Pokemon) -> bool {
    // Flying types are not grounded
    if pokemon.types.contains(&PokemonType::Flying) {
        return false;
    }

    // Check for Levitate ability
    if let Some(ref ability) = pokemon.ability {
        if normalize_name(&ability.name) == "levitate" {
            return false;
        }
    }

    // Check for Air Balloon item
    if let Some(ref item) = pokemon.item {
        if normalize_name(&item.name) == "airballoon" {
            return false;
        }
    }

    // Check for Magnet Rise or Telekinesis status
    if pokemon.volatile_statuses.contains("magnetrise") || 
       pokemon.volatile_statuses.contains("telekinesis") {
        return false;
    }

    true
}

/// Get terrain-based damage modifier
///
/// ## Parameters
/// - `state`: Current battle state
/// - `move_type`: Type of the move being used
/// - `attacker`: The attacking Pokemon
///
/// ## Returns
/// Terrain damage multiplier
pub fn get_terrain_damage_modifier(
    state: &BattleState,
    move_type: &str,
    attacker: &Pokemon,
) -> f64 {
    if !is_grounded(attacker) {
        return 1.0;
    }

    match (&state.field.terrain, move_type) {
        (Some(Terrain::Electric), "Electric") => 1.3,  // 30% boost to Electric moves
        (Some(Terrain::Grassy), "Grass") => 1.3,       // 30% boost to Grass moves
        (Some(Terrain::Psychic), "Psychic") => 1.3,    // 30% boost to Psychic moves
        _ => 1.0,
    }
}

// =============================================================================
// Screen/Barrier Modifiers
// =============================================================================

/// Get screen damage modifier (Reflect, Light Screen, Aurora Veil)
///
/// ## Parameters
/// - `state`: Current battle state
/// - `move_category`: Physical or Special
/// - `defender_side`: Side reference for the defending Pokemon
/// - `format`: Battle format for determining screen effectiveness
///
/// ## Returns
/// Screen damage multiplier
pub fn get_screen_damage_modifier(
    state: &BattleState,
    move_category: &str,
    defender_side: &crate::core::battle_format::SideReference,
    format: &BattleFormat,
) -> f64 {
    let defender_conditions = match defender_side {
        crate::core::battle_format::SideReference::SideOne => &state.side_one.side_conditions,
        crate::core::battle_format::SideReference::SideTwo => &state.side_two.side_conditions,
    };

    // Aurora Veil protects against both Physical and Special moves
    if defender_conditions.contains(&SideCondition::AuroraVeil) {
        return match format.format_type {
            FormatType::Singles => 0.5,   // 50% reduction in singles
            _ => 0.67,                    // 33% reduction in doubles/triples
        };
    }

    // Reflect protects against Physical moves
    if move_category == "Physical" && defender_conditions.contains(&SideCondition::Reflect) {
        return match format.format_type {
            FormatType::Singles => 0.5,
            _ => 0.67,
        };
    }

    // Light Screen protects against Special moves
    if move_category == "Special" && defender_conditions.contains(&SideCondition::LightScreen) {
        return match format.format_type {
            FormatType::Singles => 0.5,
            _ => 0.67,
        };
    }

    1.0
}

// =============================================================================
// Spread Move Modifiers
// =============================================================================

/// Get spread move damage modifier
///
/// Multi-target moves deal reduced damage when hitting multiple targets.
///
/// ## Parameters
/// - `target_count`: Number of targets the move is hitting
/// - `format`: Battle format
///
/// ## Returns
/// Spread move damage multiplier
pub fn get_spread_move_modifier(target_count: usize, format: &BattleFormat) -> f64 {
    match format.format_type {
        FormatType::Singles => 1.0, // No reduction in singles
        _ => {
            if target_count > 1 {
                0.75 // 25% damage reduction for multi-target moves
            } else {
                1.0
            }
        }
    }
}

// =============================================================================
// Ability Modifiers
// =============================================================================

/// Check if a Pokemon has the Adaptability ability
///
/// ## Parameters
/// - `pokemon`: Pokemon to check
///
/// ## Returns
/// True if the Pokemon has Adaptability
pub fn has_adaptability_ability(pokemon: &Pokemon) -> bool {
    if let Some(ref ability) = pokemon.ability {
        normalize_name(&ability.name) == "adaptability"
    } else {
        false
    }
}

/// Get STAB (Same Type Attack Bonus) modifier
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
/// - `move_type`: Type of the move being used
///
/// ## Returns
/// STAB multiplier (1.5x normally, 2.0x with Adaptability)
pub fn get_stab_modifier(attacker: &Pokemon, move_type: &str) -> f64 {
    let move_type_enum = PokemonType::from_str(move_type);
    
    if attacker.types.contains(&move_type_enum) {
        if has_adaptability_ability(attacker) {
            2.0 // Adaptability increases STAB from 1.5x to 2.0x
        } else {
            1.5 // Normal STAB
        }
    } else {
        1.0 // No STAB
    }
}

/// Get Terastallization STAB modifier (Gen 9)
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
/// - `move_type`: Type of the move being used
///
/// ## Returns
/// Tera STAB multiplier
pub fn get_tera_stab_modifier(attacker: &Pokemon, move_type: &str) -> f64 {
    // TODO: Implement Terastallization mechanics when available
    // For now, return normal STAB
    get_stab_modifier(attacker, move_type)
}

/// Get Filter/Solid Rock damage reduction
///
/// These abilities reduce super effective damage by 25%.
///
/// ## Parameters
/// - `defender`: The defending Pokemon
/// - `type_effectiveness`: Type effectiveness multiplier
///
/// ## Returns
/// Filter/Solid Rock modifier
pub fn get_filter_modifier(defender: &Pokemon, type_effectiveness: f64) -> f64 {
    if type_effectiveness > 1.0 {
        if let Some(ref ability) = defender.ability {
            let ability_name = normalize_name(&ability.name);
            if matches!(ability_name.as_str(), "filter" | "solidrock" | "prismarmor") {
                return 0.75; // 25% reduction to super effective moves
            }
        }
    }
    1.0
}

/// Get Tinted Lens damage boost
///
/// Tinted Lens doubles the damage of not very effective moves.
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
/// - `type_effectiveness`: Type effectiveness multiplier
///
/// ## Returns
/// Tinted Lens modifier
pub fn get_tinted_lens_modifier(attacker: &Pokemon, type_effectiveness: f64) -> f64 {
    if type_effectiveness < 1.0 && type_effectiveness > 0.0 {
        if let Some(ref ability) = attacker.ability {
            if normalize_name(&ability.name) == "tintedlens" {
                return 2.0; // Double damage for not very effective moves
            }
        }
    }
    1.0
}

// =============================================================================
// Item Modifiers
// =============================================================================

/// Get item damage modifier for Generation 2
///
/// ## Parameters
/// - `item`: Item name
/// - `move_type`: Type of the move being used
///
/// ## Returns
/// Gen 2 item damage multiplier
pub fn get_gen2_item_modifier(item: &str, move_type: &str) -> f64 {
    let item_name = item.to_lowercase().replace("-", "").replace(" ", "");
    
    match (item_name.as_str(), move_type) {
        // Type-boosting items from Gen 2 (10% boost)
        ("blackbelt", "Fighting") => 1.1,
        ("blackglasses", "Dark") => 1.1,
        ("charcoal", "Fire") => 1.1,
        ("dragonfang", "Dragon") => 1.1,
        ("hardstone", "Rock") => 1.1,
        ("magnet", "Electric") => 1.1,
        ("metalcoat", "Steel") => 1.1,
        ("miracleseed", "Grass") => 1.1,
        ("mysticwater", "Water") => 1.1,
        ("nevermeltice", "Ice") => 1.1,
        ("pinkbow" | "polkadotbow", "Normal") => 1.1,
        ("poisonbarb", "Poison") => 1.1,
        ("sharpbeak", "Flying") => 1.1,
        ("silverpowder", "Bug") => 1.1,
        ("softsand", "Ground") => 1.1,
        ("spelltag", "Ghost") => 1.1,
        ("twistedspoon", "Psychic") => 1.1,
        _ => 1.0,
    }
}

/// Get Expert Belt damage modifier
///
/// Expert Belt increases damage of super effective moves by 20%.
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
/// - `type_effectiveness`: Type effectiveness multiplier
///
/// ## Returns
/// Expert Belt modifier
pub fn get_expert_belt_modifier(attacker: &Pokemon, type_effectiveness: f64) -> f64 {
    if type_effectiveness > 1.0 {
        if let Some(ref item) = attacker.item {
            if normalize_name(&item.name) == "expertbelt" {
                return 1.2; // 20% boost to super effective moves
            }
        }
    }
    1.0
}

/// Get Life Orb damage modifier
///
/// Life Orb increases damage of all moves by 30%.
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
///
/// ## Returns
/// Life Orb modifier
pub fn get_life_orb_modifier(attacker: &Pokemon) -> f64 {
    if let Some(ref item) = attacker.item {
        if normalize_name(&item.name) == "lifeorb" {
            return 1.3; // 30% boost to all moves
        }
    }
    1.0
}

// =============================================================================
// Status Effect Modifiers
// =============================================================================

/// Get burn status damage modifier
///
/// Burn reduces the damage of physical moves by 50%.
///
/// ## Parameters
/// - `attacker`: The attacking Pokemon
/// - `move_category`: Physical or Special
///
/// ## Returns
/// Burn damage modifier
pub fn get_burn_modifier(attacker: &Pokemon, move_category: &str) -> f64 {
    if move_category == "Physical" && attacker.status == Some(crate::core::instructions::PokemonStatus::Burn) {
        // Check for Guts ability which ignores burn damage reduction
        if let Some(ref ability) = attacker.ability {
            if normalize_name(&ability.name) == "guts" {
                return 1.0;
            }
        }
        0.5 // 50% damage reduction for burned physical attackers
    } else {
        1.0
    }
}

// =============================================================================
// Type Effectiveness Helpers
// =============================================================================

/// Get type effectiveness multiplier
///
/// ## Parameters
/// - `move_type`: Type of the attacking move
/// - `defender_types`: Types of the defending Pokemon
/// - `generation`: Generation mechanics for type chart
///
/// ## Returns
/// Type effectiveness multiplier
pub fn get_type_effectiveness_modifier(
    move_type: &str,
    defender_types: &[PokemonType],
    generation: &dyn GenerationBattleMechanics,
) -> f64 {
    let attacking_type = PokemonType::from_str(move_type);
    get_type_effectiveness(&attacking_type, defender_types, generation)
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Calculate all applicable damage modifiers in correct order
///
/// This is a convenience function that applies all modifiers in the correct
/// order for damage calculation.
///
/// ## Parameters
/// - `state`: Current battle state
/// - `attacker`: The attacking Pokemon
/// - `defender`: The defending Pokemon  
/// - `move_data`: Data for the move being used
/// - `target_count`: Number of targets
/// - `generation`: Generation mechanics
///
/// ## Returns
/// Combined modifier value
pub fn calculate_all_modifiers(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    target_count: usize,
    generation: &dyn GenerationBattleMechanics,
) -> f64 {
    let mut modifier = 1.0;
    
    // Weather modifier
    modifier *= get_weather_damage_modifier(state, &move_data.move_type, attacker);
    
    // Terrain modifier
    modifier *= get_terrain_damage_modifier(state, &move_data.move_type, attacker);
    
    // Screen modifier
    let defender_side = if state.side_one.active_pokemon.iter().any(|p| std::ptr::eq(p, defender)) {
        crate::core::battle_format::SideReference::SideOne
    } else {
        crate::core::battle_format::SideReference::SideTwo
    };
    modifier *= get_screen_damage_modifier(state, &move_data.category, &defender_side, &state.format);
    
    // Spread move modifier
    modifier *= get_spread_move_modifier(target_count, &state.format);
    
    // Type effectiveness
    let type_effectiveness = get_type_effectiveness_modifier(&move_data.move_type, &defender.types, generation);
    modifier *= type_effectiveness;
    
    // STAB modifier
    modifier *= get_stab_modifier(attacker, &move_data.move_type);
    
    // Ability modifiers
    modifier *= get_filter_modifier(defender, type_effectiveness);
    modifier *= get_tinted_lens_modifier(attacker, type_effectiveness);
    
    // Item modifiers
    modifier *= get_expert_belt_modifier(attacker, type_effectiveness);
    modifier *= get_life_orb_modifier(attacker);
    
    // Status modifiers
    modifier *= get_burn_modifier(attacker, &move_data.category);
    
    modifier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::{BattleState};
use crate::core::battle_state::pokemon_state::{Pokemon, Move};
    use crate::core::battle_format::BattleFormat;
    use crate::core::instructions::{Weather, Terrain, PokemonStatus};
    use crate::engine::combat::type_effectiveness::PokemonType;

    fn create_test_pokemon() -> Pokemon {
        Pokemon {
            species_id: "pikachu".to_string(),
            types: vec![PokemonType::Electric],
            ability: None,
            item: None,
            status: None,
            volatile_statuses: std::collections::HashSet::new(),
            // ... other fields with defaults
            ..Default::default()
        }
    }

    fn create_test_state() -> BattleState {
        BattleState {
            format: BattleFormat::gen9_ou(),
            field: Field {
                weather: None,
                terrain: None,
                global_effects: std::collections::HashMap::new(),
            },
            side_one: Side::default(),
            side_two: Side::default(),
            // ... other fields with defaults
            ..Default::default()
        }
    }

    #[test]
    fn test_weather_damage_modifiers() {
        let mut state = create_test_state();
        let pokemon = create_test_pokemon();
        
        // Test sun boosting Fire moves
        state.field.weather = Some(Weather::Sun);
        assert_eq!(get_weather_damage_modifier(&state, "Fire", &pokemon), 1.5);
        assert_eq!(get_weather_damage_modifier(&state, "Water", &pokemon), 0.5);
        
        // Test rain boosting Water moves
        state.field.weather = Some(Weather::Rain);
        assert_eq!(get_weather_damage_modifier(&state, "Water", &pokemon), 1.5);
        assert_eq!(get_weather_damage_modifier(&state, "Fire", &pokemon), 0.5);
    }

    #[test]
    fn test_terrain_damage_modifiers() {
        let mut state = create_test_state();
        let pokemon = create_test_pokemon();
        
        state.field.terrain = Some(Terrain::Electric);
        assert_eq!(get_terrain_damage_modifier(&state, "Electric", &pokemon), 1.3);
        assert_eq!(get_terrain_damage_modifier(&state, "Fire", &pokemon), 1.0);
    }

    #[test]
    fn test_grounded_detection() {
        let mut pokemon = create_test_pokemon();
        assert!(is_grounded(&pokemon)); // Electric type is grounded
        
        pokemon.types = vec![PokemonType::Flying];
        assert!(!is_grounded(&pokemon)); // Flying type is not grounded
    }

    #[test]
    fn test_stab_calculation() {
        let pokemon = create_test_pokemon(); // Electric type
        
        assert_eq!(get_stab_modifier(&pokemon, "Electric"), 1.5);
        assert_eq!(get_stab_modifier(&pokemon, "Fire"), 1.0);
    }

    #[test]
    fn test_spread_move_modifier() {
        let format = BattleFormat::gen9_ou(); // Singles format
        assert_eq!(get_spread_move_modifier(2, &format), 1.0); // No reduction in singles
        
        let doubles_format = BattleFormat::gen9_vgc(); // Doubles format
        assert_eq!(get_spread_move_modifier(2, &doubles_format), 0.75); // Reduction in doubles
        assert_eq!(get_spread_move_modifier(1, &doubles_format), 1.0); // No reduction for single target
    }

    #[test]
    fn test_burn_modifier() {
        let mut pokemon = create_test_pokemon();
        
        assert_eq!(get_burn_modifier(&pokemon, "Physical"), 1.0); // No burn
        assert_eq!(get_burn_modifier(&pokemon, "Special"), 1.0); // Special moves unaffected
        
        pokemon.status = Some(PokemonStatus::Burn);
        assert_eq!(get_burn_modifier(&pokemon, "Physical"), 0.5); // Burn reduces physical damage
        assert_eq!(get_burn_modifier(&pokemon, "Special"), 1.0); // Special moves still unaffected
    }
}