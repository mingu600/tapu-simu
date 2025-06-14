//! Action queue system for turn-based battle execution

use serde::{Deserialize, Serialize};
use crate::side::{SideId, ChosenAction, ActionType};
use crate::pokemon::MoveData;
use crate::format::BattleFormat;

/// Queue of actions to be executed in priority order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionQueue {
    actions: Vec<QueuedAction>,
}

/// An action in the queue with all necessary execution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedAction {
    pub action_type: QueuedActionType,
    pub side: SideId,
    pub pokemon_position: usize,
    pub priority: i8,
    pub speed: u16,
    pub order: u8, // For same-priority ordering
    pub target_location: Option<i8>,
    pub move_data: Option<MoveData>,
    pub switch_target: Option<usize>,
    pub mega: bool,
    pub z_move: bool,
    pub dynamax: bool,
    pub terastallize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueuedActionType {
    // Pre-move actions
    MegaEvolution,
    Terastallize,
    Dynamax,
    
    // Move actions
    Move,
    
    // Switch actions
    Switch,
    InstaSwitch,
    
    // Special actions
    Pass,
    Start,        // Battle start
    Residual,     // End of turn effects
    BeforeTurn,   // Start of turn effects
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
    ) -> Vec<QueuedAction> {
        let mut resolved = Vec::new();
        
        // Get Pokemon speed
        let speed = pokemon_speeds.iter()
            .find(|(s, pos, _)| *s == side && *pos == action.pokemon_index)
            .map(|(_, _, speed)| *speed)
            .unwrap_or(0);
        
        match action.action_type {
            ActionType::Move => {
                // Add pre-move actions first
                if action.mega {
                    resolved.push(QueuedAction {
                        action_type: QueuedActionType::MegaEvolution,
                        side,
                        pokemon_position: action.pokemon_index,
                        priority: 0,
                        speed,
                        order: ActionOrder::MegaEvolution as u8,
                        target_location: None,
                        move_data: None,
                        switch_target: None,
                        mega: true,
                        z_move: false,
                        dynamax: false,
                        terastallize: false,
                    });
                }
                
                if action.terastallize {
                    resolved.push(QueuedAction {
                        action_type: QueuedActionType::Terastallize,
                        side,
                        pokemon_position: action.pokemon_index,
                        priority: 0,
                        speed,
                        order: ActionOrder::Terastallize as u8,
                        target_location: None,
                        move_data: None,
                        switch_target: None,
                        mega: false,
                        z_move: false,
                        dynamax: false,
                        terastallize: true,
                    });
                }
                
                if action.dynamax {
                    resolved.push(QueuedAction {
                        action_type: QueuedActionType::Dynamax,
                        side,
                        pokemon_position: action.pokemon_index,
                        priority: 0,
                        speed,
                        order: ActionOrder::Dynamax as u8,
                        target_location: None,
                        move_data: None,
                        switch_target: None,
                        mega: false,
                        z_move: false,
                        dynamax: true,
                        terastallize: false,
                    });
                }
                
                // Add the move action
                // Note: Move data and priority would be looked up from the move database
                resolved.push(QueuedAction {
                    action_type: QueuedActionType::Move,
                    side,
                    pokemon_position: action.pokemon_index,
                    priority: 0, // Would be move.priority
                    speed,
                    order: ActionOrder::Move as u8,
                    target_location: action.target_location,
                    move_data: None, // Would be populated with actual move data
                    switch_target: None,
                    mega: false,
                    z_move: action.z_move,
                    dynamax: false,
                    terastallize: false,
                });
            }
            
            ActionType::Switch => {
                resolved.push(QueuedAction {
                    action_type: QueuedActionType::Switch,
                    side,
                    pokemon_position: action.pokemon_index,
                    priority: 6, // Switches have priority 6
                    speed,
                    order: ActionOrder::Switch as u8,
                    target_location: None,
                    move_data: None,
                    switch_target: action.switch_target,
                    mega: false,
                    z_move: false,
                    dynamax: false,
                    terastallize: false,
                });
            }
            
            ActionType::Pass => {
                resolved.push(QueuedAction {
                    action_type: QueuedActionType::Pass,
                    side,
                    pokemon_position: action.pokemon_index,
                    priority: 0,
                    speed,
                    order: ActionOrder::Move as u8,
                    target_location: None,
                    move_data: None,
                    switch_target: None,
                    mega: false,
                    z_move: false,
                    dynamax: false,
                    terastallize: false,
                });
            }
        }
        
        resolved
    }
    
    /// Sort actions by priority order
    fn sort(&mut self) {
        self.actions.sort_by(|a, b| {
            // First by order (lower is first)
            a.order.cmp(&b.order)
                // Then by priority (higher is first for moves)
                .then_with(|| b.priority.cmp(&a.priority))
                // Then by speed (higher is first)
                .then_with(|| b.speed.cmp(&a.speed))
                // Finally by side (for deterministic ordering)
                .then_with(|| a.side.to_index().cmp(&b.side.to_index()))
        });
    }
    
    /// Get the next action to execute
    pub fn next(&mut self) -> Option<QueuedAction> {
        if self.actions.is_empty() {
            None
        } else {
            Some(self.actions.remove(0))
        }
    }
    
    /// Peek at the next action without removing it
    pub fn peek(&self) -> Option<&QueuedAction> {
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
    
    /// Add a special action (like start of turn effects)
    pub fn add_special_action(&mut self, action_type: QueuedActionType, order: u8) {
        let action = QueuedAction {
            action_type,
            side: SideId::P1, // Doesn't matter for special actions
            pokemon_position: 0,
            priority: 0,
            speed: 0,
            order,
            target_location: None,
            move_data: None,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        };
        
        self.actions.push(action);
        self.sort();
    }
    
    /// Add residual effects (end of turn)
    pub fn add_residual(&mut self) {
        self.add_special_action(QueuedActionType::Residual, ActionOrder::Residual as u8);
    }
    
    /// Add start of turn effects
    pub fn add_start_turn(&mut self) {
        self.add_special_action(QueuedActionType::BeforeTurn, ActionOrder::BeforeTurn as u8);
    }
    
    /// Get all actions for debugging
    pub fn debug_actions(&self) -> &[QueuedAction] {
        &self.actions
    }
}

/// Order values for different action types
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
    use crate::side::{ChosenAction, ActionType};
    
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
        
        let choices = vec![
            (SideId::P1, [fast_move].as_slice()),
            (SideId::P2, [slow_move, switch_action].as_slice()),
        ];
        
        queue.add_choices(&choices, &speeds);
        
        // Switches should come first
        let first_action = queue.next().unwrap();
        assert!(matches!(first_action.action_type, QueuedActionType::Switch));
        
        // Then the faster move
        let second_action = queue.next().unwrap();
        assert!(matches!(second_action.action_type, QueuedActionType::Move));
        assert_eq!(second_action.speed, 100);
        
        // Then the slower move
        let third_action = queue.next().unwrap();
        assert!(matches!(third_action.action_type, QueuedActionType::Move));
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
        let choices = vec![(SideId::P1, [mega_move].as_slice())];
        
        queue.add_choices(&choices, &speeds);
        
        // Mega evolution should come before the move
        let first_action = queue.next().unwrap();
        assert!(matches!(first_action.action_type, QueuedActionType::MegaEvolution));
        
        let second_action = queue.next().unwrap();
        assert!(matches!(second_action.action_type, QueuedActionType::Move));
    }
}