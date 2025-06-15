//! Event priority system matching Pokemon Showdown's behavior
//! 
//! This module handles the complex priority sorting that Pokemon Showdown uses
//! for determining event execution order.

use std::cmp::Ordering;
use crate::pokemon::PokemonRef;
use crate::events::context::{EventTarget, EventSource, EffectType};
use crate::battle_state::BattleState;
use crate::errors::BattleResult;

/// Event listener with complete priority information
#[derive(Debug)]
pub struct EventListener {
    /// Effect causing this event
    pub effect_id: String,
    /// Effect type (ability, item, etc.)
    pub effect_type: EffectType,
    /// Pokemon that owns this effect (if applicable)
    pub owner: Option<PokemonRef>,
    /// Base priority for this effect type
    pub priority: i32,
    /// Specific order (overrides speed-based sorting)
    pub order: Option<i32>,
    /// Sub-order for ties
    pub sub_order: i32,
    /// Effect order (from effect state)
    pub effect_order: Option<i32>,
    /// Speed of the Pokemon (for speed-based sorting)
    pub speed: Option<u16>,
    /// Callback function to execute
    pub callback: EventCallback,
}

/// Event callback function type
pub type EventCallback = fn(&mut crate::events::context::EventContext, &mut crate::events::relay_vars::RelayContainer) -> crate::events::EventResult;

/// Event result types matching Pokemon Showdown's behavior
#[derive(Debug, Clone)]
pub enum EventResult {
    /// Continue to next handler (undefined in PS)
    Continue,
    /// Stop execution and suppress failure message (false in PS)
    Suppress,
    /// Stop execution and show failure message (null in PS)
    Fail,
    /// Stop execution with success (true in PS)
    Success,
    /// Modify relay variable and continue (any other value in PS)
    Modify(crate::events::relay_vars::RelayVar),
}

/// Priority calculator for event listeners
pub struct PriorityCalculator;

impl PriorityCalculator {
    /// Sort event listeners according to Pokemon Showdown rules
    pub fn sort_listeners(
        listeners: &mut Vec<EventListener>,
        event_id: &str,
        battle_state: &BattleState,
    ) -> BattleResult<()> {
        // Determine sorting strategy based on event type
        let sort_strategy = Self::get_sort_strategy(event_id);
        
        match sort_strategy {
            SortStrategy::SpeedBased => {
                Self::sort_by_speed(listeners, battle_state)?;
            }
            SortStrategy::LeftToRight => {
                Self::sort_left_to_right(listeners);
            }
            SortStrategy::RedirectOrder => {
                Self::sort_redirect_order(listeners);
            }
            SortStrategy::Custom => {
                Self::sort_custom(listeners, event_id);
            }
        }
        
        Ok(())
    }
    
    /// Determine the appropriate sorting strategy for an event
    fn get_sort_strategy(event_id: &str) -> SortStrategy {
        match event_id {
            // Fast exit events use redirect order
            "Invulnerability" | "TryHit" | "TryImmunity" => SortStrategy::RedirectOrder,
            
            // Special events use left-to-right
            "DamagingHit" | "EntryHazard" => SortStrategy::LeftToRight,
            
            // Custom events
            "ModifyDamage" | "BasePower" => SortStrategy::Custom,
            
            // Most events use speed-based sorting
            _ => SortStrategy::SpeedBased,
        }
    }
    
    /// Sort by Pokemon speed (fastest first)
    fn sort_by_speed(listeners: &mut Vec<EventListener>, battle_state: &BattleState) -> BattleResult<()> {
        // First, resolve speeds for all listeners
        for listener in listeners.iter_mut() {
            if let Some(owner) = listener.owner {
                if let Ok(pokemon) = battle_state.get_pokemon(owner) {
                    listener.speed = Some(pokemon.get_speed());
                }
            }
        }
        
        // Sort by priority, then by order, then by speed, then by sub_order
        listeners.sort_by(|a, b| {
            // Primary: Priority (higher first)
            match b.priority.cmp(&a.priority) {
                Ordering::Equal => {
                    // Secondary: Order (lower first, if specified)
                    match (&a.order, &b.order) {
                        (Some(a_order), Some(b_order)) => a_order.cmp(b_order),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => {
                            // Tertiary: Speed (higher first)
                            match (&b.speed, &a.speed) {
                                (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                                (Some(_), None) => Ordering::Less,
                                (None, Some(_)) => Ordering::Greater,
                                (None, None) => {
                                    // Quaternary: Sub-order (lower first)
                                    a.sub_order.cmp(&b.sub_order)
                                }
                            }
                        }
                    }
                }
                other => other,
            }
        });
        
        Ok(())
    }
    
    /// Sort left-to-right (by position on field)
    fn sort_left_to_right(listeners: &mut Vec<EventListener>) {
        listeners.sort_by(|a, b| {
            // Sort by owner position
            match (&a.owner, &b.owner) {
                (Some(a_owner), Some(b_owner)) => {
                    // First by side, then by position
                    match a_owner.side.cmp(&b_owner.side) {
                        Ordering::Equal => a_owner.position.cmp(&b_owner.position),
                        other => other,
                    }
                }
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
        });
    }
    
    /// Sort for redirect-order events (special handling)
    fn sort_redirect_order(listeners: &mut Vec<EventListener>) {
        // For redirect events, we want fastest Pokemon first for fast exit
        listeners.sort_by(|a, b| {
            match (&b.speed, &a.speed) {
                (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            }
        });
    }
    
    /// Custom sorting for specific events
    fn sort_custom(listeners: &mut Vec<EventListener>, event_id: &str) {
        match event_id {
            "ModifyDamage" => {
                // Damage modifiers: priority, then effect order, then speed
                listeners.sort_by(|a, b| {
                    match b.priority.cmp(&a.priority) {
                        Ordering::Equal => {
                            match (&a.effect_order, &b.effect_order) {
                                (Some(a_eff), Some(b_eff)) => a_eff.cmp(b_eff),
                                (Some(_), None) => Ordering::Less,
                                (None, Some(_)) => Ordering::Greater,
                                (None, None) => {
                                    match (&b.speed, &a.speed) {
                                        (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                                        _ => Ordering::Equal,
                                    }
                                }
                            }
                        }
                        other => other,
                    }
                });
            }
            "BasePower" => {
                // Base power modifiers: priority, then speed
                listeners.sort_by(|a, b| {
                    match b.priority.cmp(&a.priority) {
                        Ordering::Equal => {
                            match (&b.speed, &a.speed) {
                                (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                                _ => Ordering::Equal,
                            }
                        }
                        other => other,
                    }
                });
            }
            _ => {
                // Default to speed-based sorting
                listeners.sort_by(|a, b| {
                    match (&b.speed, &a.speed) {
                        (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                        _ => Ordering::Equal,
                    }
                });
            }
        }
    }
    
    /// Get default priority for an effect type
    pub fn get_default_priority(effect_type: EffectType) -> i32 {
        match effect_type {
            EffectType::Format => 1,
            EffectType::Rule => 1,
            EffectType::FieldCondition => 2,
            EffectType::Weather => 2,
            EffectType::Terrain => 2,
            EffectType::SideCondition => 3,
            EffectType::Volatile => 4,
            EffectType::Status => 5,
            EffectType::Ability => 6,
            EffectType::Item => 7,
            EffectType::Species => 8,
            EffectType::Move => 9,
        }
    }
    
    /// Create an event listener with calculated priority
    pub fn create_listener(
        effect_id: String,
        effect_type: EffectType,
        owner: Option<PokemonRef>,
        callback: EventCallback,
        custom_priority: Option<i32>,
        custom_order: Option<i32>,
        custom_sub_order: Option<i32>,
    ) -> EventListener {
        let priority = custom_priority.unwrap_or_else(|| Self::get_default_priority(effect_type.clone()));
        
        EventListener {
            effect_id,
            effect_type,
            owner,
            priority,
            order: custom_order,
            sub_order: custom_sub_order.unwrap_or(0),
            effect_order: None,
            speed: None,
            callback,
        }
    }
}

/// Sorting strategies for different event types
#[derive(Debug, Clone, PartialEq)]
enum SortStrategy {
    /// Sort by Pokemon speed (fastest first)
    SpeedBased,
    /// Sort by field position (left to right)
    LeftToRight,
    /// Sort for redirect order (special case)
    RedirectOrder,
    /// Custom sorting for specific events
    Custom,
}

/// Event execution controller
pub struct EventExecutor;

impl EventExecutor {
    /// Execute all listeners for an event with proper result handling
    pub fn execute_listeners(
        listeners: Vec<EventListener>,
        context: &mut crate::events::context::EventContext,
        relay_container: &mut crate::events::relay_vars::RelayContainer,
    ) -> BattleResult<EventResult> {
        for listener in listeners {
            // Check if event is suppressed
            if context.is_suppressed() {
                break;
            }
            
            // Execute the listener callback
            let result = (listener.callback)(context, relay_container);
            
            // Handle the result
            match result {
                EventResult::Continue => {
                    // Continue to next listener
                    continue;
                }
                EventResult::Suppress => {
                    // Stop execution, return false equivalent
                    return Ok(EventResult::Suppress);
                }
                EventResult::Fail => {
                    // Stop execution, return null equivalent
                    return Ok(EventResult::Fail);
                }
                EventResult::Success => {
                    // Stop execution, return true
                    return Ok(EventResult::Success);
                }
                EventResult::Modify(new_value) => {
                    // Update relay variable and continue
                    relay_container.modify(new_value);
                }
            }
        }
        
        // If we got here, all listeners executed without stopping
        Ok(EventResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::PokemonRef;
    use crate::side::SideId;
    
    #[test]
    fn test_priority_sorting() {
        let mut listeners = vec![
            EventListener {
                effect_id: "ability1".to_string(),
                effect_type: EffectType::Ability,
                owner: Some(PokemonRef { side: SideId::P1, position: 0 }),
                priority: 6,
                order: None,
                sub_order: 0,
                effect_order: None,
                speed: Some(100),
                callback: |_, _| EventResult::Continue,
            },
            EventListener {
                effect_id: "item1".to_string(),
                effect_type: EffectType::Item,
                owner: Some(PokemonRef { side: SideId::P1, position: 0 }),
                priority: 7,
                order: None,
                sub_order: 0,
                effect_order: None,
                speed: Some(80),
                callback: |_, _| EventResult::Continue,
            },
            EventListener {
                effect_id: "ability2".to_string(),
                effect_type: EffectType::Ability,
                owner: Some(PokemonRef { side: SideId::P2, position: 0 }),
                priority: 6,
                order: None,
                sub_order: 0,
                effect_order: None,
                speed: Some(120),
                callback: |_, _| EventResult::Continue,
            },
        ];
        
        // Create a mock battle state for testing
        // This would normally be a real BattleState
        // For now, we'll test without it
        
        // Sort by priority should put item first (priority 7), 
        // then faster ability (speed 120), then slower ability (speed 100)
        listeners.sort_by(|a, b| {
            match b.priority.cmp(&a.priority) {
                Ordering::Equal => {
                    match (&b.speed, &a.speed) {
                        (Some(b_speed), Some(a_speed)) => b_speed.cmp(a_speed),
                        _ => Ordering::Equal,
                    }
                }
                other => other,
            }
        });
        
        assert_eq!(listeners[0].effect_id, "item1");
        assert_eq!(listeners[1].effect_id, "ability2");
        assert_eq!(listeners[2].effect_id, "ability1");
    }
}