//! # Ability System
//! 
//! This module provides the event-driven ability system for damage calculation
//! and other battle effects. Abilities can modify damage, stats, immunities,
//! and many other aspects of battle.

use crate::core::state::{Pokemon, State, MoveCategory};
use crate::data::types::EngineMoveData;
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};

/// Context for damage calculation that abilities can modify
#[derive(Debug, Clone)]
pub struct DamageContext {
    pub attacker: Pokemon,
    pub defender: Pokemon,
    pub move_data: EngineMoveData,
    pub base_power: u8,
    pub is_critical: bool,
    pub move_type: String,
    pub state: State,
}

/// Modifier result from an ability
#[derive(Debug, Clone)]
pub struct AbilityModifier {
    /// Damage multiplier (1.0 = no change)
    pub damage_multiplier: f32,
    /// Base power multiplier (1.0 = no change)
    pub power_multiplier: f32,
    /// Attack stat multiplier (1.0 = no change)
    pub attack_multiplier: f32,
    /// Defense stat multiplier (1.0 = no change)
    pub defense_multiplier: f32,
    /// Special Attack stat multiplier (1.0 = no change)
    pub special_attack_multiplier: f32,
    /// Special Defense stat multiplier (1.0 = no change)
    pub special_defense_multiplier: f32,
    /// Whether this move should be completely blocked (immunity)
    pub blocks_move: bool,
    /// Whether this move should ignore type effectiveness
    pub ignores_type_effectiveness: bool,
}

impl Default for AbilityModifier {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            power_multiplier: 1.0,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            special_attack_multiplier: 1.0,
            special_defense_multiplier: 1.0,
            blocks_move: false,
            ignores_type_effectiveness: false,
        }
    }
}

impl AbilityModifier {
    /// Create a new modifier with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set damage multiplier
    pub fn with_damage_multiplier(mut self, multiplier: f32) -> Self {
        self.damage_multiplier = multiplier;
        self
    }

    /// Set power multiplier
    pub fn with_power_multiplier(mut self, multiplier: f32) -> Self {
        self.power_multiplier = multiplier;
        self
    }

    /// Set attack multiplier
    pub fn with_attack_multiplier(mut self, multiplier: f32) -> Self {
        self.attack_multiplier = multiplier;
        self
    }

    /// Set defense multiplier
    pub fn with_defense_multiplier(mut self, multiplier: f32) -> Self {
        self.defense_multiplier = multiplier;
        self
    }

    /// Set special attack multiplier
    pub fn with_special_attack_multiplier(mut self, multiplier: f32) -> Self {
        self.special_attack_multiplier = multiplier;
        self
    }

    /// Set special defense multiplier
    pub fn with_special_defense_multiplier(mut self, multiplier: f32) -> Self {
        self.special_defense_multiplier = multiplier;
        self
    }

    /// Block the move completely
    pub fn block_move(mut self) -> Self {
        self.blocks_move = true;
        self
    }
}

/// Trait for ability effects that can modify damage calculation
pub trait AbilityEffect {
    /// Get the ability name
    fn name(&self) -> &str;

    /// Modify damage calculation before it happens
    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        AbilityModifier::default()
    }

    /// Check if this ability provides immunity to a move type
    fn provides_immunity(&self, move_type: &str) -> bool {
        false
    }

    /// Check if this ability modifies STAB calculation
    fn modify_stab(&self, context: &DamageContext) -> f32 {
        1.0 // No modification by default
    }

    /// Check if this ability negates weather effects
    fn negates_weather(&self) -> bool {
        false
    }

    /// Check if this ability bypasses screens/barriers
    fn bypasses_screens(&self) -> bool {
        false
    }
}

/// Calculate all ability modifiers for a damage calculation
pub fn calculate_ability_modifiers(
    context: &DamageContext,
    _generation_mechanics: &GenerationMechanics,
) -> AbilityModifier {
    let mut combined_modifier = AbilityModifier::new();

    // Get attacker's ability
    if let Some(attacker_ability) = get_ability_by_name(&context.attacker.ability) {
        let attacker_mod = attacker_ability.modify_damage(context);
        
        // Check for immunity first
        if attacker_mod.blocks_move {
            return attacker_mod;
        }
        
        // Apply attacker ability modifiers
        combined_modifier.damage_multiplier *= attacker_mod.damage_multiplier;
        combined_modifier.power_multiplier *= attacker_mod.power_multiplier;
        combined_modifier.attack_multiplier *= attacker_mod.attack_multiplier;
        combined_modifier.special_attack_multiplier *= attacker_mod.special_attack_multiplier;
    }

    // Get defender's ability
    if let Some(defender_ability) = get_ability_by_name(&context.defender.ability) {
        let defender_mod = defender_ability.modify_damage(context);
        
        // Check for immunity
        if defender_mod.blocks_move {
            return defender_mod;
        }
        
        // Apply defender ability modifiers
        combined_modifier.damage_multiplier *= defender_mod.damage_multiplier;
        combined_modifier.power_multiplier *= defender_mod.power_multiplier;
        combined_modifier.defense_multiplier *= defender_mod.defense_multiplier;
        combined_modifier.special_defense_multiplier *= defender_mod.special_defense_multiplier;
    }

    combined_modifier
}

/// Normalize ability names to match PS conventions (lowercase, no spaces/hyphens)
fn normalize_ability_name(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "")
        .replace("-", "")
        .replace("'", "")
        .replace(".", "")
        .replace(":", "")
}

/// Get an ability by name using normalized lookup
pub fn get_ability_by_name(ability_name: &str) -> Option<Box<dyn AbilityEffect>> {
    match normalize_ability_name(ability_name).as_str() {
        "flashfire" => Some(Box::new(FlashFire)),
        "thickfat" => Some(Box::new(ThickFat)),
        "levitate" => Some(Box::new(Levitate)),
        "waterabsorb" => Some(Box::new(WaterAbsorb)),
        "voltabsorb" => Some(Box::new(VoltAbsorb)),
        "solidrock" => Some(Box::new(SolidRock)),
        "filter" => Some(Box::new(Filter)),
        "tintedlens" => Some(Box::new(TintedLens)),
        "ironfist" => Some(Box::new(IronFist)),
        "technician" => Some(Box::new(Technician)),
        "hugepower" => Some(Box::new(HugePower)),
        "purepower" => Some(Box::new(PurePower)),
        "adaptability" => Some(Box::new(Adaptability)),
        "dryskin" => Some(Box::new(DrySkin)),
        "stormdrain" => Some(Box::new(StormDrain)),
        "lightningrod" => Some(Box::new(LightningRod)),
        "motordrive" => Some(Box::new(MotorDrive)),
        "cloudnine" => Some(Box::new(CloudNine)),
        "airlock" => Some(Box::new(AirLock)),
        "infiltrator" => Some(Box::new(Infiltrator)),
        "neuroforce" => Some(Box::new(Neuroforce)),
        "sheerforce" => Some(Box::new(SheerForce)),
        "strongjaw" => Some(Box::new(StrongJaw)),
        "toughclaws" => Some(Box::new(ToughClaws)),
        "steelworker" => Some(Box::new(Steelworker)),
        "multiscale" => Some(Box::new(Multiscale)),
        "shadowshield" => Some(Box::new(ShadowShield)),
        "prismarmor" => Some(Box::new(PrismArmor)),
        "icescales" => Some(Box::new(IceScales)),
        "fluffy" => Some(Box::new(Fluffy)),
        "reckless" => Some(Box::new(Reckless)),
        "pixilate" => Some(Box::new(Pixilate)),
        "refrigerate" => Some(Box::new(Refrigerate)),
        "aerilate" => Some(Box::new(Aerilate)),
        _ => None,
    }
}

// Core damage-affecting abilities

/// Flash Fire - Fire immunity and 1.5x boost when activated
pub struct FlashFire;

impl AbilityEffect for FlashFire {
    fn name(&self) -> &str {
        "Flash Fire"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "fire"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "fire" {
            // Check if Flash Fire boost is active
            if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::FlashFire) {
                // Flash Fire boost is active - provide 1.5x damage boost and immunity
                AbilityModifier::new()
                    .block_move()
                    .with_damage_multiplier(1.5)
            } else {
                // Flash Fire not yet activated - just provide immunity and activate boost
                // Note: The boost activation would happen in the instruction generator
                AbilityModifier::new().block_move()
            }
        } else {
            // Check if Flash Fire boost is active for non-Fire moves
            if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::FlashFire) {
                // Flash Fire boost affects all Fire-type moves user makes
                if context.move_data.move_type.to_lowercase() == "fire" {
                    AbilityModifier::new().with_damage_multiplier(1.5)
                } else {
                    AbilityModifier::default()
                }
            } else {
                AbilityModifier::default()
            }
        }
    }
}

/// Thick Fat - Fire and Ice moves deal 0.5x damage
pub struct ThickFat;

impl AbilityEffect for ThickFat {
    fn name(&self) -> &str {
        "Thick Fat"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        let move_type = context.move_type.to_lowercase();
        if move_type == "fire" || move_type == "ice" {
            AbilityModifier::new().with_damage_multiplier(0.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Levitate - Ground immunity
pub struct Levitate;

impl AbilityEffect for Levitate {
    fn name(&self) -> &str {
        "Levitate"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "ground"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "ground" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Water Absorb - Water immunity and healing
pub struct WaterAbsorb;

impl AbilityEffect for WaterAbsorb {
    fn name(&self) -> &str {
        "Water Absorb"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "water"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "water" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Volt Absorb - Electric immunity and healing
pub struct VoltAbsorb;

impl AbilityEffect for VoltAbsorb {
    fn name(&self) -> &str {
        "Volt Absorb"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "electric"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "electric" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Solid Rock - Super effective moves deal 0.75x damage
pub struct SolidRock;

impl AbilityEffect for SolidRock {
    fn name(&self) -> &str {
        "Solid Rock"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is super effective against defender
        let type_chart = TypeChart::new(context.state.get_generation().number());
        let move_type = PokemonType::from_str(&context.move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&context.defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if context.defender.types.len() > 1 {
            PokemonType::from_str(&context.defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            move_type,
            (defender_type1, defender_type2),
            None,
            Some(&context.move_data.name.to_lowercase()),
        );

        if type_effectiveness > 1.0 {
            // Super effective moves: 25% damage reduction (0.75x multiplier)
            AbilityModifier::new().with_damage_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Filter - Super effective moves deal 0.75x damage (same as Solid Rock)
pub struct Filter;

impl AbilityEffect for Filter {
    fn name(&self) -> &str {
        "Filter"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is super effective against defender
        let type_chart = TypeChart::new(context.state.get_generation().number());
        let move_type = PokemonType::from_str(&context.move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&context.defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if context.defender.types.len() > 1 {
            PokemonType::from_str(&context.defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            move_type,
            (defender_type1, defender_type2),
            None,
            Some(&context.move_data.name.to_lowercase()),
        );

        if type_effectiveness > 1.0 {
            // Super effective moves: 25% damage reduction (0.75x multiplier)
            AbilityModifier::new().with_damage_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Tinted Lens - Not very effective moves deal 2.0x damage
pub struct TintedLens;

impl AbilityEffect for TintedLens {
    fn name(&self) -> &str {
        "Tinted Lens"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is not very effective against defender
        let type_chart = TypeChart::new(context.state.get_generation().number());
        let move_type = PokemonType::from_str(&context.move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&context.defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if context.defender.types.len() > 1 {
            PokemonType::from_str(&context.defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            move_type,
            (defender_type1, defender_type2),
            None,
            Some(&context.move_data.name.to_lowercase()),
        );

        if type_effectiveness < 1.0 {
            // Not very effective moves: 2.0x damage boost
            AbilityModifier::new().with_damage_multiplier(2.0)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Iron Fist - Punch moves deal 1.2x damage
pub struct IronFist;

impl AbilityEffect for IronFist {
    fn name(&self) -> &str {
        "Iron Fist"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has punch flag
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "punch") {
            AbilityModifier::new().with_power_multiplier(1.2)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Technician - Moves with 60 or less base power deal 1.5x damage
pub struct Technician;

impl AbilityEffect for Technician {
    fn name(&self) -> &str {
        "Technician"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.base_power <= 60 && context.base_power > 0 {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Huge Power - Attack stat is doubled
pub struct HugePower;

impl AbilityEffect for HugePower {
    fn name(&self) -> &str {
        "Huge Power"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_attack_multiplier(2.0)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Pure Power - Attack stat is doubled (same as Huge Power)
pub struct PurePower;

impl AbilityEffect for PurePower {
    fn name(&self) -> &str {
        "Pure Power"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_attack_multiplier(2.0)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Adaptability - STAB is 2.0x instead of 1.5x
pub struct Adaptability;

impl AbilityEffect for Adaptability {
    fn name(&self) -> &str {
        "Adaptability"
    }

    fn modify_stab(&self, _context: &DamageContext) -> f32 {
        2.0 / 1.5 // Convert 1.5x STAB to 2.0x STAB
    }
}

/// Dry Skin - Fire moves deal 1.25x damage, Water immunity and healing
pub struct DrySkin;

impl AbilityEffect for DrySkin {
    fn name(&self) -> &str {
        "Dry Skin"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "water"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        match context.move_type.to_lowercase().as_str() {
            "water" => AbilityModifier::new().block_move(),
            "fire" => AbilityModifier::new().with_damage_multiplier(1.25),
            _ => AbilityModifier::default(),
        }
    }
}

/// Storm Drain - Water immunity and Special Attack boost
pub struct StormDrain;

impl AbilityEffect for StormDrain {
    fn name(&self) -> &str {
        "Storm Drain"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "water"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "water" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Lightning Rod - Electric immunity and Special Attack boost
pub struct LightningRod;

impl AbilityEffect for LightningRod {
    fn name(&self) -> &str {
        "Lightning Rod"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "electric"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "electric" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Motor Drive - Electric immunity and Speed boost
pub struct MotorDrive;

impl AbilityEffect for MotorDrive {
    fn name(&self) -> &str {
        "Motor Drive"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type.to_lowercase() == "electric"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "electric" {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Cloud Nine - Negates all weather effects
pub struct CloudNine;

impl AbilityEffect for CloudNine {
    fn name(&self) -> &str {
        "Cloud Nine"
    }

    fn negates_weather(&self) -> bool {
        true
    }
}

/// Air Lock - Negates all weather effects (same as Cloud Nine)
pub struct AirLock;

impl AbilityEffect for AirLock {
    fn name(&self) -> &str {
        "Air Lock"
    }

    fn negates_weather(&self) -> bool {
        true
    }
}

/// Infiltrator - Bypasses screens and barriers
pub struct Infiltrator;

impl AbilityEffect for Infiltrator {
    fn name(&self) -> &str {
        "Infiltrator"
    }

    fn bypasses_screens(&self) -> bool {
        true
    }
}

/// Neuroforce - Super effective moves deal 1.25x damage
pub struct Neuroforce;

impl AbilityEffect for Neuroforce {
    fn name(&self) -> &str {
        "Neuroforce"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is super effective against defender
        let type_chart = TypeChart::new(context.state.get_generation().number());
        let move_type = PokemonType::from_str(&context.move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&context.defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if context.defender.types.len() > 1 {
            PokemonType::from_str(&context.defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            move_type,
            (defender_type1, defender_type2),
            None,
            Some(&context.move_data.name.to_lowercase()),
        );

        if type_effectiveness > 1.0 {
            // Super effective moves: 25% damage boost (1.25x multiplier)
            AbilityModifier::new().with_damage_multiplier(1.25)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Sheer Force - Moves with secondary effects deal 1.3x damage (effects removed)
pub struct SheerForce;

impl AbilityEffect for SheerForce {
    fn name(&self) -> &str {
        "Sheer Force"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has secondary effects
        // For now, we'll use a simplified check - if effect_chance is Some, it has secondary effects
        if context.move_data.effect_chance.is_some() && context.move_data.effect_chance.unwrap() > 0 {
            // 30% damage boost for moves with secondary effects
            AbilityModifier::new().with_power_multiplier(1.3)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Strong Jaw - Bite moves deal 1.5x damage
pub struct StrongJaw;

impl AbilityEffect for StrongJaw {
    fn name(&self) -> &str {
        "Strong Jaw"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has bite flag
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "bite") {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Tough Claws - Contact moves deal 1.3x damage
pub struct ToughClaws;

impl AbilityEffect for ToughClaws {
    fn name(&self) -> &str {
        "Tough Claws"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has contact flag
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "contact") {
            AbilityModifier::new().with_power_multiplier(1.3)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Steelworker - Steel moves deal 1.5x damage
pub struct Steelworker;

impl AbilityEffect for Steelworker {
    fn name(&self) -> &str {
        "Steelworker"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "steel" {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Multiscale - Damage reduced by 50% when at full HP
pub struct Multiscale;

impl AbilityEffect for Multiscale {
    fn name(&self) -> &str {
        "Multiscale"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.defender.hp == context.defender.max_hp {
            // 50% damage reduction when at full HP
            AbilityModifier::new().with_damage_multiplier(0.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Shadow Shield - Damage reduced by 50% when at full HP (Necrozma's Multiscale)
pub struct ShadowShield;

impl AbilityEffect for ShadowShield {
    fn name(&self) -> &str {
        "Shadow Shield"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.defender.hp == context.defender.max_hp {
            // 50% damage reduction when at full HP
            AbilityModifier::new().with_damage_multiplier(0.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Prism Armor - Super effective moves deal 0.75x damage (Necrozma's Filter)
pub struct PrismArmor;

impl AbilityEffect for PrismArmor {
    fn name(&self) -> &str {
        "Prism Armor"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is super effective against defender
        let type_chart = TypeChart::new(context.state.get_generation().number());
        let move_type = PokemonType::from_str(&context.move_type).unwrap_or(PokemonType::Normal);
        
        let defender_type1 = PokemonType::from_str(&context.defender.types[0]).unwrap_or(PokemonType::Normal);
        let defender_type2 = if context.defender.types.len() > 1 {
            PokemonType::from_str(&context.defender.types[1]).unwrap_or(defender_type1)
        } else {
            defender_type1
        };

        let type_effectiveness = type_chart.calculate_damage_multiplier(
            move_type,
            (defender_type1, defender_type2),
            None,
            Some(&context.move_data.name.to_lowercase()),
        );

        if type_effectiveness > 1.0 {
            // Super effective moves: 25% damage reduction (0.75x multiplier)
            AbilityModifier::new().with_damage_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Ice Scales - Special moves deal 0.5x damage
pub struct IceScales;

impl AbilityEffect for IceScales {
    fn name(&self) -> &str {
        "Ice Scales"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Special {
            // 50% damage reduction for special moves
            AbilityModifier::new().with_damage_multiplier(0.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Fluffy - Contact moves deal 0.5x damage, Fire moves deal 2.0x damage
pub struct Fluffy;

impl AbilityEffect for Fluffy {
    fn name(&self) -> &str {
        "Fluffy"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        let mut modifier = AbilityModifier::new();
        
        // Check for contact moves (reduced damage)
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "contact") {
            modifier.damage_multiplier *= 0.5;
        }
        
        // Check for Fire moves (increased damage)
        if context.move_type.to_lowercase() == "fire" {
            modifier.damage_multiplier *= 2.0;
        }
        
        modifier
    }
}

/// Reckless - Recoil/crash moves deal 1.2x damage
pub struct Reckless;

impl AbilityEffect for Reckless {
    fn name(&self) -> &str {
        "Reckless"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has recoil or crash damage
        // For now, we'll check move flags or specific moves
        if context.move_data.flags.iter().any(|flag| {
            let flag_lower = flag.to_lowercase();
            flag_lower == "recoil" || flag_lower == "crash"
        }) {
            AbilityModifier::new().with_power_multiplier(1.2)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Pixilate - Normal moves become Fairy type with generation-specific power boost
pub struct Pixilate;

impl AbilityEffect for Pixilate {
    fn name(&self) -> &str {
        "Pixilate"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "normal" {
            // Generation-specific multiplier: 1.3x in Gen 6, 1.2x in Gen 7+
            let multiplier = if context.state.get_generation().number() <= 6 {
                1.3 // Gen 6 and earlier
            } else {
                1.2 // Gen 7 and later
            };
            
            // Note: Type change would need to be handled in damage calculation
            // For now, just apply the power multiplier
            AbilityModifier::new().with_power_multiplier(multiplier)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Refrigerate - Normal moves become Ice type with generation-specific power boost
pub struct Refrigerate;

impl AbilityEffect for Refrigerate {
    fn name(&self) -> &str {
        "Refrigerate"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "normal" {
            // Generation-specific multiplier: 1.3x in Gen 6, 1.2x in Gen 7+
            let multiplier = if context.state.get_generation().number() <= 6 {
                1.3 // Gen 6 and earlier
            } else {
                1.2 // Gen 7 and later
            };
            
            // Note: Type change would need to be handled in damage calculation
            // For now, just apply the power multiplier
            AbilityModifier::new().with_power_multiplier(multiplier)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Aerilate - Normal moves become Flying type with generation-specific power boost
pub struct Aerilate;

impl AbilityEffect for Aerilate {
    fn name(&self) -> &str {
        "Aerilate"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "normal" {
            // Generation-specific multiplier: 1.3x in Gen 6, 1.2x in Gen 7+
            let multiplier = if context.state.get_generation().number() <= 6 {
                1.3 // Gen 6 and earlier
            } else {
                1.2 // Gen 7 and later
            };
            
            // Note: Type change would need to be handled in damage calculation
            // For now, just apply the power multiplier
            AbilityModifier::new().with_power_multiplier(multiplier)
        } else {
            AbilityModifier::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{Pokemon, State, MoveCategory};
    use crate::data::types::EngineMoveData;

    fn create_test_context() -> DamageContext {
        let attacker = Pokemon::new("Attacker".to_string());
        let defender = Pokemon::new("Defender".to_string());
        let move_data = EngineMoveData {
            id: 1,
            name: "Test Move".to_string(),
            base_power: Some(80),
            accuracy: Some(100),
            pp: 10,
            move_type: "Fire".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::ps_types::PSMoveTarget::Normal,
            effect_chance: None,
            effect_description: String::new(),
            flags: vec![],
        };
        let state = State::new(crate::core::battle_format::BattleFormat::gen9_ou());

        DamageContext {
            attacker,
            defender,
            move_data,
            base_power: 80,
            is_critical: false,
            move_type: "Fire".to_string(),
            state,
        }
    }

    #[test]
    fn test_thick_fat_reduces_fire_damage() {
        let mut context = create_test_context();
        context.defender.ability = "Thick Fat".to_string();
        context.move_type = "Fire".to_string();

        let thick_fat = ThickFat;
        let modifier = thick_fat.modify_damage(&context);

        assert_eq!(modifier.damage_multiplier, 0.5);
    }

    #[test]
    fn test_thick_fat_reduces_ice_damage() {
        let mut context = create_test_context();
        context.defender.ability = "Thick Fat".to_string();
        context.move_type = "Ice".to_string();

        let thick_fat = ThickFat;
        let modifier = thick_fat.modify_damage(&context);

        assert_eq!(modifier.damage_multiplier, 0.5);
    }

    #[test]
    fn test_levitate_blocks_ground_moves() {
        let mut context = create_test_context();
        context.defender.ability = "Levitate".to_string();
        context.move_type = "Ground".to_string();

        let levitate = Levitate;
        let modifier = levitate.modify_damage(&context);

        assert!(modifier.blocks_move);
    }

    #[test]
    fn test_water_absorb_blocks_water_moves() {
        let mut context = create_test_context();
        context.defender.ability = "Water Absorb".to_string();
        context.move_type = "Water".to_string();

        let water_absorb = WaterAbsorb;
        let modifier = water_absorb.modify_damage(&context);

        assert!(modifier.blocks_move);
    }
}