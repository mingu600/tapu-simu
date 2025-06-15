//! Enhanced event context system matching Pokemon Showdown's architecture
//! 
//! This provides event handlers with access to all battle state and utilities
//! they need to make decisions and apply effects.

use std::collections::HashMap;
use crate::battle_state::BattleState;
use crate::prng::PRNGState;
use crate::side::SideId;
use crate::pokemon::{PokemonRef, Pokemon};
use crate::errors::{BattleResult, BattleError};
use crate::events::relay_vars::{RelayVar, RelayContainer};
use crate::types::{Type, MoveCategory};
use serde::{Deserialize, Serialize};

/// Enhanced event context that provides handlers with full battle access
#[derive(Debug)]
pub struct EventContext<'a> {
    /// Current event being processed
    pub event_id: String,
    
    /// Target of the event (Pokemon, Side, or Field)
    pub target: Option<EventTarget>,
    
    /// Source of the event (Pokemon, Move, Ability, etc.)
    pub source: Option<EventSource>,
    
    /// Effect that triggered this event
    pub effect: Option<EffectData>,
    
    /// Mutable reference to battle state
    pub battle_state: &'a mut BattleState,
    
    /// PRNGState for random number generation
    pub prng: &'a mut PRNGState,
    
    /// Current turn number
    pub turn: u32,
    
    /// Event depth (for recursion protection)
    pub event_depth: u8,
    
    /// Whether this is a suppressed event
    pub suppressed: bool,
    
    /// Custom event data
    pub custom_data: HashMap<String, serde_json::Value>,
}

impl<'a> EventContext<'a> {
    /// Create a new event context
    pub fn new(
        event_id: String,
        target: Option<EventTarget>,
        source: Option<EventSource>,
        effect: Option<EffectData>,
        battle_state: &'a mut BattleState,
        prng: &'a mut PRNGState,
        turn: u32,
        event_depth: u8,
    ) -> Self {
        Self {
            event_id,
            target,
            source,
            effect,
            battle_state,
            prng,
            turn,
            event_depth,
            suppressed: false,
            custom_data: HashMap::new(),
        }
    }
    
    /// Get a Pokemon by reference
    pub fn get_pokemon(&self, pokemon_ref: PokemonRef) -> BattleResult<&Pokemon> {
        let side = self.battle_state.get_side(pokemon_ref.side)?;
        side.pokemon.get(pokemon_ref.position)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid Pokemon position: {}", pokemon_ref.position)))
    }
    
    /// Get a mutable Pokemon by reference
    pub fn get_pokemon_mut(&mut self, pokemon_ref: PokemonRef) -> BattleResult<&mut Pokemon> {
        let side = self.battle_state.get_side_mut(pokemon_ref.side)?;
        side.pokemon.get_mut(pokemon_ref.position)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid Pokemon position: {}", pokemon_ref.position)))
    }
    
    /// Get the source Pokemon if the source is a Pokemon
    pub fn get_source_pokemon(&self) -> BattleResult<Option<&Pokemon>> {
        match &self.source {
            Some(EventSource::Pokemon(pokemon_ref)) => {
                Ok(Some(self.get_pokemon(*pokemon_ref)?))
            }
            _ => Ok(None),
        }
    }
    
    /// Get the target Pokemon if the target is a Pokemon
    pub fn get_target_pokemon(&self) -> BattleResult<Option<&Pokemon>> {
        match &self.target {
            Some(EventTarget::Pokemon(pokemon_ref)) => {
                Ok(Some(self.get_pokemon(*pokemon_ref)?))
            }
            _ => Ok(None),
        }
    }
    
    /// Get the target Pokemon mutably if the target is a Pokemon
    pub fn get_target_pokemon_mut(&mut self) -> BattleResult<Option<&mut Pokemon>> {
        match self.target.clone() {
            Some(EventTarget::Pokemon(pokemon_ref)) => {
                Ok(Some(self.get_pokemon_mut(pokemon_ref)?))
            }
            _ => Ok(None),
        }
    }
    
    /// Get all active Pokemon on a side
    pub fn get_active_pokemon(&self, side_id: SideId) -> BattleResult<Vec<&Pokemon>> {
        let side = self.battle_state.get_side(side_id)?;
        let mut active_pokemon = Vec::new();
        
        for &pokemon_index in &side.active {
            if let Some(pokemon_index) = pokemon_index {
                if let Some(pokemon) = side.pokemon.get(pokemon_index) {
                    active_pokemon.push(pokemon);
                }
            }
        }
        
        Ok(active_pokemon)
    }
    
    /// Get all active Pokemon on the opposing side(s)
    pub fn get_opponents(&self, side_id: SideId) -> BattleResult<Vec<&Pokemon>> {
        let mut opponents = Vec::new();
        
        for side in &self.battle_state.sides {
            if side.id != side_id {
                for &pokemon_index in &side.active {
                    if let Some(pokemon_index) = pokemon_index {
                        if let Some(pokemon) = side.pokemon.get(pokemon_index) {
                            opponents.push(pokemon);
                        }
                    }
                }
            }
        }
        
        Ok(opponents)
    }
    
    /// Check if a Pokemon has a specific ability
    pub fn has_ability(&self, pokemon_ref: PokemonRef, ability_id: &str) -> BattleResult<bool> {
        let pokemon = self.get_pokemon(pokemon_ref)?;
        Ok(pokemon.ability.id == ability_id)
    }
    
    /// Check if a Pokemon has a specific item
    pub fn has_item(&self, pokemon_ref: PokemonRef, item_id: &str) -> BattleResult<bool> {
        let pokemon = self.get_pokemon(pokemon_ref)?;
        Ok(pokemon.item.as_ref().map_or(false, |item| item.id == item_id))
    }
    
    /// Check if a Pokemon has a specific type
    pub fn has_type(&self, pokemon_ref: PokemonRef, type_: Type) -> BattleResult<bool> {
        let pokemon = self.get_pokemon(pokemon_ref)?;
        Ok(pokemon.types.contains(&type_))
    }
    
    /// Get type effectiveness between two types
    pub fn get_type_effectiveness(&self, attacking_type: Type, defending_pokemon: PokemonRef) -> BattleResult<f32> {
        let pokemon = self.get_pokemon(defending_pokemon)?;
        let defending_types = pokemon.get_effective_types();
        
        let mut effectiveness = 1.0;
        for defending_type in defending_types {
            effectiveness *= self.calculate_type_effectiveness(attacking_type, defending_type);
        }
        
        Ok(effectiveness)
    }
    
    /// Calculate type effectiveness between two individual types
    fn calculate_type_effectiveness(&self, attacking: Type, defending: Type) -> f32 {
        // This would use the type chart from battle_state or a static type chart
        // For now, return neutral effectiveness
        // TODO: Implement full type chart lookup
        1.0
    }
    
    /// Apply damage to a Pokemon
    pub fn apply_damage(&mut self, pokemon_ref: PokemonRef, damage: u16) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        pokemon.take_damage(damage);
        Ok(())
    }
    
    /// Heal a Pokemon
    pub fn heal_pokemon(&mut self, pokemon_ref: PokemonRef, amount: u16) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        pokemon.heal(amount);
        Ok(())
    }
    
    /// Apply stat boosts to a Pokemon
    pub fn boost_stat(&mut self, pokemon_ref: PokemonRef, stat: &str, amount: i8) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        
        match stat {
            "attack" | "atk" => pokemon.boosts.attack = (pokemon.boosts.attack + amount).clamp(-6, 6),
            "defense" | "def" => pokemon.boosts.defense = (pokemon.boosts.defense + amount).clamp(-6, 6),
            "spatk" | "spa" => pokemon.boosts.special_attack = (pokemon.boosts.special_attack + amount).clamp(-6, 6),
            "spdef" | "spd" => pokemon.boosts.special_defense = (pokemon.boosts.special_defense + amount).clamp(-6, 6),
            "speed" | "spe" => pokemon.boosts.speed = (pokemon.boosts.speed + amount).clamp(-6, 6),
            "accuracy" | "acc" => pokemon.boosts.accuracy = (pokemon.boosts.accuracy + amount).clamp(-6, 6),
            "evasion" | "eva" => pokemon.boosts.evasion = (pokemon.boosts.evasion + amount).clamp(-6, 6),
            _ => return Err(BattleError::InvalidMove(format!("Unknown stat: {}", stat))),
        }
        
        Ok(())
    }
    
    /// Set a status condition on a Pokemon
    pub fn set_status(&mut self, pokemon_ref: PokemonRef, status: crate::pokemon::StatusCondition) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        
        // Check if Pokemon already has a status
        if pokemon.status.is_some() {
            return Ok(()); // Cannot override major status
        }
        
        pokemon.status = Some(status);
        Ok(())
    }
    
    /// Add a volatile status to a Pokemon
    pub fn add_volatile(&mut self, pokemon_ref: PokemonRef, volatile_id: String, data: Option<HashMap<String, serde_json::Value>>) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        
        let volatile_status = crate::pokemon::VolatileStatus {
            id: volatile_id.clone(),
            duration: None,
            data: data.unwrap_or_default(),
        };
        
        pokemon.volatiles.insert(volatile_id, volatile_status);
        Ok(())
    }
    
    /// Check if Pokemon has a volatile status
    pub fn has_volatile(&self, pokemon_ref: PokemonRef, volatile_id: &str) -> BattleResult<bool> {
        let pokemon = self.get_pokemon(pokemon_ref)?;
        Ok(pokemon.volatiles.contains_key(volatile_id))
    }
    
    /// Remove a volatile status from a Pokemon
    pub fn remove_volatile(&mut self, pokemon_ref: PokemonRef, volatile_id: &str) -> BattleResult<()> {
        let pokemon = self.get_pokemon_mut(pokemon_ref)?;
        pokemon.volatiles.remove(volatile_id);
        Ok(())
    }
    
    /// Generate a random number from 0 to max-1
    pub fn random(&mut self, max: u32) -> u32 {
        self.prng.next_u32() % max
    }
    
    /// Check random chance (returns true if random roll is less than chance)
    pub fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool {
        self.random(denominator) < numerator
    }
    
    /// Log a battle event
    pub fn log(&mut self, message: String) {
        // TODO: Implement battle logging system
        // For now, we'll just store in custom_data
        let log_key = format!("log_{}", self.turn);
        self.custom_data.insert(log_key, serde_json::Value::String(message));
    }
    
    /// Add custom data to the event context
    pub fn add_data(&mut self, key: String, value: serde_json::Value) {
        self.custom_data.insert(key, value);
    }
    
    /// Get custom data from the event context
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_data.get(key)
    }
    
    /// Check if event should be suppressed
    pub fn is_suppressed(&self) -> bool {
        self.suppressed
    }
    
    /// Suppress this event
    pub fn suppress(&mut self) {
        self.suppressed = true;
    }
}

/// Target of an event - Pokemon, Side, or Field
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTarget {
    /// Specific Pokemon
    Pokemon(PokemonRef),
    /// Entire side
    Side(SideId),
    /// Battle field
    Field,
}

/// Source of an event - what caused this event to trigger
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSource {
    /// Pokemon performing an action
    Pokemon(PokemonRef),
    /// Side effect
    Side(SideId),
    /// Field effect
    Field,
    /// Move being used
    Move(String),
    /// Ability triggering
    Ability(String),
    /// Item effect
    Item(String),
    /// Status condition
    Status(String),
    /// Effect triggering
    Effect(String),
    /// Format rule
    Format(String),
}

/// Effect data for abilities, items, moves, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectData {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Type of effect
    pub effect_type: EffectType,
    /// Source Pokemon or effect
    pub source: Option<String>,
    /// Duration (if applicable)
    pub duration: Option<u32>,
    /// Custom effect data
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Types of effects in Pokemon battles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    Move,
    Ability,
    Item,
    Status,
    Volatile,
    SideCondition,
    FieldCondition,
    Weather,
    Terrain,
    Species,
    Format,
    Rule,
}

impl EffectData {
    /// Create effect data for an ability
    pub fn ability(id: String, name: String, source: Option<String>) -> Self {
        Self {
            id,
            name,
            effect_type: EffectType::Ability,
            source,
            duration: None,
            custom_data: HashMap::new(),
        }
    }
    
    /// Create effect data for an item
    pub fn item(id: String, name: String, source: Option<String>) -> Self {
        Self {
            id,
            name,
            effect_type: EffectType::Item,
            source,
            duration: None,
            custom_data: HashMap::new(),
        }
    }
    
    /// Create effect data for a move
    pub fn move_effect(id: String, name: String, source: Option<String>) -> Self {
        Self {
            id,
            name,
            effect_type: EffectType::Move,
            source,
            duration: None,
            custom_data: HashMap::new(),
        }
    }
    
    /// Create effect data for a status condition
    pub fn status(id: String, name: String, source: Option<String>) -> Self {
        Self {
            id,
            name,
            effect_type: EffectType::Status,
            source,
            duration: None,
            custom_data: HashMap::new(),
        }
    }
}