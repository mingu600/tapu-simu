//! Action queue system for turn-based battle execution
//! 
//! Based on Pokemon Showdown's battle-queue.ts implementation

use crate::pokemon::{MoveData, PokemonRef};
use crate::side::{ActionType, ChosenAction, SideId};
use serde::{Deserialize, Serialize};

/// Queue of actions to be executed in priority order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionQueue {
    actions: Vec<Action>,
}

/// Base action trait - all actions implement these fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub choice: ActionChoice,
    pub priority: i32,
    pub fractional_priority: f64,
    pub speed: u16,
    pub pokemon: Option<PokemonRef>,
    pub order: u8,
}


/// Different types of actions in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionChoice {
    /// Move action
    Move {
        move_id: String,
        move_data: Option<MoveData>,
        target_location: Option<i8>,
        original_target: Option<PokemonRef>,
        mega: MegaType,
        z_move: Option<String>,
        max_move: Option<String>,
    },
    
    /// Switch action
    Switch {
        target: PokemonRef,
    },
    
    /// Instant switch (forced)
    InstaSwitch {
        target: PokemonRef,
    },
    
    /// Revival Blessing switch
    RevivalBlessing {
        target: PokemonRef,
    },
    
    /// Team preview choice
    Team {
        index: usize,
    },
    
    /// Mega Evolution
    MegaEvo {
        mega_type: MegaType,
    },
    
    /// Dynamax
    Dynamax,
    
    /// Terastallize
    Terastallize,
    
    /// Generic field actions
    Field {
        field_type: FieldActionType,
    },
    
    /// Pass (do nothing)
    Pass,
}

/// Types of Mega Evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MegaType {
    None,
    Mega,
    MegaX,
    MegaY,
    Done, // Already mega evolved this turn
}

/// Types of field actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldActionType {
    Start,
    Residual,
    BeforeTurn,
}

impl ActionQueue {
    /// Create a new empty action queue
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add actions from player choices
    pub fn add_choices(
        &mut self,
        choices: &[(SideId, &[ChosenAction])],
        pokemon_speeds: &[(SideId, usize, u16)], // (side, position, speed)
    ) {
        for (side_id, actions) in choices {
            for action in *actions {
                let resolved_actions = self.resolve_action(*side_id, action, pokemon_speeds);
                self.actions.extend(resolved_actions);
            }
        }

        self.sort();
    }

    /// Resolve a chosen action into one or more queued actions
    fn resolve_action(
        &self,
        side: SideId,
        action: &ChosenAction,
        pokemon_speeds: &[(SideId, usize, u16)],
    ) -> Vec<Action> {
        let mut resolved = Vec::new();

        // Get Pokemon speed
        let speed = pokemon_speeds
            .iter()
            .find(|(s, pos, _)| *s == side && *pos == action.pokemon_index)
            .map(|(_, _, speed)| *speed)
            .unwrap_or(0);

        let pokemon_ref = Some(PokemonRef {
            side,
            position: action.pokemon_index,
        });

        match action.action_type {
            ActionType::Move => {
                // Add pre-move actions first
                if action.mega {
                    resolved.push(Action {
                        choice: ActionChoice::MegaEvo {
                            mega_type: MegaType::Mega,
                        },
                        priority: 0,
                        fractional_priority: 0.0,
                        speed,
                        pokemon: pokemon_ref.clone(),
                        order: ActionOrder::MegaEvolution as u8,
                    });
                }

                if action.terastallize {
                    resolved.push(Action {
                        choice: ActionChoice::Terastallize,
                        priority: 0,
                        fractional_priority: 0.0,
                        speed,
                        pokemon: pokemon_ref.clone(),
                        order: ActionOrder::Terastallize as u8,
                    });
                }

                if action.dynamax {
                    resolved.push(Action {
                        choice: ActionChoice::Dynamax,
                        priority: 0,
                        fractional_priority: 0.0,
                        speed,
                        pokemon: pokemon_ref.clone(),
                        order: ActionOrder::Dynamax as u8,
                    });
                }

                // Add the move action
                let mega_type = if action.mega {
                    MegaType::Mega
                } else {
                    MegaType::None
                };

                resolved.push(Action {
                    choice: ActionChoice::Move {
                        move_id: format!("move_{}", action.move_index.unwrap_or(0)),
                        move_data: None, // Would be populated with actual move data
                        target_location: action.target_location,
                        original_target: None,
                        mega: mega_type,
                        z_move: if action.z_move { Some("zmove".to_string()) } else { None },
                        max_move: if action.dynamax { Some("maxmove".to_string()) } else { None },
                    },
                    priority: 0, // Would be move.priority
                    fractional_priority: 0.0,
                    speed,
                    pokemon: pokemon_ref,
                    order: ActionOrder::Move as u8,
                });
            }

            ActionType::Switch => {
                let target = if let Some(target_idx) = action.switch_target {
                    PokemonRef {
                        side,
                        position: target_idx,
                    }
                } else {
                    // Default to switching to index 1 if not specified
                    PokemonRef {
                        side,
                        position: 1,
                    }
                };

                resolved.push(Action {
                    choice: ActionChoice::Switch { target },
                    priority: 6, // Switches have priority 6
                    fractional_priority: 0.0,
                    speed,
                    pokemon: pokemon_ref,
                    order: ActionOrder::Switch as u8,
                });
            }

            ActionType::Pass => {
                resolved.push(Action {
                    choice: ActionChoice::Pass,
                    priority: 0,
                    fractional_priority: 0.0,
                    speed,
                    pokemon: pokemon_ref,
                    order: ActionOrder::Move as u8,
                });
            }
        }

        resolved
    }

    /// Sort actions by priority order (based on Pokemon Showdown rules)
    fn sort(&mut self) {
        self.actions.sort_by(|a, b| {
            // First by order (lower is first)
            a.order
                .cmp(&b.order)
                // Then by priority (higher is first for moves)
                .then_with(|| b.priority.cmp(&a.priority))
                // Then by fractional priority (lower is first)
                .then_with(|| a.fractional_priority.partial_cmp(&b.fractional_priority).unwrap_or(std::cmp::Ordering::Equal))
                // Then by speed (higher is first)
                .then_with(|| b.speed.cmp(&a.speed))
                // Finally by side for deterministic ordering
                .then_with(|| {
                    if let (Some(a_poke), Some(b_poke)) = (&a.pokemon, &b.pokemon) {
                        a_poke.side.to_index().cmp(&b_poke.side.to_index())
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
        });
    }

    /// Get the next action to execute
    pub fn next(&mut self) -> Option<Action> {
        if self.actions.is_empty() {
            None
        } else {
            Some(self.actions.remove(0))
        }
    }

    /// Peek at the next action without removing it
    pub fn peek(&self) -> Option<&Action> {
        self.actions.first()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Get the number of actions in the queue
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Clear all actions
    pub fn clear(&mut self) {
        self.actions.clear();
    }

    /// Add a field action (like start of turn effects)
    pub fn add_field_action(&mut self, field_type: FieldActionType, order: u8) {
        let action = Action {
            choice: ActionChoice::Field { field_type },
            priority: 0,
            fractional_priority: 0.0,
            speed: 1,
            pokemon: None, // Field actions don't have a pokemon
            order,
        };

        self.actions.push(action);
        self.sort();
    }

    /// Add residual effects (end of turn)
    pub fn add_residual(&mut self) {
        self.add_field_action(FieldActionType::Residual, ActionOrder::Residual as u8);
    }

    /// Add start of turn effects
    pub fn add_start_turn(&mut self) {
        self.add_field_action(FieldActionType::BeforeTurn, ActionOrder::BeforeTurn as u8);
    }

    /// Add battle start action
    pub fn add_battle_start(&mut self) {
        self.add_field_action(FieldActionType::Start, ActionOrder::Start as u8);
    }

    /// Get all actions for debugging
    pub fn debug_actions(&self) -> &[Action] {
        &self.actions
    }

    /// Insert an action at a specific position (used for forced actions)
    pub fn insert_action(&mut self, action: Action) {
        self.actions.push(action);
        self.sort();
    }

    /// Get actions by type for analysis
    pub fn get_actions_by_choice(&self, choice_type: &str) -> Vec<&Action> {
        self.actions.iter().filter(|action| {
            match &action.choice {
                ActionChoice::Move { .. } => choice_type == "move",
                ActionChoice::Switch { .. } => choice_type == "switch",
                ActionChoice::Pass => choice_type == "pass",
                _ => false,
            }
        }).collect()
    }
    
    /// Update move priority based on move data (called when move data is loaded)
    pub fn update_move_priorities(&mut self, get_move_priority: impl Fn(&str) -> i32) {
        for action in &mut self.actions {
            if let ActionChoice::Move { move_id, .. } = &action.choice {
                action.priority = get_move_priority(move_id);
            }
        }
        self.sort();
    }
}

/// Order values for different action types (based on Pokemon Showdown)
#[repr(u8)]
enum ActionOrder {
    Start = 1,
    BeforeTurn = 2,
    Switch = 3,
    MegaEvolution = 4,
    Terastallize = 5,
    Dynamax = 6,
    Move = 100,
    Residual = 200,
}

impl Default for ActionQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::side::{ActionType, ChosenAction};

    #[test]
    fn test_action_queue_ordering() {
        let mut queue = ActionQueue::new();

        // Create test actions
        let fast_move = ChosenAction {
            action_type: ActionType::Move,
            pokemon_index: 0,
            move_index: Some(0),
            target_location: None,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        };

        let slow_move = ChosenAction {
            action_type: ActionType::Move,
            pokemon_index: 0,
            move_index: Some(1),
            target_location: None,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        };

        let switch_action = ChosenAction {
            action_type: ActionType::Switch,
            pokemon_index: 0,
            move_index: None,
            target_location: None,
            switch_target: Some(1),
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        };

        let speeds = vec![
            (SideId::P1, 0, 100), // Fast Pokemon
            (SideId::P2, 0, 50),  // Slow Pokemon
        ];

        let p1_actions = [fast_move];
        let p2_actions = [slow_move, switch_action];
        let choices = vec![
            (SideId::P1, p1_actions.as_slice()),
            (SideId::P2, p2_actions.as_slice()),
        ];

        queue.add_choices(&choices, &speeds);

        // Switches should come first
        let first_action = queue.next().unwrap();
        assert!(matches!(first_action.choice, ActionChoice::Switch { .. }));

        // Then the faster move
        let second_action = queue.next().unwrap();
        assert!(matches!(second_action.choice, ActionChoice::Move { .. }));
        assert_eq!(second_action.speed, 100);

        // Then the slower move
        let third_action = queue.next().unwrap();
        assert!(matches!(third_action.choice, ActionChoice::Move { .. }));
        assert_eq!(third_action.speed, 50);
    }

    #[test]
    fn test_mega_evolution_ordering() {
        let mut queue = ActionQueue::new();

        let mega_move = ChosenAction {
            action_type: ActionType::Move,
            pokemon_index: 0,
            move_index: Some(0),
            target_location: None,
            switch_target: None,
            mega: true,
            z_move: false,
            dynamax: false,
            terastallize: false,
        };

        let speeds = vec![(SideId::P1, 0, 100)];
        let p1_actions = [mega_move];
        let choices = vec![(SideId::P1, p1_actions.as_slice())];

        queue.add_choices(&choices, &speeds);

        // Mega evolution should come before the move
        let first_action = queue.next().unwrap();
        assert!(matches!(
            first_action.choice,
            ActionChoice::MegaEvo { .. }
        ));

        let second_action = queue.next().unwrap();
        assert!(matches!(second_action.choice, ActionChoice::Move { .. }));
    }

    #[test]
    fn test_field_actions() {
        let mut queue = ActionQueue::new();
        
        queue.add_battle_start();
        queue.add_start_turn();
        queue.add_residual();
        
        // Start should come first
        let first = queue.next().unwrap();
        assert!(matches!(first.choice, ActionChoice::Field { field_type: FieldActionType::Start }));
        
        // Then before turn
        let second = queue.next().unwrap();
        assert!(matches!(second.choice, ActionChoice::Field { field_type: FieldActionType::BeforeTurn }));
        
        // Then residual
        let third = queue.next().unwrap();
        assert!(matches!(third.choice, ActionChoice::Field { field_type: FieldActionType::Residual }));
    }

    #[test]
    fn test_priority_sorting() {
        let mut queue = ActionQueue::new();
        
        // Create a high priority action
        let high_priority = Action {
            choice: ActionChoice::Move {
                move_id: "quickattack".to_string(),
                move_data: None,
                target_location: None,
                original_target: None,
                mega: MegaType::None,
                z_move: None,
                max_move: None,
            },
            priority: 1, // High priority
            fractional_priority: 0.0,
            speed: 50,
            pokemon: Some(PokemonRef { side: SideId::P1, position: 0 }),
            order: ActionOrder::Move as u8,
        };
        
        // Create a normal priority action with higher speed
        let normal_priority = Action {
            choice: ActionChoice::Move {
                move_id: "tackle".to_string(),
                move_data: None,
                target_location: None,
                original_target: None,
                mega: MegaType::None,
                z_move: None,
                max_move: None,
            },
            priority: 0, // Normal priority
            fractional_priority: 0.0,
            speed: 100, // Higher speed
            pokemon: Some(PokemonRef { side: SideId::P2, position: 0 }),
            order: ActionOrder::Move as u8,
        };
        
        queue.insert_action(normal_priority);
        queue.insert_action(high_priority);
        
        // High priority should come first despite lower speed
        let first = queue.next().unwrap();
        assert_eq!(first.priority, 1);
        
        let second = queue.next().unwrap();
        assert_eq!(second.priority, 0);
        assert_eq!(second.speed, 100);
    }
}
