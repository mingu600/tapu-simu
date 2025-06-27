use crate::types::identifiers::{AbilityId, TypeId};
use crate::types::PokemonType;
use crate::core::battle_state::Pokemon;
use crate::core::battle_state::BattleState;
use crate::core::battle_format::BattlePosition;
use crate::engine::combat::damage_context::DamageContext;


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

        // Type immunity bypass
        "mindseye" | "mind's eye" => apply_minds_eye(context),

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
    use crate::engine::combat::type_effectiveness::TypeChart;
    use crate::types::PokemonType;
    
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
    
    // Get defender types (already PokemonType)
    let defender_type1 = defender.types[0];
    let defender_type2 = if defender.types.len() > 1 {
        defender.types[1]
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_normalized_str(&move_type_str)
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
    use crate::engine::combat::type_effectiveness::TypeChart;
    use crate::types::PokemonType;
    
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
    
    // Get defender types (already PokemonType)
    let defender_type1 = defender.types[0];
    let defender_type2 = if defender.types.len() > 1 {
        defender.types[1]
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_normalized_str(&move_type_str)
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
    use crate::engine::combat::type_effectiveness::TypeChart;
    use crate::types::PokemonType;
    
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
    
    // Get defender types (already PokemonType)
    let defender_type1 = defender.types[0];
    let defender_type2 = if defender.types.len() > 1 {
        defender.types[1]
    } else {
        defender_type1
    };
    
    // Convert TypeId to string and then to PokemonType
    let move_type_str = format!("{:?}", move_type_id).to_lowercase();
    let move_type = PokemonType::from_normalized_str(&move_type_str)
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
    
    // Check all ally positions for Plus or Minus ability
    let same_side_positions = context.user_position.same_side_positions(&context.state.format);
    for ally_position in same_side_positions {
        // Skip self
        if ally_position == context.user_position {
            continue;
        }
        
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
    }
    
    AbilityEffectResult::none()
}

// STAB modifiers
fn apply_adaptability(context: AbilityContext) -> AbilityEffectResult {
    // Check if move type matches user's type for STAB
    // This needs access to Pokemon's types and move type matching
    if let Some(move_type) = context.move_type {
        if let Some(pokemon) = context.state.get_pokemon_at_position(context.user_position) {
            let move_pokemon_type = PokemonType::from(&move_type);
            if pokemon.types.contains(&move_pokemon_type) {
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

fn apply_minds_eye(context: AbilityContext) -> AbilityEffectResult {
    // Mind's Eye allows Normal and Fighting type moves to hit Ghost types
    if let Some(move_type) = context.move_type {
        if move_type.as_str() == "normal" || move_type.as_str() == "fighting" {
            return AbilityEffectResult {
                ignore_type_effectiveness: true,
                ..Default::default()
            };
        }
    }
    AbilityEffectResult::none()
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






