//! # Complete Item System
//!
//! This module provides comprehensive item effects system with generation-aware mechanics.
//! Items can modify damage, stats, provide abilities, immunities, and many other battle effects.

use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instruction::{Stat, PokemonStatus};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, StatsInstruction};
use std::collections::HashMap;

/// Stat boosts for reactive items
#[derive(Debug, Clone, PartialEq)]
pub struct StatBoosts {
    pub attack: i8,
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
    pub accuracy: i8,
}

impl StatBoosts {
    pub fn new() -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn attack(attack: i8) -> Self {
        Self {
            attack,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn special_attack(special_attack: i8) -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn defense(defense: i8) -> Self {
        Self {
            attack: 0,
            defense,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn special_defense(special_defense: i8) -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn attack_and_special_attack(attack: i8, special_attack: i8) -> Self {
        Self {
            attack,
            defense: 0,
            special_attack,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
}

/// Modifier result from an item
#[derive(Debug, Clone)]
pub struct ItemModifier {
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
    /// Speed stat multiplier (1.0 = no change)
    pub speed_multiplier: f32,
    /// Whether this move should be completely blocked
    pub blocks_move: bool,
    /// Recoil damage percentage (e.g., 0.1 for 10% recoil)
    pub recoil_percentage: f32,
    /// Move type change (None = no change)
    pub type_change: Option<String>,
    /// Whether item is consumed after use
    pub is_consumed: bool,
    /// Ground immunity (Air Balloon)
    pub ground_immunity: bool,
    /// Hazard immunity (Heavy Duty Boots)
    pub hazard_immunity: bool,
    /// Contact damage to attacker (Rocky Helmet)
    pub contact_recoil: f32,
    /// Stat boosts to apply (for reactive items)
    pub stat_boosts: Option<StatBoosts>,
    /// Prevents KO at full HP (Focus Sash)
    pub prevents_ko_at_full_hp: bool,
    /// Drain percentage for Shell Bell (fraction of damage dealt restored as HP)
    pub drain_percentage: f32,
    /// Removes contact flag from move (Punching Glove)
    pub removes_contact: bool,
    /// Accuracy multiplier (1.0 = no change, >1.0 = more accurate)
    pub accuracy_multiplier: f32,
    /// Priority modifier (0 = no change, +1 = +1 priority, etc.)
    pub priority_modifier: i8,
}

impl Default for ItemModifier {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            power_multiplier: 1.0,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            special_attack_multiplier: 1.0,
            special_defense_multiplier: 1.0,
            speed_multiplier: 1.0,
            blocks_move: false,
            recoil_percentage: 0.0,
            type_change: None,
            is_consumed: false,
            ground_immunity: false,
            hazard_immunity: false,
            contact_recoil: 0.0,
            stat_boosts: None,
            prevents_ko_at_full_hp: false,
            drain_percentage: 0.0,
            removes_contact: false,
            accuracy_multiplier: 1.0,
            priority_modifier: 0,
        }
    }
}

impl ItemModifier {
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

    /// Set speed multiplier
    pub fn with_speed_multiplier(mut self, multiplier: f32) -> Self {
        self.speed_multiplier = multiplier;
        self
    }

    /// Set recoil percentage
    pub fn with_recoil(mut self, percentage: f32) -> Self {
        self.recoil_percentage = percentage;
        self
    }

    /// Block the move completely
    pub fn block_move(mut self) -> Self {
        self.blocks_move = true;
        self
    }

    /// Change move type
    pub fn with_type_change(mut self, new_type: String) -> Self {
        self.type_change = Some(new_type);
        self
    }

    /// Mark item as consumed
    pub fn consumed(mut self) -> Self {
        self.is_consumed = true;
        self
    }

    /// Provide ground immunity
    pub fn with_ground_immunity(mut self) -> Self {
        self.ground_immunity = true;
        self
    }

    /// Provide hazard immunity
    pub fn with_hazard_immunity(mut self) -> Self {
        self.hazard_immunity = true;
        self
    }

    /// Set contact recoil damage
    pub fn with_contact_recoil(mut self, recoil: f32) -> Self {
        self.contact_recoil = recoil;
        self
    }
    
    /// Apply stat boosts
    pub fn with_stat_boosts(mut self, boosts: StatBoosts) -> Self {
        self.stat_boosts = Some(boosts);
        self
    }
    
    /// Prevent KO at full HP (Focus Sash)
    pub fn with_ko_prevention(mut self) -> Self {
        self.prevents_ko_at_full_hp = true;
        self
    }
    
    /// Set drain percentage (Shell Bell)
    pub fn with_drain(mut self, percentage: f32) -> Self {
        self.drain_percentage = percentage;
        self
    }
    
    /// Remove contact from move (Punching Glove)
    pub fn with_contact_removal(mut self) -> Self {
        self.removes_contact = true;
        self
    }
    
    /// Set accuracy multiplier
    pub fn with_accuracy_multiplier(mut self, multiplier: f32) -> Self {
        self.accuracy_multiplier = multiplier;
        self
    }
    
    /// Set priority modifier
    pub fn with_priority_modifier(mut self, modifier: i8) -> Self {
        self.priority_modifier = modifier;
        self
    }
}

/// Trait for item effects that can modify damage calculation
pub trait ItemEffect {
    /// Get the item name
    fn name(&self) -> &str;

    /// Modify damage calculation before it happens
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        ItemModifier::default()
    }

    /// Check if this is an attacker item (affects the user's own moves)
    fn is_attacker_item(&self) -> bool {
        true // Default to attacker item
    }

    /// Check if this is a defender item (affects incoming attacks)
    fn is_defender_item(&self) -> bool {
        false // Default to not defender item
    }

    /// Check if this item provides immunity to a move type
    fn provides_immunity(&self, _move_type: &str) -> bool {
        false
    }

    /// Modify STAB calculation
    fn modify_stab(&self, _context: &DamageContext) -> f32 {
        1.0
    }

    /// Check if weather effects should be negated
    fn negates_weather(&self) -> bool {
        false
    }

    /// Check if this item boosts super effective moves
    fn boosts_super_effective(&self) -> bool {
        false
    }

    /// Get super effective boost multiplier
    fn super_effective_multiplier(&self) -> f32 {
        1.0
    }

    /// Check if this item affects type effectiveness calculation
    fn affects_type_effectiveness(&self) -> bool {
        false
    }

    /// Modify type effectiveness multiplier
    fn modify_type_effectiveness(&self, effectiveness: f32, _context: &DamageContext) -> f32 {
        effectiveness
    }
    
    /// Check if this item should activate based on the damage context (reactive items)
    fn check_reactive_trigger(&self, _context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        ItemModifier::default()
    }
    
    /// Generate instructions for reactive item effects (status curing, healing, stat boosts, etc.)
    fn generate_reactive_instructions(
        &self, 
        _context: &DamageContext, 
        _type_effectiveness: f32,
        _holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        vec![]
    }
    
    /// Generate instructions for end-of-turn item effects
    fn generate_end_of_turn_instructions(
        &self,
        _holder_position: BattlePosition,
        _current_status: &PokemonStatus
    ) -> Vec<BattleInstructions> {
        vec![]
    }
    
    /// Check if this item should activate when a move misses
    fn check_miss_trigger(&self, _holder_position: BattlePosition) -> Vec<BattleInstructions> {
        vec![]
    }
    
    /// Check if this item should activate when affected by an ability
    fn check_ability_trigger(&self, _holder_position: BattlePosition, _ability_name: &str) -> Vec<BattleInstructions> {
        vec![]
    }
    
    /// Get the form change for this item when held by a specific Pokemon species
    fn get_form_change(&self, _pokemon_species: &str) -> Option<String> {
        None
    }
    
    /// Check if this item provides stat changes when changing form
    fn get_form_stat_changes(&self, _pokemon_species: &str) -> Option<HashMap<Stat, i16>> {
        None
    }
}

// =============================================================================
// CHOICE ITEMS (Move Locking + Stat Boost)
// =============================================================================

/// Choice Band - Boosts Attack by 1.5x but locks into first move
#[derive(Debug, Clone)]
pub struct ChoiceBand;

impl ItemEffect for ChoiceBand {
    fn name(&self) -> &str {
        "Choice Band"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.data.category == MoveCategory::Physical {
            ItemModifier::new().with_attack_multiplier(1.5)
        } else {
            ItemModifier::default()
        }
    }
}

/// Choice Specs - Boosts Special Attack by 1.5x but locks into first move
#[derive(Debug, Clone)]
pub struct ChoiceSpecs;

impl ItemEffect for ChoiceSpecs {
    fn name(&self) -> &str {
        "Choice Specs"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.data.category == MoveCategory::Special {
            ItemModifier::new().with_special_attack_multiplier(1.5)
        } else {
            ItemModifier::default()
        }
    }
}

/// Choice Scarf - Boosts Speed by 1.5x but locks into first move
#[derive(Debug, Clone)]
pub struct ChoiceScarf;

impl ItemEffect for ChoiceScarf {
    fn name(&self) -> &str {
        "Choice Scarf"
    }

    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_speed_multiplier(1.5)
    }
}

// =============================================================================
// POWER AMPLIFICATION ITEMS
// =============================================================================

/// Life Orb - Boosts all moves by 1.3x but causes 10% recoil
#[derive(Debug, Clone)]
pub struct LifeOrb;

impl ItemEffect for LifeOrb {
    fn name(&self) -> &str {
        "Life Orb"
    }

    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new()
            .with_power_multiplier(1.3)
            .with_recoil(0.1) // 10% recoil
    }
}

/// Expert Belt - Boosts super effective moves by 1.2x
#[derive(Debug, Clone)]
pub struct ExpertBelt;

impl ItemEffect for ExpertBelt {
    fn name(&self) -> &str {
        "Expert Belt"
    }

    fn boosts_super_effective(&self) -> bool {
        true
    }
    fn super_effective_multiplier(&self) -> f32 {
        1.2
    }
}

/// Muscle Band - Boosts physical moves by 1.1x
#[derive(Debug, Clone)]
pub struct MuscleBand;

impl ItemEffect for MuscleBand {
    fn name(&self) -> &str {
        "Muscle Band"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.data.category == MoveCategory::Physical {
            ItemModifier::new().with_power_multiplier(1.1)
        } else {
            ItemModifier::default()
        }
    }
}

/// Wise Glasses - Boosts special moves by 1.1x
#[derive(Debug, Clone)]
pub struct WiseGlasses;

impl ItemEffect for WiseGlasses {
    fn name(&self) -> &str {
        "Wise Glasses"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.data.category == MoveCategory::Special {
            ItemModifier::new().with_power_multiplier(1.1)
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// SPECIAL GENERATION-SPECIFIC ITEMS
// =============================================================================

/// Sea Incense - Water-type moves with special generation logic
/// Gen 3: 1.05x boost | Gen 4+: 1.2x boost
#[derive(Debug, Clone)]
pub struct SeaIncense;

impl ItemEffect for SeaIncense {
    fn name(&self) -> &str {
        "Sea Incense"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.move_type == "Water" {
            let generation_multiplier = match context.get_generation() {
                crate::generation::Generation::Gen3 => 1.05,
                crate::generation::Generation::Gen4 | crate::generation::Generation::Gen5 |
                crate::generation::Generation::Gen6 | crate::generation::Generation::Gen7 |
                crate::generation::Generation::Gen8 | crate::generation::Generation::Gen9 => 1.2,
                _ => 1.0, // No effect in Gen 1-2
            };
            ItemModifier::new().with_power_multiplier(generation_multiplier)
        } else {
            ItemModifier::default()
        }
    }
}

/// Pink Bow - Normal-type moves with consistent 1.1x boost across all generations
#[derive(Debug, Clone)]
pub struct PinkBow;

impl ItemEffect for PinkBow {
    fn name(&self) -> &str {
        "Pink Bow"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.move_type == "Normal" {
            ItemModifier::new().with_power_multiplier(1.1)
        } else {
            ItemModifier::default()
        }
    }
}

/// Polkadot Bow - Normal-type moves with consistent 1.1x boost across all generations
#[derive(Debug, Clone)]
pub struct PolkadotBow;

impl ItemEffect for PolkadotBow {
    fn name(&self) -> &str {
        "Polkadot Bow"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.move_type == "Normal" {
            ItemModifier::new().with_power_multiplier(1.1)
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// TYPE BOOSTING ITEMS (1.2x power for matching types)
// =============================================================================

/// Generic type booster implementation
#[derive(Debug, Clone)]
pub struct TypeBooster {
    item_name: String,
    boosted_type: String,
}

impl TypeBooster {
    pub fn new(item_name: String, boosted_type: String) -> Self {
        Self {
            item_name,
            boosted_type,
        }
    }

    // Normal type boosters
    pub fn silk_scarf() -> Self {
        Self::new("Silk Scarf".to_string(), "Normal".to_string())
    }

    // Physical type boosters
    pub fn black_belt() -> Self {
        Self::new("Black Belt".to_string(), "Fighting".to_string())
    }
    pub fn black_glasses() -> Self {
        Self::new("Black Glasses".to_string(), "Dark".to_string())
    }
    pub fn charcoal() -> Self {
        Self::new("Charcoal".to_string(), "Fire".to_string())
    }
    pub fn dragon_fang() -> Self {
        Self::new("Dragon Fang".to_string(), "Dragon".to_string())
    }
    pub fn dragon_scale() -> Self {
        Self::new("Dragon Scale".to_string(), "Dragon".to_string())
    }
    pub fn hard_stone() -> Self {
        Self::new("Hard Stone".to_string(), "Rock".to_string())
    }
    pub fn magnet() -> Self {
        Self::new("Magnet".to_string(), "Electric".to_string())
    }
    pub fn metal_coat() -> Self {
        Self::new("Metal Coat".to_string(), "Steel".to_string())
    }
    pub fn mystic_water() -> Self {
        Self::new("Mystic Water".to_string(), "Water".to_string())
    }
    pub fn never_melt_ice() -> Self {
        Self::new("Never-Melt Ice".to_string(), "Ice".to_string())
    }
    pub fn poison_barb() -> Self {
        Self::new("Poison Barb".to_string(), "Poison".to_string())
    }
    pub fn sharp_beak() -> Self {
        Self::new("Sharp Beak".to_string(), "Flying".to_string())
    }
    pub fn silver_powder() -> Self {
        Self::new("Silver Powder".to_string(), "Bug".to_string())
    }
    pub fn soft_sand() -> Self {
        Self::new("Soft Sand".to_string(), "Ground".to_string())
    }
    pub fn spell_tag() -> Self {
        Self::new("Spell Tag".to_string(), "Ghost".to_string())
    }
    pub fn miracle_seed() -> Self {
        Self::new("Miracle Seed".to_string(), "Grass".to_string())
    }
    pub fn twisted_spoon() -> Self {
        Self::new("Twisted Spoon".to_string(), "Psychic".to_string())
    }
    pub fn fairy_feather() -> Self {
        Self::new("Fairy Feather".to_string(), "Fairy".to_string())
    }

    // Incense items
    pub fn wave_incense() -> Self {
        Self::new("Wave Incense".to_string(), "Water".to_string())
    }
    pub fn odd_incense() -> Self {
        Self::new("Odd Incense".to_string(), "Psychic".to_string())
    }
}

impl ItemEffect for TypeBooster {
    fn name(&self) -> &str {
        &self.item_name
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.move_type == self.boosted_type {
            // Generation-aware multipliers:
            // Gen 2-3: 1.1x multiplier
            // Gen 4+: 1.2x multiplier
            let generation_multiplier = match context.get_generation() {
                crate::generation::Generation::Gen2 | crate::generation::Generation::Gen3 => 1.1,
                _ => 1.2, // Gen 4 and later
            };
            ItemModifier::new().with_power_multiplier(generation_multiplier)
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// ARCEUS PLATES (Type Change + Power Boost)
// =============================================================================

/// Arceus plate that changes Judgment type and boosts matching moves
#[derive(Debug, Clone)]
pub struct ArceusPlate {
    plate_name: String,
    plate_type: String,
}

impl ArceusPlate {
    pub fn new(plate_name: String, plate_type: String) -> Self {
        Self {
            plate_name,
            plate_type,
        }
    }

    pub fn fist_plate() -> Self {
        Self::new("Fist Plate".to_string(), "Fighting".to_string())
    }
    pub fn sky_plate() -> Self {
        Self::new("Sky Plate".to_string(), "Flying".to_string())
    }
    pub fn toxic_plate() -> Self {
        Self::new("Toxic Plate".to_string(), "Poison".to_string())
    }
    pub fn earth_plate() -> Self {
        Self::new("Earth Plate".to_string(), "Ground".to_string())
    }
    pub fn stone_plate() -> Self {
        Self::new("Stone Plate".to_string(), "Rock".to_string())
    }
    pub fn insect_plate() -> Self {
        Self::new("Insect Plate".to_string(), "Bug".to_string())
    }
    pub fn spooky_plate() -> Self {
        Self::new("Spooky Plate".to_string(), "Ghost".to_string())
    }
    pub fn iron_plate() -> Self {
        Self::new("Iron Plate".to_string(), "Steel".to_string())
    }
    pub fn flame_plate() -> Self {
        Self::new("Flame Plate".to_string(), "Fire".to_string())
    }
    pub fn splash_plate() -> Self {
        Self::new("Splash Plate".to_string(), "Water".to_string())
    }
    pub fn meadow_plate() -> Self {
        Self::new("Meadow Plate".to_string(), "Grass".to_string())
    }
    pub fn zap_plate() -> Self {
        Self::new("Zap Plate".to_string(), "Electric".to_string())
    }
    pub fn mind_plate() -> Self {
        Self::new("Mind Plate".to_string(), "Psychic".to_string())
    }
    pub fn icicle_plate() -> Self {
        Self::new("Icicle Plate".to_string(), "Ice".to_string())
    }
    pub fn draco_plate() -> Self {
        Self::new("Draco Plate".to_string(), "Dragon".to_string())
    }
    pub fn dread_plate() -> Self {
        Self::new("Dread Plate".to_string(), "Dark".to_string())
    }
    pub fn pixie_plate() -> Self {
        Self::new("Pixie Plate".to_string(), "Fairy".to_string())
    }
}

impl ItemEffect for ArceusPlate {
    fn name(&self) -> &str {
        &self.plate_name
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let mut modifier = ItemModifier::new();

        // Change Judgment to plate type
        if context.move_info.data.name.to_lowercase() == "judgment" {
            modifier.type_change = Some(self.plate_type.clone());
        }

        // Boost matching type moves
        if context.move_info.move_type == self.plate_type {
            modifier.power_multiplier = 1.2;
        }

        modifier
    }
}

// =============================================================================
// GEMS (One-Time Type Boost)
// =============================================================================

/// Gem that boosts a type once then is consumed
#[derive(Debug, Clone)]
pub struct Gem {
    gem_name: String,
    gem_type: String,
    multiplier: f32,
}

impl Gem {
    pub fn new(gem_name: String, gem_type: String, generation: u8) -> Self {
        let multiplier = if generation <= 5 { 1.5 } else { 1.3 };
        Self {
            gem_name,
            gem_type,
            multiplier,
        }
    }

    pub fn normal_gem(gen: u8) -> Self {
        Self::new("Normal Gem".to_string(), "Normal".to_string(), gen)
    }
    pub fn fighting_gem(gen: u8) -> Self {
        Self::new("Fighting Gem".to_string(), "Fighting".to_string(), gen)
    }
    pub fn flying_gem(gen: u8) -> Self {
        Self::new("Flying Gem".to_string(), "Flying".to_string(), gen)
    }
    pub fn poison_gem(gen: u8) -> Self {
        Self::new("Poison Gem".to_string(), "Poison".to_string(), gen)
    }
    pub fn ground_gem(gen: u8) -> Self {
        Self::new("Ground Gem".to_string(), "Ground".to_string(), gen)
    }
    pub fn rock_gem(gen: u8) -> Self {
        Self::new("Rock Gem".to_string(), "Rock".to_string(), gen)
    }
    pub fn bug_gem(gen: u8) -> Self {
        Self::new("Bug Gem".to_string(), "Bug".to_string(), gen)
    }
    pub fn ghost_gem(gen: u8) -> Self {
        Self::new("Ghost Gem".to_string(), "Ghost".to_string(), gen)
    }
    pub fn steel_gem(gen: u8) -> Self {
        Self::new("Steel Gem".to_string(), "Steel".to_string(), gen)
    }
    pub fn fire_gem(gen: u8) -> Self {
        Self::new("Fire Gem".to_string(), "Fire".to_string(), gen)
    }
    pub fn water_gem(gen: u8) -> Self {
        Self::new("Water Gem".to_string(), "Water".to_string(), gen)
    }
    pub fn grass_gem(gen: u8) -> Self {
        Self::new("Grass Gem".to_string(), "Grass".to_string(), gen)
    }
    pub fn electric_gem(gen: u8) -> Self {
        Self::new("Electric Gem".to_string(), "Electric".to_string(), gen)
    }
    pub fn psychic_gem(gen: u8) -> Self {
        Self::new("Psychic Gem".to_string(), "Psychic".to_string(), gen)
    }
    pub fn ice_gem(gen: u8) -> Self {
        Self::new("Ice Gem".to_string(), "Ice".to_string(), gen)
    }
    pub fn dragon_gem(gen: u8) -> Self {
        Self::new("Dragon Gem".to_string(), "Dragon".to_string(), gen)
    }
    pub fn dark_gem(gen: u8) -> Self {
        Self::new("Dark Gem".to_string(), "Dark".to_string(), gen)
    }
    pub fn fairy_gem(gen: u8) -> Self {
        Self::new("Fairy Gem".to_string(), "Fairy".to_string(), gen)
    }
}

impl ItemEffect for Gem {
    fn name(&self) -> &str {
        &self.gem_name
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        if context.move_info.move_type == self.gem_type {
            ItemModifier::new()
                .with_power_multiplier(self.multiplier)
                .consumed() // Gem is consumed after use
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// DAMAGE REDUCTION BERRIES (50% damage reduction)
// =============================================================================

/// Berry that reduces super effective damage by 50%
#[derive(Debug, Clone)]
pub struct DamageReductionBerry {
    berry_name: String,
    resisted_type: String,
}

impl DamageReductionBerry {
    pub fn new(berry_name: String, resisted_type: String) -> Self {
        Self {
            berry_name,
            resisted_type,
        }
    }

    pub fn chople_berry() -> Self {
        Self::new("Chople Berry".to_string(), "Fighting".to_string())
    }
    pub fn coba_berry() -> Self {
        Self::new("Coba Berry".to_string(), "Flying".to_string())
    }
    pub fn kebia_berry() -> Self {
        Self::new("Kebia Berry".to_string(), "Poison".to_string())
    }
    pub fn shuca_berry() -> Self {
        Self::new("Shuca Berry".to_string(), "Ground".to_string())
    }
    pub fn charti_berry() -> Self {
        Self::new("Charti Berry".to_string(), "Rock".to_string())
    }
    pub fn tanga_berry() -> Self {
        Self::new("Tanga Berry".to_string(), "Bug".to_string())
    }
    pub fn kasib_berry() -> Self {
        Self::new("Kasib Berry".to_string(), "Ghost".to_string())
    }
    pub fn babiri_berry() -> Self {
        Self::new("Babiri Berry".to_string(), "Steel".to_string())
    }
    pub fn occa_berry() -> Self {
        Self::new("Occa Berry".to_string(), "Fire".to_string())
    }
    pub fn passho_berry() -> Self {
        Self::new("Passho Berry".to_string(), "Water".to_string())
    }
    pub fn rindo_berry() -> Self {
        Self::new("Rindo Berry".to_string(), "Grass".to_string())
    }
    pub fn wacan_berry() -> Self {
        Self::new("Wacan Berry".to_string(), "Electric".to_string())
    }
    pub fn payapa_berry() -> Self {
        Self::new("Payapa Berry".to_string(), "Psychic".to_string())
    }
    pub fn yache_berry() -> Self {
        Self::new("Yache Berry".to_string(), "Ice".to_string())
    }
    pub fn haban_berry() -> Self {
        Self::new("Haban Berry".to_string(), "Dragon".to_string())
    }
    pub fn colbur_berry() -> Self {
        Self::new("Colbur Berry".to_string(), "Dark".to_string())
    }
    pub fn roseli_berry() -> Self {
        Self::new("Roseli Berry".to_string(), "Fairy".to_string())
    }
    pub fn chilan_berry() -> Self {
        Self::new("Chilan Berry".to_string(), "Normal".to_string())
    }
}

impl ItemEffect for DamageReductionBerry {
    fn name(&self) -> &str {
        &self.berry_name
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
        
        // Check if this move targets the berry's resisted type
        if context.move_info.move_type != self.resisted_type {
            return ItemModifier::default();
        }
        
        // For Chilan Berry, activate on any Normal-type move
        if self.resisted_type == "Normal" {
            return ItemModifier::new().with_power_multiplier(0.5).consumed();
        }
        
        // For other berries, only activate on super effective moves
        // We need to check type effectiveness
        let type_chart = TypeChart::new(9); // Gen 9 type chart
        if let Some(attacking_type) = PokemonType::from_str(&context.move_info.move_type) {
            let defender_type1 = PokemonType::from_str(&context.defender.pokemon.types[0]).unwrap_or(PokemonType::Normal);
            let defender_type2 = if context.defender.pokemon.types.len() > 1 {
                PokemonType::from_str(&context.defender.pokemon.types[1]).unwrap_or(defender_type1)
            } else {
                defender_type1
            };
            
            let effectiveness = type_chart.calculate_damage_multiplier(
                attacking_type,
                (defender_type1, defender_type2),
                None,
                None,
            );
            
            if effectiveness > 1.0 {
                return ItemModifier::new().with_power_multiplier(0.5).consumed();
            }
        }
        
        ItemModifier::default()
    }
}

// =============================================================================
// SPECIES-SPECIFIC ITEMS
// =============================================================================

/// Thick Club - Doubles Attack for Cubone and Marowak
#[derive(Debug, Clone)]
pub struct ThickClub;

impl ItemEffect for ThickClub {
    fn name(&self) -> &str {
        "Thick Club"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let species_name = context.attacker.pokemon.species.to_lowercase();
        if (species_name.contains("cubone") || species_name.contains("marowak"))
            && context.move_info.data.category == MoveCategory::Physical
        {
            ItemModifier::new().with_attack_multiplier(2.0)
        } else {
            ItemModifier::default()
        }
    }
}

/// Light Ball - Doubles Attack and Special Attack for Pikachu
#[derive(Debug, Clone)]
pub struct LightBall;

impl ItemEffect for LightBall {
    fn name(&self) -> &str {
        "Light Ball"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let species_name = context.attacker.pokemon.species.to_lowercase();
        if species_name.contains("pikachu") {
            match context.move_info.data.category {
                MoveCategory::Physical => ItemModifier::new().with_attack_multiplier(2.0),
                MoveCategory::Special => ItemModifier::new().with_special_attack_multiplier(2.0),
                MoveCategory::Status => ItemModifier::default(),
            }
        } else {
            ItemModifier::default()
        }
    }
}

/// Soul Dew - Boosts Latios/Latias moves
#[derive(Debug, Clone)]
pub struct SoulDew {
    generation: u8,
}

impl SoulDew {
    pub fn new(generation: u8) -> Self {
        Self { generation }
    }
}

impl ItemEffect for SoulDew {
    fn name(&self) -> &str {
        "Soul Dew"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let species_name = context.attacker.pokemon.species.to_lowercase();
        if species_name.contains("latios") || species_name.contains("latias") {
            if self.generation >= 7 {
                // Gen 7+: Boost Dragon/Psychic moves by 20%
                if context.move_info.move_type == "Dragon" || context.move_info.move_type == "Psychic" {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            } else {
                // Gen 3-6: Boost special moves by 50%, special defense by 50%
                if context.move_info.data.category == MoveCategory::Special {
                    ItemModifier::new()
                        .with_special_attack_multiplier(1.5)
                        .with_special_defense_multiplier(1.5)
                } else {
                    ItemModifier::new().with_special_defense_multiplier(1.5)
                }
            }
        } else {
            ItemModifier::default()
        }
    }
}

/// Legendary signature orbs
#[derive(Debug, Clone)]
pub struct LegendaryOrb {
    orb_name: String,
    species: String,
    boosted_types: Vec<String>,
}

impl LegendaryOrb {
    pub fn new(orb_name: String, species: String, boosted_types: Vec<String>) -> Self {
        Self {
            orb_name,
            species,
            boosted_types,
        }
    }

    pub fn adamant_orb() -> Self {
        Self::new(
            "Adamant Orb".to_string(),
            "dialga".to_string(),
            vec!["Dragon".to_string(), "Steel".to_string()],
        )
    }

    pub fn lustrous_orb() -> Self {
        Self::new(
            "Lustrous Orb".to_string(),
            "palkia".to_string(),
            vec!["Dragon".to_string(), "Water".to_string()],
        )
    }

    pub fn griseous_orb() -> Self {
        Self::new(
            "Griseous Orb".to_string(),
            "giratina".to_string(),
            vec!["Dragon".to_string(), "Ghost".to_string()],
        )
    }
}

impl ItemEffect for LegendaryOrb {
    fn name(&self) -> &str {
        &self.orb_name
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let species_name = context.attacker.pokemon.species.to_lowercase();
        if species_name.contains(&self.species) && self.boosted_types.contains(&context.move_info.move_type) {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// DEFENSIVE ITEMS
// =============================================================================

/// Eviolite - Boosts Defense and Special Defense by 1.5x for not fully evolved Pokemon
#[derive(Debug, Clone)]
pub struct Eviolite;

impl ItemEffect for Eviolite {
    fn name(&self) -> &str {
        "Eviolite"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }

    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new()
            .with_defense_multiplier(1.5)
            .with_special_defense_multiplier(1.5)
    }
}

/// Assault Vest - Boosts Special Defense by 1.5x but prevents status moves
#[derive(Debug, Clone)]
pub struct AssaultVest;

impl ItemEffect for AssaultVest {
    fn name(&self) -> &str {
        "Assault Vest"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Assault Vest provides 1.5x Special Defense when held by defender
        // Note: The move blocking effect should be handled in move selection, not damage calc
        if context.move_info.data.category == MoveCategory::Special {
            ItemModifier::new().with_special_defense_multiplier(1.5)
        } else {
            ItemModifier::default()
        }
    }
}

/// Air Balloon - Provides Ground immunity until hit by damaging move
#[derive(Debug, Clone)]
pub struct AirBalloon;

impl ItemEffect for AirBalloon {
    fn name(&self) -> &str {
        "Air Balloon"
    }

    fn provides_immunity(&self, move_type: &str) -> bool {
        move_type == "Ground"
    }

    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_ground_immunity()
    }
}

/// Heavy Duty Boots - Provides hazard immunity
#[derive(Debug, Clone)]
pub struct HeavyDutyBoots;

impl ItemEffect for HeavyDutyBoots {
    fn name(&self) -> &str {
        "Heavy-Duty Boots"
    }

    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_hazard_immunity()
    }
}

/// Rocky Helmet - Contact moves cause 1/6 max HP recoil to attacker
#[derive(Debug, Clone)]
pub struct RockyHelmet;

impl ItemEffect for RockyHelmet {
    fn name(&self) -> &str {
        "Rocky Helmet"
    }

    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Check if move makes contact (would need flag checking)
        if context.move_info.data.flags.contains(&"contact".to_string()) {
            ItemModifier::new().with_contact_recoil(1.0 / 6.0) // 1/6 max HP
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// REACTIVE ITEMS (Triggered by incoming damage/effects)
// =============================================================================

/// Weakness Policy - +2 Attack/Special Attack when hit by super effective moves
#[derive(Debug, Clone)]
pub struct WeaknessPolicy;

impl ItemEffect for WeaknessPolicy {
    fn name(&self) -> &str {
        "Weakness Policy"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, type_effectiveness: f32) -> ItemModifier {
        if context.move_info.data.category != MoveCategory::Status && type_effectiveness > 1.0 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::attack_and_special_attack(2, 2))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Focus Sash - Survive any attack that would KO at full HP
#[derive(Debug, Clone)]
pub struct FocusSash;

impl ItemEffect for FocusSash {
    fn name(&self) -> &str {
        "Focus Sash"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Check if defender is at full HP
        if context.defender.pokemon.hp == context.defender.pokemon.max_hp {
            ItemModifier::new()
                .with_ko_prevention()
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Absorb Bulb - +1 Special Attack when hit by Water moves
#[derive(Debug, Clone)]
pub struct AbsorbBulb;

impl ItemEffect for AbsorbBulb {
    fn name(&self) -> &str {
        "Absorb Bulb"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.move_info.move_type == "Water" {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::special_attack(1))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Cell Battery - +1 Attack when hit by Electric moves
#[derive(Debug, Clone)]
pub struct CellBattery;

impl ItemEffect for CellBattery {
    fn name(&self) -> &str {
        "Cell Battery"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.move_info.move_type == "Electric" {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::attack(1))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Shell Bell - Restore 1/8 of damage dealt as HP
#[derive(Debug, Clone)]
pub struct ShellBell;

impl ItemEffect for ShellBell {
    fn name(&self) -> &str {
        "Shell Bell"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_drain(0.125) // 1/8 = 12.5%
    }
}

/// Leftovers - Restore 1/16 max HP each turn
#[derive(Debug, Clone)]
pub struct Leftovers;

impl ItemEffect for Leftovers {
    fn name(&self) -> &str {
        "Leftovers"
    }
    
    fn generate_end_of_turn_instructions(
        &self,
        holder_position: BattlePosition,
        _current_status: &PokemonStatus
    ) -> Vec<BattleInstructions> {
        // Leftovers heal 1/16 of max HP each turn
        // We'll use a placeholder amount that the battle engine can calculate properly
        let heal_instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
            target: holder_position,
            amount: 1, // Placeholder - battle engine should calculate 1/16 max HP
            previous_hp: None,
        });
        
        vec![BattleInstructions::new(100.0, vec![heal_instruction])]
    }
}

/// Metal Powder - Reduce damage by 50% when held by Ditto
#[derive(Debug, Clone)]
pub struct MetalPowder;

impl ItemEffect for MetalPowder {
    fn name(&self) -> &str {
        "Metal Powder"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Metal Powder only works when the defender (item holder) is Ditto
        let species_name = context.defender.pokemon.species.to_lowercase();
        if species_name == "ditto" {
            ItemModifier::new().with_damage_multiplier(0.5)
        } else {
            ItemModifier::default()
        }
    }
}

/// Punching Glove - 1.1x punch moves, removes contact, no Iron Fist boost
#[derive(Debug, Clone)]
pub struct PunchingGlove;

impl ItemEffect for PunchingGlove {
    fn name(&self) -> &str {
        "Punching Glove"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Check if move has punch flag
        if context.move_info.data.flags.contains(&"punch".to_string()) {
            ItemModifier::new()
                .with_power_multiplier(1.1)
                .with_contact_removal()
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// STATUS BERRIES (Consumable, cure status conditions)
// =============================================================================

/// Pecha Berry - Cures poison status
#[derive(Debug, Clone)]
pub struct PechaBerry;

impl ItemEffect for PechaBerry {
    fn name(&self) -> &str {
        "Pecha Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if matches!(context.defender.pokemon.status, PokemonStatus::Poison | PokemonStatus::Toxic) {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if matches!(context.defender.pokemon.status, PokemonStatus::Poison | PokemonStatus::Toxic) {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: PokemonStatus::Poison,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Pecha Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Cheri Berry - Cures paralysis status
#[derive(Debug, Clone)]
pub struct CheriBerry;

impl ItemEffect for CheriBerry {
    fn name(&self) -> &str {
        "Cheri Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.defender.pokemon.status == PokemonStatus::Paralysis {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if context.defender.pokemon.status == PokemonStatus::Paralysis {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: PokemonStatus::Paralysis,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Cheri Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Rawst Berry - Cures burn status
#[derive(Debug, Clone)]
pub struct RawstBerry;

impl ItemEffect for RawstBerry {
    fn name(&self) -> &str {
        "Rawst Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.defender.pokemon.status == PokemonStatus::Burn {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if context.defender.pokemon.status == PokemonStatus::Burn {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: PokemonStatus::Burn,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Rawst Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Aspear Berry - Cures freeze status
#[derive(Debug, Clone)]
pub struct AspearBerry;

impl ItemEffect for AspearBerry {
    fn name(&self) -> &str {
        "Aspear Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.defender.pokemon.status == PokemonStatus::Freeze {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if context.defender.pokemon.status == PokemonStatus::Freeze {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: PokemonStatus::Freeze,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Aspear Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Lum Berry - Cures all status conditions
#[derive(Debug, Clone)]
pub struct LumBerry;

impl ItemEffect for LumBerry {
    fn name(&self) -> &str {
        "Lum Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.defender.pokemon.status != PokemonStatus::None {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if context.defender.pokemon.status != PokemonStatus::None {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: context.defender.pokemon.status,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Lum Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Sitrus Berry - Heals HP when below threshold
#[derive(Debug, Clone)]
pub struct SitrusBerry {
    generation: u8,
}

impl SitrusBerry {
    pub fn new(generation: u8) -> Self {
        Self { generation }
    }
}

impl ItemEffect for SitrusBerry {
    fn name(&self) -> &str {
        "Sitrus Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Check HP threshold based on generation
        let threshold = if self.generation >= 4 { 0.25 } else { 0.5 };
        let hp_percentage = context.defender.pokemon.hp as f32 / context.defender.pokemon.max_hp as f32;
        
        if hp_percentage <= threshold {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        // Check HP threshold based on generation
        let threshold = if self.generation >= 4 { 0.25 } else { 0.5 };
        let hp_percentage = context.defender.pokemon.hp as f32 / context.defender.pokemon.max_hp as f32;
        
        if hp_percentage <= threshold {
            // Heal 1/4 of max HP or remaining HP, whichever is less (following V1 pattern)
            let heal_amount = std::cmp::min(
                context.defender.pokemon.max_hp / 4,
                context.defender.pokemon.max_hp - context.defender.pokemon.hp
            );
            
            let instructions = vec![
                BattleInstruction::Pokemon(PokemonInstruction::Heal {
                    target: holder_position,
                    amount: heal_amount,
                    previous_hp: Some(0),
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Sitrus Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Chesto Berry - Cures sleep status
#[derive(Debug, Clone)]
pub struct ChestoBerry;

impl ItemEffect for ChestoBerry {
    fn name(&self) -> &str {
        "Chesto Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        if context.defender.pokemon.status == PokemonStatus::Sleep {
            ItemModifier::new().consumed()
        } else {
            ItemModifier::default()
        }
    }
    
    fn generate_reactive_instructions(
        &self, 
        context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        if context.defender.pokemon.status == PokemonStatus::Sleep {
            let instructions = vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: holder_position,
                    status: PokemonStatus::Sleep,
                    previous_duration: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: holder_position,
                    new_item: None,
                    previous_item: Some("Chesto Berry".to_string()),
                })
            ];
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            vec![]
        }
    }
}

/// Miracle Berry - Gen 2 exclusive, cures all status conditions
#[derive(Debug, Clone)]
pub struct MiracleBerry;

impl ItemEffect for MiracleBerry {
    fn name(&self) -> &str {
        "Miracle Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Only available in Gen 2
        match context.get_generation() {
            crate::generation::Generation::Gen2 => {
                if context.defender.pokemon.status != PokemonStatus::None {
                    ItemModifier::new().consumed()
                } else {
                    ItemModifier::default()
                }
            },
            _ => ItemModifier::default(), // No effect in other generations
        }
    }
}

/// Mint Berry - Gen 2 exclusive, cures sleep status
#[derive(Debug, Clone)]
pub struct MintBerry;

impl ItemEffect for MintBerry {
    fn name(&self) -> &str {
        "Mint Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Only available in Gen 2
        match context.get_generation() {
            crate::generation::Generation::Gen2 => {
                if context.defender.pokemon.status == PokemonStatus::Sleep {
                    ItemModifier::new().consumed()
                } else {
                    ItemModifier::default()
                }
            },
            _ => ItemModifier::default(), // No effect in other generations
        }
    }
}

// =============================================================================
// STAT BOOST BERRIES (Consumable, activate at 25% HP)
// =============================================================================

/// Liechi Berry - +1 Attack when HP ≤ 25%
#[derive(Debug, Clone)]
pub struct LiechiBerry;

impl ItemEffect for LiechiBerry {
    fn name(&self) -> &str {
        "Liechi Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        let hp_percentage = context.defender.pokemon.hp as f32 / context.defender.pokemon.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::attack(1))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Petaya Berry - +1 Special Attack when HP ≤ 25%
#[derive(Debug, Clone)]
pub struct PetayaBerry;

impl ItemEffect for PetayaBerry {
    fn name(&self) -> &str {
        "Petaya Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        let hp_percentage = context.defender.pokemon.hp as f32 / context.defender.pokemon.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::special_attack(1))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Salac Berry - +1 Speed when HP ≤ 25%
#[derive(Debug, Clone)]
pub struct SalacBerry;

impl ItemEffect for SalacBerry {
    fn name(&self) -> &str {
        "Salac Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        let hp_percentage = context.defender.pokemon.hp as f32 / context.defender.pokemon.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts {
                    attack: 0,
                    defense: 0,
                    special_attack: 0,
                    special_defense: 0,
                    speed: 1,
                    accuracy: 0,
                })
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// TERRAIN SEEDS (Consumable, terrain-dependent stat boosts)
// =============================================================================

/// Electric Seed - +1 Defense when Electric Terrain is active
#[derive(Debug, Clone)]
pub struct ElectricSeed;

impl ItemEffect for ElectricSeed {
    fn name(&self) -> &str {
        "Electric Seed"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        ItemModifier::new().consumed()
    }
    
    fn generate_reactive_instructions(
        &self, 
        _context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Defense, 1);
        
        let instructions = vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: holder_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            }),
            BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: holder_position,
                new_item: None,
                previous_item: Some("Electric Seed".to_string()),
            })
        ];
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

/// Grassy Seed - +1 Defense when Grassy Terrain is active
#[derive(Debug, Clone)]
pub struct GrassySeed;

impl ItemEffect for GrassySeed {
    fn name(&self) -> &str {
        "Grassy Seed"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        ItemModifier::new().consumed()
    }
    
    fn generate_reactive_instructions(
        &self, 
        _context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Defense, 1);
        
        let instructions = vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: holder_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            }),
            BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: holder_position,
                new_item: None,
                previous_item: Some("Grassy Seed".to_string()),
            })
        ];
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

/// Misty Seed - +1 Special Defense when Misty Terrain is active
#[derive(Debug, Clone)]
pub struct MistySeed;

impl ItemEffect for MistySeed {
    fn name(&self) -> &str {
        "Misty Seed"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        ItemModifier::new().consumed()
    }
    
    fn generate_reactive_instructions(
        &self, 
        _context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::SpecialDefense, 1);
        
        let instructions = vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: holder_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            }),
            BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: holder_position,
                new_item: None,
                previous_item: Some("Misty Seed".to_string()),
            })
        ];
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

/// Psychic Seed - +1 Special Defense when Psychic Terrain is active
#[derive(Debug, Clone)]
pub struct PsychicSeed;

impl ItemEffect for PsychicSeed {
    fn name(&self) -> &str {
        "Psychic Seed"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        ItemModifier::new().consumed()
    }
    
    fn generate_reactive_instructions(
        &self, 
        _context: &DamageContext, 
        _type_effectiveness: f32,
        holder_position: BattlePosition
    ) -> Vec<BattleInstructions> {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::SpecialDefense, 1);
        
        let instructions = vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: holder_position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            }),
            BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: holder_position,
                new_item: None,
                previous_item: Some("Psychic Seed".to_string()),
            })
        ];
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

// =============================================================================
// END-OF-TURN STATUS ITEMS (Inflict status at end of turn)
// =============================================================================

/// Black Sludge - Heals Poison-types, damages others
#[derive(Debug, Clone)]
pub struct BlackSludge;

impl ItemEffect for BlackSludge {
    fn name(&self) -> &str {
        "Black Sludge"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn generate_end_of_turn_instructions(
        &self,
        holder_position: BattlePosition,
        _current_status: &PokemonStatus
    ) -> Vec<BattleInstructions> {
        // Black Sludge heals Poison-types by 1/16 max HP, damages others by 1/8 max HP
        // Since we can't check Pokemon type here, this is a placeholder implementation
        // The battle engine should check the Pokemon's type and apply the appropriate effect
        
        // For now, create a generic instruction that the battle engine can interpret
        // In a more complete implementation, this would check the Pokemon's types
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
            target: holder_position,
            amount: 0, // Placeholder - battle engine should check type and heal/damage accordingly
            previous_hp: None,
        });
        
        vec![BattleInstructions::new(100.0, vec![instruction])]
    }
}

/// Flame Orb - Inflicts burn status at end of turn
#[derive(Debug, Clone)]
pub struct FlameOrb;

impl ItemEffect for FlameOrb {
    fn name(&self) -> &str {
        "Flame Orb"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn generate_end_of_turn_instructions(
        &self,
        holder_position: BattlePosition,
        current_status: &PokemonStatus
    ) -> Vec<BattleInstructions> {
        // Only apply burn if Pokemon doesn't already have a status condition
        if *current_status == PokemonStatus::None {
            let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                target: holder_position,
                status: PokemonStatus::Burn,
                duration: None,
                previous_status: Some(PokemonStatus::None),
                previous_duration: None,
            });
            vec![BattleInstructions::new(100.0, vec![instruction])]
        } else {
            vec![]
        }
    }
}

/// Toxic Orb - Inflicts badly poisoned status at end of turn
#[derive(Debug, Clone)]
pub struct ToxicOrb;

impl ItemEffect for ToxicOrb {
    fn name(&self) -> &str {
        "Toxic Orb"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn generate_end_of_turn_instructions(
        &self,
        holder_position: BattlePosition,
        current_status: &PokemonStatus
    ) -> Vec<BattleInstructions> {
        // Only apply badly poisoned if Pokemon doesn't already have a status condition
        if *current_status == PokemonStatus::None {
            let instruction = BattleInstruction::Status(StatusInstruction::Apply {
                target: holder_position,
                status: PokemonStatus::Toxic,
                duration: None,
                previous_status: Some(PokemonStatus::None),
                previous_duration: None,
            });
            vec![BattleInstructions::new(100.0, vec![instruction])]
        } else {
            vec![]
        }
    }
}

// =============================================================================
// UTILITY ITEMS (Various battle mechanics)
// =============================================================================

/// Protective Pads - Removes contact flag from moves
#[derive(Debug, Clone)]
pub struct ProtectivePads;

impl ItemEffect for ProtectivePads {
    fn name(&self) -> &str {
        "Protective Pads"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_contact_removal()
    }
}

/// Throat Spray - +1 Special Attack when using sound moves
#[derive(Debug, Clone)]
pub struct ThroatSpray;

impl ItemEffect for ThroatSpray {
    fn name(&self) -> &str {
        "Throat Spray"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn check_reactive_trigger(&self, context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Check if move has sound flag
        if context.move_info.data.flags.contains(&"sound".to_string()) {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::special_attack(1))
                .consumed()
        } else {
            ItemModifier::default()
        }
    }
}

/// Wide Lens - Increases move accuracy by 1.1x
#[derive(Debug, Clone)]
pub struct WideLens;

impl ItemEffect for WideLens {
    fn name(&self) -> &str {
        "Wide Lens"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        // Wide Lens increases move accuracy by 1.1x
        ItemModifier::default().with_accuracy_multiplier(1.1)
    }
}

/// Zoom Lens - Increases accuracy when moving after target
#[derive(Debug, Clone)]
pub struct ZoomLens;

impl ItemEffect for ZoomLens {
    fn name(&self) -> &str {
        "Zoom Lens"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Zoom Lens increases accuracy by 20% (1.2x) when user moves after target
        // For now, we'll implement a simple version that checks if the user has moved
        // A more complete implementation would require move order context
        
        // This is a placeholder - ideally we'd check if user moves after specific target
        // For now, apply the boost if it seems likely the user moved second
        ItemModifier::default().with_accuracy_multiplier(1.2)
    }
}

/// Iron Ball - Halves speed, makes user grounded
#[derive(Debug, Clone)]
pub struct IronBall;

impl ItemEffect for IronBall {
    fn name(&self) -> &str {
        "Iron Ball"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        ItemModifier::new().with_speed_multiplier(0.5)
    }
}

/// Loaded Dice - Multi-hit moves always hit maximum number of times
#[derive(Debug, Clone)]
pub struct LoadedDice;

impl ItemEffect for LoadedDice {
    fn name(&self) -> &str {
        "Loaded Dice"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    // Multi-hit move logic is now handled in move_effects.rs apply_multi_hit_move function
    // Loaded Dice forces multi-hit moves to always hit the maximum number of times (5)
}

/// Blunder Policy - +2 Speed when missing a move
#[derive(Debug, Clone)]
pub struct BlunderPolicy;

impl ItemEffect for BlunderPolicy {
    fn name(&self) -> &str {
        "Blunder Policy"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn check_reactive_trigger(&self, _context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Blunder Policy activates on move miss, which is handled by check_miss_trigger
        ItemModifier::default()
    }
    
    fn check_miss_trigger(&self, holder_position: BattlePosition) -> Vec<BattleInstructions> {
        // Blunder Policy: +2 Speed when missing a move (single use)
        let mut stat_changes = HashMap::new();
        stat_changes.insert(Stat::Speed, 2);
        let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: holder_position,
            stat_changes: stat_changes,
            previous_boosts: HashMap::new(),
        });
        
        // Also consume the item since it's single use
        let consume_instruction = BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: holder_position,
            new_item: None,
            previous_item: Some("Blunder Policy".to_string()),
        });
        
        vec![BattleInstructions::new(100.0, vec![instruction, consume_instruction])]
    }
}

/// Custap Berry - Provides +1 priority when HP ≤ 25%
#[derive(Debug, Clone)]
pub struct CustapBerry;

impl ItemEffect for CustapBerry {
    fn name(&self) -> &str {
        "Custap Berry"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        // Custap Berry increases priority when HP ≤ 25%
        let pokemon = &context.attacker.pokemon;
        let hp_threshold = pokemon.max_hp / 4; // 25% of max HP
        if pokemon.hp <= hp_threshold {
            return ItemModifier::default()
                .with_priority_modifier(1)
                .consumed();
        }
        ItemModifier::default()
    }
}

/// Quick Claw - May move first regardless of speed (20% chance)
#[derive(Debug, Clone)]
pub struct QuickClaw;

impl ItemEffect for QuickClaw {
    fn name(&self) -> &str {
        "Quick Claw"
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, _context: &DamageContext) -> ItemModifier {
        // Quick Claw has a 20% chance to provide +1 priority
        // For simplicity, we'll apply it deterministically in this implementation
        // A more complete implementation would use probability branching
        ItemModifier::default().with_priority_modifier(1)
    }
}

/// Adrenaline Orb - +1 Speed when intimidated
#[derive(Debug, Clone)]
pub struct AdrenalineOrb;

impl ItemEffect for AdrenalineOrb {
    fn name(&self) -> &str {
        "Adrenaline Orb"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        true
    }
    
    fn check_reactive_trigger(&self, _context: &DamageContext, _type_effectiveness: f32) -> ItemModifier {
        // Adrenaline Orb activates on Intimidate, which is handled by check_ability_trigger
        ItemModifier::default()
    }
    
    fn check_ability_trigger(&self, holder_position: BattlePosition, ability_name: &str) -> Vec<BattleInstructions> {
        // Adrenaline Orb: +1 Speed when intimidated (single use)
        if ability_name.to_lowercase() == "intimidate" {
            let mut stat_changes = HashMap::new();
            stat_changes.insert(Stat::Speed, 1);
            let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: holder_position,
                stat_changes: stat_changes,
                previous_boosts: HashMap::new(),
            });
            
            // Consume the item since it's single use
            let consume_instruction = BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: holder_position,
                new_item: None,
                previous_item: Some("Adrenaline Orb".to_string()),
            });
            
            return vec![BattleInstructions::new(100.0, vec![instruction, consume_instruction])];
        }
        
        vec![]
    }
}

/// Booster Energy - Activates Protosynthesis/Quark Drive abilities
#[derive(Debug, Clone)]
pub struct BoosterEnergy;

impl ItemEffect for BoosterEnergy {
    fn name(&self) -> &str {
        "Booster Energy"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    // TODO: Ability activation needs to be handled in ability system
}

// =============================================================================
// LEGENDARY/MYTHICAL ITEMS (Form changes and signature items)
// =============================================================================

/// Rusted Sword - Zacian forme item
#[derive(Debug, Clone)]
pub struct RustedSword;

impl ItemEffect for RustedSword {
    fn name(&self) -> &str {
        "Rusted Sword"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn get_form_change(&self, pokemon_species: &str) -> Option<String> {
        match pokemon_species.to_lowercase().as_str() {
            "zacian" => Some("Zacian-Crowned".to_string()),
            _ => None,
        }
    }
    
    fn get_form_stat_changes(&self, pokemon_species: &str) -> Option<HashMap<Stat, i16>> {
        if pokemon_species.to_lowercase() == "zacian" {
            // Crowned form stat changes: +10 Attack, +10 Defense
            let mut stat_changes = HashMap::new();
            stat_changes.insert(Stat::Attack, 10);
            stat_changes.insert(Stat::Defense, 10);
            Some(stat_changes)
        } else {
            None
        }
    }
}

/// Rusted Shield - Zamazenta forme item
#[derive(Debug, Clone)]
pub struct RustedShield;

impl ItemEffect for RustedShield {
    fn name(&self) -> &str {
        "Rusted Shield"
    }
    
    fn is_attacker_item(&self) -> bool {
        false
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn get_form_change(&self, pokemon_species: &str) -> Option<String> {
        match pokemon_species.to_lowercase().as_str() {
            "zamazenta" => Some("Zamazenta-Crowned".to_string()),
            _ => None,
        }
    }
    
    fn get_form_stat_changes(&self, pokemon_species: &str) -> Option<HashMap<Stat, i16>> {
        if pokemon_species.to_lowercase() == "zamazenta" {
            // Crowned form stat changes: +10 Defense, +10 Special Defense
            let mut stat_changes = HashMap::new();
            stat_changes.insert(Stat::Defense, 10);
            stat_changes.insert(Stat::SpecialDefense, 10);
            Some(stat_changes)
        } else {
            None
        }
    }
}

/// Ogerpon Masks - 1.2x power boost for specific Ogerpon forms
#[derive(Debug, Clone)]
pub struct OgerponMask {
    mask_name: String,
    forme_name: String,
}

impl OgerponMask {
    pub fn cornerstone_mask() -> Self {
        Self {
            mask_name: "Cornerstone Mask".to_string(),
            forme_name: "ogerpon-cornerstone".to_string(),
        }
    }
    
    pub fn hearthflame_mask() -> Self {
        Self {
            mask_name: "Hearthflame Mask".to_string(),
            forme_name: "ogerpon-hearthflame".to_string(),
        }
    }
    
    pub fn wellspring_mask() -> Self {
        Self {
            mask_name: "Wellspring Mask".to_string(),
            forme_name: "ogerpon-wellspring".to_string(),
        }
    }
}

impl ItemEffect for OgerponMask {
    fn name(&self) -> &str {
        &self.mask_name
    }
    
    fn is_attacker_item(&self) -> bool {
        true
    }
    
    fn is_defender_item(&self) -> bool {
        false
    }
    
    fn modify_damage(&self, context: &DamageContext) -> ItemModifier {
        let species_name = context.attacker.pokemon.species.to_lowercase();
        if species_name.contains("ogerpon") && species_name.contains(&self.forme_name.split('-').last().unwrap_or("")) {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    }
}

// =============================================================================
// ITEM LOOKUP AND CALCULATION FUNCTIONS
// =============================================================================

/// Get item effect by name with generation awareness
pub fn get_item_by_name_with_generation(item_name: &str, generation: u8) -> Option<Box<dyn ItemEffect>> {
    let normalized_name = item_name.to_lowercase().replace(" ", "").replace("-", "");

    match normalized_name.as_str() {
        // Choice Items
        "choiceband" => Some(Box::new(ChoiceBand)),
        "choicespecs" => Some(Box::new(ChoiceSpecs)),
        "choicescarf" => Some(Box::new(ChoiceScarf)),

        // Power Items
        "lifeorb" => Some(Box::new(LifeOrb)),
        "expertbelt" => Some(Box::new(ExpertBelt)),
        "muscleband" => Some(Box::new(MuscleBand)),
        "wiseglasses" => Some(Box::new(WiseGlasses)),

        // Type Boosters
        "silkscarf" => Some(Box::new(TypeBooster::silk_scarf())),
        "pinkbow" => Some(Box::new(PinkBow)),
        "polkadotbow" => Some(Box::new(PolkadotBow)),
        "blackbelt" => Some(Box::new(TypeBooster::black_belt())),
        "blackglasses" => Some(Box::new(TypeBooster::black_glasses())),
        "charcoal" => Some(Box::new(TypeBooster::charcoal())),
        "dragonfang" => Some(Box::new(TypeBooster::dragon_fang())),
        "dragonscale" => Some(Box::new(TypeBooster::dragon_scale())),
        "hardstone" => Some(Box::new(TypeBooster::hard_stone())),
        "magnet" => Some(Box::new(TypeBooster::magnet())),
        "metalcoat" => Some(Box::new(TypeBooster::metal_coat())),
        "mysticwater" => Some(Box::new(TypeBooster::mystic_water())),
        "nevermeltice" => Some(Box::new(TypeBooster::never_melt_ice())),
        "poisonbarb" => Some(Box::new(TypeBooster::poison_barb())),
        "sharpbeak" => Some(Box::new(TypeBooster::sharp_beak())),
        "silverpowder" => Some(Box::new(TypeBooster::silver_powder())),
        "softsand" => Some(Box::new(TypeBooster::soft_sand())),
        "spelltag" => Some(Box::new(TypeBooster::spell_tag())),
        "miracleseed" => Some(Box::new(TypeBooster::miracle_seed())),
        "twistedspoon" => Some(Box::new(TypeBooster::twisted_spoon())),
        "fairyfeather" => Some(Box::new(TypeBooster::fairy_feather())),
        "seaincense" => Some(Box::new(SeaIncense)),
        "waveincense" => Some(Box::new(TypeBooster::wave_incense())),
        "oddincense" => Some(Box::new(TypeBooster::odd_incense())),

        // Arceus Plates
        "fistplate" => Some(Box::new(ArceusPlate::fist_plate())),
        "skyplate" => Some(Box::new(ArceusPlate::sky_plate())),
        "toxicplate" => Some(Box::new(ArceusPlate::toxic_plate())),
        "earthplate" => Some(Box::new(ArceusPlate::earth_plate())),
        "stoneplate" => Some(Box::new(ArceusPlate::stone_plate())),
        "insectplate" => Some(Box::new(ArceusPlate::insect_plate())),
        "spookyplate" => Some(Box::new(ArceusPlate::spooky_plate())),
        "ironplate" => Some(Box::new(ArceusPlate::iron_plate())),
        "flameplate" => Some(Box::new(ArceusPlate::flame_plate())),
        "splashplate" => Some(Box::new(ArceusPlate::splash_plate())),
        "meadowplate" => Some(Box::new(ArceusPlate::meadow_plate())),
        "zapplate" => Some(Box::new(ArceusPlate::zap_plate())),
        "mindplate" => Some(Box::new(ArceusPlate::mind_plate())),
        "icicleplate" => Some(Box::new(ArceusPlate::icicle_plate())),
        "dracoplate" => Some(Box::new(ArceusPlate::draco_plate())),
        "dreadplate" => Some(Box::new(ArceusPlate::dread_plate())),
        "pixieplate" => Some(Box::new(ArceusPlate::pixie_plate())),

        // Damage Reduction Berries
        "chopleberry" => Some(Box::new(DamageReductionBerry::chople_berry())),
        "cobaberry" => Some(Box::new(DamageReductionBerry::coba_berry())),
        "kebiaberry" => Some(Box::new(DamageReductionBerry::kebia_berry())),
        "shucaberry" => Some(Box::new(DamageReductionBerry::shuca_berry())),
        "chartiberry" => Some(Box::new(DamageReductionBerry::charti_berry())),
        "tangaberry" => Some(Box::new(DamageReductionBerry::tanga_berry())),
        "kasibberry" => Some(Box::new(DamageReductionBerry::kasib_berry())),
        "babiriberry" => Some(Box::new(DamageReductionBerry::babiri_berry())),
        "occaberry" => Some(Box::new(DamageReductionBerry::occa_berry())),
        "passhoberry" => Some(Box::new(DamageReductionBerry::passho_berry())),
        "rindoberry" => Some(Box::new(DamageReductionBerry::rindo_berry())),
        "wacanberry" => Some(Box::new(DamageReductionBerry::wacan_berry())),
        "payapaberry" => Some(Box::new(DamageReductionBerry::payapa_berry())),
        "yacheberry" => Some(Box::new(DamageReductionBerry::yache_berry())),
        "habanberry" => Some(Box::new(DamageReductionBerry::haban_berry())),
        "colburberry" => Some(Box::new(DamageReductionBerry::colbur_berry())),
        "roseliberry" => Some(Box::new(DamageReductionBerry::roseli_berry())),
        "chilanberry" => Some(Box::new(DamageReductionBerry::chilan_berry())),

        // Species-Specific Items
        "thickclub" => Some(Box::new(ThickClub)),
        "lightball" => Some(Box::new(LightBall)),
        "souldew" => Some(Box::new(SoulDew::new(generation))),
        "adamantorb" => Some(Box::new(LegendaryOrb::adamant_orb())),
        "lustrousorb" => Some(Box::new(LegendaryOrb::lustrous_orb())),
        "griseousorb" => Some(Box::new(LegendaryOrb::griseous_orb())),

        // Gems (generation-aware multipliers)
        "normalgem" => Some(Box::new(Gem::normal_gem(generation))),
        "fightinggem" => Some(Box::new(Gem::fighting_gem(generation))),
        "flyinggem" => Some(Box::new(Gem::flying_gem(generation))),
        "poisongem" => Some(Box::new(Gem::poison_gem(generation))),
        "groundgem" => Some(Box::new(Gem::ground_gem(generation))),
        "rockgem" => Some(Box::new(Gem::rock_gem(generation))),
        "buggem" => Some(Box::new(Gem::bug_gem(generation))),
        "ghostgem" => Some(Box::new(Gem::ghost_gem(generation))),
        "steelgem" => Some(Box::new(Gem::steel_gem(generation))),
        "firegem" => Some(Box::new(Gem::fire_gem(generation))),
        "watergem" => Some(Box::new(Gem::water_gem(generation))),
        "grassgem" => Some(Box::new(Gem::grass_gem(generation))),
        "electricgem" => Some(Box::new(Gem::electric_gem(generation))),
        "psychicgem" => Some(Box::new(Gem::psychic_gem(generation))),
        "icegem" => Some(Box::new(Gem::ice_gem(generation))),
        "dragongem" => Some(Box::new(Gem::dragon_gem(generation))),
        "darkgem" => Some(Box::new(Gem::dark_gem(generation))),
        "fairygem" => Some(Box::new(Gem::fairy_gem(generation))),

        // Defensive Items
        "eviolite" => Some(Box::new(Eviolite)),
        "assaultvest" => Some(Box::new(AssaultVest)),
        "airballoon" => Some(Box::new(AirBalloon)),
        "heavydutyboots" => Some(Box::new(HeavyDutyBoots)),
        "rockyhelmet" => Some(Box::new(RockyHelmet)),

        // Reactive Items
        "weaknesspolicy" => Some(Box::new(WeaknessPolicy)),
        "focussash" => Some(Box::new(FocusSash)),
        "absorbbulb" => Some(Box::new(AbsorbBulb)),
        "cellbattery" => Some(Box::new(CellBattery)),
        "shellbell" => Some(Box::new(ShellBell)),
        "leftovers" => Some(Box::new(Leftovers)),
        "metalpowder" => Some(Box::new(MetalPowder)),
        "punchingglove" => Some(Box::new(PunchingGlove)),

        // Status Berries
        "lumberry" => Some(Box::new(LumBerry)),
        "sitrusberry" => Some(Box::new(SitrusBerry::new(generation))),
        "chestoberry" => Some(Box::new(ChestoBerry)),
        "miracleberry" => Some(Box::new(MiracleBerry)),
        "mintberry" => Some(Box::new(MintBerry)),

        // Stat Boost Berries
        "liechiberry" => Some(Box::new(LiechiBerry)),
        "petayaberry" => Some(Box::new(PetayaBerry)),
        "salacberry" => Some(Box::new(SalacBerry)),

        // Terrain Seeds
        "electricseed" => Some(Box::new(ElectricSeed)),
        "grassyseed" => Some(Box::new(GrassySeed)),
        "mistyseed" => Some(Box::new(MistySeed)),
        "psychicseed" => Some(Box::new(PsychicSeed)),

        // End-of-Turn Items
        "blacksludge" => Some(Box::new(BlackSludge)),
        "flameorb" => Some(Box::new(FlameOrb)),
        "toxicorb" => Some(Box::new(ToxicOrb)),

        // Utility Items
        "protectivepads" => Some(Box::new(ProtectivePads)),
        "throatspray" => Some(Box::new(ThroatSpray)),
        "widelens" => Some(Box::new(WideLens)),
        "ironball" => Some(Box::new(IronBall)),
        "loadeddice" => Some(Box::new(LoadedDice)),
        "blunderpolicy" => Some(Box::new(BlunderPolicy)),
        "custapberry" => Some(Box::new(CustapBerry)),
        "quickclaw" => Some(Box::new(QuickClaw)),
        "adrenalineorb" => Some(Box::new(AdrenalineOrb)),
        "boosterenergy" => Some(Box::new(BoosterEnergy)),

        // Legendary Items
        "rustedsword" => Some(Box::new(RustedSword)),
        "rustedshield" => Some(Box::new(RustedShield)),
        "cornerstonemask" => Some(Box::new(OgerponMask::cornerstone_mask())),
        "hearthflamemask" => Some(Box::new(OgerponMask::hearthflame_mask())),
        "wellspringmask" => Some(Box::new(OgerponMask::wellspring_mask())),

        _ => None,
    }
}

/// Get item effect by name (defaults to Gen 9)
pub fn get_item_by_name(item_name: &str) -> Option<Box<dyn ItemEffect>> {
    get_item_by_name_with_generation(item_name, 9)
}

/// Calculate attacker's item modifiers (items that boost their own attacks)
pub fn calculate_attacker_item_modifiers(
    context: &DamageContext,
    generation_mechanics: &GenerationMechanics,
) -> ItemModifier {
    let mut modifier = ItemModifier::default();
    let generation = generation_mechanics.generation.number();

    if let Some(ref item_name) = context.attacker.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            // Only process if this is actually an attacker item
            if item.is_attacker_item() {
                let item_modifier = item.modify_damage(context);
                
                modifier.power_multiplier *= item_modifier.power_multiplier;
                modifier.attack_multiplier *= item_modifier.attack_multiplier;
                modifier.special_attack_multiplier *= item_modifier.special_attack_multiplier;
                modifier.recoil_percentage += item_modifier.recoil_percentage;
                modifier.removes_contact |= item_modifier.removes_contact;
                modifier.drain_percentage += item_modifier.drain_percentage;

                if item_modifier.type_change.is_some() {
                    modifier.type_change = item_modifier.type_change;
                }

                if item_modifier.is_consumed {
                    modifier.is_consumed = true;
                }
            }
        }
    }

    modifier
}

/// Calculate defender's item modifiers (items that affect incoming attacks)
pub fn calculate_defender_item_modifiers(
    context: &DamageContext,
    type_effectiveness: f32,
    generation_mechanics: &GenerationMechanics,
) -> ItemModifier {
    let mut modifier = ItemModifier::default();
    let generation = generation_mechanics.generation.number();

    if let Some(ref item_name) = context.defender.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            // Only process if this is actually a defender item
            if item.is_defender_item() {
                let item_modifier = item.modify_damage(context);
                
                modifier.damage_multiplier *= item_modifier.damage_multiplier;
                modifier.defense_multiplier *= item_modifier.defense_multiplier;
                modifier.special_defense_multiplier *= item_modifier.special_defense_multiplier;
                modifier.contact_recoil += item_modifier.contact_recoil;
                modifier.ground_immunity |= item_modifier.ground_immunity;
                modifier.hazard_immunity |= item_modifier.hazard_immunity;
                modifier.prevents_ko_at_full_hp |= item_modifier.prevents_ko_at_full_hp;

                if item_modifier.blocks_move {
                    modifier.blocks_move = true;
                }
                
                // Handle reactive items
                let reactive_modifier = item.check_reactive_trigger(context, type_effectiveness);
                
                // Apply reactive modifiers
                if let Some(stat_boosts) = reactive_modifier.stat_boosts {
                    modifier.stat_boosts = Some(stat_boosts);
                }
                
                // Apply other reactive modifiers
                modifier.damage_multiplier *= reactive_modifier.damage_multiplier;
                modifier.power_multiplier *= reactive_modifier.power_multiplier;
            }
        }
    }

    modifier
}

/// Calculate all item modifiers for a damage context (for backward compatibility)
pub fn calculate_item_modifiers(
    context: &DamageContext,
    type_effectiveness: f32,
    generation_mechanics: &GenerationMechanics,
) -> ItemModifier {
    let attacker_modifier = calculate_attacker_item_modifiers(context, generation_mechanics);
    let defender_modifier = calculate_defender_item_modifiers(context, type_effectiveness, generation_mechanics);
    
    // Combine attacker and defender modifiers
    let mut combined_modifier = ItemModifier::default();
    
    // Attacker modifiers
    combined_modifier.power_multiplier *= attacker_modifier.power_multiplier;
    combined_modifier.attack_multiplier *= attacker_modifier.attack_multiplier;
    combined_modifier.special_attack_multiplier *= attacker_modifier.special_attack_multiplier;
    combined_modifier.recoil_percentage += attacker_modifier.recoil_percentage;
    combined_modifier.removes_contact |= attacker_modifier.removes_contact;
    combined_modifier.drain_percentage += attacker_modifier.drain_percentage;
    
    if attacker_modifier.type_change.is_some() {
        combined_modifier.type_change = attacker_modifier.type_change;
    }
    
    // Defender modifiers
    combined_modifier.damage_multiplier *= defender_modifier.damage_multiplier;
    combined_modifier.defense_multiplier *= defender_modifier.defense_multiplier;
    combined_modifier.special_defense_multiplier *= defender_modifier.special_defense_multiplier;
    combined_modifier.contact_recoil += defender_modifier.contact_recoil;
    combined_modifier.ground_immunity |= defender_modifier.ground_immunity;
    combined_modifier.hazard_immunity |= defender_modifier.hazard_immunity;
    combined_modifier.prevents_ko_at_full_hp |= defender_modifier.prevents_ko_at_full_hp;
    
    if defender_modifier.blocks_move {
        combined_modifier.blocks_move = true;
    }
    
    // Consumption can come from either side
    combined_modifier.is_consumed = attacker_modifier.is_consumed || defender_modifier.is_consumed;

    combined_modifier
}

/// Check if Expert Belt should boost damage based on type effectiveness
pub fn apply_expert_belt_boost(context: &DamageContext, type_effectiveness: f32, generation: u8) -> f32 {
    if let Some(ref item_name) = context.attacker.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            if item.boosts_super_effective() && type_effectiveness > 1.0 {
                return item.super_effective_multiplier();
            }
        }
    }
    1.0
}

/// Apply type effectiveness modifications from items (berries)
pub fn apply_item_type_effectiveness_modifiers(
    context: &DamageContext,
    type_effectiveness: f32,
    generation: u8,
) -> f32 {
    let mut modified_effectiveness = type_effectiveness;

    // Check defender's item for type effectiveness modifications
    if let Some(ref item_name) = context.defender.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            if item.affects_type_effectiveness() {
                modified_effectiveness =
                    item.modify_type_effectiveness(modified_effectiveness, context);
            }
        }
    }

    modified_effectiveness
}

/// Handle move miss triggers for items like Blunder Policy
pub fn handle_miss_triggers(
    holder_position: BattlePosition,
    pokemon: &Pokemon,
    generation: u8,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(ref item_name) = pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            let miss_instructions = item.check_miss_trigger(holder_position);
            instructions.extend(miss_instructions);
        }
    }
    
    instructions
}

/// Calculate priority modifiers from items
pub fn calculate_item_priority_modifier(
    context: &DamageContext,
    generation_mechanics: &GenerationMechanics,
) -> i8 {
    let mut priority_modifier = 0;
    let generation = generation_mechanics.generation.number();

    // Check attacker's item for priority modifications
    if let Some(ref item_name) = context.attacker.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            if item.is_attacker_item() {
                let item_modifier = item.modify_damage(context);
                priority_modifier += item_modifier.priority_modifier;
            }
        }
    }

    priority_modifier
}

/// Handle ability activation triggers for items like Adrenaline Orb
pub fn handle_ability_triggers(
    holder_position: BattlePosition,
    pokemon: &Pokemon,
    ability_name: &str,
    generation: u8,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(ref item_name) = pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            let ability_instructions = item.check_ability_trigger(holder_position, ability_name);
            instructions.extend(ability_instructions);
        }
    }
    
    instructions
}

/// Check if an item provides a form change for a Pokemon
pub fn get_item_form_change(pokemon_species: &str, item_name: &str, generation: u8) -> Option<String> {
    if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
        item.get_form_change(pokemon_species)
    } else {
        None
    }
}

/// Get stat changes from form-changing items
pub fn get_item_form_stat_changes(pokemon_species: &str, item_name: &str, generation: u8) -> Option<HashMap<Stat, i16>> {
    if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
        item.get_form_stat_changes(pokemon_species)
    } else {
        None
    }
}

/// Check reactive item triggers (for items that activate when hit)
pub fn check_reactive_item_triggers(
    context: &DamageContext,
    type_effectiveness: f32,
    generation_mechanics: &GenerationMechanics,
) -> ItemModifier {
    let generation = generation_mechanics.generation.number();
    
    // Check defender's item for reactive triggers
    if let Some(ref item_name) = context.defender.pokemon.item {
        if let Some(item) = get_item_by_name_with_generation(item_name, generation) {
            return item.check_reactive_trigger(context, type_effectiveness);
        }
    }
    
    ItemModifier::default()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::EngineMoveData;
    use crate::core::battle_state::Pokemon;


    #[test]
    fn test_comprehensive_item_lookup() {
        // Test all major item categories
        assert!(get_item_by_name("Life Orb").is_some());
        assert!(get_item_by_name("Choice Band").is_some());
        assert!(get_item_by_name("Expert Belt").is_some());
        assert!(get_item_by_name("Charcoal").is_some());
        assert!(get_item_by_name("Fist Plate").is_some());
        assert!(get_item_by_name("Chople Berry").is_some());
        assert!(get_item_by_name("Thick Club").is_some());
        assert!(get_item_by_name("Eviolite").is_some());
        assert!(get_item_by_name("Air Balloon").is_some());
        assert!(get_item_by_name("nonexistent_item").is_none());
    }

    #[test]
    fn test_type_booster_coverage() {
        // Test that we have coverage for all major types
        assert!(get_item_by_name("Silk Scarf").is_some()); // Normal
        assert!(get_item_by_name("Charcoal").is_some()); // Fire
        assert!(get_item_by_name("Mystic Water").is_some()); // Water
        assert!(get_item_by_name("Miracle Seed").is_some()); // Grass
        assert!(get_item_by_name("Magnet").is_some()); // Electric
        assert!(get_item_by_name("Never-Melt Ice").is_some()); // Ice
        assert!(get_item_by_name("Black Belt").is_some()); // Fighting
        assert!(get_item_by_name("Poison Barb").is_some()); // Poison
        assert!(get_item_by_name("Soft Sand").is_some()); // Ground
        assert!(get_item_by_name("Sharp Beak").is_some()); // Flying
        assert!(get_item_by_name("Twisted Spoon").is_some()); // Psychic
        assert!(get_item_by_name("Silver Powder").is_some()); // Bug
        assert!(get_item_by_name("Hard Stone").is_some()); // Rock
        assert!(get_item_by_name("Spell Tag").is_some()); // Ghost
        assert!(get_item_by_name("Dragon Fang").is_some()); // Dragon
        assert!(get_item_by_name("Black Glasses").is_some()); // Dark
        assert!(get_item_by_name("Metal Coat").is_some()); // Steel
        assert!(get_item_by_name("Fairy Feather").is_some()); // Fairy
    }

    #[test]
    fn test_arceus_plate_coverage() {
        // Test all Arceus plates
        let plates = [
            "Fist Plate",
            "Sky Plate",
            "Toxic Plate",
            "Earth Plate",
            "Stone Plate",
            "Insect Plate",
            "Spooky Plate",
            "Iron Plate",
            "Flame Plate",
            "Splash Plate",
            "Meadow Plate",
            "Zap Plate",
            "Mind Plate",
            "Icicle Plate",
            "Draco Plate",
            "Dread Plate",
            "Pixie Plate",
        ];

        for plate in &plates {
            assert!(
                get_item_by_name(plate).is_some(),
                "Missing plate: {}",
                plate
            );
        }
    }

    #[test]
    fn test_damage_reduction_berry_coverage() {
        // Test all damage reduction berries
        let berries = [
            "Chople Berry",
            "Coba Berry",
            "Kebia Berry",
            "Shuca Berry",
            "Charti Berry",
            "Tanga Berry",
            "Kasib Berry",
            "Babiri Berry",
            "Occa Berry",
            "Passho Berry",
            "Rindo Berry",
            "Wacan Berry",
            "Payapa Berry",
            "Yache Berry",
            "Haban Berry",
            "Colbur Berry",
            "Roseli Berry",
            "Chilan Berry",
        ];

        for berry in &berries {
            assert!(
                get_item_by_name(berry).is_some(),
                "Missing berry: {}",
                berry
            );
        }
    }

    #[test]
    fn test_reactive_items() {
        // Test Weakness Policy
        assert!(get_item_by_name("Weakness Policy").is_some());
        
        // Test Focus Sash
        assert!(get_item_by_name("Focus Sash").is_some());
        
        // Test Absorb Bulb
        assert!(get_item_by_name("Absorb Bulb").is_some());
        
        // Test Cell Battery
        assert!(get_item_by_name("Cell Battery").is_some());
        
        // Test Shell Bell
        assert!(get_item_by_name("Shell Bell").is_some());
        
        // Test Leftovers
        assert!(get_item_by_name("Leftovers").is_some());
    }

    #[test]
    fn test_weakness_policy_reactive_trigger() {
        // Create test context
        let attacker = Pokemon::new("Garchomp".to_string());
        let mut defender = Pokemon::new("Tyranitar".to_string());
        defender.item = Some("Weakness Policy".to_string());

        let move_data = EngineMoveData {
            id: 1,
            name: "Earthquake".to_string(),
            base_power: Some(100),
            accuracy: Some(100),
            pp: 10,
            move_type: "Ground".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::showdown_types::MoveTarget::Normal,
            effect_chance: None,
            effect_description: "".to_string(),
            flags: vec![],
        };

        let context = DamageContext {
            attacker,
            defender,
            attacker_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
            defender_position: BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, 0),
            move_data,
            move_id: "Earthquake".to_string(),
            base_power: 100,
            is_critical: false,
            is_contact: false,
            move_type: "Ground".to_string(),
            state: crate::core::battle_state::BattleState::default(),
        };

        // Test Weakness Policy with super effective move (Ground vs Rock/Dark)
        let weakness_policy = WeaknessPolicy;
        let modifier = weakness_policy.check_reactive_trigger(&context, 2.0); // Super effective
        
        assert!(modifier.stat_boosts.is_some());
        if let Some(boosts) = modifier.stat_boosts {
            assert_eq!(boosts.attack, 2);
            assert_eq!(boosts.special_attack, 2);
        }
        assert!(modifier.is_consumed);
        
        // Test with neutral effectiveness - should not trigger
        let modifier_neutral = weakness_policy.check_reactive_trigger(&context, 1.0);
        assert!(modifier_neutral.stat_boosts.is_none());
        assert!(!modifier_neutral.is_consumed);
    }

    #[test]
    fn test_focus_sash_reactive_trigger() {
        // Create test context with full HP defender
        let attacker = Pokemon::new("Garchomp".to_string());
        let mut defender = Pokemon::new("Alakazam".to_string());
        defender.hp = 251;
        defender.max_hp = 251;
        defender.item = Some("Focus Sash".to_string());

        let move_data = EngineMoveData {
            id: 1,
            name: "Earthquake".to_string(),
            base_power: Some(100),
            accuracy: Some(100),
            pp: 10,
            move_type: "Ground".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::showdown_types::MoveTarget::Normal,
            effect_chance: None,
            effect_description: "".to_string(),
            flags: vec![],
        };

        let context = DamageContext {
            attacker,
            defender,
            attacker_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
            defender_position: BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, 0),
            move_data,
            move_id: "Earthquake".to_string(),
            base_power: 100,
            is_critical: false,
            is_contact: false,
            move_type: "Ground".to_string(),
            state: crate::core::battle_state::BattleState::default(),
        };

        // Test Focus Sash at full HP
        let focus_sash = FocusSash;
        let modifier = focus_sash.check_reactive_trigger(&context, 1.0);
        assert!(modifier.prevents_ko_at_full_hp);
        assert!(modifier.is_consumed);
        
        // Test with damaged Pokemon - should not trigger
        let mut damaged_context = context.clone();
        damaged_context.defender.pokemon.hp = 100; // Not at full HP
        let modifier_damaged = focus_sash.check_reactive_trigger(&damaged_context, 1.0);
        assert!(!modifier_damaged.prevents_ko_at_full_hp);
        assert!(!modifier_damaged.is_consumed);
    }

    #[test]
    fn test_absorb_bulb_and_cell_battery() {
        let attacker = Pokemon::new("Vaporeon".to_string());
        let mut defender = Pokemon::new("Venusaur".to_string());
        defender.item = Some("Absorb Bulb".to_string());

        let water_move = EngineMoveData {
            id: 1,
            name: "Surf".to_string(),
            base_power: Some(90),
            accuracy: Some(100),
            pp: 15,
            move_type: "Water".to_string(),
            category: MoveCategory::Special,
            priority: 0,
            target: crate::data::showdown_types::MoveTarget::Normal,
            effect_chance: None,
            effect_description: "".to_string(),
            flags: vec![],
        };

        let context = DamageContext {
            attacker: attacker.clone(),
            defender: defender.clone(),
            attacker_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
            defender_position: BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, 0),
            move_data: water_move,
            move_id: "Hydro Pump".to_string(),
            base_power: 90,
            is_critical: false,
            is_contact: false,
            move_type: "Water".to_string(),
            state: crate::core::battle_state::BattleState::default(),
        };

        // Test Absorb Bulb with Water move
        let absorb_bulb = AbsorbBulb;
        let modifier = absorb_bulb.check_reactive_trigger(&context, 1.0);
        assert!(modifier.stat_boosts.is_some());
        if let Some(boosts) = modifier.stat_boosts {
            assert_eq!(boosts.special_attack, 1);
        }
        assert!(modifier.is_consumed);

        // Test Cell Battery with Electric move
        let mut electric_context = context.clone();
        electric_context.move_info.move_type = "Electric".to_string();
        electric_context.defender.item = Some("Cell Battery".to_string());
        
        let cell_battery = CellBattery;
        let modifier = cell_battery.check_reactive_trigger(&electric_context, 1.0);
        assert!(modifier.stat_boosts.is_some());
        if let Some(boosts) = modifier.stat_boosts {
            assert_eq!(boosts.attack, 1);
        }
        assert!(modifier.is_consumed);
    }

    #[test]
    fn test_shell_bell_drain() {
        let mut attacker = Pokemon::new("Garchomp".to_string());
        attacker.item = Some("Shell Bell".to_string());
        let defender = Pokemon::new("Latios".to_string());

        let move_data = EngineMoveData {
            id: 1,
            name: "Earthquake".to_string(),
            base_power: Some(100),
            accuracy: Some(100),
            pp: 10,
            move_type: "Ground".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::showdown_types::MoveTarget::Normal,
            effect_chance: None,
            effect_description: "".to_string(),
            flags: vec![],
        };

        let context = DamageContext {
            attacker,
            defender,
            attacker_position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
            defender_position: BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, 0),
            move_data,
            move_id: "Earthquake".to_string(),
            base_power: 100,
            is_critical: false,
            is_contact: false,
            move_type: "Ground".to_string(),
            state: crate::core::battle_state::BattleState::default(),
        };

        // Test Shell Bell drain
        let shell_bell = ShellBell;
        let modifier = shell_bell.modify_damage(&context);
        assert_eq!(modifier.drain_percentage, 0.125); // 1/8 = 12.5%
    }

}
