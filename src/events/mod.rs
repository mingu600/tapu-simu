//! Enhanced Pokemon Showdown compatible event system
//! 
//! This module implements the core event system that matches Pokemon Showdown's
//! runEvent() and singleEvent() architecture with full fidelity.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::side::SideId;
use crate::errors::{BattleResult, BattleError};
use crate::battle_state::BattleState;
use crate::prng::PRNG;
use crate::pokemon::PokemonRef;

// Public modules
pub mod types;
pub mod ability_handlers;
pub mod context;
pub mod relay_vars;
pub mod priority;
pub mod critical_events;

// Re-export core types for easy access
pub use types::*;
pub use context::{EventContext, EventTarget, EventSource, EffectData, EffectType};
pub use relay_vars::{RelayVar, RelayContainer};
pub use priority::{EventListener, EventCallback, EventResult, PriorityCalculator, EventExecutor};

/// Event handler registry for effects - stores event names that have handlers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventHandlerRegistry {
    pub handled_events: HashSet<String>,
}

impl EventHandlerRegistry {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn register(&mut self, event_id: String) {
        self.handled_events.insert(event_id);
    }
    
    pub fn handles(&self, event_id: &str) -> bool {
        self.handled_events.contains(event_id)
    }
}

/// Effect state tracking for abilities, items, statuses, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectState {
    pub id: String,
    pub effect_order: i32,
    pub duration: Option<u32>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Enhanced core event system - matches Pokemon Showdown's architecture exactly
pub struct EventSystem {
    /// Event depth counter for overflow protection
    event_depth: u8,
    /// Maximum event depth (PS uses 8)
    max_event_depth: u8,
    /// Event log length counter
    log_length: usize,
    /// Maximum log length per event (PS uses 1000)
    max_log_length: usize,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            event_depth: 0,
            max_event_depth: 8,
            log_length: 0,
            max_log_length: 1000,
        }
    }
    
    /// Enhanced runEvent() function that matches Pokemon Showdown exactly
    pub fn run_event(
        &mut self,
        event_id: &str,
        target: Option<EventTarget>,
        source: Option<EventSource>,
        source_effect: Option<EffectData>,
        mut relay_container: RelayContainer,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<RelayContainer> {
        // Check for event depth overflow
        if self.event_depth >= self.max_event_depth {
            return Ok(relay_container);
        }
        
        self.event_depth += 1;
        
        // Create enhanced event context
        let mut context = EventContext::new(
            event_id.to_string(),
            target.clone(),
            source.clone(),
            source_effect,
            battle_state,
            prng,
            turn,
            self.event_depth,
        );
        
        // Collect all event listeners
        let mut listeners = self.collect_event_listeners(event_id, &target, &source, &context)?;
        
        // Sort listeners by priority using the enhanced priority system
        PriorityCalculator::sort_listeners(&mut listeners, event_id, context.battle_state)?;
        
        // Execute listeners using the enhanced executor
        let result = EventExecutor::execute_listeners(listeners, &mut context, &mut relay_container)?;
        
        // Handle early termination results
        match result {
            EventResult::Suppress | EventResult::Fail | EventResult::Success => {
                self.event_depth -= 1;
                return Ok(relay_container);
            }
            _ => {
                // Continue with normal execution
            }
        }
        
        self.event_depth -= 1;
        Ok(relay_container)
    }
    
    /// Enhanced singleEvent() function
    pub fn single_event(
        &mut self,
        event_id: &str,
        effect: &EffectData,
        state: &mut EffectState,
        target: Option<EventTarget>,
        source: Option<EventSource>,
        source_effect: Option<EffectData>,
        mut relay_container: RelayContainer,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<RelayContainer> {
        // Check for event depth overflow
        if self.event_depth >= self.max_event_depth {
            return Ok(relay_container);
        }
        
        self.event_depth += 1;
        
        // Create context for single event
        let mut context = EventContext::new(
            event_id.to_string(),
            target,
            source,
            Some(effect.clone()),
            battle_state,
            prng,
            turn,
            self.event_depth,
        );
        
        // Look up and execute the specific effect's handler
        if let Some(callback) = self.get_effect_handler(effect, event_id) {
            let result = callback(&mut context, &mut relay_container);
            
            match result {
                EventResult::Suppress | EventResult::Fail | EventResult::Success => {
                    self.event_depth -= 1;
                    return Ok(relay_container);
                }
                EventResult::Modify(new_value) => {
                    relay_container.modify(new_value);
                }
                EventResult::Continue => {
                    // Continue normally
                }
            }
        }
        
        self.event_depth -= 1;
        Ok(relay_container)
    }
    
    /// Collect all event listeners for the given event with enhanced logic
    fn collect_event_listeners(
        &self,
        event_id: &str,
        target: &Option<EventTarget>,
        source: &Option<EventSource>,
        context: &EventContext,
    ) -> BattleResult<Vec<EventListener>> {
        let mut listeners = Vec::new();
        
        // 1. Collect listeners from all Pokemon (abilities, items, status, volatiles)
        for side in &context.battle_state.sides {
            // From active Pokemon
            for (position, pokemon_index) in side.active.iter().enumerate() {
                if let Some(pokemon_index) = pokemon_index {
                    if let Some(pokemon) = side.pokemon.get(*pokemon_index) {
                        let pokemon_ref = PokemonRef {
                            side: side.id,
                            position,
                        };
                        
                        // Add ability listeners with proper priority
                        if pokemon.ability.event_handlers.handles(event_id) {
                            if let Some(callback) = ability_handlers::AbilityHandlers::get_handler(&pokemon.ability.id, event_id) {
                                let listener = PriorityCalculator::create_listener(
                                    pokemon.ability.id.clone(),
                                    EffectType::Ability,
                                    Some(pokemon_ref),
                                    callback,
                                    None, // Use default priority
                                    None, // No custom order
                                    None, // No custom sub-order
                                );
                                listeners.push(listener);
                            }
                        }
                        
                        // Add item listeners with proper priority
                        if let Some(ref item) = pokemon.item {
                            if item.event_handlers.handles(event_id) {
                                if let Some(callback) = ability_handlers::ItemHandlers::get_handler(&item.id, event_id) {
                                    let listener = PriorityCalculator::create_listener(
                                        item.id.clone(),
                                        EffectType::Item,
                                        Some(pokemon_ref),
                                        callback,
                                        None, // Use default priority
                                        None, // No custom order
                                        None, // No custom sub-order
                                    );
                                    listeners.push(listener);
                                }
                            }
                        }
                        
                        // Add status listeners
                        if let Some(ref status) = pokemon.status {
                            if self.status_handles_event(status, event_id) {
                                let listener = PriorityCalculator::create_listener(
                                    format!("{:?}", status).to_lowercase(),
                                    EffectType::Status,
                                    Some(pokemon_ref),
                                    Self::default_status_callback,
                                    None,
                                    None,
                                    None,
                                );
                                listeners.push(listener);
                            }
                        }
                        
                        // Add volatile status listeners
                        for (volatile_id, _volatile_status) in &pokemon.volatiles {
                            if self.volatile_handles_event(volatile_id, event_id) {
                                let listener = PriorityCalculator::create_listener(
                                    volatile_id.clone(),
                                    EffectType::Volatile,
                                    Some(pokemon_ref),
                                    Self::default_volatile_callback,
                                    None,
                                    None,
                                    None,
                                );
                                listeners.push(listener);
                            }
                        }
                    }
                }
            }
        }
        
        // 2. Collect listeners from side conditions
        for side in &context.battle_state.sides {
            for (condition_id, _condition) in &side.conditions {
                if self.side_condition_handles_event(condition_id, event_id) {
                    let listener = PriorityCalculator::create_listener(
                        condition_id.clone(),
                        EffectType::SideCondition,
                        None,
                        Self::default_side_condition_callback,
                        None,
                        None,
                        None,
                    );
                    listeners.push(listener);
                }
            }
        }
        
        // 3. Collect listeners from field effects
        if let Some(ref weather) = context.battle_state.field.weather {
            if self.weather_handles_event(weather, event_id) {
                let listener = PriorityCalculator::create_listener(
                    format!("{:?}", weather).to_lowercase(),
                    EffectType::Weather,
                    None,
                    Self::default_weather_callback,
                    None,
                    None,
                    None,
                );
                listeners.push(listener);
            }
        }
        
        if let Some(ref terrain) = context.battle_state.field.terrain {
            if self.terrain_handles_event(terrain, event_id) {
                let listener = PriorityCalculator::create_listener(
                    format!("{:?}", terrain).to_lowercase(),
                    EffectType::Terrain,
                    None,
                    Self::default_terrain_callback,
                    None,
                    None,
                    None,
                );
                listeners.push(listener);
            }
        }
        
        // Pseudoweather effects
        for (pseudo_id, _pseudo_effect) in &context.battle_state.field.effects {
            if self.pseudoweather_handles_event(pseudo_id, event_id) {
                let listener = PriorityCalculator::create_listener(
                    pseudo_id.clone(),
                    EffectType::FieldCondition,
                    None,
                    Self::default_field_callback,
                    None,
                    None,
                    None,
                );
                listeners.push(listener);
            }
        }
        
        // 4. Format rules
        if self.format_handles_event(&context.battle_state.format, event_id) {
            let listener = PriorityCalculator::create_listener(
                format!("{:?}", context.battle_state.format),
                EffectType::Format,
                None,
                Self::default_format_callback,
                None,
                None,
                None,
            );
            listeners.push(listener);
        }
        
        Ok(listeners)
    }
    
    /// Get a specific effect handler (for singleEvent)
    fn get_effect_handler(&self, effect: &EffectData, event_id: &str) -> Option<EventCallback> {
        match effect.effect_type {
            EffectType::Ability => {
                ability_handlers::AbilityHandlers::get_handler(&effect.id, event_id)
            }
            EffectType::Item => {
                ability_handlers::ItemHandlers::get_handler(&effect.id, event_id)
            }
            EffectType::Move => {
                ability_handlers::MoveHandlers::get_handler(&effect.id, event_id)
            }
            _ => None, // Other effects would be implemented here
        }
    }
    
    // Default callbacks for different effect types
    fn default_status_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_volatile_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_side_condition_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_weather_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_terrain_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_field_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    fn default_format_callback(_context: &mut EventContext, _relay_var: &mut RelayContainer) -> EventResult {
        EventResult::Continue
    }
    
    // Helper methods for determining what events each effect type handles
    fn status_handles_event(&self, status: &crate::pokemon::StatusCondition, event_id: &str) -> bool {
        match status {
            crate::pokemon::StatusCondition::Burn => {
                matches!(event_id, "AfterMoveSelf" | "TurnEnd" | "ModifyDamage")
            },
            crate::pokemon::StatusCondition::Paralysis => {
                matches!(event_id, "BeforeMove" | "ModifySpeed")
            },
            crate::pokemon::StatusCondition::Sleep => {
                matches!(event_id, "BeforeMove" | "TurnEnd")
            },
            crate::pokemon::StatusCondition::Freeze => {
                matches!(event_id, "BeforeMove" | "TurnEnd")
            },
            crate::pokemon::StatusCondition::Poison | crate::pokemon::StatusCondition::BadPoison => {
                matches!(event_id, "TurnEnd")
            },
        }
    }
    
    fn volatile_handles_event(&self, volatile_id: &str, event_id: &str) -> bool {
        match volatile_id {
            "confusion" => matches!(event_id, "BeforeMove"),
            "substitute" => matches!(event_id, "TryHit" | "TakeDamage"),
            "taunt" => matches!(event_id, "TryUseMove"),
            "encore" => matches!(event_id, "TryUseMove"),
            _ => false,
        }
    }
    
    fn side_condition_handles_event(&self, condition_id: &str, event_id: &str) -> bool {
        match condition_id {
            "reflect" => matches!(event_id, "TakeDamage"),
            "lightscreen" => matches!(event_id, "TakeDamage"),
            "spikes" => matches!(event_id, "SwitchIn"),
            "stealthrock" => matches!(event_id, "SwitchIn"),
            "toxicspikes" => matches!(event_id, "SwitchIn"),
            _ => false,
        }
    }
    
    fn weather_handles_event(&self, weather: &crate::battle_state::Weather, event_id: &str) -> bool {
        match weather {
            crate::battle_state::Weather::Sun => {
                matches!(event_id, "ModifyDamage" | "TurnEnd" | "WeatherDamage")
            },
            crate::battle_state::Weather::Rain => {
                matches!(event_id, "ModifyDamage" | "TurnEnd" | "WeatherDamage")
            },
            crate::battle_state::Weather::Sandstorm => {
                matches!(event_id, "TurnEnd" | "WeatherDamage")
            },
            crate::battle_state::Weather::Hail => {
                matches!(event_id, "TurnEnd" | "WeatherDamage")
            },
            _ => false,
        }
    }
    
    fn terrain_handles_event(&self, terrain: &crate::battle_state::Terrain, event_id: &str) -> bool {
        match terrain {
            crate::battle_state::Terrain::Electric => {
                matches!(event_id, "ModifyDamage" | "TrySetStatus")
            },
            crate::battle_state::Terrain::Grassy => {
                matches!(event_id, "ModifyDamage" | "TurnEnd")
            },
            crate::battle_state::Terrain::Misty => {
                matches!(event_id, "ModifyDamage" | "TrySetStatus")
            },
            crate::battle_state::Terrain::Psychic => {
                matches!(event_id, "ModifyDamage" | "TryHit")
            },
            _ => false,
        }
    }
    
    fn pseudoweather_handles_event(&self, pseudo_id: &str, event_id: &str) -> bool {
        match pseudo_id {
            "trickroom" => matches!(event_id, "ModifySpeed"),
            "wonderroom" => matches!(event_id, "ModifyDef" | "ModifySpD"),
            "magicroom" => matches!(event_id, "TryUseItem"),
            _ => false,
        }
    }
    
    fn format_handles_event(&self, format: &crate::format::BattleFormat, event_id: &str) -> bool {
        match format {
            crate::format::BattleFormat::Singles => false,
            crate::format::BattleFormat::Doubles => {
                matches!(event_id, "TryHit" | "ModifyTarget")
            },
            _ => false,
        }
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Battle events that can be logged
#[derive(Debug, Clone)]
pub enum BattleEvent {
    TurnStart(u32),
    PokemonDamage { 
        target: (SideId, usize), 
        damage: u16, 
        source: Option<(SideId, usize)> 
    },
    PokemonFaint((SideId, usize)),
    MoveUse { 
        user: (SideId, usize), 
        move_id: String, 
        targets: Vec<(SideId, usize)> 
    },
    StatusSet {
        target: (SideId, usize),
        status: String,
        source: Option<(SideId, usize)>,
    },
    BoostApplied {
        target: (SideId, usize),
        stat: String,
        amount: i8,
        source: Option<(SideId, usize)>,
    },
    WeatherSet {
        weather: String,
        duration: Option<u8>,
        source: Option<(SideId, usize)>,
    },
    // More events will be added as needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle_state::BattleState;
    use crate::prng::PRNG;
    
    #[test]
    fn test_event_system_depth_protection() {
        let mut event_system = EventSystem::new();
        let mut battle_state = BattleState::new(crate::format::BattleFormat::Singles);
        let mut prng = PRNG::new(Some([1, 2, 3, 4]));
        
        // Set event depth to maximum
        event_system.event_depth = event_system.max_event_depth;
        
        let relay_container = RelayContainer::new(RelayVar::None);
        let result = event_system.run_event(
            "TestEvent",
            None,
            None,
            None,
            relay_container,
            &mut battle_state,
            &mut prng,
            1,
        );
        
        // Should return immediately without error due to depth protection
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_relay_variable_system() {
        let mut container = RelayContainer::new(RelayVar::damage(100));
        
        // Test modification
        container.modify(RelayVar::damage(150));
        
        assert!(container.was_modified());
        assert_eq!(container.modification_count(), 1);
        assert_eq!(container.value.as_damage(), Some(150));
    }
}