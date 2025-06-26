//! # Focused Damage Calculation Context
//! 
//! This module provides focused context structs for damage calculation,
//! eliminating the need to pass the entire battle state.

use crate::core::battle_format::{BattleFormat, BattlePosition};
use crate::core::battle_state::{FieldConditions, WeatherState, TerrainState, GlobalEffects};
use crate::core::instructions::{Weather, Terrain, Stat};
use crate::core::battle_state::Pokemon;
use crate::core::instructions::MoveCategory;
use crate::core::battle_state::BattleState;
use crate::types::identifiers::AbilityId;
use crate::data::showdown_types::MoveData;
use crate::data::types::BaseStats;
use serde::{Deserialize, Serialize};

/// Comprehensive context for damage calculation
#[derive(Debug, Clone)]
pub struct DamageContext {
    /// Information about the attacking Pokemon
    pub attacker: AttackerContext,
    /// Information about the defending Pokemon
    pub defender: DefenderContext,
    /// Information about the move being used
    pub move_info: MoveContext,
    /// Battlefield conditions that affect damage
    pub field: FieldContext,
    /// Format-specific context
    pub format: FormatContext,
}

/// Context for the attacking Pokemon
#[derive(Debug, Clone)]
pub struct AttackerContext {
    /// The attacking Pokemon
    pub pokemon: Pokemon, // TODO: Consider using &Pokemon with lifetimes
    /// Position of the attacker on the battlefield
    pub position: BattlePosition,
    /// Effective stats after all modifiers
    pub effective_stats: EffectiveStats,
    /// Current ability state
    pub ability_state: AbilityState,
    /// Item effects affecting the attacker
    pub item_effects: ItemEffects,
}

/// Context for the defending Pokemon
#[derive(Debug, Clone)]
pub struct DefenderContext {
    /// The defending Pokemon
    pub pokemon: Pokemon,
    /// Position of the defender on the battlefield
    pub position: BattlePosition,
    /// Effective stats after all modifiers
    pub effective_stats: EffectiveStats,
    /// Current ability state
    pub ability_state: AbilityState,
    /// Item effects affecting the defender
    pub item_effects: ItemEffects,
}

/// Context for the move being used
#[derive(Debug, Clone)]
pub struct MoveContext {
    /// Name/ID of the move
    pub name: String,
    /// Base power after initial modifications
    pub base_power: u8,
    /// Whether this is a critical hit
    pub is_critical: bool,
    /// Whether this move makes contact
    pub is_contact: bool,
    /// Whether this move is a punch move (for Iron Fist, Punching Glove)
    pub is_punch: bool,
    /// Whether this move is a sound move (for Throat Chop, Soundproof)
    pub is_sound: bool,
    /// Whether this move is a multi-hit move (for Loaded Dice)
    pub is_multihit: bool,
    /// Type of the move (may differ from original due to abilities)
    pub move_type: String,
    /// Category of the move
    pub category: MoveCategory,
}

/// Battlefield context affecting damage
#[derive(Debug, Clone)]
pub struct FieldContext {
    /// Current weather state
    pub weather: WeatherState,
    /// Current terrain state
    pub terrain: TerrainState,
    /// Global effects like Trick Room
    pub global_effects: GlobalEffects,
}

/// Format-specific context
#[derive(Debug, Clone)]
pub struct FormatContext {
    /// The battle format
    pub format: BattleFormat,
    /// Number of targets (for spread move calculations)
    pub target_count: usize,
}

/// Effective stats after all modifiers
#[derive(Debug, Clone)]
pub struct EffectiveStats {
    /// Current stat values
    pub stats: BaseStats,
    /// Stat stage modifiers (-6 to +6)
    pub stat_stages: StatStages,
}

/// Stat stage modifiers
#[derive(Debug, Clone, Default)]
pub struct StatStages {
    pub attack: i8,
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
}

/// Ability state affecting damage
#[derive(Debug, Clone, Default)]
pub struct AbilityState {
    /// The ability ID
    pub ability_id: Option<AbilityId>,
    /// Whether the ability is suppressed
    pub is_suppressed: bool,
    /// Whether the ability was triggered this turn
    pub triggered_this_turn: bool,
}

/// Item effects affecting damage
#[derive(Debug, Clone, Default)]
pub struct ItemEffects {
    /// The item name
    pub item_name: Option<String>,
    /// Whether the item can be used
    pub is_active: bool,
    /// Whether the item was consumed
    pub is_consumed: bool,
}

/// Result of damage calculation
#[derive(Debug, Clone)]
pub struct DamageResult {
    /// Final damage amount
    pub damage: i16,
    /// Whether the move was completely blocked
    pub blocked: bool,
    /// Whether it was a critical hit
    pub was_critical: bool,
    /// Type effectiveness multiplier
    pub type_effectiveness: f32,
    /// Whether the move hit a substitute instead of the Pokemon
    pub hit_substitute: bool,
    /// Any special effects that occurred
    pub effects: Vec<DamageEffect>,
}

/// Special effects that can occur during damage calculation
#[derive(Debug, Clone)]
pub enum DamageEffect {
    /// Critical hit occurred
    Critical,
    /// Ability activated during calculation
    AbilityActivated { ability: String, position: BattlePosition },
    /// Item was consumed or activated
    ItemActivated { item: String, position: BattlePosition },
    /// Weather affected the calculation
    WeatherEffect { weather: Weather },
    /// Terrain affected the calculation
    TerrainEffect { terrain: Terrain },
}

impl DamageContext {
    /// Create a new damage context from battle state components
    pub fn new(
        attacker: AttackerContext,
        defender: DefenderContext,
        move_info: MoveContext,
        field: FieldContext,
        format: FormatContext,
    ) -> Self {
        Self {
            attacker,
            defender,
            move_info,
            field,
            format,
        }
    }

    /// Get the generation from the format
    pub fn get_generation(&self) -> crate::generation::Generation {
        // Extract generation from format - implement based on format structure
        // For now, default to Gen 9
        crate::generation::Generation::Gen9
    }

    /// Helper to create context from battle state
    pub fn from_battle_state(
        attacker_pokemon: &Pokemon,
        attacker_position: BattlePosition,
        defender_pokemon: &Pokemon,
        defender_position: BattlePosition,
        move_data: &MoveData,
        field_conditions: &FieldConditions,
        battle_format: &BattleFormat,
        target_count: usize,
        is_critical: bool,
    ) -> Self {
        let attacker = AttackerContext {
            pokemon: attacker_pokemon.clone(),
            position: attacker_position,
            effective_stats: EffectiveStats::from_pokemon(attacker_pokemon),
            ability_state: AbilityState::from_pokemon(attacker_pokemon),
            item_effects: ItemEffects::from_pokemon(attacker_pokemon),
        };

        let defender = DefenderContext {
            pokemon: defender_pokemon.clone(),
            position: defender_position,
            effective_stats: EffectiveStats::from_pokemon(defender_pokemon),
            ability_state: AbilityState::from_pokemon(defender_pokemon),
            item_effects: ItemEffects::from_pokemon(defender_pokemon),
        };

        let move_info = MoveContext {
            name: move_data.name.clone(),
            base_power: move_data.base_power as u8,
            is_critical,
            is_contact: move_data.flags.contains_key("contact"),
            is_punch: move_data.flags.contains_key("punch"),
            is_sound: move_data.flags.contains_key("sound"),
            is_multihit: move_data.flags.contains_key("multihit"),
            move_type: move_data.move_type.clone(),
            category: MoveCategory::from_str(&move_data.category),
        };

        let field = FieldContext {
            weather: field_conditions.weather.clone(),
            terrain: field_conditions.terrain.clone(),
            global_effects: field_conditions.global_effects.clone(),
        };

        let format = FormatContext {
            format: battle_format.clone(),
            target_count,
        };

        Self::new(attacker, defender, move_info, field, format)
    }
}

impl EffectiveStats {
    /// Create effective stats from a Pokemon
    pub fn from_pokemon(pokemon: &Pokemon) -> Self {
        Self {
            stats: pokemon.stats,
            stat_stages: StatStages::from_pokemon(pokemon),
        }
    }

    /// Get the effective value of a stat after stages
    pub fn get_effective_stat(&self, stat: Stat) -> i16 {
        let base_value = match stat {
            Stat::Attack => self.stats.attack,
            Stat::Defense => self.stats.defense,
            Stat::SpecialAttack => self.stats.special_attack,
            Stat::SpecialDefense => self.stats.special_defense,
            Stat::Speed => self.stats.speed,
            _ => return 0, // HP doesn't use stages
        };

        let stage = match stat {
            Stat::Attack => self.stat_stages.attack,
            Stat::Defense => self.stat_stages.defense,
            Stat::SpecialAttack => self.stat_stages.special_attack,
            Stat::SpecialDefense => self.stat_stages.special_defense,
            Stat::Speed => self.stat_stages.speed,
            _ => 0,
        };

        apply_stat_stage_multiplier(base_value, stage)
    }

    /// Get the effective value of a stat with critical hit considerations
    /// Critical hits ignore positive defensive boosts and negative offensive drops
    pub fn get_effective_stat_with_crit(&self, stat: Stat, is_critical: bool, is_attacker: bool) -> i16 {
        let base_value = match stat {
            Stat::Attack => self.stats.attack,
            Stat::Defense => self.stats.defense,
            Stat::SpecialAttack => self.stats.special_attack,
            Stat::SpecialDefense => self.stats.special_defense,
            Stat::Speed => self.stats.speed,
            _ => return 0, // HP doesn't use stages
        };

        let original_stage = match stat {
            Stat::Attack => self.stat_stages.attack,
            Stat::Defense => self.stat_stages.defense,
            Stat::SpecialAttack => self.stat_stages.special_attack,
            Stat::SpecialDefense => self.stat_stages.special_defense,
            Stat::Speed => self.stat_stages.speed,
            _ => 0,
        };

        // Apply critical hit stat boost rules
        let effective_stage = if is_critical {
            match (stat, is_attacker) {
                // For attacker's offensive stats: ignore negative boosts (drops)
                (Stat::Attack | Stat::SpecialAttack, true) => {
                    if original_stage < 0 { 0 } else { original_stage }
                }
                // For defender's defensive stats: ignore positive boosts
                (Stat::Defense | Stat::SpecialDefense, false) => {
                    if original_stage > 0 { 0 } else { original_stage }
                }
                // All other cases: use original stage
                _ => original_stage,
            }
        } else {
            original_stage
        };

        apply_stat_stage_multiplier(base_value, effective_stage)
    }

    /// Get the effective value of a stat with critical hit considerations for a specific generation
    /// Gen 1 critical hits ignore ALL stat boosts (positive and negative)
    pub fn get_effective_stat_with_crit_gen(&self, stat: Stat, is_critical: bool, is_attacker: bool, generation: crate::generation::Generation) -> i16 {
        let base_value = match stat {
            Stat::Attack => self.stats.attack,
            Stat::Defense => self.stats.defense,
            Stat::SpecialAttack => self.stats.special_attack,
            Stat::SpecialDefense => self.stats.special_defense,
            Stat::Speed => self.stats.speed,
            _ => return 0, // HP doesn't use stages
        };

        let original_stage = match stat {
            Stat::Attack => self.stat_stages.attack,
            Stat::Defense => self.stat_stages.defense,
            Stat::SpecialAttack => self.stat_stages.special_attack,
            Stat::SpecialDefense => self.stat_stages.special_defense,
            Stat::Speed => self.stat_stages.speed,
            _ => 0,
        };

        // Apply critical hit stat boost rules
        let effective_stage = if is_critical {
            match generation {
                crate::generation::Generation::Gen1 => {
                    // Gen 1: Critical hits ignore ALL stat boosts completely
                    0
                }
                _ => {
                    // Gen 2+: Modern critical hit rules
                    match (stat, is_attacker) {
                        // For attacker's offensive stats: ignore negative boosts (drops)
                        (Stat::Attack | Stat::SpecialAttack, true) => {
                            if original_stage < 0 { 0 } else { original_stage }
                        }
                        // For defender's defensive stats: ignore positive boosts
                        (Stat::Defense | Stat::SpecialDefense, false) => {
                            if original_stage > 0 { 0 } else { original_stage }
                        }
                        // All other cases: use original stage
                        _ => original_stage,
                    }
                }
            }
        } else {
            original_stage
        };

        apply_stat_stage_multiplier(base_value, effective_stage)
    }
}

impl StatStages {
    /// Create stat stages from a Pokemon's current boosts
    pub fn from_pokemon(pokemon: &Pokemon) -> Self {
        Self {
            attack: *pokemon.stat_boosts.get(&Stat::Attack).unwrap_or(&0),
            defense: *pokemon.stat_boosts.get(&Stat::Defense).unwrap_or(&0),
            special_attack: *pokemon.stat_boosts.get(&Stat::SpecialAttack).unwrap_or(&0),
            special_defense: *pokemon.stat_boosts.get(&Stat::SpecialDefense).unwrap_or(&0),
            speed: *pokemon.stat_boosts.get(&Stat::Speed).unwrap_or(&0),
        }
    }
}

impl AbilityState {
    /// Create ability state from a Pokemon
    pub fn from_pokemon(pokemon: &Pokemon) -> Self {
        Self {
            ability_id: Some(crate::types::identifiers::AbilityId::from(pokemon.ability.clone())),
            is_suppressed: pokemon.ability_suppressed,
            triggered_this_turn: pokemon.ability_triggered_this_turn,
        }
    }
}

impl ItemEffects {
    /// Create item effects from a Pokemon
    pub fn from_pokemon(pokemon: &Pokemon) -> Self {
        Self {
            item_name: pokemon.item.clone(),
            is_active: pokemon.item.is_some() && !pokemon.item_consumed,
            is_consumed: pokemon.item_consumed,
        }
    }
}

/// Apply stat stage multiplier according to Pokemon mechanics
fn apply_stat_stage_multiplier(base_value: i16, stage: i8) -> i16 {
    if stage == 0 {
        return base_value;
    }

    let multiplier = if stage > 0 {
        (2 + stage as i16) as f32 / 2.0
    } else {
        2.0 / (2 - stage as i16) as f32
    };

    (base_value as f32 * multiplier) as i16
}


// Default implementations for testing and compatibility
impl Default for DamageContext {
    fn default() -> Self {
        Self {
            attacker: AttackerContext::default(),
            defender: DefenderContext::default(),
            move_info: MoveContext::default(),
            field: FieldContext::default(),
            format: FormatContext::default(),
        }
    }
}

impl Default for AttackerContext {
    fn default() -> Self {
        Self {
            pokemon: Pokemon::default(),
            position: BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0),
            effective_stats: EffectiveStats::default(),
            ability_state: AbilityState::default(),
            item_effects: ItemEffects::default(),
        }
    }
}

impl Default for DefenderContext {
    fn default() -> Self {
        Self {
            pokemon: Pokemon::default(),
            position: BattlePosition::new(crate::core::battle_format::SideReference::SideTwo, 0),
            effective_stats: EffectiveStats::default(),
            ability_state: AbilityState::default(),
            item_effects: ItemEffects::default(),
        }
    }
}

impl Default for MoveContext {
    fn default() -> Self {
        Self {
            name: "tackle".to_string(),
            base_power: 40,
            is_critical: false,
            is_contact: true,
            is_punch: false,
            is_sound: false,
            is_multihit: false,
            move_type: "normal".to_string(),
            category: MoveCategory::Physical,
        }
    }
}

impl Default for FieldContext {
    fn default() -> Self {
        Self {
            weather: crate::core::battle_state::WeatherState {
                condition: crate::core::instructions::Weather::None,
                turns_remaining: None,
                source: None,
            },
            terrain: crate::core::battle_state::TerrainState {
                condition: crate::core::instructions::Terrain::None,
                turns_remaining: None,
                source: None,
            },
            global_effects: crate::core::battle_state::GlobalEffects {
                gravity: None,
                trick_room: None,
            },
        }
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self {
            format: crate::core::battle_format::BattleFormat::gen9_ou(),
            target_count: 1,
        }
    }
}

impl Default for EffectiveStats {
    fn default() -> Self {
        Self {
            stats: crate::data::types::BaseStats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            stat_stages: StatStages::default(),
        }
    }
}