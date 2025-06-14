//! Main battle engine

use crate::battle_state::BattleState;
use crate::side::{SideId, ChosenAction};
use crate::dex::Dex;
use crate::errors::{BattleError, BattleResult};
use crate::format::BattleFormat;
use serde::{Deserialize, Serialize};

/// Battle snapshot for undo functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSnapshot {
    pub turn: u32,
    pub state: BattleState,
}

/// Main battle controller
pub struct Battle {
    state: BattleState,
    dex: Box<dyn Dex>,
    history: Vec<BattleSnapshot>,
}

impl Battle {
    /// Create a new battle
    pub fn new(
        state: BattleState,
        dex: Box<dyn Dex>,
    ) -> Self {
        Self {
            state,
            dex,
            history: Vec::new(),
        }
    }
    
    /// Add a choice for a side
    pub fn add_choice(&mut self, side_id: SideId, actions: Vec<ChosenAction>) -> BattleResult<()> {
        let side = self.state.get_side_mut(side_id)
            .ok_or(BattleError::SideNotFound { side: side_id })?;
        
        for action in actions {
            side.add_choice(action)?;
        }
        
        Ok(())
    }
    
    /// Execute one step of the battle
    pub fn step(&mut self) -> BattleResult<bool> {
        if self.state.ended {
            return Ok(true);
        }
        
        // Save snapshot before executing
        self.save_snapshot();
        
        // Check if all choices are made
        if !self.state.all_choices_made() {
            return Ok(false); // Waiting for more choices
        }
        
        // Start new turn if queue is empty
        if self.state.queue.is_empty() {
            self.state.start_turn();
            
            // Collect choices and add to queue
            let mut all_choices = Vec::new();
            for side in &self.state.sides {
                all_choices.push((side.id, side.choice.actions.as_slice()));
            }
            
            let speeds = self.state.get_pokemon_speeds();
            self.state.queue.add_choices(&all_choices, &speeds);
        }
        
        // Execute next action
        if let Some(action) = self.state.queue.next() {
            self.execute_action(action)?;
        } else {
            // End of turn - add residual effects
            self.state.queue.add_residual();
            self.state.field.process_turn_end();
        }
        
        // Check if battle ended
        let ended = self.state.check_battle_end();
        Ok(ended)
    }
    
    /// Execute an action (placeholder implementation)
    fn execute_action(&mut self, _action: crate::action_queue::QueuedAction) -> BattleResult<()> {
        // TODO: Implement action execution
        Ok(())
    }
    
    /// Save current state as snapshot
    fn save_snapshot(&mut self) {
        let snapshot = BattleSnapshot {
            turn: self.state.turn,
            state: self.state.clone(),
        };
        self.history.push(snapshot);
    }
    
    /// Undo to a specific turn
    pub fn undo_to_turn(&mut self, turn: u32) -> BattleResult<()> {
        if let Some(snapshot) = self.history.iter().find(|s| s.turn == turn) {
            self.state = snapshot.state.clone();
            // Remove snapshots after this turn
            self.history.retain(|s| s.turn <= turn);
            Ok(())
        } else {
            Err(BattleError::TurnNotFound { turn })
        }
    }
    
    /// Get current state
    pub fn state(&self) -> &BattleState {
        &self.state
    }
    
    /// Serialize current state
    pub fn serialize_state(&self) -> BattleResult<Vec<u8>> {
        self.state.to_bytes()
    }
    
    /// Deserialize and restore state
    pub fn deserialize_state(&mut self, data: &[u8]) -> BattleResult<()> {
        self.state = BattleState::from_bytes(data)?;
        Ok(())
    }
}