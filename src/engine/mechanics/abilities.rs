//! # Ability System
//! 
//! This module provides the event-driven ability system for damage calculation
//! and other battle effects. Abilities can modify damage, stats, immunities,
//! and many other aspects of battle.

use crate::core::state::{Pokemon, State, MoveCategory};
use crate::core::battle_format::BattlePosition;
use crate::core::instruction::{StateInstructions, Instruction, BoostStatsInstruction, Stat, ChangeAbilityInstruction, ChangeTypeInstruction, PositionDamageInstruction, ChangeWeatherInstruction, Weather, ChangeTerrainInstruction, Terrain, ApplySideConditionInstruction, SideCondition, ApplyVolatileStatusInstruction, VolatileStatus, ChangeItemInstruction, ApplyStatusInstruction};
use crate::data::types::EngineMoveData;
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};
use std::collections::HashMap;

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
    /// Optional type change for the move
    pub changed_move_type: Option<String>,
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
            changed_move_type: None,
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

    /// Set the move type change
    pub fn with_move_type_change(mut self, new_type: String) -> Self {
        self.changed_move_type = Some(new_type);
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
        
        // Copy type change from attacker ability (only attacker abilities can change move type)
        if attacker_mod.changed_move_type.is_some() {
            combined_modifier.changed_move_type = attacker_mod.changed_move_type;
        }
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

/// Process before-move abilities that trigger when a Pokemon is about to use a move
pub fn process_before_move_abilities(
    state: &State,
    user_position: BattlePosition,
    move_data: &crate::core::state::Move,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Get the user Pokemon
    let user_side = state.get_side(user_position.side);
    let user_pokemon = match user_side.get_active_pokemon_at_slot(user_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    // Process user's before-move abilities
    match user_pokemon.ability.to_lowercase().replace(" ", "").as_str() {
        "protean" => {
            instructions.extend(apply_protean_type_change(state, user_position, move_data, generation));
        }
        "libero" => {
            instructions.extend(apply_protean_type_change(state, user_position, move_data, generation));
        }
        _ => {}
    }
    
    instructions
}

/// Process after-damage-hit abilities that trigger when damage is dealt
pub fn process_after_damage_abilities(
    state: &State,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
    damage_dealt: i16,
    was_ko: bool,
    was_contact: bool,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Get attacker and defender Pokemon
    let attacker_side = state.get_side(attacker_position.side);
    let defender_side = state.get_side(defender_position.side);
    
    let attacker = match attacker_side.get_active_pokemon_at_slot(attacker_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    let defender = match defender_side.get_active_pokemon_at_slot(defender_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    // Process attacker's after-damage abilities (KO abilities like Moxie, Beast Boost)
    if damage_dealt > 0 && attacker.hp > 0 {
        match attacker.ability.to_lowercase().replace(" ", "").as_str() {
            "moxie" | "chillingneigh" | "asoneglastrier" => {
                if was_ko {
                    instructions.extend(apply_attack_boost_on_ko(state, attacker_position, generation));
                }
            }
            "grimneigh" | "asonespectrier" => {
                if was_ko {
                    instructions.extend(apply_special_attack_boost_on_ko(state, attacker_position, generation));
                }
            }
            "beastboost" => {
                if was_ko {
                    instructions.extend(apply_beast_boost_on_ko(state, attacker_position, generation, attacker));
                }
            }
            "battlebond" => {
                if was_ko {
                    instructions.extend(apply_battle_bond_on_ko(state, attacker_position, generation));
                }
            }
            "magician" => {
                if damage_dealt > 0 && !was_ko {
                    instructions.extend(apply_item_steal(state, attacker_position, defender_position, generation, "magician"));
                }
            }
            "pickpocket" => {
                if was_contact {
                    instructions.extend(apply_item_steal(state, defender_position, attacker_position, generation, "pickpocket"));
                }
            }
            _ => {}
        }
    }
    
    // Process defender's after-damage abilities (triggered when hit)
    if damage_dealt > 0 {
        match defender.ability.to_lowercase().replace(" ", "").as_str() {
            "mummy" | "lingeringaroma" | "wanderingspirit" => {
                if was_contact && attacker.hp > 0 {
                    instructions.extend(apply_ability_change_on_contact(state, attacker_position, defender_position, generation, &defender.ability));
                }
            }
            "colorchange" => {
                instructions.extend(apply_color_change(state, defender_position, generation, attacker));
            }
            "stamina" => {
                instructions.extend(apply_defense_boost_on_hit(state, defender_position, generation));
            }
            "cottondown" => {
                if was_contact {
                    instructions.extend(apply_speed_drop_on_contact(state, attacker_position, generation));
                }
            }
            "sandspit" => {
                instructions.extend(apply_weather_change_on_hit(state, generation, crate::core::instruction::Weather::Sand));
            }
            "seedsower" => {
                instructions.extend(apply_terrain_change_on_hit(state, generation, crate::core::instruction::Terrain::GrassyTerrain));
            }
            "toxicdebris" => {
                // Only triggers on physical moves
                if was_contact {
                    instructions.extend(apply_toxic_spikes_on_physical_hit(state, attacker_position.side.opposite(), generation));
                }
            }
            "berserk" => {
                // Boost Special Attack when HP drops below 50%
                if defender.hp <= defender.max_hp / 2 && (defender.hp + damage_dealt) > defender.max_hp / 2 {
                    instructions.extend(apply_special_attack_boost_on_hit(state, defender_position, generation));
                }
            }
            "roughskin" | "ironbarbs" => {
                if was_contact && attacker.hp > 0 {
                    instructions.extend(apply_contact_damage(state, attacker_position, generation, attacker.max_hp / 8));
                }
            }
            "aftermath" => {
                if was_ko && was_contact && attacker.hp > 0 {
                    instructions.extend(apply_contact_damage(state, attacker_position, generation, attacker.max_hp / 4));
                }
            }
            "innardsout" => {
                if was_ko && attacker.hp > 0 {
                    instructions.extend(apply_contact_damage(state, attacker_position, generation, damage_dealt));
                }
            }
            "perishbody" => {
                if was_contact {
                    instructions.extend(apply_perish_song_to_both(state, attacker_position, defender_position, generation));
                }
            }
            "poisonpoint" => {
                if was_contact && attacker.hp > 0 {
                    instructions.extend(apply_status_on_contact(state, attacker_position, generation, crate::core::instruction::PokemonStatus::Poison, 33.0));
                }
            }
            "static" => {
                if was_contact && attacker.hp > 0 {
                    instructions.extend(apply_status_on_contact(state, attacker_position, generation, crate::core::instruction::PokemonStatus::Paralysis, 30.0));
                }
            }
            "flamebody" => {
                if was_contact && attacker.hp > 0 {
                    instructions.extend(apply_status_on_contact(state, attacker_position, generation, crate::core::instruction::PokemonStatus::Burn, 30.0));
                }
            }
            "effectspore" => {
                if was_contact && attacker.hp > 0 {
                    // Effect Spore has 9% chance each for poison, paralysis, and sleep (27% total)
                    instructions.extend(apply_effect_spore_on_contact(state, attacker_position, generation));
                }
            }
            _ => {}
        }
    }
    
    instructions
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
        "tabletsofruin" => Some(Box::new(TabletsOfRuin)),
        "swordofruin" => Some(Box::new(SwordOfRuin)),
        "vesselofruin" => Some(Box::new(VesselOfRuin)),
        "beadsofruin" => Some(Box::new(BeadsOfRuin)),
        "torrent" => Some(Box::new(Torrent)),
        "serenegrace" => Some(Box::new(SereneGrace)),
        "compoundeyes" => Some(Box::new(CompoundEyes)),
        "stench" => Some(Box::new(Stench)),
        "swarm" => Some(Box::new(Swarm)),
        "blaze" => Some(Box::new(Blaze)),
        "overgrow" => Some(Box::new(Overgrow)),
        "hustle" => Some(Box::new(Hustle)),
        "guts" => Some(Box::new(Guts)),
        "soundproof" => Some(Box::new(Soundproof)),
        "poisonpoint" => Some(Box::new(PoisonPoint)),
        "marvelscale" => Some(Box::new(MarvelScale)),
        "effectspore" => Some(Box::new(EffectSpore)),
        "flamebody" => Some(Box::new(FlameBody)),
        "suctioncups" => Some(Box::new(SuctionCups)),
        "wonderguard" => Some(Box::new(WonderGuard)),
        "static" => Some(Box::new(Static)),
        "liquidooze" => Some(Box::new(LiquidOoze)),
        "shielddust" => Some(Box::new(ShieldDust)),
        "damp" => Some(Box::new(Damp)),
        "scrappy" => Some(Box::new(Scrappy)),
        "mindseye" => Some(Box::new(MindEye)),
        "unaware" => Some(Box::new(Unaware)),
        "moldbreaker" => Some(Box::new(Moldbreaker)),
        "purifyingsalt" => Some(Box::new(PurifyingSalt)),
        "comatose" => Some(Box::new(Comatose)),
        "leafguard" => Some(Box::new(LeafGuard)),
        "waterveil" => Some(Box::new(WaterVeil)),
        "waterbubble" => Some(Box::new(WaterBubble)),
        "thermalexchange" => Some(Box::new(ThermalExchange)),
        "magmaarmor" => Some(Box::new(MagmaArmor)),
        "insomnia" => Some(Box::new(Insomnia)),
        "sweetveil" => Some(Box::new(SweetVeil)),
        "vitalspirit" => Some(Box::new(VitalSpirit)),
        "limber" => Some(Box::new(Limber)),
        "immunity" => Some(Box::new(Immunity)),
        "pastelveil" => Some(Box::new(PastelVeil)),
        "sturdy" => Some(Box::new(Sturdy)),
        "pressure" => Some(Box::new(Pressure)),
        "contrary" => Some(Box::new(Contrary)),
        "corrosion" => Some(Box::new(Corrosion)),
        "magicguard" => Some(Box::new(MagicGuard)),
        "neutralizinggas" => Some(Box::new(NeutralizingGas)),
        // After-damage abilities (KO abilities)
        "moxie" => Some(Box::new(Moxie)),
        "chillingneigh" => Some(Box::new(ChillingNeigh)),
        "asoneglastrier" => Some(Box::new(AsOneGlastrier)),
        "grimneigh" => Some(Box::new(GrimNeigh)),
        "asonespectrier" => Some(Box::new(AsOneSpectrier)),
        "beastboost" => Some(Box::new(BeastBoost)),
        "battlebond" => Some(Box::new(BattleBond)),
        "magician" => Some(Box::new(Magician)),
        "pickpocket" => Some(Box::new(Pickpocket)),
        // After-damage abilities (contact abilities)
        "mummy" => Some(Box::new(Mummy)),
        "lingeringaroma" => Some(Box::new(LingeringAroma)),
        "wanderingspirit" => Some(Box::new(WanderingSpirit)),
        "colorchange" => Some(Box::new(ColorChange)),
        "stamina" => Some(Box::new(Stamina)),
        "cottondown" => Some(Box::new(CottonDown)),
        "sandspit" => Some(Box::new(SandSpit)),
        "seedsower" => Some(Box::new(SeedSower)),
        "toxicdebris" => Some(Box::new(ToxicDebris)),
        "berserk" => Some(Box::new(Berserk)),
        "roughskin" => Some(Box::new(RoughSkin)),
        "ironbarbs" => Some(Box::new(IronBarbs)),
        "aftermath" => Some(Box::new(Aftermath)),
        "innardsout" => Some(Box::new(InnardsOut)),
        "perishbody" => Some(Box::new(PerishBody)),
        // Attack modification abilities
        "protean" => Some(Box::new(Protean)),
        "libero" => Some(Box::new(Libero)),
        "gorillatactics" => Some(Box::new(GorillaTactics)),
        "prankster" => Some(Box::new(Prankster)),
        // Critical missing abilities for 100% parity
        "disguise" => Some(Box::new(Disguise)),
        "gulpmissile" => Some(Box::new(GulpMissile)),
        "schooling" => Some(Box::new(Schooling)),
        "shieldsdown" => Some(Box::new(ShieldsDown)),
        "normalize" => Some(Box::new(Normalize)),
        "liquidvoice" => Some(Box::new(LiquidVoice)),
        "galvanize" => Some(Box::new(Galvanize)),
        "megalauncher" => Some(Box::new(MegaLauncher)),
        "punkrock" => Some(Box::new(PunkRock)),
        "mirrorarmor" => Some(Box::new(MirrorArmor)),
        "bulletproof" => Some(Box::new(Bulletproof)),
        "overcoat" => Some(Box::new(Overcoat)),
        "goodasgold" => Some(Box::new(GoodAsGold)),
        "primordialsea" => Some(Box::new(PrimordialSea)), 
        "desolateland" => Some(Box::new(DesolateLand)),
        "orichalcumpulse" => Some(Box::new(OrichalcumPulse)),
        "hadronengine" => Some(Box::new(HadronEngine)),
        // Gen 9 Legendary Abilities
        "intrepidsword" => Some(Box::new(IntrepidSword)),
        "dauntlessshield" => Some(Box::new(DauntlessShield)),
        // Paradox Pokemon Abilities
        "protosynthesis" => Some(Box::new(Protosynthesis)),
        "quarkdrive" => Some(Box::new(QuarkDrive)),
        // Ogerpon Abilities
        "embodyaspect" => Some(Box::new(EmbodyAspect)),
        // Utility Abilities
        "screencleaner" => Some(Box::new(ScreenCleaner)),
        "slowstart" => Some(Box::new(SlowStart)),
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

/// Apply type change for Protean/Libero abilities
fn apply_protean_type_change(
    state: &State,
    user_position: BattlePosition,
    move_data: &crate::core::state::Move,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Get the user Pokemon to check current types
    let user_side = state.get_side(user_position.side);
    let user_pokemon = match user_side.get_active_pokemon_at_slot(user_position.slot) {
        Some(pokemon) => pokemon,
        None => return vec![],
    };
    
    // Get the move type
    let move_type = &move_data.move_type;
    
    // Check if Pokemon already has this type
    if user_pokemon.types.contains(move_type) {
        return vec![]; // No change needed
    }
    
    // For Generation 9+, Protean/Libero only works once per switch-in
    if generation.generation as u8 >= 9 {
        // Check if Pokemon has already used type-changing ability this switch-in
        if user_pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::TypeChange) {
            return vec![]; // Already used this switch-in
        }
    }
    
    // Change to the move's type
    let mut new_types = vec![move_type.clone()];
    if user_pokemon.types.len() > 1 && !user_pokemon.types[1].is_empty() {
        // Keep secondary type if it's different from the new type
        if &user_pokemon.types[1] != move_type {
            new_types.push(user_pokemon.types[1].clone());
        }
    }
    
    let mut instructions = vec![
        Instruction::ChangeType(ChangeTypeInstruction {
            target_position: user_position,
            new_types: new_types.clone(),
            previous_types: Some(user_pokemon.types.clone()),
        })
    ];
    
    // For Gen 9+, mark that type change has been used
    if generation.generation as u8 >= 9 {
        instructions.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: user_position,
            volatile_status: crate::core::instruction::VolatileStatus::TypeChange,
            duration: None, // Permanent until switch out
        }));
    }
    
    vec![StateInstructions::new(100.0, instructions)]
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
            
            // Change Normal moves to Fairy type and apply power boost
            AbilityModifier::new()
                .with_power_multiplier(multiplier)
                .with_move_type_change("fairy".to_string())
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
            
            // Change Normal moves to Ice type and apply power boost
            AbilityModifier::new()
                .with_power_multiplier(multiplier)
                .with_move_type_change("ice".to_string())
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
            
            // Change Normal moves to Flying type and apply power boost
            AbilityModifier::new()
                .with_power_multiplier(multiplier)
                .with_move_type_change("flying".to_string())
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// DEFENSIVE ABILITIES (TABLETS/SWORD/VESSEL/BEADS OF RUIN SERIES)
// =============================================================================

/// Tablets of Ruin - Reduces physical move power by 25%
pub struct TabletsOfRuin;

impl AbilityEffect for TabletsOfRuin {
    fn name(&self) -> &str {
        "Tablets of Ruin"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_power_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Sword of Ruin - Reduces physical move power by 25%
pub struct SwordOfRuin;

impl AbilityEffect for SwordOfRuin {
    fn name(&self) -> &str {
        "Sword of Ruin"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_power_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Vessel of Ruin - Reduces special move power by 25%
pub struct VesselOfRuin;

impl AbilityEffect for VesselOfRuin {
    fn name(&self) -> &str {
        "Vessel of Ruin"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Special {
            AbilityModifier::new().with_power_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Beads of Ruin - Reduces special move power by 25%
pub struct BeadsOfRuin;

impl AbilityEffect for BeadsOfRuin {
    fn name(&self) -> &str {
        "Beads of Ruin"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Special {
            AbilityModifier::new().with_power_multiplier(0.75)
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// TYPE-BASED BOOSTING ABILITIES
// =============================================================================

/// Torrent - Boosts Water moves by 1.5x when at low HP
pub struct Torrent;

impl AbilityEffect for Torrent {
    fn name(&self) -> &str {
        "Torrent"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "water" && context.attacker.hp <= context.attacker.max_hp / 3 {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Swarm - Boosts Bug moves by 1.5x when at low HP
pub struct Swarm;

impl AbilityEffect for Swarm {
    fn name(&self) -> &str {
        "Swarm"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "bug" && context.attacker.hp <= context.attacker.max_hp / 3 {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Blaze - Boosts Fire moves by 1.5x when at low HP
pub struct Blaze;

impl AbilityEffect for Blaze {
    fn name(&self) -> &str {
        "Blaze"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "fire" && context.attacker.hp <= context.attacker.max_hp / 3 {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Overgrow - Boosts Grass moves by 1.5x when at low HP
pub struct Overgrow;

impl AbilityEffect for Overgrow {
    fn name(&self) -> &str {
        "Overgrow"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "grass" && context.attacker.hp <= context.attacker.max_hp / 3 {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// ACCURACY AND EFFECT CHANCE ABILITIES
// =============================================================================

/// Serene Grace - Doubles secondary effect chances
pub struct SereneGrace;

impl AbilityEffect for SereneGrace {
    fn name(&self) -> &str {
        "Serene Grace"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Secondary effect doubling would need to be handled in move effect processing
        AbilityModifier::default()
    }
}

/// Compound Eyes - Increases accuracy by 1.3x
pub struct CompoundEyes;

impl AbilityEffect for CompoundEyes {
    fn name(&self) -> &str {
        "Compound Eyes"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Accuracy boost would need to be handled in accuracy calculation
        AbilityModifier::default()
    }
}

/// Stench - Adds 10% flinch chance to moves
pub struct Stench;

impl AbilityEffect for Stench {
    fn name(&self) -> &str {
        "Stench"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Flinch chance would need to be handled in move effect processing
        AbilityModifier::default()
    }
}

// =============================================================================
// STAT MODIFICATION ABILITIES
// =============================================================================

/// Hustle - Boosts physical move power by 1.5x, reduces accuracy to 80%
pub struct Hustle;

impl AbilityEffect for Hustle {
    fn name(&self) -> &str {
        "Hustle"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Guts - Boosts move power by 1.5x when statused (with Burn interaction)
pub struct Guts;

impl AbilityEffect for Guts {
    fn name(&self) -> &str {
        "Guts"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        use crate::core::instruction::PokemonStatus;
        
        if context.attacker.status != PokemonStatus::None && context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Marvel Scale - Reduces physical damage by 1.5x when statused
pub struct MarvelScale;

impl AbilityEffect for MarvelScale {
    fn name(&self) -> &str {
        "Marvel Scale"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        use crate::core::instruction::PokemonStatus;
        
        if context.defender.status != PokemonStatus::None && context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_damage_multiplier(0.67) // Approximate 1.5x reduction
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// SOUND-BASED AND MOVE BLOCKING ABILITIES
// =============================================================================

/// Soundproof - Blocks sound-based moves
pub struct Soundproof;

impl AbilityEffect for Soundproof {
    fn name(&self) -> &str {
        "Soundproof"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move has sound flag
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "sound") {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Damp - Prevents explosion moves
pub struct Damp;

impl AbilityEffect for Damp {
    fn name(&self) -> &str {
        "Damp"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move name contains explosion/self-destruct patterns
        let move_name = context.move_data.name.to_lowercase();
        if move_name.contains("explosion") || move_name.contains("self-destruct") || move_name.contains("selfdestruct") {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// TYPE EFFECTIVENESS MODIFICATION ABILITIES
// =============================================================================

/// Scrappy - Normal/Fighting moves hit Ghost types
pub struct Scrappy;

impl AbilityEffect for Scrappy {
    fn name(&self) -> &str {
        "Scrappy"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        let move_type = context.move_type.to_lowercase();
        let is_ghost_defender = context.defender.types.iter().any(|t| t.to_lowercase() == "ghost");
        
        if (move_type == "normal" || move_type == "fighting") && is_ghost_defender {
            AbilityModifier::new().with_damage_multiplier(1.0) // Override immunity
        } else {
            AbilityModifier::default()
        }
    }
}

/// Mind's Eye - Normal/Fighting moves hit Ghost types
pub struct MindEye;

impl AbilityEffect for MindEye {
    fn name(&self) -> &str {
        "Mind's Eye"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        let move_type = context.move_type.to_lowercase();
        let is_ghost_defender = context.defender.types.iter().any(|t| t.to_lowercase() == "ghost");
        
        if (move_type == "normal" || move_type == "fighting") && is_ghost_defender {
            AbilityModifier::new().with_damage_multiplier(1.0) // Override immunity
        } else {
            AbilityModifier::default()
        }
    }
}

/// Unaware - Ignores stat changes when calculating damage
pub struct Unaware;

impl AbilityEffect for Unaware {
    fn name(&self) -> &str {
        "Unaware"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Stat boost ignoring would need to be handled in damage calculation
        AbilityModifier::default()
    }
}

/// Moldbreaker - Ignores certain defensive abilities
pub struct Moldbreaker;

impl AbilityEffect for Moldbreaker {
    fn name(&self) -> &str {
        "Moldbreaker"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Ability bypassing would need to be handled in damage calculation pipeline
        AbilityModifier::default()
    }
}

/// Wonder Guard - Only super effective moves deal damage
pub struct WonderGuard;

impl AbilityEffect for WonderGuard {
    fn name(&self) -> &str {
        "Wonder Guard"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Status {
            return AbilityModifier::default();
        }
        
        // Check type effectiveness
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

        if type_effectiveness <= 1.0 {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// CONTACT ABILITIES
// =============================================================================

/// Poison Point - 33% chance to poison on contact
pub struct PoisonPoint;

impl AbilityEffect for PoisonPoint {
    fn name(&self) -> &str {
        "Poison Point"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact effect would need to be handled in after-damage processing
        AbilityModifier::default()
    }
}

/// Effect Spore - Chance to inflict Poison/Paralysis/Sleep on contact
pub struct EffectSpore;

impl AbilityEffect for EffectSpore {
    fn name(&self) -> &str {
        "Effect Spore"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact effect would need to be handled in after-damage processing
        AbilityModifier::default()
    }
}

/// Flame Body - 30% chance to burn on contact
pub struct FlameBody;

impl AbilityEffect for FlameBody {
    fn name(&self) -> &str {
        "Flame Body"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact effect would need to be handled in after-damage processing
        AbilityModifier::default()
    }
}

/// Static - 30% chance to paralyze on contact
pub struct Static;

impl AbilityEffect for Static {
    fn name(&self) -> &str {
        "Static"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact effect would need to be handled in after-damage processing
        AbilityModifier::default()
    }
}

// =============================================================================
// STATUS PROTECTION ABILITIES
// =============================================================================

/// Purifying Salt - Status immunity
pub struct PurifyingSalt;

impl AbilityEffect for PurifyingSalt {
    fn name(&self) -> &str {
        "Purifying Salt"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Status immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Comatose - Sleep immunity (always asleep)
pub struct Comatose;

impl AbilityEffect for Comatose {
    fn name(&self) -> &str {
        "Comatose"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Sleep immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Leaf Guard - Status immunity in Sun
pub struct LeafGuard;

impl AbilityEffect for LeafGuard {
    fn name(&self) -> &str {
        "Leaf Guard"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Conditional status immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Water Veil - Burn immunity
pub struct WaterVeil;

impl AbilityEffect for WaterVeil {
    fn name(&self) -> &str {
        "Water Veil"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Burn immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Water Bubble - Burn immunity
pub struct WaterBubble;

impl AbilityEffect for WaterBubble {
    fn name(&self) -> &str {
        "Water Bubble"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Burn immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Thermal Exchange - Burn immunity
pub struct ThermalExchange;

impl AbilityEffect for ThermalExchange {
    fn name(&self) -> &str {
        "Thermal Exchange"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Burn immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Magma Armor - Freeze immunity
pub struct MagmaArmor;

impl AbilityEffect for MagmaArmor {
    fn name(&self) -> &str {
        "Magma Armor"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Freeze immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Insomnia - Sleep immunity
pub struct Insomnia;

impl AbilityEffect for Insomnia {
    fn name(&self) -> &str {
        "Insomnia"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Sleep immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Sweet Veil - Sleep immunity
pub struct SweetVeil;

impl AbilityEffect for SweetVeil {
    fn name(&self) -> &str {
        "Sweet Veil"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Sleep immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Vital Spirit - Sleep immunity
pub struct VitalSpirit;

impl AbilityEffect for VitalSpirit {
    fn name(&self) -> &str {
        "Vital Spirit"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Sleep immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Limber - Paralysis immunity
pub struct Limber;

impl AbilityEffect for Limber {
    fn name(&self) -> &str {
        "Limber"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Paralysis immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Immunity - Poison immunity
pub struct Immunity;

impl AbilityEffect for Immunity {
    fn name(&self) -> &str {
        "Immunity"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Poison immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Pastel Veil - Poison immunity
pub struct PastelVeil;

impl AbilityEffect for PastelVeil {
    fn name(&self) -> &str {
        "Pastel Veil"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Poison immunity would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Sturdy - Prevents OHKO from full HP
pub struct Sturdy;

impl AbilityEffect for Sturdy {
    fn name(&self) -> &str {
        "Sturdy"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // OHKO prevention would need special handling in damage calculation
        if context.defender.hp == context.defender.max_hp {
            // This would need more complex logic to prevent OHKO
            AbilityModifier::default()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Pressure - Increases PP usage
pub struct Pressure;

impl AbilityEffect for Pressure {
    fn name(&self) -> &str {
        "Pressure"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // PP increase would need to be handled in move execution
        AbilityModifier::default()
    }
}

// =============================================================================
// SPECIAL MECHANICS ABILITIES
// =============================================================================

/// Contrary - Reverses stat changes
pub struct Contrary;

impl AbilityEffect for Contrary {
    fn name(&self) -> &str {
        "Contrary"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Stat change reversal would need to be handled in stat modification
        AbilityModifier::default()
    }
}

/// Corrosion - Can poison Steel and Poison types
pub struct Corrosion;

impl AbilityEffect for Corrosion {
    fn name(&self) -> &str {
        "Corrosion"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Special poisoning would need to be handled in status application
        AbilityModifier::default()
    }
}

/// Magic Guard - Prevents indirect damage
pub struct MagicGuard;

impl AbilityEffect for MagicGuard {
    fn name(&self) -> &str {
        "Magic Guard"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Indirect damage prevention would need to be handled in various damage sources
        AbilityModifier::default()
    }
}

/// Neutralizing Gas - Suppresses all abilities
pub struct NeutralizingGas;

impl AbilityEffect for NeutralizingGas {
    fn name(&self) -> &str {
        "Neutralizing Gas"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Ability suppression would need to be handled globally
        AbilityModifier::default()
    }
}

// =============================================================================
// MISC ABILITIES
// =============================================================================

/// Suction Cups - Prevents forced switching
pub struct SuctionCups;

impl AbilityEffect for SuctionCups {
    fn name(&self) -> &str {
        "Suction Cups"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Switch prevention would need to be handled in move effects
        AbilityModifier::default()
    }
}

/// Liquid Ooze - Reverses HP drain effects
pub struct LiquidOoze;

impl AbilityEffect for LiquidOoze {
    fn name(&self) -> &str {
        "Liquid Ooze"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // HP drain reversal would need to be handled in drain move effects
        AbilityModifier::default()
    }
}

/// Shield Dust - Prevents secondary effects on the user
pub struct ShieldDust;

impl AbilityEffect for ShieldDust {
    fn name(&self) -> &str {
        "Shield Dust"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Secondary effect prevention would need to be handled in move effects
        AbilityModifier::default()
    }
}

// =============================================================================
// AFTER-DAMAGE ABILITIES (KO ABILITIES)
// =============================================================================

/// Moxie - Boosts Attack by 1 when KOing opponent
pub struct Moxie;

impl AbilityEffect for Moxie {
    fn name(&self) -> &str {
        "Moxie"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Chilling Neigh - Boosts Attack by 1 when KOing opponent  
pub struct ChillingNeigh;

impl AbilityEffect for ChillingNeigh {
    fn name(&self) -> &str {
        "Chilling Neigh"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// As One (Glastrier) - Boosts Attack by 1 when KOing opponent
pub struct AsOneGlastrier;

impl AbilityEffect for AsOneGlastrier {
    fn name(&self) -> &str {
        "As One (Glastrier)"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Grim Neigh - Boosts Special Attack by 1 when KOing opponent
pub struct GrimNeigh;

impl AbilityEffect for GrimNeigh {
    fn name(&self) -> &str {
        "Grim Neigh"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// As One (Spectrier) - Boosts Special Attack by 1 when KOing opponent
pub struct AsOneSpectrier;

impl AbilityEffect for AsOneSpectrier {
    fn name(&self) -> &str {
        "As One (Spectrier)"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Beast Boost - Boosts highest stat by 1 when KOing opponent
pub struct BeastBoost;

impl AbilityEffect for BeastBoost {
    fn name(&self) -> &str {
        "Beast Boost"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Battle Bond - Boosts Attack/SpA/Speed by 1 when KOing opponent
pub struct BattleBond;

impl AbilityEffect for BattleBond {
    fn name(&self) -> &str {
        "Battle Bond"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Magician - Steals opponent's item when dealing damage
pub struct Magician;

impl AbilityEffect for Magician {
    fn name(&self) -> &str {
        "Magician"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Item stealing is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Pickpocket - Steals opponent's item when hit by contact moves
pub struct Pickpocket;

impl AbilityEffect for Pickpocket {
    fn name(&self) -> &str {
        "Pickpocket"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Item stealing is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

// =============================================================================
// AFTER-DAMAGE ABILITIES (CONTACT ABILITIES)
// =============================================================================

/// Mummy - Changes attacker's ability to Mummy on contact
pub struct Mummy;

impl AbilityEffect for Mummy {
    fn name(&self) -> &str {
        "Mummy"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Ability change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Lingering Aroma - Changes attacker's ability on contact
pub struct LingeringAroma;

impl AbilityEffect for LingeringAroma {
    fn name(&self) -> &str {
        "Lingering Aroma"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Ability change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Wandering Spirit - Changes attacker's ability on contact
pub struct WanderingSpirit;

impl AbilityEffect for WanderingSpirit {
    fn name(&self) -> &str {
        "Wandering Spirit"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Ability change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Color Change - Changes type to match move that hit
pub struct ColorChange;

impl AbilityEffect for ColorChange {
    fn name(&self) -> &str {
        "Color Change"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Type change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Stamina - Boosts Defense by 1 when hit
pub struct Stamina;

impl AbilityEffect for Stamina {
    fn name(&self) -> &str {
        "Stamina"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Defense boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Cotton Down - Lowers attacker's Speed by 1 when hit by contact moves
pub struct CottonDown;

impl AbilityEffect for CottonDown {
    fn name(&self) -> &str {
        "Cotton Down"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Speed drop is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Sand Spit - Sets Sand weather when hit
pub struct SandSpit;

impl AbilityEffect for SandSpit {
    fn name(&self) -> &str {
        "Sand Spit"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Weather change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Seed Sower - Sets Grassy Terrain when hit
pub struct SeedSower;

impl AbilityEffect for SeedSower {
    fn name(&self) -> &str {
        "Seed Sower"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Terrain change is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Toxic Debris - Sets Toxic Spikes when hit by physical moves
pub struct ToxicDebris;

impl AbilityEffect for ToxicDebris {
    fn name(&self) -> &str {
        "Toxic Debris"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Toxic Spikes application is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Berserk - Boosts Special Attack when HP drops below 50%
pub struct Berserk;

impl AbilityEffect for Berserk {
    fn name(&self) -> &str {
        "Berserk"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Special Attack boost is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Rough Skin - Damages attacker 1/8 HP on contact
pub struct RoughSkin;

impl AbilityEffect for RoughSkin {
    fn name(&self) -> &str {
        "Rough Skin"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact damage is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Iron Barbs - Damages attacker 1/8 HP on contact
pub struct IronBarbs;

impl AbilityEffect for IronBarbs {
    fn name(&self) -> &str {
        "Iron Barbs"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Contact damage is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Aftermath - Damages attacker 1/4 HP when KO'd by contact
pub struct Aftermath;

impl AbilityEffect for Aftermath {
    fn name(&self) -> &str {
        "Aftermath"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO contact damage is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Innards Out - Damages attacker equal to damage taken when KO'd
pub struct InnardsOut;

impl AbilityEffect for InnardsOut {
    fn name(&self) -> &str {
        "Innards Out"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // KO damage is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

/// Perish Body - Applies Perish Song to both Pokemon on contact
pub struct PerishBody;

impl AbilityEffect for PerishBody {
    fn name(&self) -> &str {
        "Perish Body"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Perish Song application is handled in process_after_damage_abilities
        AbilityModifier::default()
    }
}

// =============================================================================
// ATTACK MODIFICATION ABILITIES
// =============================================================================

/// Protean - Changes type to match move being used
pub struct Protean;

impl AbilityEffect for Protean {
    fn name(&self) -> &str {
        "Protean"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Type change would be handled in move execution
        AbilityModifier::default()
    }
}

/// Libero - Changes type to match move being used (same as Protean)
pub struct Libero;

impl AbilityEffect for Libero {
    fn name(&self) -> &str {
        "Libero"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Type change would be handled in move execution
        AbilityModifier::default()
    }
}

/// Gorilla Tactics - Disables other moves after using one
pub struct GorillaTactics;

impl AbilityEffect for GorillaTactics {
    fn name(&self) -> &str {
        "Gorilla Tactics"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Provides Attack boost but limits move selection
        if context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_attack_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Prankster - Gives priority to status moves
pub struct Prankster;

impl AbilityEffect for Prankster {
    fn name(&self) -> &str {
        "Prankster"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Priority modification would be handled in move execution
        AbilityModifier::default()
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

// =============================================================================
// AFTER DAMAGE ABILITY HELPER FUNCTIONS
// =============================================================================

/// Apply Attack boost on KO for abilities like Moxie
fn apply_attack_boost_on_ko(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply Special Attack boost on KO for abilities like Grim Neigh
fn apply_special_attack_boost_on_ko(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::SpecialAttack, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply Beast Boost on KO - boosts highest stat
fn apply_beast_boost_on_ko(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
    pokemon: &Pokemon,
) -> Vec<StateInstructions> {
    // Find the highest stat (ignoring HP)
    let mut highest_stat = Stat::Attack;
    let mut highest_value = pokemon.stats.attack;
    
    if pokemon.stats.defense > highest_value {
        highest_stat = Stat::Defense;
        highest_value = pokemon.stats.defense;
    }
    if pokemon.stats.special_attack > highest_value {
        highest_stat = Stat::SpecialAttack;
        highest_value = pokemon.stats.special_attack;
    }
    if pokemon.stats.special_defense > highest_value {
        highest_stat = Stat::SpecialDefense;
        highest_value = pokemon.stats.special_defense;
    }
    if pokemon.stats.speed > highest_value {
        highest_stat = Stat::Speed;
        highest_value = pokemon.stats.speed;
    }
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(highest_stat, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply Battle Bond on KO - Attack/SpA/Speed boost and forme change
fn apply_battle_bond_on_ko(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::SpecialAttack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply item stealing for Magician/Pickpocket
fn apply_item_steal(
    _state: &State,
    stealer_position: BattlePosition,
    victim_position: BattlePosition,
    _generation: &GenerationMechanics,
    _ability_name: &str,
) -> Vec<StateInstructions> {
    // For now, simplified item stealing (would need to check if stealer has no item, victim has item, etc.)
    vec![StateInstructions::new(100.0, vec![
        Instruction::ChangeItem(ChangeItemInstruction {
            target_position: stealer_position,
            new_item: Some("stolen_item".to_string()),
            previous_item: None,
        })
    ])]
}

/// Apply ability change on contact for Mummy-like abilities
fn apply_ability_change_on_contact(
    _state: &State,
    attacker_position: BattlePosition,
    _defender_position: BattlePosition,
    _generation: &GenerationMechanics,
    new_ability: &str,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::ChangeAbility(ChangeAbilityInstruction {
            target_position: attacker_position,
            new_ability: new_ability.to_string(),
            previous_ability: None,
        })
    ])]
}

/// Apply Color Change - change type to match attacking move
fn apply_color_change(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
    attacker: &Pokemon,
) -> Vec<StateInstructions> {
    // Get the type of the move that hit (would need move context)
    let new_type = "Normal".to_string(); // Placeholder - would need actual move type
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::ChangeType(ChangeTypeInstruction {
            target_position: position,
            new_types: vec![new_type],
            previous_types: None,
        })
    ])]
}

/// Apply Defense boost on hit for Stamina
fn apply_defense_boost_on_hit(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Defense, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply Speed drop on contact for Cotton Down
fn apply_speed_drop_on_contact(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Speed, -1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply weather change on hit
fn apply_weather_change_on_hit(
    state: &State,
    _generation: &GenerationMechanics,
    new_weather: Weather,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::ChangeWeather(ChangeWeatherInstruction {
            weather: new_weather,
            duration: Some(5),
            previous_weather: Some(state.weather),
            previous_duration: Some(state.weather_turns_remaining),
        })
    ])]
}

/// Apply terrain change on hit
fn apply_terrain_change_on_hit(
    state: &State,
    _generation: &GenerationMechanics,
    new_terrain: Terrain,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::ChangeTerrain(ChangeTerrainInstruction {
            terrain: new_terrain,
            duration: Some(5),
            previous_terrain: Some(state.terrain),
            previous_duration: Some(state.terrain_turns_remaining),
        })
    ])]
}

/// Apply Toxic Spikes on physical hit
fn apply_toxic_spikes_on_physical_hit(
    _state: &State,
    target_side: crate::core::battle_format::SideReference,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: target_side,
            condition: SideCondition::ToxicSpikes,
            duration: None,
        })
    ])]
}

/// Apply Special Attack boost on hit for Berserk
fn apply_special_attack_boost_on_hit(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::SpecialAttack, 1);
    
    vec![StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ])]
}

/// Apply contact damage for abilities like Rough Skin
fn apply_contact_damage(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
    damage: i16,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::PositionDamage(PositionDamageInstruction {
            target_position: position,
            damage_amount: damage,
            previous_hp: Some(0),
        })
    ])]
}

/// Apply Perish Song to both Pokemon for Perish Body
fn apply_perish_song_to_both(
    _state: &State,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(100.0, vec![
        Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: attacker_position,
            volatile_status: VolatileStatus::Perish3,
            duration: Some(3),
        }),
        Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: defender_position,
            volatile_status: VolatileStatus::Perish3,
            duration: Some(3),
        })
    ])]
}

// ==============================================================================
// CRITICAL MISSING ABILITIES - IMPLEMENTING FOR 100% PARITY WITH POKE-ENGINE
// ==============================================================================

/// Disguise - Blocks first damaging move and changes to Busted form
pub struct Disguise;

impl AbilityEffect for Disguise {
    fn name(&self) -> &str {
        "Disguise"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Disguise blocks the first physical or special attack and changes forme
        // This should only trigger on Mimikyu in its base form
        if (context.move_data.category == MoveCategory::Physical || 
            context.move_data.category == MoveCategory::Special) &&
           context.defender.species.to_lowercase().contains("mimikyu") &&
           !context.defender.species.to_lowercase().contains("busted") {
            
            // Block the move completely and trigger forme change
            // In Gen 8+, Disguise also takes 1/8 HP damage
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Gulp Missile - Forme change ability for Cramorant
pub struct GulpMissile;

impl AbilityEffect for GulpMissile {
    fn name(&self) -> &str {
        "Gulp Missile" 
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Gulp Missile forme changes are handled elsewhere
        // Damage effects when hit are handled in after_damage abilities
        AbilityModifier::default()
    }
}

/// Schooling - Forme change ability for Wishiwashi
pub struct Schooling;

impl AbilityEffect for Schooling {
    fn name(&self) -> &str {
        "Schooling"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Schooling forme changes are handled based on HP thresholds
        AbilityModifier::default()
    }
}

/// Shields Down - Forme change and protection for Minior
pub struct ShieldsDown;

impl AbilityEffect for ShieldsDown {
    fn name(&self) -> &str {
        "Shields Down"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Above 50% HP in Meteor form: immune to status moves
        if context.defender.hp > context.defender.max_hp / 2 &&
           context.defender.species.to_lowercase().contains("minior") &&
           !context.defender.species.to_lowercase().contains("core") &&
           context.move_data.category == MoveCategory::Status {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Normalize - Changes all moves to Normal type
pub struct Normalize;

impl AbilityEffect for Normalize {
    fn name(&self) -> &str {
        "Normalize"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // All moves become Normal-type with 1.2x power boost
        if context.move_type.to_lowercase() != "normal" {
            AbilityModifier::new().with_power_multiplier(1.2)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Liquid Voice - Sound moves become Water-type with power boost
pub struct LiquidVoice;

impl AbilityEffect for LiquidVoice {
    fn name(&self) -> &str {
        "Liquid Voice"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Sound moves become Water-type with 20% power boost
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "sound") {
            AbilityModifier::new().with_power_multiplier(1.2)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Galvanize - Normal moves become Electric-type with power boost  
pub struct Galvanize;

impl AbilityEffect for Galvanize {
    fn name(&self) -> &str {
        "Galvanize"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_type.to_lowercase() == "normal" {
            // Gen 7+: 1.2x power boost, Gen 6: 1.3x power boost
            let multiplier = if context.state.get_generation().number() >= 7 { 1.2 } else { 1.3 };
            AbilityModifier::new().with_power_multiplier(multiplier)
        } else {
            AbilityModifier::default()
        }
    }
}


/// Mega Launcher - Pulse/Aura moves get 50% power boost
pub struct MegaLauncher;

impl AbilityEffect for MegaLauncher {
    fn name(&self) -> &str {
        "Mega Launcher"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is a pulse/aura move
        let pulse_moves = [
            "waterpulse", "originpulse", "aurasphere", "darkpulse", "dragonpulse",
            "healingpulse", "terrainpulse"
        ];
        
        if pulse_moves.iter().any(|&move_name| 
            context.move_data.name.to_lowercase().replace(" ", "").contains(move_name)) {
            AbilityModifier::new().with_power_multiplier(1.5)
        } else {
            AbilityModifier::default()
        }
    }
}

/// Punk Rock - Sound moves get 30% power boost and sound move immunity
pub struct PunkRock;

impl AbilityEffect for PunkRock {
    fn name(&self) -> &str {
        "Punk Rock"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "sound") {
            // Attacking: 30% power boost for sound moves
            // Defending: immune to sound moves
            if std::ptr::eq(&context.attacker, &context.defender) {
                // This is the attacker using a sound move
                AbilityModifier::new().with_power_multiplier(1.3)
            } else {
                // This is defending against a sound move
                AbilityModifier::new().block_move()
            }
        } else {
            AbilityModifier::default()
        }
    }
}

/// Mirror Armor - Reflects stat drops back to the attacker
pub struct MirrorArmor;

impl AbilityEffect for MirrorArmor {
    fn name(&self) -> &str {
        "Mirror Armor"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Stat drop reflection is handled in the instruction processing
        AbilityModifier::default()
    }
}

/// Bulletproof - Blocks ball and bomb moves
pub struct Bulletproof;

impl AbilityEffect for Bulletproof {
    fn name(&self) -> &str {
        "Bulletproof"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is a ball or bomb move
        let bullet_moves = [
            "shadowball", "focusblast", "energyball", "aurasphere", "mistball",
            "octazooka", "rockblast", "seedbomb", "sludgebomb", "eggbomb",
            "gyroball", "electrobomb", "bulletpunch"
        ];
        
        if bullet_moves.iter().any(|&move_name| 
            context.move_data.name.to_lowercase().replace(" ", "").contains(move_name)) {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Overcoat - Blocks powder moves and weather damage
pub struct Overcoat;

impl AbilityEffect for Overcoat {
    fn name(&self) -> &str {
        "Overcoat"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if move is a powder move
        if context.move_data.flags.iter().any(|flag| flag.to_lowercase() == "powder") {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Good as Gold - Blocks status moves
pub struct GoodAsGold;

impl AbilityEffect for GoodAsGold {
    fn name(&self) -> &str {
        "Good as Gold"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        if context.move_data.category == MoveCategory::Status {
            AbilityModifier::new().block_move()
        } else {
            AbilityModifier::default()
        }
    }
}

/// Primordial Sea - Sets Heavy Rain weather
pub struct PrimordialSea;

impl AbilityEffect for PrimordialSea {
    fn name(&self) -> &str {
        "Primordial Sea"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Weather setting is handled in switch-in effects
        AbilityModifier::default()
    }
}

/// Desolate Land - Sets Harsh Sun weather
pub struct DesolateLand;

impl AbilityEffect for DesolateLand {
    fn name(&self) -> &str {
        "Desolate Land"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Weather setting is handled in switch-in effects
        AbilityModifier::default()
    }
}

/// Orichalcum Pulse - Sets Sun weather (Koraidon signature)
pub struct OrichalcumPulse;

impl AbilityEffect for OrichalcumPulse {
    fn name(&self) -> &str {
        "Orichalcum Pulse"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Boost Attack in Sun weather
        if context.state.weather == Weather::Sun &&
           context.move_data.category == MoveCategory::Physical {
            AbilityModifier::new().with_attack_multiplier(1.33) // 4/3 boost
        } else {
            AbilityModifier::default()
        }
    }
}

/// Hadron Engine - Sets Electric Terrain (Miraidon signature)
pub struct HadronEngine;

impl AbilityEffect for HadronEngine {
    fn name(&self) -> &str {
        "Hadron Engine"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Boost Special Attack in Electric Terrain
        if context.state.terrain == Terrain::ElectricTerrain &&
           context.move_data.category == MoveCategory::Special {
            AbilityModifier::new().with_special_attack_multiplier(1.33) // 4/3 boost
        } else {
            AbilityModifier::default()
        }
    }
}

// =============================================================================
// GEN 9 LEGENDARY ABILITIES
// =============================================================================

/// Intrepid Sword - Boosts Attack by 1 on switch-in (Zacian signature)
pub struct IntrepidSword;

impl AbilityEffect for IntrepidSword {
    fn name(&self) -> &str {
        "Intrepid Sword"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Switch-in effect is handled in switch_effects.rs
        AbilityModifier::default()
    }
}

/// Dauntless Shield - Boosts Defense by 1 on switch-in (Zamazenta signature)
pub struct DauntlessShield;

impl AbilityEffect for DauntlessShield {
    fn name(&self) -> &str {
        "Dauntless Shield"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Switch-in effect is handled in switch_effects.rs
        AbilityModifier::default()
    }
}

// =============================================================================
// PARADOX POKEMON ABILITIES
// =============================================================================

/// Protosynthesis - Boosts highest stat in Sun weather (or with Booster Energy)
pub struct Protosynthesis;

impl AbilityEffect for Protosynthesis {
    fn name(&self) -> &str {
        "Protosynthesis"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if Protosynthesis is active through volatile status
        let protosynthesis_active = context.attacker.volatile_statuses.iter().any(|status| {
            matches!(status, 
                crate::core::instruction::VolatileStatus::ProtosynthesisAttack |
                crate::core::instruction::VolatileStatus::ProtosynthesisDefense |
                crate::core::instruction::VolatileStatus::ProtosynthesisSpecialAttack |
                crate::core::instruction::VolatileStatus::ProtosynthesisSpecialDefense |
                crate::core::instruction::VolatileStatus::ProtosynthesisSpeed
            )
        });

        if protosynthesis_active {
            // Check which stat is boosted and apply appropriate multiplier
            if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::ProtosynthesisAttack) {
                if context.move_data.category == MoveCategory::Physical {
                    return AbilityModifier::new().with_attack_multiplier(1.3);
                }
            } else if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::ProtosynthesisSpecialAttack) {
                if context.move_data.category == MoveCategory::Special {
                    return AbilityModifier::new().with_special_attack_multiplier(1.3);
                }
            }
        }

        AbilityModifier::default()
    }
}

/// Quark Drive - Boosts highest stat in Electric Terrain (or with Booster Energy)
pub struct QuarkDrive;

impl AbilityEffect for QuarkDrive {
    fn name(&self) -> &str {
        "Quark Drive"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if Quark Drive is active through volatile status
        let quark_drive_active = context.attacker.volatile_statuses.iter().any(|status| {
            matches!(status,
                crate::core::instruction::VolatileStatus::QuarkDriveAttack |
                crate::core::instruction::VolatileStatus::QuarkDriveDefense |
                crate::core::instruction::VolatileStatus::QuarkDriveSpecialAttack |
                crate::core::instruction::VolatileStatus::QuarkDriveSpecialDefense |
                crate::core::instruction::VolatileStatus::QuarkDriveSpeed
            )
        });

        if quark_drive_active {
            // Check which stat is boosted and apply appropriate multiplier
            if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::QuarkDriveAttack) {
                if context.move_data.category == MoveCategory::Physical {
                    return AbilityModifier::new().with_attack_multiplier(1.3);
                }
            } else if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::QuarkDriveSpecialAttack) {
                if context.move_data.category == MoveCategory::Special {
                    return AbilityModifier::new().with_special_attack_multiplier(1.3);
                }
            }
        }

        AbilityModifier::default()
    }
}

// =============================================================================
// OGERPON ABILITIES
// =============================================================================

/// Embody Aspect - Boosts different stats based on Ogerpon's forme
pub struct EmbodyAspect;

impl AbilityEffect for EmbodyAspect {
    fn name(&self) -> &str {
        "Embody Aspect"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Switch-in effect is handled in switch_effects.rs
        AbilityModifier::default()
    }
}

// =============================================================================
// UTILITY ABILITIES
// =============================================================================

/// Screen Cleaner - Removes all screens from both sides on switch-in
pub struct ScreenCleaner;

impl AbilityEffect for ScreenCleaner {
    fn name(&self) -> &str {
        "Screen Cleaner"
    }

    fn modify_damage(&self, _context: &DamageContext) -> AbilityModifier {
        // Switch-in effect is handled in switch_effects.rs
        AbilityModifier::default()
    }
}

/// Slow Start - Halves Attack and Speed for 5 turns (Regigigas signature)
pub struct SlowStart;

impl AbilityEffect for SlowStart {
    fn name(&self) -> &str {
        "Slow Start"
    }

    fn modify_damage(&self, context: &DamageContext) -> AbilityModifier {
        // Check if Slow Start is active
        if context.attacker.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::SlowStart) {
            if context.move_data.category == MoveCategory::Physical {
                AbilityModifier::new().with_attack_multiplier(0.5)
            } else {
                AbilityModifier::default()
            }
        } else {
            AbilityModifier::default()
        }
    }
}

/// Apply status effect on contact for abilities like Poison Point, Static, Flame Body
fn apply_status_on_contact(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
    status: crate::core::instruction::PokemonStatus,
    chance: f32,
) -> Vec<StateInstructions> {
    vec![StateInstructions::new(chance, vec![
        Instruction::ApplyStatus(ApplyStatusInstruction {
            target_position: position,
            status,
            previous_status: Some(crate::core::instruction::PokemonStatus::None),
            previous_status_duration: None,
        })
    ])]
}

/// Apply Effect Spore status effects on contact (poison, paralysis, or sleep)
fn apply_effect_spore_on_contact(
    _state: &State,
    position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![
        // 9% chance for poison
        StateInstructions::new(9.0, vec![
            Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: crate::core::instruction::PokemonStatus::Poison,
                previous_status: Some(crate::core::instruction::PokemonStatus::None),
                previous_status_duration: None,
            })
        ]),
        // 9% chance for paralysis
        StateInstructions::new(9.0, vec![
            Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: crate::core::instruction::PokemonStatus::Paralysis,
                previous_status: Some(crate::core::instruction::PokemonStatus::None),
                previous_status_duration: None,
            })
        ]),
        // 9% chance for sleep
        StateInstructions::new(9.0, vec![
            Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: crate::core::instruction::PokemonStatus::Sleep,
                previous_status: Some(crate::core::instruction::PokemonStatus::None),
                previous_status_duration: None,
            })
        ])
    ]
}