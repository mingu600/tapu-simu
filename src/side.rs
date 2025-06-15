//! Side (player) management and choice handling

use serde::{Deserialize, Serialize};
use crate::pokemon::Pokemon;
use crate::format::BattleFormat;
use crate::errors::{BattleError, BattleResult};
use std::collections::{HashMap, HashSet};

/// Side identifier (P1, P2, P3, P4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum SideId {
    P1, P2, P3, P4,
}

impl SideId {
    pub fn to_index(&self) -> usize {
        match self {
            SideId::P1 => 0,
            SideId::P2 => 1,
            SideId::P3 => 2,
            SideId::P4 => 3,
        }
    }
    
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(SideId::P1),
            1 => Some(SideId::P2),
            2 => Some(SideId::P3),
            3 => Some(SideId::P4),
            _ => None,
        }
    }
    
    pub fn opponent(&self) -> SideId {
        match self {
            SideId::P1 => SideId::P2,
            SideId::P2 => SideId::P1,
            SideId::P3 => SideId::P4,
            SideId::P4 => SideId::P3,
        }
    }
}

impl std::fmt::Display for SideId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SideId::P1 => write!(f, "P1"),
            SideId::P2 => write!(f, "P2"),
            SideId::P3 => write!(f, "P3"),
            SideId::P4 => write!(f, "P4"),
        }
    }
}

/// A player's side in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Side {
    /// Side identifier
    pub id: SideId,
    
    /// Player name
    pub name: String,
    
    /// All Pokemon on this side
    pub pokemon: Vec<Pokemon>,
    
    /// Currently active Pokemon (indices into pokemon vec)
    pub active: Vec<Option<usize>>,
    
    /// Current choice for this turn
    pub choice: Choice,
    
    /// Side conditions (Reflect, Light Screen, etc.)
    pub conditions: HashMap<String, SideCondition>,
    
    /// Pokemon that fainted last turn
    pub fainted_last_turn: Option<usize>,
    
    /// Pokemon that fainted this turn
    pub fainted_this_turn: Option<usize>,
    
    /// Total Pokemon fainted
    pub total_fainted: u8,
    
    /// Whether Z-move has been used this battle
    pub z_move_used: bool,
    
    /// Whether Mega Evolution has been used this turn
    pub mega_used: bool,
    
    /// Whether Dynamax has been used this battle
    pub dynamax_used: bool,
    
    /// Whether Terastallization has been used this battle
    pub tera_used: bool,
}

/// Player's choice for a turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Individual actions for each active Pokemon
    pub actions: Vec<ChosenAction>,
    
    /// Number of forced switches remaining
    pub forced_switches_left: u8,
    
    /// Whether this choice cannot be undone
    pub cant_undo: bool,
    
    /// Error message if choice is invalid
    pub error: Option<String>,
    
    /// Set of Pokemon indices that are switching in
    pub switch_ins: HashSet<usize>,
}

/// A single chosen action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChosenAction {
    pub action_type: ActionType,
    pub pokemon_index: usize,
    pub move_index: Option<usize>,
    pub target_location: Option<i8>,
    pub switch_target: Option<usize>,
    pub mega: bool,
    pub z_move: bool,
    pub dynamax: bool,
    pub terastallize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Switch,
    Pass,
}

impl ChosenAction {
    // Factory methods for easy action creation
    
    /// Create a move action - most common case
    /// Example: ChosenAction::move_action(0, 0, None) // Pokemon 0 uses move 0
    pub fn move_action(
        pokemon_index: usize,
        move_index: usize,
        target_location: Option<i8>,
    ) -> Self {
        Self {
            action_type: ActionType::Move,
            pokemon_index,
            move_index: Some(move_index),
            target_location,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        }
    }
    
    /// Create a switch action
    /// Example: ChosenAction::switch_action(0, 1) // Pokemon 0 switches to Pokemon 1
    pub fn switch_action(pokemon_index: usize, switch_target: usize) -> Self {
        Self {
            action_type: ActionType::Switch,
            pokemon_index,
            move_index: None,
            target_location: None,
            switch_target: Some(switch_target),
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        }
    }
    
    /// Create a pass action (for fainted Pokemon)
    pub fn pass_action(pokemon_index: usize) -> Self {
        Self {
            action_type: ActionType::Pass,
            pokemon_index,
            move_index: None,
            target_location: None,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: false,
        }
    }
    
    /// Create a move action with Mega Evolution
    pub fn mega_move_action(
        pokemon_index: usize,
        move_index: usize,
        target_location: Option<i8>,
    ) -> Self {
        Self {
            action_type: ActionType::Move,
            pokemon_index,
            move_index: Some(move_index),
            target_location,
            switch_target: None,
            mega: true,
            z_move: false,
            dynamax: false,
            terastallize: false,
        }
    }
    
    /// Create a Z-move action
    pub fn z_move_action(
        pokemon_index: usize,
        move_index: usize,
        target_location: Option<i8>,
    ) -> Self {
        Self {
            action_type: ActionType::Move,
            pokemon_index,
            move_index: Some(move_index),
            target_location,
            switch_target: None,
            mega: false,
            z_move: true,
            dynamax: false,
            terastallize: false,
        }
    }
    
    /// Create a Dynamax move action
    pub fn dynamax_move_action(
        pokemon_index: usize,
        move_index: usize,
        target_location: Option<i8>,
    ) -> Self {
        Self {
            action_type: ActionType::Move,
            pokemon_index,
            move_index: Some(move_index),
            target_location,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: true,
            terastallize: false,
        }
    }
    
    /// Create a move action with Terastallization
    pub fn tera_move_action(
        pokemon_index: usize,
        move_index: usize,
        target_location: Option<i8>,
    ) -> Self {
        Self {
            action_type: ActionType::Move,
            pokemon_index,
            move_index: Some(move_index),
            target_location,
            switch_target: None,
            mega: false,
            z_move: false,
            dynamax: false,
            terastallize: true,
        }
    }
    
    /// Create a simple attack action targeting the opponent
    /// Most common case for testing - Pokemon 0 uses move 0 against opponent
    pub fn attack() -> Self {
        Self::move_action(0, 0, Some(1))
    }
    
    /// Create a simple switch action for testing
    /// Pokemon 0 switches to Pokemon 1
    pub fn switch() -> Self {
        Self::switch_action(0, 1)
    }
}

/// Side condition data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideCondition {
    pub id: String,
    pub duration: Option<u8>,
    pub layers: u8, // For stackable conditions like Spikes
    pub data: HashMap<String, serde_json::Value>,
}

impl Side {
    /// Create a new side with a team of Pokemon
    pub fn new(
        id: SideId, 
        name: String, 
        team: Vec<Pokemon>, 
        format: &BattleFormat
    ) -> BattleResult<Self> {
        if team.is_empty() {
            return Err(BattleError::DataError("Team cannot be empty".to_string()));
        }
        
        let active_slots = format.active_per_side();
        let mut active = vec![None; active_slots];
        
        // Set initial active Pokemon
        for i in 0..active_slots.min(team.len()) {
            active[i] = Some(i);
        }
        
        Ok(Self {
            id,
            name,
            pokemon: team,
            active,
            choice: Choice::new(),
            conditions: HashMap::new(),
            fainted_last_turn: None,
            fainted_this_turn: None,
            total_fainted: 0,
            z_move_used: false,
            mega_used: false,
            dynamax_used: false,
            tera_used: false,
        })
    }
    
    /// Get active Pokemon at position
    pub fn get_active(&self, position: usize) -> Option<&Pokemon> {
        self.active.get(position)
            .and_then(|&idx| idx)
            .and_then(|idx| self.pokemon.get(idx))
    }
    
    /// Get mutable reference to active Pokemon at position
    pub fn get_active_mut(&mut self, position: usize) -> Option<&mut Pokemon> {
        if let Some(&Some(idx)) = self.active.get(position) {
            self.pokemon.get_mut(idx)
        } else {
            None
        }
    }
    
    /// Get all active Pokemon
    pub fn get_all_active(&self) -> Vec<&Pokemon> {
        self.active.iter()
            .filter_map(|&idx| idx)
            .filter_map(|idx| self.pokemon.get(idx))
            .collect()
    }
    
    /// Check if this side has any Pokemon left
    pub fn has_pokemon_left(&self) -> bool {
        self.pokemon.iter().any(|p| !p.is_fainted())
    }
    
    /// Count unfainted Pokemon
    pub fn pokemon_left(&self) -> usize {
        self.pokemon.iter().filter(|p| !p.is_fainted()).count()
    }
    
    /// Switch Pokemon at position
    pub fn switch_pokemon(&mut self, position: usize, new_pokemon_index: usize) -> BattleResult<()> {
        if position >= self.active.len() {
            return Err(BattleError::InvalidSwitch(
                format!("Invalid position: {}", position)
            ));
        }
        
        if new_pokemon_index >= self.pokemon.len() {
            return Err(BattleError::PokemonNotFound { position: new_pokemon_index });
        }
        
        if self.pokemon[new_pokemon_index].is_fainted() {
            return Err(BattleError::InvalidSwitch(
                "Cannot switch to a fainted Pokemon".to_string()
            ));
        }
        
        // Check if Pokemon is already active
        if self.active.contains(&Some(new_pokemon_index)) {
            return Err(BattleError::InvalidSwitch(
                "Pokemon is already active".to_string()
            ));
        }
        
        self.active[position] = Some(new_pokemon_index);
        Ok(())
    }
    
    /// Add a choice for this turn
    pub fn add_choice(&mut self, action: ChosenAction) -> BattleResult<()> {
        // Validate the action
        self.validate_action(&action)?;
        
        self.choice.actions.push(action);
        Ok(())
    }
    
    /// Validate an action
    fn validate_action(&self, action: &ChosenAction) -> BattleResult<()> {
        // Check if Pokemon exists
        if action.pokemon_index >= self.pokemon.len() {
            return Err(BattleError::PokemonNotFound { 
                position: action.pokemon_index 
            });
        }
        
        let pokemon = &self.pokemon[action.pokemon_index];
        
        match action.action_type {
            ActionType::Move => {
                if let Some(move_index) = action.move_index {
                    if !pokemon.can_use_move(move_index) {
                        return Err(BattleError::InvalidMove(
                            "Move cannot be used".to_string()
                        ));
                    }
                } else {
                    return Err(BattleError::InvalidMove(
                        "Move index required for move action".to_string()
                    ));
                }
            }
            ActionType::Switch => {
                if !matches!(pokemon.trapped, crate::pokemon::TrappedState::None) {
                    return Err(BattleError::InvalidSwitch(
                        "Pokemon is trapped and cannot switch".to_string()
                    ));
                }
                
                if let Some(target) = action.switch_target {
                    if target >= self.pokemon.len() {
                        return Err(BattleError::PokemonNotFound { position: target });
                    }
                    
                    if self.pokemon[target].is_fainted() {
                        return Err(BattleError::InvalidSwitch(
                            "Cannot switch to fainted Pokemon".to_string()
                        ));
                    }
                }
            }
            ActionType::Pass => {
                // Pass is always valid for fainted Pokemon
                if !pokemon.is_fainted() {
                    return Err(BattleError::InvalidMove(
                        "Can only pass with fainted Pokemon".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Clear choices for the next turn
    pub fn clear_choice(&mut self) {
        self.choice = Choice::new();
        self.mega_used = false;
    }
    
    /// Check if this side is ready (has made all required choices)
    pub fn is_choice_done(&self) -> bool {
        let required_actions = self.active.iter()
            .filter(|slot| slot.is_some())
            .count();
            
        self.choice.actions.len() >= required_actions
    }
    
    /// Add a side condition
    pub fn add_condition(&mut self, condition: SideCondition) {
        self.conditions.insert(condition.id.clone(), condition);
    }
    
    /// Remove a side condition
    pub fn remove_condition(&mut self, condition_id: &str) -> bool {
        self.conditions.remove(condition_id).is_some()
    }
    
    /// Check if a side condition is active
    pub fn has_condition(&self, condition_id: &str) -> bool {
        self.conditions.contains_key(condition_id)
    }
}

impl Choice {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            forced_switches_left: 0,
            cant_undo: false,
            error: None,
            switch_ins: HashSet::new(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

impl Default for Choice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::pokemon::*;
    use crate::types::*;
    use crate::format::BattleFormat;
    use crate::dex::ShowdownDex;
    
    pub fn create_test_pokemon() -> Pokemon {
        // Use factory method - 95% less code!
        let dex = ShowdownDex::test_dex();
        Pokemon::test_pokemon(dex.as_ref(), Some(50))
            .unwrap_or_else(|_| {
                // Fallback for when no data is available
                Pokemon::new(
                    SpeciesData {
                        id: "pikachu".to_string(),
                        name: "Pikachu".to_string(),
                        types: [Type::Electric, Type::Electric],
                        base_stats: StatsTable { hp: 35, attack: 55, defense: 40, special_attack: 50, special_defense: 50, speed: 90 },
                        abilities: vec!["static".to_string()],
                        height: 0.4,
                        weight: 6.0,
                        gender_ratio: GenderRatio::Ratio { male: 0.5, female: 0.5 },
                    },
                    50,
                    [
                        MoveData { id: "tackle".to_string(), name: "Tackle".to_string(), type_: Type::Normal, category: MoveCategory::Physical, base_power: 40, accuracy: Some(100), pp: 35, target: MoveTarget::Normal, priority: 0, flags: MoveFlags::default(), secondary_effect: None, crit_ratio: 1, multihit: None, drain: None, recoil: None },
                        MoveData { id: "thundershock".to_string(), name: "Thunder Shock".to_string(), type_: Type::Electric, category: MoveCategory::Special, base_power: 40, accuracy: Some(100), pp: 30, target: MoveTarget::Normal, priority: 0, flags: MoveFlags::default(), secondary_effect: None, crit_ratio: 1, multihit: None, drain: None, recoil: None },
                        MoveData { id: "growl".to_string(), name: "Growl".to_string(), type_: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: Some(100), pp: 40, target: MoveTarget::AllAdjacentFoes, priority: 0, flags: MoveFlags::default(), secondary_effect: None, crit_ratio: 1, multihit: None, drain: None, recoil: None },
                        MoveData { id: "tailwhip".to_string(), name: "Tail Whip".to_string(), type_: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: Some(100), pp: 30, target: MoveTarget::AllAdjacentFoes, priority: 0, flags: MoveFlags::default(), secondary_effect: None, crit_ratio: 1, multihit: None, drain: None, recoil: None },
                    ],
                    AbilityData { id: "static".to_string(), name: "Static".to_string(), description: "Contact may paralyze attacker".to_string(), event_handlers: crate::events::EventHandlerRegistry::default() },
                    None,
                    Nature::Hardy,
                    StatsTable::max(),
                    StatsTable::default(),
                    Gender::Male,
                )
            })
    }
    
    #[test]
    fn test_side_creation() {
        let team = vec![create_test_pokemon()];
        let format = BattleFormat::Singles;
        
        let side = Side::new(SideId::P1, "Test Player".to_string(), team, &format).unwrap();
        
        assert_eq!(side.id, SideId::P1);
        assert_eq!(side.name, "Test Player");
        assert_eq!(side.pokemon.len(), 1);
        assert_eq!(side.active.len(), 1);
        assert_eq!(side.active[0], Some(0));
    }
    
    #[test]
    fn test_choice_validation() {
        let team = vec![create_test_pokemon()];
        let format = BattleFormat::Singles;
        let mut side = Side::new(SideId::P1, "Test Player".to_string(), team, &format).unwrap();
        
        // Valid move action using factory method
        let action = ChosenAction::move_action(0, 0, None);
        
        assert!(side.add_choice(action).is_ok());
    }
}