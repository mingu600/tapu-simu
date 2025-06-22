use crate::types::identifiers::{AbilityId, TypeId};
use crate::core::battle_state::Pokemon;
use crate::core::battle_state::BattleState;
use crate::core::battle_format::BattlePosition;
use crate::engine::combat::damage_context::DamageContext;

// Legacy DamageContext type alias removed - using modern DamageContext directly

#[derive(Debug, Clone)]
pub struct AbilityContext<'a> {
    pub user_position: BattlePosition,
    pub target_position: Option<BattlePosition>,
    pub move_type: Option<TypeId>,
    pub move_id: Option<&'a str>,
    pub base_power: Option<u16>,
    pub is_critical: bool,
    pub is_contact: bool,
    pub state: &'a BattleState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilityEffectResult {
    pub damage_multiplier: f32,
    pub power_multiplier: f32,
    pub attack_multiplier: f32,
    pub defense_multiplier: f32,
    pub special_attack_multiplier: f32,
    pub special_defense_multiplier: f32,
    pub immunity: bool,
    pub ignore_type_effectiveness: bool,
    pub stab_multiplier: f32,
    pub negates_weather: bool,
    pub bypasses_screens: bool,
}

impl Default for AbilityEffectResult {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            power_multiplier: 1.0,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            special_attack_multiplier: 1.0,
            special_defense_multiplier: 1.0,
            immunity: false,
            ignore_type_effectiveness: false,
            stab_multiplier: 1.0,
            negates_weather: false,
            bypasses_screens: false,
        }
    }
}

impl AbilityEffectResult {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn immunity() -> Self {
        Self {
            immunity: true,
            ..Default::default()
        }
    }

    pub fn damage_multiplier(multiplier: f32) -> Self {
        Self {
            damage_multiplier: multiplier,
            ..Default::default()
        }
    }

    pub fn power_multiplier(multiplier: f32) -> Self {
        Self {
            power_multiplier: multiplier,
            ..Default::default()
        }
    }

    pub fn stab_multiplier(multiplier: f32) -> Self {
        Self {
            stab_multiplier: multiplier,
            ..Default::default()
        }
    }

    pub fn negates_weather() -> Self {
        Self {
            negates_weather: true,
            ..Default::default()
        }
    }

    pub fn bypasses_screens() -> Self {
        Self {
            bypasses_screens: true,
            ..Default::default()
        }
    }

    pub fn stat_multiplier(attack: f32, defense: f32, special_attack: f32, special_defense: f32) -> Self {
        Self {
            attack_multiplier: attack,
            defense_multiplier: defense,
            special_attack_multiplier: special_attack,
            special_defense_multiplier: special_defense,
            ..Default::default()
        }
    }
}

pub fn apply_ability_effect(ability: &AbilityId, context: AbilityContext) -> AbilityEffectResult {
    match ability.as_str() {
        // Type immunities
        "levitate" => apply_levitate(context),
        "flashfire" => apply_flash_fire(context),
        "waterabsorb" => apply_water_absorb(context),
        "voltabsorb" => apply_volt_absorb(context),
        "sapsipper" => apply_sap_sipper(context),
        "stormdrain" => apply_storm_drain(context),
        "lightningrod" => apply_lightning_rod(context),
        "motordrive" => apply_motor_drive(context),
        "dryskin" => apply_dry_skin(context),
        "wonderguard" => apply_wonder_guard(context),

        // Damage reduction
        "filter" | "solidrock" => apply_damage_reduction(context, 0.75),
        "multiscale" => apply_multiscale(context),
        "thickfat" => apply_thick_fat(context),

        // Damage boost
        "neuroforce" => apply_neuroforce(context),
        "tintedlens" => apply_tinted_lens(context),
        "steelworker" => apply_steelworker(context),

        // Power boost
        "technician" => apply_technician(context),
        "skilllink" => apply_skill_link(context),
        "strongjaw" => apply_strong_jaw(context),
        "toughclaws" => apply_tough_claws(context),
        "punkrock" => apply_punk_rock(context),

        // Stat multipliers
        "hugepower" | "purepower" => apply_stat_doubler(1.0, 1.0, 2.0, 1.0),
        "guts" => apply_guts(context),
        "marvelscale" => apply_marvel_scale(context),
        "plus" | "minus" => apply_plus_minus(context),

        // STAB modifiers
        "adaptability" => apply_adaptability(context),

        // Weather/environment
        "cloudnine" | "airlock" => apply_weather_negation(),

        // Screen bypass
        "infiltrator" => apply_infiltrator(),

        // Status immunity
        "waterveil" => apply_water_veil(context),
        "magmaarmor" => apply_magma_armor(context),
        "immunity" => apply_immunity_ability(context),
        "limber" => apply_limber(context),
        "insomnia" | "vitalspirit" => apply_sleep_immunity(context),
        "owntempo" => apply_own_tempo(context),
        "oblivious" => apply_oblivious(context),

        // Special cases
        "normalize" => apply_normalize(context),
        "refrigerate" => apply_type_change(context, "ice", 1.2),
        "pixilate" => apply_type_change(context, "fairy", 1.2),
        "aerilate" => apply_type_change(context, "flying", 1.2),
        "galvanize" => apply_type_change(context, "electric", 1.2),

        _ => AbilityEffectResult::none(),
    }
}

// Type immunity abilities
fn apply_levitate(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "ground" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_flash_fire(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "fire" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_water_absorb(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "water" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_volt_absorb(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "electric" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_sap_sipper(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "grass" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_storm_drain(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "water" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_lightning_rod(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "electric" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_motor_drive(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "electric" {
            return AbilityEffectResult::immunity();
        }
    }
    AbilityEffectResult::none()
}

fn apply_dry_skin(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        match move_type.as_str() {
            "water" => AbilityEffectResult::immunity(),
            "fire" => AbilityEffectResult::damage_multiplier(1.25),
            _ => AbilityEffectResult::none(),
        }
    } else {
        AbilityEffectResult::none()
    }
}

fn apply_wonder_guard(context: AbilityContext) -> AbilityEffectResult {
    // Wonder Guard only allows super effective moves to hit
    use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
    
    // If no move type or target position, can't check effectiveness
    let move_type_id = match context.move_type {
        Some(type_id) => type_id,
        None => return AbilityEffectResult::none(),
    };
    
    let target_position = match context.target_position {
        Some(pos) => pos,
        None => return AbilityEffectResult::none(),
    };
    
    // Get the defending Pokemon's types
    let defender = match context.state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return AbilityEffectResult::none(),
    };
    
    // Convert type string to PokemonType enum
    let defender_type1 = PokemonType::from_str(&defender.types[0])
        .unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1])
            .unwrap_or(defender_type1)
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_str(&move_type_str)
        .unwrap_or(PokemonType::Normal);
    
    // Check type effectiveness
    let type_chart = TypeChart::default();
    let effectiveness1 = type_chart.get_effectiveness(move_type, defender_type1);
    let effectiveness2 = if defender_type1 != defender_type2 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };
    
    let total_effectiveness = effectiveness1 * effectiveness2;
    
    // Wonder Guard grants immunity unless the move is super effective (>1.0)
    if total_effectiveness <= 1.0 {
        AbilityEffectResult::immunity()
    } else {
        AbilityEffectResult::none()
    }
}

// Damage reduction abilities
fn apply_damage_reduction(_context: AbilityContext, multiplier: f32) -> AbilityEffectResult {
    AbilityEffectResult::damage_multiplier(multiplier)
}

fn apply_multiscale(context: AbilityContext) -> AbilityEffectResult {
    // Check if Pokemon is at full HP
    if let Some(pokemon) = context.state.get_pokemon_at_position(context.user_position) {
        if pokemon.hp == pokemon.max_hp {
            return AbilityEffectResult::damage_multiplier(0.5);
        }
    }
    AbilityEffectResult::none()
}

fn apply_thick_fat(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "fire" || move_type.as_str() == "ice" {
            return AbilityEffectResult::damage_multiplier(0.5);
        }
    }
    AbilityEffectResult::none()
}

// Damage boost abilities
fn apply_neuroforce(context: AbilityContext) -> AbilityEffectResult {
    // Boosts super effective moves by 25%
    use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
    
    // If no move type or target position, can't check effectiveness
    let move_type_id = match context.move_type {
        Some(type_id) => type_id,
        None => return AbilityEffectResult::none(),
    };
    
    let target_position = match context.target_position {
        Some(pos) => pos,
        None => return AbilityEffectResult::none(),
    };
    
    // Get the defending Pokemon's types
    let defender = match context.state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return AbilityEffectResult::none(),
    };
    
    // Convert type string to PokemonType enum
    let defender_type1 = PokemonType::from_str(&defender.types[0])
        .unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1])
            .unwrap_or(defender_type1)
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_str(&move_type_str)
        .unwrap_or(PokemonType::Normal);
    
    // Check type effectiveness
    let type_chart = TypeChart::default();
    let effectiveness1 = type_chart.get_effectiveness(move_type, defender_type1);
    let effectiveness2 = if defender_type1 != defender_type2 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };
    
    let total_effectiveness = effectiveness1 * effectiveness2;
    
    // Neuroforce boosts super effective moves by 25% (1.25x multiplier)
    if total_effectiveness > 1.0 {
        AbilityEffectResult::damage_multiplier(1.25)
    } else {
        AbilityEffectResult::none()
    }
}

fn apply_tinted_lens(context: AbilityContext) -> AbilityEffectResult {
    // Doubles damage of not very effective moves
    use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
    
    // If no move type or target position, can't check effectiveness
    let move_type_id = match context.move_type {
        Some(type_id) => type_id,
        None => return AbilityEffectResult::none(),
    };
    
    let target_position = match context.target_position {
        Some(pos) => pos,
        None => return AbilityEffectResult::none(),
    };
    
    // Get the defending Pokemon's types
    let defender = match context.state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return AbilityEffectResult::none(),
    };
    
    // Convert type string to PokemonType enum
    let defender_type1 = PokemonType::from_str(&defender.types[0])
        .unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1])
            .unwrap_or(defender_type1)
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_str(&move_type_str)
        .unwrap_or(PokemonType::Normal);
    
    // Check type effectiveness
    let type_chart = TypeChart::default();
    let effectiveness1 = type_chart.get_effectiveness(move_type, defender_type1);
    let effectiveness2 = if defender_type1 != defender_type2 {
        type_chart.get_effectiveness(move_type, defender_type2)
    } else {
        1.0
    };
    
    let total_effectiveness = effectiveness1 * effectiveness2;
    
    // Tinted Lens doubles damage of not very effective moves (<1.0)
    if total_effectiveness < 1.0 {
        AbilityEffectResult::damage_multiplier(2.0)
    } else {
        AbilityEffectResult::none()
    }
}

fn apply_steelworker(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "steel" {
            return AbilityEffectResult::power_multiplier(1.5);
        }
    }
    AbilityEffectResult::none()
}

// Power boost abilities
fn apply_technician(context: AbilityContext) -> AbilityEffectResult {
    if let Some(base_power) = context.base_power {
        if base_power <= 60 {
            return AbilityEffectResult::power_multiplier(1.5);
        }
    }
    AbilityEffectResult::none()
}

fn apply_skill_link(context: AbilityContext) -> AbilityEffectResult {
    // Skill Link makes multi-hit moves always hit the maximum number of times
    // This is handled in move execution, not damage calculation
    AbilityEffectResult::none()
}

fn apply_strong_jaw(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_id) = context.move_id {
        // Check if move is a biting move
        let biting_moves = [
            "bite", "crunch", "firefang", "icefang", "thunderfang",
            "poisonfang", "psychicfangs", "hyperfang", "superfang",
        ];
        if biting_moves.contains(&move_id) {
            return AbilityEffectResult::power_multiplier(1.5);
        }
    }
    AbilityEffectResult::none()
}

fn apply_tough_claws(context: AbilityContext) -> AbilityEffectResult {
    if context.is_contact {
        AbilityEffectResult::power_multiplier(1.3)
    } else {
        AbilityEffectResult::none()
    }
}

fn apply_punk_rock(context: AbilityContext) -> AbilityEffectResult {
    if let Some(move_id) = context.move_id {
        // Check if move is a sound move
        let sound_moves = [
            "boomburst", "bugbuzz", "chatter", "clangoroussoul",
            "disarmingvoice", "echoedvoice", "grasswhistle", "growl",
            "hypervoice", "metalsound", "noiseburst", "overdrive",
            "perishsong", "relicsong", "roar", "round", "screech",
            "sing", "snore", "sparklingaria", "supersonic", "uproar",
        ];
        if sound_moves.contains(&move_id) {
            return AbilityEffectResult::power_multiplier(1.3);
        }
    }
    AbilityEffectResult::none()
}

// Stat multiplier abilities
fn apply_stat_doubler(attack: f32, defense: f32, special_attack: f32, special_defense: f32) -> AbilityEffectResult {
    AbilityEffectResult::stat_multiplier(attack, defense, special_attack, special_defense)
}

fn apply_guts(context: AbilityContext) -> AbilityEffectResult {
    // Check if Pokemon has a status condition
    if let Some(pokemon) = context.state.get_pokemon_at_position(context.user_position) {
        if pokemon.status != crate::core::instructions::PokemonStatus::None {
            return AbilityEffectResult::stat_multiplier(1.5, 1.0, 1.0, 1.0);
        }
    }
    AbilityEffectResult::none()
}

fn apply_marvel_scale(context: AbilityContext) -> AbilityEffectResult {
    // Check if Pokemon has a status condition
    if let Some(pokemon) = context.state.get_pokemon_at_position(context.user_position) {
        if pokemon.status != crate::core::instructions::PokemonStatus::None {
            return AbilityEffectResult::stat_multiplier(1.0, 1.5, 1.0, 1.0);
        }
    }
    AbilityEffectResult::none()
}

fn apply_plus_minus(context: AbilityContext) -> AbilityEffectResult {
    // Plus and Minus boost Special Attack by 50% when an ally has Plus or Minus
    use crate::core::battle_format::BattlePosition;
    
    // Only works in doubles/multi battles where there can be allies
    if context.state.format.active_pokemon_count() <= 1 {
        return AbilityEffectResult::none();
    }
    
    // Get ally position (other slot on same side)
    let ally_slot = 1 - context.user_position.slot;
    let ally_position = BattlePosition::new(context.user_position.side, ally_slot);
    
    // Check if ally position is active and has Plus or Minus
    if context.state.is_position_active(ally_position) {
        if let Some(ally_pokemon) = context.state.get_pokemon_at_position(ally_position) {
            let ally_ability = ally_pokemon.ability.as_str();
            if ally_ability == "plus" || ally_ability == "minus" {
                // Boost Special Attack by 50% (1.5x multiplier)
                return AbilityEffectResult {
                    special_attack_multiplier: 1.5,
                    ..Default::default()
                };
            }
        }
    }
    
    AbilityEffectResult::none()
}

// STAB modifiers
fn apply_adaptability(context: AbilityContext) -> AbilityEffectResult {
    // Check if move type matches user's type for STAB
    // This needs access to Pokemon's types and move type matching
    if let Some(move_type) = context.move_type {
        if let Some(pokemon) = context.state.get_pokemon_at_position(context.user_position) {
            if pokemon.types.contains(&move_type.as_str().to_string()) {
                return AbilityEffectResult::stab_multiplier(2.0);
            }
        }
    }
    AbilityEffectResult::none()
}

// Weather/environment abilities
fn apply_weather_negation() -> AbilityEffectResult {
    AbilityEffectResult::negates_weather()
}

fn apply_infiltrator() -> AbilityEffectResult {
    AbilityEffectResult::bypasses_screens()
}

// Status immunity abilities
fn apply_water_veil(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents burn status - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_magma_armor(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents freeze status - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_immunity_ability(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents poison status - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_limber(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents paralysis status - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_sleep_immunity(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents sleep status - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_own_tempo(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents confusion - handled in status application, not damage
    AbilityEffectResult::none()
}

fn apply_oblivious(_context: AbilityContext) -> AbilityEffectResult {
    // Prevents infatuation and taunt - handled in status application, not damage
    AbilityEffectResult::none()
}

// Type-changing abilities
fn apply_normalize(context: AbilityContext) -> AbilityEffectResult {
    // Changes all moves to Normal type with 1.2x power boost
    // Type change handled elsewhere, power boost here
    AbilityEffectResult::power_multiplier(1.2)
}

fn apply_type_change(context: AbilityContext, target_type: &str, power_multiplier: f32) -> AbilityEffectResult {
    // Changes Normal-type moves to specified type with power boost
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "normal" {
            return AbilityEffectResult::power_multiplier(power_multiplier);
        }
    }
    AbilityEffectResult::none()
}

// Compatibility layer for existing codebase
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityModifier {
    pub damage_multiplier: f32,
    pub power_multiplier: f32,
    pub attack_multiplier: f32,
    pub defense_multiplier: f32,
    pub special_attack_multiplier: f32,
    pub special_defense_multiplier: f32,
    pub blocks_move: bool,
    pub ignores_type_effectiveness: bool,
    pub changed_move_type: Option<String>,
}

impl AbilityModifier {
    pub fn new() -> Self {
        Self {
            damage_multiplier: 1.0,
            power_multiplier: 1.0,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            special_attack_multiplier: 1.0,
            special_defense_multiplier: 1.0,
            blocks_move: false,
            ignores_type_effectiveness: false,
            changed_move_type: None,
        }
    }
}

impl From<AbilityEffectResult> for AbilityModifier {
    fn from(effect: AbilityEffectResult) -> Self {
        Self {
            damage_multiplier: effect.damage_multiplier,
            power_multiplier: effect.power_multiplier,
            attack_multiplier: effect.attack_multiplier,
            defense_multiplier: effect.defense_multiplier,
            special_attack_multiplier: effect.special_attack_multiplier,
            special_defense_multiplier: effect.special_defense_multiplier,
            blocks_move: effect.immunity,
            ignores_type_effectiveness: effect.ignore_type_effectiveness,
            changed_move_type: None, // Type changes handled separately
        }
    }
}

/// Calculate all ability modifiers for a damage calculation (compatibility function)
pub fn calculate_ability_modifiers(
    context: &crate::engine::combat::damage_context::DamageContext,
    state: &BattleState,
    _generation_mechanics: &crate::generation::GenerationMechanics,
) -> AbilityModifier {
    // Use modern context directly
    calculate_ability_modifiers_modern(context, state, _generation_mechanics)
}

/// Modern function that works with the new DamageContext
pub fn calculate_ability_modifiers_modern(
    context: &DamageContext,
    state: &BattleState,
    _generation_mechanics: &crate::generation::GenerationMechanics,
) -> AbilityModifier {
    let mut combined_modifier = AbilityModifier::new();

    // Convert DamageContext to AbilityContext for attacker
    let attacker_context = AbilityContext {
        user_position: context.attacker.position,
        target_position: Some(context.defender.position),
        move_type: Some(TypeId::from(context.move_info.move_type.clone())),
        move_id: Some(&context.move_info.name),
        base_power: Some(context.move_info.base_power as u16),
        is_critical: context.move_info.is_critical,
        is_contact: context.move_info.is_contact,
        state: state,
    };

    // Apply attacker ability
    let attacker_effect = apply_ability_effect(&AbilityId::from(context.attacker.pokemon.ability.as_str()), attacker_context);
    let attacker_mod = AbilityModifier::from(attacker_effect);
    
    if attacker_mod.blocks_move {
        return attacker_mod;
    }

    // Convert DamageContext to AbilityContext for defender
    let defender_context = AbilityContext {
        user_position: context.defender.position,
        target_position: Some(context.attacker.position),
        move_type: Some(TypeId::from(context.move_info.move_type.clone())),
        move_id: Some(&context.move_info.name),
        base_power: Some(context.move_info.base_power as u16),
        is_critical: context.move_info.is_critical,
        is_contact: context.move_info.is_contact,
        state: state,
    };

    // Apply defender ability
    let defender_effect = apply_ability_effect(&AbilityId::from(context.defender.pokemon.ability.as_str()), defender_context);
    let defender_mod = AbilityModifier::from(defender_effect);
    
    if defender_mod.blocks_move {
        return defender_mod;
    }

    // Combine modifiers
    combined_modifier.damage_multiplier = attacker_mod.damage_multiplier * defender_mod.damage_multiplier;
    combined_modifier.power_multiplier = attacker_mod.power_multiplier * defender_mod.power_multiplier;
    combined_modifier.attack_multiplier = attacker_mod.attack_multiplier;
    combined_modifier.defense_multiplier = defender_mod.defense_multiplier;
    combined_modifier.special_attack_multiplier = attacker_mod.special_attack_multiplier;
    combined_modifier.special_defense_multiplier = defender_mod.special_defense_multiplier;

    combined_modifier
}

// Simple helper functions to replace legacy system usage
/// Check if an ability provides immunity to a move type
pub fn ability_provides_immunity(ability_name: &str, move_type: &str) -> bool {
    let ability_id = AbilityId::from(ability_name);
    let context = AbilityContext {
        user_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
        target_position: None,
        move_type: Some(TypeId::from(move_type)),
        move_id: None,
        base_power: None,
        is_critical: false,
        is_contact: false,
        state: &BattleState::default(),
    };
    
    apply_ability_effect(&ability_id, context).immunity
}

/// Check if an ability negates weather
pub fn ability_negates_weather(ability_name: &str) -> bool {
    let ability_id = AbilityId::from(ability_name);
    let context = AbilityContext {
        user_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
        target_position: None,
        move_type: None,
        move_id: None,
        base_power: None,
        is_critical: false,
        is_contact: false,
        state: &BattleState::default(),
    };
    
    apply_ability_effect(&ability_id, context).negates_weather
}

/// Check if an ability bypasses screens
pub fn ability_bypasses_screens(ability_name: &str) -> bool {
    let ability_id = AbilityId::from(ability_name);
    let context = AbilityContext {
        user_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
        target_position: None,
        move_type: None,
        move_id: None,
        base_power: None,
        is_critical: false,
        is_contact: false,
        state: &BattleState::default(),
    };
    
    apply_ability_effect(&ability_id, context).bypasses_screens
}




