//! # Modern Battle State System

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::instructions::{
    BattleInstruction, FieldInstruction, PokemonInstruction, PokemonStatus, SideCondition, Stat,
    StatsInstruction, StatusInstruction, Terrain, VolatileStatus, Weather,
};
use crate::core::move_choice::{MoveChoice, MoveIndex, PokemonIndex};
use crate::data::types::Stats;
use crate::generation::GenerationBattleMechanics;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Re-export MoveCategory for compatibility
pub use crate::core::instructions::MoveCategory;

/// Pokemon gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

/// Side-wide volatile statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideVolatileStatus {
    TailWind,
    WideGuard,
    QuickGuard,
}

/// Tracks damage dealt to a side for counter moves
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageDealt {
    /// Amount of damage dealt
    pub damage: i16,
    /// Category of the move that dealt damage
    pub move_category: MoveCategory,
    /// Whether the damage hit a substitute
    pub hit_substitute: bool,
}

impl DamageDealt {
    /// Create a new DamageDealt with default values
    pub fn new() -> Self {
        Self {
            damage: 0,
            move_category: MoveCategory::Physical,
            hit_substitute: false,
        }
    }

    /// Reset damage tracking (called at start of turn)
    pub fn reset(&mut self) {
        self.damage = 0;
        self.move_category = MoveCategory::Physical;
        self.hit_substitute = false;
    }

    /// Set damage information
    pub fn set_damage(&mut self, damage: i16, move_category: MoveCategory, hit_substitute: bool) {
        self.damage = damage;
        self.move_category = move_category;
        self.hit_substitute = hit_substitute;
    }
}

impl Default for DamageDealt {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about damage taken this turn (for moves like Avalanche)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageInfo {
    /// Amount of damage taken
    pub damage: i16,
    /// Category of the move that dealt damage
    pub move_category: MoveCategory,
    /// Position of the attacker that dealt damage
    pub attacker_position: BattlePosition,
    /// Whether the damage was from a direct attack
    pub is_direct_damage: bool,
}

impl DamageInfo {
    /// Create new damage info
    pub fn new(
        damage: i16,
        move_category: MoveCategory,
        attacker_position: BattlePosition,
        is_direct_damage: bool,
    ) -> Self {
        Self {
            damage,
            move_category,
            attacker_position,
            is_direct_damage,
        }
    }
}

/// Represents a Pokemon's move in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    /// Move name/identifier
    pub name: String,
    /// Base power (0 for status moves)
    pub base_power: u8,
    /// Accuracy (1-100, 0 for never-miss moves)
    pub accuracy: u8,
    /// Move type
    pub move_type: String,
    /// Current PP
    pub pp: u8,
    /// Maximum PP
    pub max_pp: u8,
    /// Move target type (Pokemon Showdown format)
    pub target: crate::data::showdown_types::MoveTarget,
    /// Move category
    pub category: MoveCategory,
    /// Move priority
    pub priority: i8,
}

impl Move {
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_power: 60,
            accuracy: 100,
            move_type: "Normal".to_string(),
            pp: 15,
            max_pp: 15,
            target: crate::data::showdown_types::MoveTarget::Normal,
            category: MoveCategory::Physical,
            priority: 0,
        }
    }

    /// Get the move's type
    pub fn get_type(&self) -> &str {
        &self.move_type
    }

    /// Get the move's name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Create a new move with detailed parameters
    pub fn new_with_details(
        name: String,
        base_power: u8,
        accuracy: u8,
        move_type: String,
        pp: u8,
        max_pp: u8,
        target: crate::data::showdown_types::MoveTarget,
        category: MoveCategory,
        priority: i8,
    ) -> Self {
        Self {
            name,
            base_power,
            accuracy,
            move_type,
            pp,
            max_pp,
            target,
            category,
            priority,
        }
    }
}

/// Pokemon representation in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Pokemon species name/ID
    pub species: String,
    /// Current HP
    pub hp: i16,
    /// Maximum HP
    pub max_hp: i16,
    /// Base stats
    pub stats: Stats,
    /// Current stat boosts (-6 to +6)
    pub stat_boosts: HashMap<crate::core::instructions::Stat, i8>,
    /// Current status condition
    pub status: PokemonStatus,
    /// Status duration (for sleep/freeze)
    pub status_duration: Option<u8>,
    /// Volatile statuses
    pub volatile_statuses: HashSet<VolatileStatus>,
    /// Volatile status durations (turns remaining for each status)
    pub volatile_status_durations: HashMap<VolatileStatus, u8>,
    /// Substitute health (when Substitute volatile status is active)
    pub substitute_health: i16,
    /// Current moves
    pub moves: HashMap<MoveIndex, Move>,
    /// Current ability
    pub ability: String,
    /// Held item
    pub item: Option<String>,
    /// Types (can change due to moves like Soak)
    pub types: Vec<String>,
    /// Level
    pub level: u8,
    /// Gender
    pub gender: Gender,
    /// Tera type (if Terastallized) - Gen 9+ only
    pub tera_type: Option<crate::core::move_choice::PokemonType>,
    /// Whether this Pokemon is Terastallized - Gen 9+ only
    pub is_terastallized: bool,
    /// Whether the ability is suppressed (by moves like Gastro Acid)
    pub ability_suppressed: bool,
    /// Whether the ability has triggered this turn (for once-per-turn abilities)
    pub ability_triggered_this_turn: bool,
    /// Whether the held item has been consumed this battle
    pub item_consumed: bool,
    /// Weight in kilograms (for moves like Heavy Slam, Heat Crash)
    pub weight_kg: f32,
}

impl Pokemon {
    /// Create a new Pokemon with default values
    pub fn new(species: String) -> Self {
        Self {
            species,
            hp: 100,
            max_hp: 100,
            stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            stat_boosts: HashMap::new(),
            status: PokemonStatus::None,
            status_duration: None,
            volatile_statuses: HashSet::new(),
            volatile_status_durations: HashMap::new(),
            substitute_health: 0,
            moves: HashMap::new(),
            ability: String::new(),
            item: None,
            types: vec!["Normal".to_string()],
            level: 50,
            gender: Gender::Unknown,
            tera_type: None,
            is_terastallized: false,
            ability_suppressed: false,
            ability_triggered_this_turn: false,
            item_consumed: false,
            weight_kg: 50.0, // Default weight for unknown Pokemon
        }
    }

    /// Get a specific move from Pokemon's moveset
    pub fn get_move(&self, move_index: MoveIndex) -> Option<&Move> {
        self.moves.get(&move_index)
    }

    /// Get mutable reference to a move
    pub fn get_move_mut(&mut self, move_index: MoveIndex) -> Option<&mut Move> {
        self.moves.get_mut(&move_index)
    }

    /// Check if the Pokemon is fainted
    pub fn is_fainted(&self) -> bool {
        self.hp <= 0
    }

    /// Get effective stat after boosts, items, abilities, etc.
    pub fn get_effective_stat(&self, stat: crate::core::instructions::Stat) -> f64 {
        let base_stat = match stat {
            crate::core::instructions::Stat::Hp => self.stats.hp as f64,
            crate::core::instructions::Stat::Attack => self.stats.attack as f64,
            crate::core::instructions::Stat::Defense => self.stats.defense as f64,
            crate::core::instructions::Stat::SpecialAttack => self.stats.special_attack as f64,
            crate::core::instructions::Stat::SpecialDefense => self.stats.special_defense as f64,
            crate::core::instructions::Stat::Speed => self.stats.speed as f64,
            crate::core::instructions::Stat::Accuracy => 100.0, // Base accuracy
            crate::core::instructions::Stat::Evasion => 100.0,  // Base evasion
        };

        // Apply stat boosts
        let boost = self.stat_boosts.get(&stat).copied().unwrap_or(0);
        let boost_multiplier = if boost >= 0 {
            (2.0 + boost as f64) / 2.0
        } else {
            2.0 / (2.0 - boost as f64)
        };

        base_stat * boost_multiplier
    }

    /// Add a move to the Pokemon's moveset
    pub fn add_move(&mut self, move_index: MoveIndex, move_data: Move) {
        self.moves.insert(move_index, move_data);
    }
}

impl Default for Pokemon {
    fn default() -> Self {
        Self::new("MissingNo".to_string())
    }
}

/// Represents one side of a battle (a player/trainer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSide {
    /// All Pokemon on this side's team
    pub pokemon: Vec<Pokemon>,
    /// Indices of currently active Pokemon
    pub active_pokemon_indices: Vec<Option<usize>>,
    /// Side conditions affecting this side
    pub side_conditions: HashMap<SideCondition, u8>,
    /// Volatile statuses that affect the entire side
    pub side_volatile_statuses: HashSet<SideVolatileStatus>,
    /// Wish healing scheduled for specific slots (heal_amount, turns_remaining)
    pub wish_healing: HashMap<usize, (i16, u8)>,
    /// Future Sight attacks scheduled for specific slots (attacker_position, damage_amount, turns_remaining, move_name)
    pub future_sight_attacks: HashMap<usize, (BattlePosition, i16, u8, String)>,
    /// Damage tracking for counter moves
    pub damage_dealt: DamageDealt,
    /// Whether Terastallization has been used this battle (Gen 9+ only)
    pub tera_used: bool,
}

impl BattleSide {
    /// Create a new battle side
    pub fn new() -> Self {
        Self {
            pokemon: Vec::new(),
            active_pokemon_indices: vec![None; 3], // Max 3 for triples, unused slots ignored
            side_conditions: HashMap::new(),
            side_volatile_statuses: HashSet::new(),
            wish_healing: HashMap::new(),
            future_sight_attacks: HashMap::new(),
            damage_dealt: DamageDealt::new(),
            tera_used: false,
        }
    }

    /// Add a Pokemon to this side's team
    pub fn add_pokemon(&mut self, pokemon: Pokemon) {
        self.pokemon.push(pokemon);
    }

    /// Set the active Pokemon at a specific slot
    pub fn set_active_pokemon_at_slot(&mut self, slot: usize, pokemon_index: Option<usize>) {
        if slot < self.active_pokemon_indices.len() {
            self.active_pokemon_indices[slot] = pokemon_index;
        }
    }

    /// Get the active Pokemon at a specific slot
    pub fn get_active_pokemon_at_slot(&self, slot: usize) -> Option<&Pokemon> {
        if let Some(Some(pokemon_index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get(*pokemon_index)
        } else {
            None
        }
    }

    /// Get the active Pokemon at a specific slot (mutable)
    pub fn get_active_pokemon_at_slot_mut(&mut self, slot: usize) -> Option<&mut Pokemon> {
        if let Some(Some(pokemon_index)) = self.active_pokemon_indices.get(slot).copied() {
            self.pokemon.get_mut(pokemon_index)
        } else {
            None
        }
    }
}

/// The main battle state with decomposed components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    /// The battle format determining rules and active Pokemon count
    pub format: BattleFormat,
    /// The two battle sides (always exactly 2)
    pub sides: [BattleSide; 2],
    /// Field conditions affecting the entire battlefield
    pub field: FieldConditions,
    /// Turn-related state information
    pub turn_info: TurnState,
}

/// Field conditions that affect the entire battlefield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConditions {
    /// Current weather state
    pub weather: WeatherState,
    /// Current terrain state
    pub terrain: TerrainState,
    /// Global battlefield effects
    pub global_effects: GlobalEffects,
}

/// Weather state with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    /// Current weather condition
    pub condition: Weather,
    /// Turns remaining (None for permanent weather)
    pub turns_remaining: Option<u8>,
    /// The position that set this weather (for ability interactions)
    pub source: Option<BattlePosition>,
}

/// Terrain state with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainState {
    /// Current terrain condition
    pub condition: Terrain,
    /// Turns remaining (None for permanent terrain)
    pub turns_remaining: Option<u8>,
    /// The position that set this terrain (for ability interactions)
    pub source: Option<BattlePosition>,
}

/// Global effects that affect the entire battlefield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEffects {
    /// Trick Room state
    pub trick_room: Option<TrickRoomState>,
    /// Gravity state
    pub gravity: Option<GravityState>,
}

/// Trick Room effect state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrickRoomState {
    /// Turns remaining
    pub turns_remaining: u8,
    /// The position that set Trick Room
    pub source: Option<BattlePosition>,
}

/// Gravity effect state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityState {
    /// Turns remaining
    pub turns_remaining: u8,
    /// The position that set Gravity
    pub source: Option<BattlePosition>,
}

/// Turn-related state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    /// Current turn number
    pub number: u32,
    /// Current phase of the turn
    pub phase: TurnPhase,
    /// Positions that have moved this turn (for turn order tracking)
    pub moved_this_turn: Vec<BattlePosition>,
    /// Positions that have taken damage this turn (for Avalanche-like mechanics)
    pub damaged_this_turn: HashMap<BattlePosition, DamageInfo>,
}

/// Phase of the current turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TurnPhase {
    /// Waiting for move selection
    Selection,
    /// Executing moves
    Execution,
    /// End of turn effects
    EndOfTurn,
}

impl Default for BattleState {
    fn default() -> Self {
        Self::new(BattleFormat::gen9_ou())
    }
}

impl BattleState {
    /// Create a new battle state with the specified format
    pub fn new(format: BattleFormat) -> Self {
        let side_one = BattleSide::new();
        let side_two = BattleSide::new();
        Self {
            format,
            sides: [side_one, side_two],
            field: FieldConditions::default(),
            turn_info: TurnState::default(),
        }
    }

    /// Create a new battle state with teams from random team data
    pub fn new_with_teams(
        format: BattleFormat,
        team_one: Vec<crate::data::RandomPokemonSet>,
        team_two: Vec<crate::data::RandomPokemonSet>,
    ) -> Self {
        let mut state = Self::new(format.clone());

        // Create PS repository for proper move and Pokemon data
        let repository = crate::data::GameDataRepository::from_path("data/ps-extracted")
            .expect("Failed to load Pokemon data from data/ps-extracted. Ensure the data directory exists and contains valid JSON files.");

        // Convert and add Pokemon to each side
        for pokemon_set in team_one {
            let pokemon = pokemon_set.to_battle_pokemon(&repository);
            state.sides[0].add_pokemon(pokemon);
        }

        for pokemon_set in team_two {
            let pokemon = pokemon_set.to_battle_pokemon(&repository);
            state.sides[1].add_pokemon(pokemon);
        }

        // Set initial active Pokemon based on format
        let active_count = format.active_pokemon_count();
        for slot in 0..active_count {
            if slot < state.sides[0].pokemon.len() {
                state.sides[0].set_active_pokemon_at_slot(slot, Some(slot));
            }
            if slot < state.sides[1].pokemon.len() {
                state.sides[1].set_active_pokemon_at_slot(slot, Some(slot));
            }
        }

        state
    }

    /// Create a new battle state with pre-constructed Pokemon (for tests, direct team creation)
    pub fn new_with_pokemon(
        format: BattleFormat,
        team_one: Vec<Pokemon>,
        team_two: Vec<Pokemon>,
    ) -> Self {
        let mut state = Self::new(format);
        
        // Add Pokemon to each side
        for pokemon in team_one {
            state.sides[0].add_pokemon(pokemon);
        }
        
        for pokemon in team_two {
            state.sides[1].add_pokemon(pokemon);
        }
        
        // Set initial active Pokemon based on format
        let active_count = state.format.active_pokemon_count();
        for slot in 0..active_count {
            if slot < state.sides[0].pokemon.len() {
                state.sides[0].set_active_pokemon_at_slot(slot, Some(slot));
            }
            if slot < state.sides[1].pokemon.len() {
                state.sides[1].set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        state
    }

    /// Get a reference to a specific side
    pub fn get_side(&self, side_index: usize) -> Option<&BattleSide> {
        self.sides.get(side_index)
    }

    /// Get a mutable reference to a specific side
    pub fn get_side_mut(&mut self, side_index: usize) -> &mut BattleSide {
        &mut self.sides[side_index]
    }

    /// Check if Trick Room is active
    pub fn is_trick_room_active(&self) -> bool {
        self.field.global_effects.trick_room.is_some()
    }

    /// Check if Gravity is active
    pub fn is_gravity_active(&self) -> bool {
        self.field.global_effects.gravity.is_some()
    }

    /// Get the Pokemon at the specified position
    pub fn get_pokemon_at_position(&self, position: BattlePosition) -> Option<&Pokemon> {
        let side_index = match position.side {
            SideReference::SideOne => 0,
            SideReference::SideTwo => 1,
        };
        let side = self.get_side(side_index)?;
        side.get_active_pokemon_at_slot(position.slot)
    }

    /// Get a mutable reference to the Pokemon at the specified position
    pub fn get_pokemon_at_position_mut(
        &mut self,
        position: BattlePosition,
    ) -> Option<&mut Pokemon> {
        let side_index = match position.side {
            SideReference::SideOne => 0,
            SideReference::SideTwo => 1,
        };
        if side_index >= self.sides.len() {
            return None;
        }
        let side = &mut self.sides[side_index];
        side.get_active_pokemon_at_slot_mut(position.slot)
    }

    /// Get current weather
    pub fn weather(&self) -> Weather {
        self.field.weather.condition
    }

    /// Get current terrain
    pub fn terrain(&self) -> Terrain {
        self.field.terrain.condition
    }

    /// Get a reference to a side by SideReference
    pub fn get_side_by_ref(&self, side_ref: SideReference) -> &BattleSide {
        match side_ref {
            SideReference::SideOne => &self.sides[0],
            SideReference::SideTwo => &self.sides[1],
        }
    }

    /// Get a mutable reference to a side by SideReference
    pub fn get_side_by_ref_mut(&mut self, side_ref: SideReference) -> &mut BattleSide {
        match side_ref {
            SideReference::SideOne => &mut self.sides[0],
            SideReference::SideTwo => &mut self.sides[1],
        }
    }

    /// Get generation mechanics
    pub fn get_generation_mechanics(&self) -> crate::generation::GenerationMechanics {
        self.format.generation.get_mechanics()
    }

    /// Get the generation for this battle
    pub fn get_generation(&self) -> crate::generation::Generation {
        self.format.generation
    }

    /// Check if a generation feature is available in this battle
    pub fn has_generation_feature(&self, feature: crate::generation::GenerationFeature) -> bool {
        self.format.generation.get_mechanics().has_feature(feature)
    }

    /// Check if a position is active (has a Pokemon that isn't fainted)
    pub fn is_position_active(&self, position: BattlePosition) -> bool {
        if let Some(pokemon) = self.get_pokemon_at_position(position) {
            pokemon.hp > 0
        } else {
            false
        }
    }

    /// Apply a list of battle instructions to modify the state
    pub fn apply_instructions(&mut self, instructions: &[BattleInstruction]) {
        for instruction in instructions {
            self.apply_single_instruction(instruction);
        }
    }

    /// Apply a single battle instruction
    pub fn apply_instruction(&mut self, instruction: &BattleInstruction) {
        self.apply_single_instruction(instruction);
    }

    /// Apply a single battle instruction (internal helper)
    fn apply_single_instruction(&mut self, instruction: &BattleInstruction) {
        match instruction {
            BattleInstruction::Pokemon(pokemon_instr) => {
                self.apply_pokemon_instruction(pokemon_instr);
            }
            BattleInstruction::Field(field_instr) => {
                self.apply_field_instruction(field_instr);
            }
            BattleInstruction::Status(status_instr) => {
                self.apply_status_instruction(status_instr);
            }
            BattleInstruction::Stats(stats_instr) => {
                self.apply_stats_instruction(stats_instr);
            }
        }
    }

    /// Apply Pokemon instruction (damage, healing, switching, etc.)
    fn apply_pokemon_instruction(&mut self, instruction: &PokemonInstruction) {
        match instruction {
            PokemonInstruction::Damage { target, amount, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    // Check if Pokemon has a substitute
                    if pokemon.volatile_statuses.contains(&VolatileStatus::Substitute) && pokemon.substitute_health > 0 {
                        // Damage goes to substitute first
                        let remaining_substitute_health = pokemon.substitute_health - amount;
                        if remaining_substitute_health <= 0 {
                            // Substitute is broken, apply excess damage to Pokemon
                            pokemon.substitute_health = 0;
                            pokemon.volatile_statuses.remove(&VolatileStatus::Substitute);
                            let excess_damage = amount - pokemon.substitute_health;
                            if excess_damage > 0 {
                                pokemon.hp = (pokemon.hp - excess_damage).max(0);
                            }
                        } else {
                            // Substitute absorbs all damage
                            pokemon.substitute_health = remaining_substitute_health;
                        }
                    } else {
                        // No substitute or substitute broken, damage goes to Pokemon directly
                        pokemon.hp = (pokemon.hp - amount).max(0);
                    }
                }
            }
            PokemonInstruction::Heal { target, amount, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.hp = (pokemon.hp + amount).min(pokemon.max_hp);
                }
            }
            PokemonInstruction::MultiTargetDamage { target_damages, .. } => {
                for (target, damage) in target_damages {
                    if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                        // Apply damage directly to Pokemon HP
                        // Substitute handling should be done through explicit instructions
                        pokemon.hp = (pokemon.hp - damage).max(0);
                    }
                }
            }
            PokemonInstruction::Faint { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.hp = 0;
                    pokemon.status = PokemonStatus::None;
                    pokemon.volatile_statuses.clear();
                    pokemon.volatile_status_durations.clear();
                }
            }
            PokemonInstruction::Switch {
                position,
                new_pokemon,
                ..
            } => {
                let side_index = match position.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .set_active_pokemon_at_slot(position.slot, Some(*new_pokemon));
                }
            }
            PokemonInstruction::ChangeAbility {
                target,
                new_ability,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.ability = new_ability.to_string();
                }
            }
            PokemonInstruction::ChangeItem {
                target, new_item, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.item = new_item.clone();
                }
            }
            PokemonInstruction::ChangeType {
                target, new_types, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.types = new_types.clone();
                }
            }
            PokemonInstruction::ToggleTerastallized {
                target,
                terastallized,
                tera_type,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.is_terastallized = *terastallized;
                    pokemon.tera_type = *tera_type;
                }
            }
            PokemonInstruction::ChangeSubstituteHealth {
                target, new_health, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.substitute_health = *new_health;
                }
            }
            PokemonInstruction::SetWish {
                target,
                heal_amount,
                turns_remaining,
                ..
            } => {
                let side_index = match target.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .wish_healing
                        .insert(target.slot, (*heal_amount, *turns_remaining));
                }
            }
            PokemonInstruction::DecrementWish { target, .. } => {
                let side_index = match target.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    // First, check and update wish turns, storing heal amount for later use
                    let mut should_heal = false;
                    let mut heal_amount = 0;

                    if let Some((wish_heal_amount, turns)) =
                        self.sides[side_index].wish_healing.get_mut(&target.slot)
                    {
                        if *turns > 0 {
                            *turns -= 1;
                            if *turns == 0 {
                                should_heal = true;
                                heal_amount = *wish_heal_amount;
                            }
                        }
                    }

                    // Apply healing and cleanup separately to avoid borrowing conflicts
                    if should_heal {
                        if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                            pokemon.hp = (pokemon.hp + heal_amount).min(pokemon.max_hp);
                        }
                        self.sides[side_index].wish_healing.remove(&target.slot);
                    }
                }
            }
            // Handle other Pokemon instructions as needed
            _ => {
                // For now, log unhandled instructions for debugging
                eprintln!("Unhandled Pokemon instruction: {:?}", instruction);
            }
        }
    }

    /// Apply Field instruction (weather, terrain, global effects, side conditions)
    fn apply_field_instruction(&mut self, instruction: &FieldInstruction) {
        match instruction {
            FieldInstruction::Weather {
                new_weather,
                turns,
                source,
                ..
            } => {
                self.field.weather.set(*new_weather, *turns, *source);
            }
            FieldInstruction::Terrain {
                new_terrain,
                turns,
                source,
                ..
            } => {
                self.field.terrain.set(*new_terrain, *turns, *source);
            }
            FieldInstruction::TrickRoom {
                active,
                turns,
                source,
                ..
            } => {
                if *active {
                    if let Some(turn_count) = turns {
                        self.field
                            .global_effects
                            .set_trick_room(*turn_count, *source);
                    }
                } else {
                    self.field.global_effects.clear_trick_room();
                }
            }
            FieldInstruction::Gravity {
                active,
                turns,
                source,
                ..
            } => {
                if *active {
                    if let Some(turn_count) = turns {
                        self.field.global_effects.set_gravity(*turn_count, *source);
                    }
                } else {
                    self.field.global_effects.clear_gravity();
                }
            }
            FieldInstruction::ApplySideCondition {
                side,
                condition,
                duration,
                ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .side_conditions
                        .insert(*condition, *duration);
                }
            }
            FieldInstruction::RemoveSideCondition {
                side, condition, ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index].side_conditions.remove(condition);
                }
            }
            FieldInstruction::DecrementSideConditionDuration {
                side, condition, ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    if let Some(duration) =
                        self.sides[side_index].side_conditions.get_mut(condition)
                    {
                        if *duration > 0 {
                            *duration -= 1;
                            if *duration == 0 {
                                self.sides[side_index].side_conditions.remove(condition);
                            }
                        }
                    }
                }
            }
            FieldInstruction::DecrementWeatherTurns { .. } => {
                self.field.weather.decrement_turn();
            }
            FieldInstruction::DecrementTerrainTurns { .. } => {
                self.field.terrain.decrement_turn();
            }
            FieldInstruction::DecrementTrickRoomTurns { .. } => {
                self.field.global_effects.decrement_turn();
            }
            FieldInstruction::DecrementGravityTurns { .. } => {
                self.field.global_effects.decrement_turn();
            }
            FieldInstruction::ToggleForceSwitch { .. } => {
                // Force switch logic would be handled at a higher level
                // This is more of a metadata instruction for battle flow
            }
            FieldInstruction::ToggleBatonPassing { .. } => {
                // Baton passing logic would be handled at a higher level
                // This is more of a metadata instruction for switch mechanics
            }
            FieldInstruction::Message { .. } => {
                // Messages are for logging/debugging purposes and don't change state
                // Could be logged to a battle log if needed
            }
        }
    }

    /// Apply Status instruction (status conditions, volatile statuses)
    fn apply_status_instruction(&mut self, instruction: &StatusInstruction) {
        match instruction {
            StatusInstruction::Apply {
                target,
                status,
                duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = *status;
                    pokemon.status_duration = *duration;
                }
            }
            StatusInstruction::Remove { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::None;
                    pokemon.status_duration = None;
                }
            }
            StatusInstruction::ChangeDuration {
                target,
                new_duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status_duration = *new_duration;
                    if new_duration.is_none() || new_duration == &Some(0) {
                        pokemon.status = PokemonStatus::None;
                    }
                }
            }
            StatusInstruction::ApplyVolatile {
                target,
                status,
                duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.insert(*status);
                    if let Some(dur) = duration {
                        pokemon.volatile_status_durations.insert(*status, *dur);
                    }
                }
            }
            StatusInstruction::RemoveVolatile { target, status, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.remove(status);
                    pokemon.volatile_status_durations.remove(status);
                }
            }
            StatusInstruction::ChangeVolatileDuration {
                target,
                status,
                new_duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(new_dur) = new_duration {
                        pokemon.volatile_status_durations.insert(*status, *new_dur);
                        if *new_dur == 0 {
                            pokemon.volatile_statuses.remove(status);
                            pokemon.volatile_status_durations.remove(status);
                        }
                    } else {
                        pokemon.volatile_status_durations.remove(status);
                    }
                }
            }
            StatusInstruction::SetSleepTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::Sleep;
                    pokemon.status_duration = Some(*turns);
                }
            }
            StatusInstruction::SetRestTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::Sleep;
                    pokemon.status_duration = Some(*turns);
                    // Rest also heals to full HP
                    pokemon.hp = pokemon.max_hp;
                }
            }
            StatusInstruction::DecrementRestTurns { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(turns) = pokemon.status_duration {
                        if turns > 0 {
                            pokemon.status_duration = Some(turns - 1);
                            if turns - 1 == 0 {
                                pokemon.status = PokemonStatus::None;
                                pokemon.status_duration = None;
                            }
                        }
                    }
                }
            }
            StatusInstruction::DecrementPP {
                target,
                move_index,
                amount,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(move_data) = pokemon.get_move_mut(*move_index) {
                        move_data.pp = move_data.pp.saturating_sub(*amount);
                    }
                }
            }
            // Handle other status instructions as needed
            _ => {
                eprintln!("Unhandled Status instruction: {:?}", instruction);
            }
        }
    }

    /// Apply Stats instruction (stat boosts, raw stat changes)
    fn apply_stats_instruction(&mut self, instruction: &StatsInstruction) {
        match instruction {
            StatsInstruction::BoostStats {
                target,
                stat_changes,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    for (stat, change) in stat_changes {
                        let current_boost = pokemon.stat_boosts.get(stat).copied().unwrap_or(0);
                        let new_boost = (current_boost + change).clamp(-6, 6);
                        pokemon.stat_boosts.insert(*stat, new_boost);
                    }
                }
            }
            StatsInstruction::ChangeAttack {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.attack = *new_value;
                }
            }
            StatsInstruction::ChangeDefense {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.defense = *new_value;
                }
            }
            StatsInstruction::ChangeSpecialAttack {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.special_attack = *new_value;
                }
            }
            StatsInstruction::ChangeSpecialDefense {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.special_defense = *new_value;
                }
            }
            StatsInstruction::ChangeSpeed {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.speed = *new_value;
                }
            }
            StatsInstruction::ClearBoosts { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stat_boosts.clear();
                }
            }
            StatsInstruction::CopyBoosts {
                target,
                source,
                stats_to_copy,
                ..
            } => {
                if let Some(source_pokemon) = self.get_pokemon_at_position(*source) {
                    // Copy the boosts to apply to target
                    let mut boosts_to_copy = HashMap::new();
                    for stat in stats_to_copy {
                        if let Some(boost) = source_pokemon.stat_boosts.get(stat) {
                            boosts_to_copy.insert(*stat, *boost);
                        }
                    }

                    // Apply to target (need to get mutable reference after immutable one)
                    if let Some(target_pokemon) = self.get_pokemon_at_position_mut(*target) {
                        for (stat, boost) in boosts_to_copy {
                            target_pokemon.stat_boosts.insert(stat, boost);
                        }
                    }
                }
            }
            StatsInstruction::SwapBoosts {
                target1,
                target2,
                stats_to_swap,
                ..
            } => {
                // Collect boosts from both Pokemon
                let mut target1_boosts = HashMap::new();
                let mut target2_boosts = HashMap::new();

                if let Some(pokemon1) = self.get_pokemon_at_position(*target1) {
                    for stat in stats_to_swap {
                        target1_boosts
                            .insert(*stat, pokemon1.stat_boosts.get(stat).copied().unwrap_or(0));
                    }
                }

                if let Some(pokemon2) = self.get_pokemon_at_position(*target2) {
                    for stat in stats_to_swap {
                        target2_boosts
                            .insert(*stat, pokemon2.stat_boosts.get(stat).copied().unwrap_or(0));
                    }
                }

                // Apply swapped boosts
                if let Some(pokemon1) = self.get_pokemon_at_position_mut(*target1) {
                    for (stat, boost) in &target2_boosts {
                        pokemon1.stat_boosts.insert(*stat, *boost);
                    }
                }

                if let Some(pokemon2) = self.get_pokemon_at_position_mut(*target2) {
                    for (stat, boost) in &target1_boosts {
                        pokemon2.stat_boosts.insert(*stat, *boost);
                    }
                }
            }
            StatsInstruction::InvertBoosts {
                target,
                stats_to_invert,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    for stat in stats_to_invert {
                        if let Some(boost) = pokemon.stat_boosts.get_mut(stat) {
                            *boost = -*boost;
                        }
                    }
                }
            }
        }
    }

    /// Check if the battle has ended (all Pokemon fainted on one side)
    pub fn is_battle_over(&self) -> bool {
        let side_one_has_usable = self.sides[0].pokemon.iter().any(|p| p.hp > 0);
        let side_two_has_usable = self.sides[1].pokemon.iter().any(|p| p.hp > 0);

        !side_one_has_usable || !side_two_has_usable
    }

    /// Determine which side (0 or 1) has won, if any
    pub fn get_winner(&self) -> Option<usize> {
        let side_one_has_usable = self.sides[0].pokemon.iter().any(|p| p.hp > 0);
        let side_two_has_usable = self.sides[1].pokemon.iter().any(|p| p.hp > 0);

        match (side_one_has_usable, side_two_has_usable) {
            (false, true) => Some(1), // Side two wins
            (true, false) => Some(0), // Side one wins
            _ => None,                // Battle ongoing or tie
        }
    }

    /// Get all legal move options for both sides
    pub fn get_all_options(&self) -> (Vec<MoveChoice>, Vec<MoveChoice>) {
        let side_one_options = self.get_side_options(0);
        let side_two_options = self.get_side_options(1);
        (side_one_options, side_two_options)
    }

    /// Get move options for a specific side
    fn get_side_options(&self, side_index: usize) -> Vec<MoveChoice> {
        let mut options = Vec::new();

        if let Some(side) = self.get_side(side_index) {
            let active_count = self.format.active_pokemon_count();

            for slot in 0..active_count {
                if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                    if pokemon.hp > 0 {
                        // Add move options
                        for (move_index, move_data) in &pokemon.moves {
                            if move_data.pp > 0 {
                                let targets = self.get_valid_targets_for_move(
                                    side_index,
                                    slot,
                                    &move_data.target,
                                );
                                options.push(MoveChoice::new_move(*move_index, targets));
                            }
                        }

                        // Add switch options if there are benched Pokemon
                        for (i, bench_pokemon) in side.pokemon.iter().enumerate() {
                            if bench_pokemon.hp > 0
                                && !side.active_pokemon_indices.contains(&Some(i))
                            {
                                if let Some(pokemon_index) = PokemonIndex::from_index(i) {
                                    options.push(MoveChoice::new_switch(pokemon_index));
                                }
                            }
                        }
                    }
                }
            }
        }

        options
    }

    /// Get valid targets for a move based on its target type and format
    fn get_valid_targets_for_move(
        &self,
        user_side_index: usize,
        user_slot: usize,
        move_target: &crate::data::showdown_types::MoveTarget,
    ) -> Vec<BattlePosition> {
        let mut targets = Vec::new();
        let active_count = self.format.active_pokemon_count();
        let opponent_side_index = 1 - user_side_index;

        let user_side_ref = if user_side_index == 0 {
            SideReference::SideOne
        } else {
            SideReference::SideTwo
        };

        let opponent_side_ref = if opponent_side_index == 0 {
            SideReference::SideOne
        } else {
            SideReference::SideTwo
        };

        match move_target {
            crate::data::showdown_types::MoveTarget::Self_ => {
                // Target the user
                targets.push(BattlePosition {
                    side: user_side_ref,
                    slot: user_slot,
                });
            }
            crate::data::showdown_types::MoveTarget::Normal
            | crate::data::showdown_types::MoveTarget::AdjacentFoe => {
                // Target adjacent opponents
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
                // In singles, just return the first valid target
                if active_count == 1 && !targets.is_empty() {
                    targets.truncate(1);
                }
            }
            crate::data::showdown_types::MoveTarget::AllAdjacentFoes => {
                // All adjacent opponents (spread move)
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AllAdjacent => {
                // All adjacent Pokemon (including allies)
                for slot in 0..active_count {
                    if slot != user_slot {
                        let ally_position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(ally_position) {
                            targets.push(ally_position);
                        }
                    }
                    let opponent_position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(opponent_position) {
                        targets.push(opponent_position);
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AdjacentAlly => {
                // Adjacent allies only
                for slot in 0..active_count {
                    if slot != user_slot {
                        let position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AdjacentAllyOrSelf => {
                // User or adjacent ally
                targets.push(BattlePosition {
                    side: user_side_ref,
                    slot: user_slot,
                });
                for slot in 0..active_count {
                    if slot != user_slot {
                        let position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::All
            | crate::data::showdown_types::MoveTarget::AllySide
            | crate::data::showdown_types::MoveTarget::FoeSide
            | crate::data::showdown_types::MoveTarget::AllyTeam => {
                // Field-wide moves don't need specific targets
                // Return empty vector as they affect the field/side itself
            }
            crate::data::showdown_types::MoveTarget::Any => {
                // Can target any active Pokemon
                for side_idx in 0..2 {
                    let side_ref = if side_idx == 0 {
                        SideReference::SideOne
                    } else {
                        SideReference::SideTwo
                    };
                    for slot in 0..active_count {
                        let position = BattlePosition {
                            side: side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::RandomNormal => {
                // Random opponent - collect all valid opponents
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
            _ => {
                // For other target types, default to normal targeting
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
        }

        targets
    }

    /// Get damage tracking information for counter moves
    pub fn get_damage_dealt(&self, side_index: usize) -> &DamageDealt {
        &self.sides[side_index].damage_dealt
    }

    /// Reset damage tracking at turn start
    pub fn reset_damage_dealt(&mut self) {
        for side in &mut self.sides {
            side.damage_dealt.reset();
        }
    }

    /// Advance turn counter and handle turn-based effects
    pub fn update_turn(&mut self) {
        self.turn_info.next_turn();

        // Reset ability triggered flags for all Pokemon
        for side in &mut self.sides {
            for pokemon in &mut side.pokemon {
                pokemon.ability_triggered_this_turn = false;
            }
        }

        // Decrement field effect durations
        self.field.weather.decrement_turn();
        self.field.terrain.decrement_turn();
        self.field.global_effects.decrement_turn();
    }

    /// Track that a position has used a move this turn
    pub fn track_move_used(&mut self, position: BattlePosition) {
        self.turn_info.mark_moved(position);
    }

    /// Track that a position has taken damage this turn
    pub fn track_damage_taken(
        &mut self,
        target: BattlePosition,
        attacker: BattlePosition,
        damage: i16,
        move_category: MoveCategory,
        is_direct: bool,
    ) {
        let damage_info = DamageInfo::new(damage, move_category, attacker, is_direct);
        self.turn_info.mark_damaged(target, damage_info);
    }

    /// Check if user took damage from a physical/special move and moved second this turn
    pub fn user_moved_after_taking_damage(&self, user_position: BattlePosition) -> bool {
        self.turn_info.took_damage_from_attack(user_position)
            && self.turn_info.has_moved(user_position)
    }

    /// Generate a human-readable string representation of the battle state
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Battle Turn {} ===\n", self.turn_info.number));
        output.push_str(&format!("Format: {}\n", self.format.name));
        output.push_str(&format!("Weather: {:?}\n", self.field.weather.condition));
        output.push_str(&format!("Terrain: {:?}\n", self.field.terrain.condition));

        if self.is_trick_room_active() {
            output.push_str("Trick Room is active\n");
        }
        if self.is_gravity_active() {
            output.push_str("Gravity is active\n");
        }

        output.push_str("\n--- Side One ---\n");
        output.push_str(&self.format_side(&self.sides[0], 0));

        output.push_str("\n--- Side Two ---\n");
        output.push_str(&self.format_side(&self.sides[1], 1));

        output
    }

    /// Format a side for pretty printing
    fn format_side(&self, side: &BattleSide, side_index: usize) -> String {
        let mut output = String::new();
        let active_count = self.format.active_pokemon_count();

        output.push_str("Active Pokemon:\n");
        for slot in 0..active_count {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                output.push_str(&format!(
                    "  Slot {}: {} ({}/{} HP)\n",
                    slot, pokemon.species, pokemon.hp, pokemon.max_hp
                ));

                if pokemon.status != PokemonStatus::None {
                    output.push_str(&format!("    Status: {:?}\n", pokemon.status));
                }

                if !pokemon.volatile_statuses.is_empty() {
                    output.push_str(&format!("    Volatile: {:?}\n", pokemon.volatile_statuses));
                }
            } else {
                output.push_str(&format!("  Slot {}: Empty\n", slot));
            }
        }

        if !side.side_conditions.is_empty() {
            output.push_str(&format!("Side Conditions: {:?}\n", side.side_conditions));
        }

        output
    }
}

impl Default for FieldConditions {
    fn default() -> Self {
        Self {
            weather: WeatherState::default(),
            terrain: TerrainState::default(),
            global_effects: GlobalEffects::default(),
        }
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            condition: Weather::None,
            turns_remaining: None,
            source: None,
        }
    }
}

impl WeatherState {
    /// Set weather with specified duration and source
    pub fn set(&mut self, condition: Weather, turns: Option<u8>, source: Option<BattlePosition>) {
        self.condition = condition;
        self.turns_remaining = turns;
        self.source = source;
    }

    /// Clear weather
    pub fn clear(&mut self) {
        self.condition = Weather::None;
        self.turns_remaining = None;
        self.source = None;
    }

    /// Decrement weather duration by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(turns) = &mut self.turns_remaining {
            if *turns > 0 {
                *turns -= 1;
                if *turns == 0 {
                    self.clear();
                }
            }
        }
    }
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            condition: Terrain::None,
            turns_remaining: None,
            source: None,
        }
    }
}

impl TerrainState {
    /// Set terrain with specified duration and source
    pub fn set(&mut self, condition: Terrain, turns: Option<u8>, source: Option<BattlePosition>) {
        self.condition = condition;
        self.turns_remaining = turns;
        self.source = source;
    }

    /// Clear terrain
    pub fn clear(&mut self) {
        self.condition = Terrain::None;
        self.turns_remaining = None;
        self.source = None;
    }

    /// Decrement terrain duration by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(turns) = &mut self.turns_remaining {
            if *turns > 0 {
                *turns -= 1;
                if *turns == 0 {
                    self.clear();
                }
            }
        }
    }
}

impl Default for GlobalEffects {
    fn default() -> Self {
        Self {
            trick_room: None,
            gravity: None,
        }
    }
}

impl GlobalEffects {
    /// Set Trick Room with specified duration and source
    pub fn set_trick_room(&mut self, turns: u8, source: Option<BattlePosition>) {
        self.trick_room = Some(TrickRoomState {
            turns_remaining: turns,
            source,
        });
    }

    /// Clear Trick Room
    pub fn clear_trick_room(&mut self) {
        self.trick_room = None;
    }

    /// Set Gravity with specified duration and source
    pub fn set_gravity(&mut self, turns: u8, source: Option<BattlePosition>) {
        self.gravity = Some(GravityState {
            turns_remaining: turns,
            source,
        });
    }

    /// Clear Gravity
    pub fn clear_gravity(&mut self) {
        self.gravity = None;
    }

    /// Decrement all global effect durations by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(trick_room) = &mut self.trick_room {
            if trick_room.turns_remaining > 0 {
                trick_room.turns_remaining -= 1;
                if trick_room.turns_remaining == 0 {
                    self.trick_room = None;
                }
            }
        }

        if let Some(gravity) = &mut self.gravity {
            if gravity.turns_remaining > 0 {
                gravity.turns_remaining -= 1;
                if gravity.turns_remaining == 0 {
                    self.gravity = None;
                }
            }
        }
    }
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            number: 1,
            phase: TurnPhase::Selection,
            moved_this_turn: Vec::new(),
            damaged_this_turn: HashMap::new(),
        }
    }
}

impl TurnState {
    /// Advance to the next turn
    pub fn next_turn(&mut self) {
        self.number += 1;
        self.phase = TurnPhase::Selection;
        self.moved_this_turn.clear();
        self.damaged_this_turn.clear();
    }

    /// Set the current turn phase
    pub fn set_phase(&mut self, phase: TurnPhase) {
        self.phase = phase;
    }

    /// Mark a position as having moved this turn
    pub fn mark_moved(&mut self, position: BattlePosition) {
        if !self.moved_this_turn.contains(&position) {
            self.moved_this_turn.push(position);
        }
    }

    /// Mark a position as having taken damage this turn
    pub fn mark_damaged(&mut self, position: BattlePosition, damage_info: DamageInfo) {
        self.damaged_this_turn.insert(position, damage_info);
    }

    /// Check if a position has moved this turn
    pub fn has_moved(&self, position: BattlePosition) -> bool {
        self.moved_this_turn.contains(&position)
    }

    /// Check if a position took damage this turn from a physical or special move
    pub fn took_damage_from_attack(&self, position: BattlePosition) -> bool {
        if let Some(damage_info) = self.damaged_this_turn.get(&position) {
            damage_info.is_direct_damage
                && (damage_info.move_category == MoveCategory::Physical
                    || damage_info.move_category == MoveCategory::Special)
        } else {
            false
        }
    }

    /// Check if user moved after taking damage (for Avalanche mechanics)
    pub fn moved_after_damage(
        &self,
        user_position: BattlePosition,
        attacker_position: BattlePosition,
    ) -> bool {
        // If user took damage from the attacker, check if user moved after attacker
        if let Some(damage_info) = self.damaged_this_turn.get(&user_position) {
            if damage_info.attacker_position == attacker_position {
                // Check if user moved after the attacker by looking at move order
                let user_move_index = self
                    .moved_this_turn
                    .iter()
                    .position(|&pos| pos == user_position);
                let attacker_move_index = self
                    .moved_this_turn
                    .iter()
                    .position(|&pos| pos == attacker_position);

                match (user_move_index, attacker_move_index) {
                    (Some(user_idx), Some(attacker_idx)) => user_idx > attacker_idx,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}
