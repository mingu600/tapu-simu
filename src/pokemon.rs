//! Pokemon data structures and state management

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::side::SideId;
use std::collections::HashMap;

/// A Pokemon's current battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Species data
    pub species: SpeciesData,
    
    /// Current HP
    pub hp: u16,
    
    /// Maximum HP
    pub max_hp: u16,
    
    /// Current stats (including stat changes from items/abilities)
    pub stats: StatsTable,
    
    /// Stat boosts (-6 to +6)
    pub boosts: BoostsTable,
    
    /// Major status condition (burn, sleep, etc.)
    pub status: Option<StatusCondition>,
    
    /// Volatile status effects (confusion, substitute, etc.)
    pub volatiles: HashMap<String, VolatileStatus>,
    
    /// Move slots
    pub moves: [MoveSlot; 4],
    
    /// Current ability
    pub ability: AbilityData,
    
    /// Held item
    pub item: Option<ItemData>,
    
    /// Current types (can change due to moves/abilities)
    pub types: [Type; 2],
    
    /// Level
    pub level: u8,
    
    /// Gender
    pub gender: Gender,
    
    /// Nature
    pub nature: Nature,
    
    /// Individual Values
    pub ivs: StatsTable,
    
    /// Effort Values
    pub evs: StatsTable,
    
    /// Position on the field (0, 1, 2 for triples)
    pub position: usize,
    
    /// Pokemon details string (visible to opponents)
    pub details: String,
    
    /// Base species (before forme changes)
    pub base_species: SpeciesData,
    
    /// Status condition state
    pub status_state: Option<StatusState>,
    
    /// Base stored stats (before modifications)
    pub base_stored_stats: StatsTable,
    
    /// Current stored stats (can be modified by Power Trick, Transform, etc)
    pub stored_stats: StatsTable,
    
    /// Base ability before any changes
    pub base_ability: AbilityData,
    
    /// Last item held (for recycle, etc)
    pub last_item: Option<ItemData>,
    
    /// Used item this turn flag
    pub used_item_this_turn: bool,
    
    /// Ate berry flag
    pub ate_berry: bool,
    
    /// Whether this Pokemon can be switched out
    pub trapped: TrappedState,
    
    /// Maybe trapped (for move validation)
    pub maybe_trapped: bool,
    
    /// Maybe disabled (for move validation)
    pub maybe_disabled: bool,
    
    /// Maybe locked into a move
    pub maybe_locked: Option<bool>,
    
    /// Illusion Pokemon (if any)
    pub illusion: Option<Box<Pokemon>>,
    
    /// Whether this Pokemon is transformed
    pub transformed: bool,
    
    /// Base max HP (before Dynamax)
    pub base_max_hp: u16,
    
    /// Fainted this turn
    pub fainted: bool,
    
    /// Faint queued for end of turn
    pub faint_queued: bool,
    
    /// Substitute fainted
    pub sub_fainted: Option<bool>,
    
    /// Should revert forme when fainting
    pub forme_regression: bool,
    
    /// Added type (from Forest's Curse, Trick-or-Treat)
    pub added_type: Option<Type>,
    
    /// Whether type is known to opponent
    pub known_type: bool,
    
    /// Type that client sees
    pub apparent_type: Option<Type>,
    
    /// Switch flag (effect that caused switch)
    pub switch_flag: SwitchFlag,
    
    /// Force switch flag
    pub force_switch_flag: bool,
    
    /// Skip before switch out event
    pub skip_before_switch_out_event_flag: bool,
    
    /// Dragged in by effect
    pub dragged_in: Option<u32>,
    
    /// Newly switched flag
    pub newly_switched: bool,
    
    /// Being called back flag
    pub being_called_back: bool,
    
    /// Last move used
    pub last_move: Option<String>,
    
    /// Last move target location
    pub last_move_target_loc: Option<i8>,
    
    /// Move this turn
    pub move_this_turn: Option<String>,
    
    /// Stats raised this turn
    pub stats_raised_this_turn: bool,
    
    /// Stats lowered this turn
    pub stats_lowered_this_turn: bool,
    
    /// Result of last move used previous turn
    pub move_last_turn_result: Option<MoveResult>,
    
    /// Result of most recent move this turn
    pub move_this_turn_result: Option<MoveResult>,
    
    /// Turn when this Pokemon switched in
    pub switch_in_turn: u32,
    
    // CRITICAL BATTLE MECHANICS FIELDS
    
    /// Number of damage taken from last hit (Counter, Mirror Coat, etc.)
    pub last_damage: u16,
    
    /// Damage taken this turn for Assurance/Avalanche mechanics
    pub hurt_this_turn: Option<u16>,
    
    /// Number of turns this Pokemon has been active
    pub active_turns: u32,
    
    /// Whether this Pokemon is currently active on the field
    pub is_active: bool,
    
    /// Whether this Pokemon has entered battle (Start events)
    pub is_started: bool,
    
    /// Current cached speed stat (updated when stats change)
    pub speed_cache: u16,
    
    /// Weight in hectograms (for Low Kick, Grass Knot, etc.)
    pub weight_hg: u16,
    
    /// Height in decimeters (for certain moves)
    pub height_dm: u16,
    
    /// Move actions taken while active (for Fake Out, Truant)
    pub active_move_actions: u8,
    
    /// List of attackers this turn (for Weakness Policy, etc.)
    pub attacked_by: Vec<AttackerInfo>,
    
    /// Tera type for Terastallization
    pub tera_type: Option<Type>,
    
    /// Whether this Pokemon is Terastallized
    pub terastallized: Option<Type>,
    
    /// Base types before any modifications
    pub base_types: [Type; 2],
}

/// Pokemon species data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesData {
    pub id: String,
    pub name: String,
    pub types: [Type; 2],
    pub base_stats: StatsTable,
    pub abilities: Vec<String>,
    pub height: f32,   // meters
    pub weight: f32,   // kg
    pub gender_ratio: GenderRatio,
}

/// Gender ratio for a species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenderRatio {
    Genderless,
    MaleOnly,
    FemaleOnly,
    Ratio { male: f32, female: f32 },
}

/// A move slot with PP tracking (based on Pokemon Showdown MoveSlot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSlot {
    pub id: String,
    pub move_data: MoveData,
    pub pp: u8,
    pub max_pp: u8,
    pub target: Option<String>,
    pub disabled: MoveDisabled,
    pub disabled_source: Option<String>,
    pub used: bool,
    pub virtual_: bool, // 'virtual' is a reserved keyword in Rust
}

/// Move disabled state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveDisabled {
    None,
    True,
    Reason(String),
}

impl Default for MoveDisabled {
    fn default() -> Self {
        MoveDisabled::None
    }
}

/// Move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub type_: Type,
    pub category: MoveCategory,
    pub base_power: u16,
    pub accuracy: Option<u8>, // None = always hits
    pub pp: u8,
    pub target: MoveTarget,
    pub priority: i8,
    pub flags: MoveFlags,
    pub secondary_effect: Option<SecondaryEffect>,
    pub crit_ratio: u8,
    pub multihit: Option<MultihitData>,
    pub drain: Option<[u8; 2]>,  // [numerator, denominator]
    pub recoil: Option<[u8; 2]>, // [numerator, denominator]
}

/// Move flags from Pokemon Showdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFlags {
    pub contact: bool,
    pub sound: bool,
    pub bullet: bool,
    pub pulse: bool,
    pub bite: bool,
    pub punch: bool,
    pub powder: bool,
    pub reflectable: bool,
    pub charge: bool,
    pub recharge: bool,
    pub gravity: bool,
    pub defrost: bool,
    pub distance: bool,
    pub heal: bool,
    pub authentic: bool,
}

impl Default for MoveFlags {
    fn default() -> Self {
        Self {
            contact: false,
            sound: false,
            bullet: false,
            pulse: false,
            bite: false,
            punch: false,
            powder: false,
            reflectable: false,
            charge: false,
            recharge: false,
            gravity: false,
            defrost: false,
            distance: false,
            heal: false,
            authentic: false,
        }
    }
}

/// Secondary effect data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub chance: u8, // Percentage chance
    pub effect: SecondaryEffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecondaryEffectType {
    Status(StatusCondition),
    StatBoost { stat: String, amount: i8 },
    Flinch,
    Burn,
    Freeze,
    Paralyze,
    Poison,
    BadlyPoison,
    Sleep,
}

/// Multi-hit move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultihitData {
    pub min_hits: u8,
    pub max_hits: u8,
}

/// Ability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_handlers: crate::events::EventHandlerRegistry,
}

/// Item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub natural_gift: Option<NaturalGiftData>,
    pub event_handlers: crate::events::EventHandlerRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalGiftData {
    pub type_: Type,
    pub base_power: u16,
}

/// Major status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCondition {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison,
    Sleep,
}

/// Volatile status effects (temporary conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatileStatus {
    pub id: String,
    pub duration: Option<u8>,
    pub data: HashMap<String, serde_json::Value>,
}

/// Status condition state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusState {
    pub effect_order: i32,
    pub duration: Option<u8>,
    pub data: HashMap<String, serde_json::Value>,
}

/// Trapped state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrappedState {
    None,
    True,
    Hidden,
}

/// Switch flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchFlag {
    None,
    Effect(String), // ID of the effect that caused the switch
    Force,
}

/// Move result
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MoveResult {
    Success,    // Move executed successfully
    Failure,    // Move failed completely
    Skipped,    // Move was skipped (recharge, Sky Drop, etc)
}

/// Information about a Pokemon that attacked this Pokemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackerInfo {
    pub pokemon_ref: PokemonRef,
    pub move_id: String,
    pub damage: u16,
    pub this_turn: bool,
}

/// Reference to a Pokemon (side + position)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PokemonRef {
    pub side: SideId,
    pub position: usize,
}

impl Pokemon {
    /// Create a new Pokemon from team data
    pub fn new(
        species: SpeciesData,
        level: u8,
        moves: [MoveData; 4],
        ability: AbilityData,
        item: Option<ItemData>,
        nature: Nature,
        ivs: StatsTable,
        evs: StatsTable,
        gender: Gender,
    ) -> Self {
        let stats = Self::calculate_stats(&species.base_stats, level, nature, ivs, evs);
        let max_hp = stats.hp;
        let speed_cache = stats.speed;
        let base_types = species.types;
        let species_weight = species.weight;
        let species_height = species.height;
        let species_name = species.name.clone();
        
        let move_slots = moves.map(|move_data| MoveSlot {
            id: move_data.id.clone(),
            pp: move_data.pp,
            max_pp: move_data.pp,
            target: None,
            disabled: MoveDisabled::None,
            disabled_source: None,
            used: false,
            virtual_: false,
            move_data,
        });
        
        Self {
            hp: max_hp,
            max_hp,
            stats,
            boosts: BoostsTable::default(),
            status: None,
            volatiles: HashMap::new(),
            moves: move_slots,
            ability: ability.clone(),
            item,
            types: base_types,
            level,
            gender,
            nature,
            ivs,
            evs,
            position: 0,
            fainted: false,
            trapped: TrappedState::None,
            last_move: None,
            last_move_target_loc: None,
            switch_in_turn: 0,
            // New fields
            species: species.clone(),
            details: format!("{}, L{}, {}", species_name, level, gender_string(gender)),
            base_species: species,
            status_state: None,
            base_stored_stats: stats,
            stored_stats: StatsTable {
                hp: 0, // HP is not included in stored stats
                attack: stats.attack,
                defense: stats.defense,
                special_attack: stats.special_attack,
                special_defense: stats.special_defense,
                speed: stats.speed,
            },
            base_ability: ability,
            last_item: None,
            used_item_this_turn: false,
            ate_berry: false,
            maybe_trapped: false,
            maybe_disabled: false,
            maybe_locked: None,
            illusion: None,
            transformed: false,
            base_max_hp: max_hp,
            faint_queued: false,
            sub_fainted: None,
            forme_regression: false,
            added_type: None,
            known_type: true,
            apparent_type: None,
            switch_flag: SwitchFlag::None,
            force_switch_flag: false,
            skip_before_switch_out_event_flag: false,
            dragged_in: None,
            newly_switched: false,
            being_called_back: false,
            move_this_turn: None,
            stats_raised_this_turn: false,
            stats_lowered_this_turn: false,
            move_last_turn_result: None,
            move_this_turn_result: None,
            // New battle mechanics fields
            last_damage: 0,
            hurt_this_turn: None,
            active_turns: 0,
            is_active: false,
            is_started: false,
            speed_cache,
            weight_hg: (species_weight * 10.0) as u16, // Convert kg to hectograms
            height_dm: (species_height * 10.0) as u16, // Convert m to decimeters
            active_move_actions: 0,
            attacked_by: Vec::new(),
            tera_type: None,
            terastallized: None,
            base_types,
        }
    }
    
    /// Calculate stats from base stats, level, nature, IVs, and EVs
    pub fn calculate_stats(
        base: &StatsTable,
        level: u8,
        nature: Nature,
        ivs: StatsTable,
        evs: StatsTable,
    ) -> StatsTable {
        let level = level as u32;
        
        // HP calculation: ((Base + IV + EV/4) * 2 * Level / 100) + Level + 10
        let hp = if base.hp == 1 {
            // Shedinja special case
            1
        } else {
            ((base.hp as u32 + ivs.hp as u32 + evs.hp as u32 / 4) * 2 * level / 100 + level + 10) as u16
        };
        
        // Other stats: ((Base + IV + EV/4) * 2 * Level / 100 + 5) * Nature
        let attack = Self::calc_stat(base.attack, ivs.attack, evs.attack, level, nature, StatType::Attack);
        let defense = Self::calc_stat(base.defense, ivs.defense, evs.defense, level, nature, StatType::Defense);
        let special_attack = Self::calc_stat(base.special_attack, ivs.special_attack, evs.special_attack, level, nature, StatType::SpecialAttack);
        let special_defense = Self::calc_stat(base.special_defense, ivs.special_defense, evs.special_defense, level, nature, StatType::SpecialDefense);
        let speed = Self::calc_stat(base.speed, ivs.speed, evs.speed, level, nature, StatType::Speed);
        
        StatsTable {
            hp,
            attack,
            defense,
            special_attack,
            special_defense,
            speed,
        }
    }
    
    fn calc_stat(base: u16, iv: u16, ev: u16, level: u32, nature: Nature, stat_type: StatType) -> u16 {
        let base_calc = ((base as u32 + iv as u32 + ev as u32 / 4) * 2 * level / 100 + 5) as f32;
        let nature_mod = nature.modifier(stat_type);
        (base_calc * nature_mod) as u16
    }
    
    /// Check if this Pokemon has fainted
    pub fn is_fainted(&self) -> bool {
        self.hp == 0 || self.fainted
    }
    
    /// Apply damage to this Pokemon
    pub fn take_damage(&mut self, damage: u16) {
        self.last_damage = damage;
        self.hurt_this_turn = Some(self.hurt_this_turn.unwrap_or(0) + damage);
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            self.fainted = true;
        }
    }
    
    /// Add an attacker to the list (for abilities like Justified)
    pub fn add_attacker(&mut self, attacker: PokemonRef, move_id: String, damage: u16) {
        // Remove any existing attacker from this turn (replace with latest)
        self.attacked_by.retain(|a| !(a.pokemon_ref == attacker && a.this_turn));
        
        self.attacked_by.push(AttackerInfo {
            pokemon_ref: attacker,
            move_id,
            damage,
            this_turn: true,
        });
    }
    
    /// Mark start of new turn for this Pokemon
    pub fn start_turn(&mut self) {
        self.hurt_this_turn = None;
        
        // Mark previous turn's attackers as not this turn
        for attacker in &mut self.attacked_by {
            attacker.this_turn = false;
        }
        
        if self.is_active {
            self.active_turns += 1;
        }
    }
    
    /// Switch this Pokemon in
    pub fn switch_in(&mut self, turn: u32) {
        self.is_active = true;
        self.newly_switched = true;
        self.switch_in_turn = turn;
        self.active_turns = 0;
        self.active_move_actions = 0;
        self.hurt_this_turn = None;
        self.attacked_by.clear();
        
        if !self.is_started {
            self.is_started = true;
        }
    }
    
    /// Switch this Pokemon out
    pub fn switch_out(&mut self) {
        self.is_active = false;
        self.newly_switched = false;
        self.hurt_this_turn = None;
        self.attacked_by.clear();
    }
    
    /// Get current speed considering all modifiers
    pub fn get_speed(&self) -> u16 {
        // For now, return cached speed
        // In full implementation, this would consider abilities, items, etc.
        self.speed_cache
    }
    
    /// Update speed cache when stats change
    pub fn update_speed_cache(&mut self) {
        self.speed_cache = self.effective_stat(StatType::Speed);
    }
    
    /// Check if this Pokemon was attacked this turn
    pub fn was_attacked_this_turn(&self) -> bool {
        self.attacked_by.iter().any(|a| a.this_turn)
    }
    
    /// Get damage taken this turn
    pub fn damage_this_turn(&self) -> u16 {
        self.hurt_this_turn.unwrap_or(0)
    }
    
    /// Heal this Pokemon
    pub fn heal(&mut self, amount: u16) {
        self.hp = (self.hp + amount).min(self.max_hp);
        if self.hp > 0 {
            self.fainted = false;
        }
    }
    
    /// Get effective stat considering boosts
    pub fn effective_stat(&self, stat_type: StatType) -> u16 {
        let base_stat = match stat_type {
            StatType::Attack => self.stats.attack,
            StatType::Defense => self.stats.defense,
            StatType::SpecialAttack => self.stats.special_attack,
            StatType::SpecialDefense => self.stats.special_defense,
            StatType::Speed => self.stats.speed,
        };
        
        let boost = match stat_type {
            StatType::Attack => self.boosts.attack,
            StatType::Defense => self.boosts.defense,
            StatType::SpecialAttack => self.boosts.special_attack,
            StatType::SpecialDefense => self.boosts.special_defense,
            StatType::Speed => self.boosts.speed,
        };
        
        Self::apply_boost(base_stat, boost)
    }
    
    /// Apply stat boost multiplier
    fn apply_boost(base_stat: u16, boost: i8) -> u16 {
        let multiplier = if boost >= 0 {
            (2.0 + boost as f32) / 2.0
        } else {
            2.0 / (2.0 - boost as f32)
        };
        
        (base_stat as f32 * multiplier) as u16
    }
    
    /// Check if this Pokemon can use a move
    pub fn can_use_move(&self, move_index: usize) -> bool {
        if move_index >= 4 {
            return false;
        }
        
        let move_slot = &self.moves[move_index];
        !matches!(move_slot.disabled, MoveDisabled::True | MoveDisabled::Reason(_)) && move_slot.pp > 0
    }
    
    /// Check if move slot can be used (more detailed check)
    pub fn can_use_move_slot(&self, slot_index: usize) -> bool {
        if let Some(slot) = self.moves.get(slot_index) {
            !matches!(slot.disabled, MoveDisabled::True | MoveDisabled::Reason(_)) 
                && slot.pp > 0 
                && !slot.virtual_
        } else {
            false
        }
    }
    
    /// Get the Pokemon that should be displayed (considering illusion)
    pub fn get_apparent_pokemon(&self) -> &Pokemon {
        if let Some(ref illusion_mon) = self.illusion {
            illusion_mon
        } else {
            self
        }
    }
    
    /// Check if this Pokemon is under the effect of an illusion
    pub fn has_illusion(&self) -> bool {
        self.illusion.is_some()
    }
    
    /// Get the effective types (including added type)
    pub fn get_effective_types(&self) -> Vec<Type> {
        let mut types = self.types.to_vec();
        if let Some(added) = self.added_type {
            if !types.contains(&added) {
                types.push(added);
            }
        }
        types
    }
    
    /// Set illusion
    pub fn set_illusion(&mut self, illusion_pokemon: Option<Pokemon>) {
        self.illusion = illusion_pokemon.map(Box::new);
    }
    
    /// Clear illusion
    pub fn clear_illusion(&mut self) {
        self.illusion = None;
    }
    
    /// Set transformed state
    pub fn set_transformed(&mut self, is_transformed: bool) {
        self.transformed = is_transformed;
    }
    
    /// Check if this Pokemon should be forced to switch
    pub fn should_force_switch(&self) -> bool {
        self.force_switch_flag || matches!(self.switch_flag, SwitchFlag::Force)
    }
    
    /// Use PP for a move
    pub fn use_pp(&mut self, move_index: usize) {
        if move_index < 4 {
            self.moves[move_index].pp = self.moves[move_index].pp.saturating_sub(1);
        }
    }
    
    // Factory methods for easier Pokemon creation using dex data
    
    /// Create a Pokemon using species name and move names from dex data
    /// This is much easier to use for testing and general Pokemon creation
    pub fn from_dex(
        dex: &dyn crate::dex::Dex,
        species_name: &str,
        level: u8,
        move_names: &[&str],
        ability_name: Option<&str>,
        item_name: Option<&str>,
        nature: Option<Nature>,
        gender: Option<Gender>,
    ) -> crate::errors::BattleResult<Self> {
        // Get species data
        let species = dex.get_species(species_name)
            .ok_or_else(|| crate::errors::BattleError::InvalidMove(format!("Species '{}' not found", species_name)))?
            .clone();
            
        // Get move data, padding with empty moves if needed
        let mut moves_data = Vec::new();
        for &move_name in move_names.iter().take(4) {
            let move_data = dex.get_move(move_name)
                .ok_or_else(|| crate::errors::BattleError::InvalidMove(format!("Move '{}' not found", move_name)))?
                .clone();
            moves_data.push(move_data);
        }
        
        // Pad with a default move if needed
        while moves_data.len() < 4 {
            let default_move = dex.get_move("tackle")
                .or_else(|| dex.get_move("pound"))
                .ok_or_else(|| crate::errors::BattleError::InvalidMove("No default move (tackle/pound) found".to_string()))?
                .clone();
            moves_data.push(default_move);
        }
        
        let moves: [MoveData; 4] = moves_data.try_into()
            .map_err(|_| crate::errors::BattleError::InvalidMove("Failed to create move array".to_string()))?;
            
        // Get ability data
        let ability_name = ability_name.unwrap_or(&species.abilities[0]);
        let ability = dex.get_ability(ability_name)
            .ok_or_else(|| crate::errors::BattleError::InvalidMove(format!("Ability '{}' not found", ability_name)))?
            .clone();
            
        // Get item data if specified
        let item = if let Some(item_name) = item_name {
            Some(dex.get_item(item_name)
                .ok_or_else(|| crate::errors::BattleError::InvalidMove(format!("Item '{}' not found", item_name)))?
                .clone())
        } else {
            None
        };
        
        // Use defaults for other values
        let nature = nature.unwrap_or(Nature::Hardy);
        let gender = gender.unwrap_or(Gender::Genderless);
        let ivs = crate::types::StatsTable::max(); // Perfect IVs for testing
        let evs = StatsTable::default(); // No EVs
        
        Ok(Self::new(species, level, moves, ability, item, nature, ivs, evs, gender))
    }
    
    /// Create a simple Pokemon for testing with minimal configuration
    /// Uses commonly available Pokemon like Pikachu for reliable testing
    pub fn test_pokemon(
        dex: &dyn crate::dex::Dex,
        level: Option<u8>,
    ) -> crate::errors::BattleResult<Self> {
        Self::from_dex(
            dex,
            "pikachu",
            level.unwrap_or(50),
            &["thunderbolt", "quick-attack", "double-team", "substitute"],
            None, // Use default ability
            None, // No item
            Some(Nature::Modest), // +SpA, -Atk
            Some(Gender::Male),
        )
    }
    
    /// Create a Pokemon for competitive testing with optimal stats
    pub fn competitive_pokemon(
        dex: &dyn crate::dex::Dex,
        species_name: &str,
        level: u8,
        move_names: &[&str],
        ability_name: &str,
        item_name: Option<&str>,
        nature: Nature,
        evs: Option<StatsTable>,
    ) -> crate::errors::BattleResult<Self> {
        let mut pokemon = Self::from_dex(
            dex,
            species_name,
            level,
            move_names,
            Some(ability_name),
            item_name,
            Some(nature),
            Some(Gender::Genderless), // For consistency
        )?;
        
        // Apply custom EVs if provided
        if let Some(custom_evs) = evs {
            pokemon.evs = custom_evs;
            // Recalculate stats with new EVs
            pokemon.stats = Self::calculate_stats(
                &pokemon.species.base_stats,
                pokemon.level,
                pokemon.nature,
                pokemon.ivs,
                pokemon.evs,
            );
            pokemon.max_hp = pokemon.stats.hp;
            pokemon.hp = pokemon.max_hp;
            pokemon.speed_cache = pokemon.stats.speed;
        }
        
        Ok(pokemon)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

impl Nature {
    /// Get the stat modifier for this nature
    fn modifier(&self, stat: StatType) -> f32 {
        use Nature::*;
        use StatType::*;
        
        match (self, stat) {
            // Increased Attack
            (Lonely | Brave | Adamant | Naughty, Attack) => 1.1,
            // Decreased Attack
            (Bold | Timid | Modest | Calm, Attack) => 0.9,
            
            // Increased Defense
            (Bold | Relaxed | Impish | Lax, Defense) => 1.1,
            // Decreased Defense
            (Lonely | Hasty | Mild | Gentle, Defense) => 0.9,
            
            // Increased Special Attack
            (Modest | Mild | Quiet | Rash, SpecialAttack) => 1.1,
            // Decreased Special Attack
            (Adamant | Impish | Jolly | Careful, SpecialAttack) => 0.9,
            
            // Increased Special Defense
            (Calm | Gentle | Sassy | Careful, SpecialDefense) => 1.1,
            // Decreased Special Defense
            (Naughty | Lax | Naive | Rash, SpecialDefense) => 0.9,
            
            // Increased Speed
            (Timid | Hasty | Jolly | Naive, Speed) => 1.1,
            // Decreased Speed
            (Brave | Relaxed | Quiet | Sassy, Speed) => 0.9,
            
            // Neutral natures
            _ => 1.0,
        }
    }
}

/// Helper function to convert gender to string
fn gender_string(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "M",
        Gender::Female => "F",
        Gender::Genderless => "",
    }
}

impl Default for TrappedState {
    fn default() -> Self {
        TrappedState::None
    }
}

impl Default for SwitchFlag {
    fn default() -> Self {
        SwitchFlag::None
    }
}