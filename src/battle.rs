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
    
    // Factory methods for easy battle creation
    
    /// Create a quick test battle with minimal setup
    /// Perfect for unit tests and quick development
    pub fn quick_test_battle(dex: Box<dyn Dex>) -> BattleResult<Self> {
        Self::test_battle_with_teams(
            dex,
            None, // Use default test teams
            None,
            None, // Singles format
        )
    }
    
    /// Create a test battle with specific teams
    pub fn test_battle_with_teams(
        dex: Box<dyn Dex>,
        team1: Option<Vec<crate::pokemon::Pokemon>>,
        team2: Option<Vec<crate::pokemon::Pokemon>>,
        format: Option<crate::format::BattleFormat>,
    ) -> BattleResult<Self> {
        use crate::pokemon::Pokemon;
        use crate::side::{Side, SideId};
        use crate::format::{BattleFormat, FormatRules};
        
        let format = format.unwrap_or(BattleFormat::Singles);
        
        // Create teams - use provided teams or create test teams
        let team1 = if let Some(team) = team1 {
            team
        } else {
            vec![
                Pokemon::test_pokemon(dex.as_ref(), Some(50))
                    .unwrap_or_else(|_| Self::create_fallback_pokemon()),
                Pokemon::test_pokemon(dex.as_ref(), Some(50))
                    .unwrap_or_else(|_| Self::create_fallback_pokemon()),
            ]
        };
        
        let team2 = if let Some(team) = team2 {
            team
        } else {
            vec![
                Pokemon::test_pokemon(dex.as_ref(), Some(50))
                    .unwrap_or_else(|_| Self::create_fallback_pokemon()),
                Pokemon::test_pokemon(dex.as_ref(), Some(50))
                    .unwrap_or_else(|_| Self::create_fallback_pokemon()),
            ]
        };
        
        // Create sides
        let side1 = Side::new(SideId::P1, "Player 1".to_string(), team1, &format)?;
        let side2 = Side::new(SideId::P2, "Player 2".to_string(), team2, &format)?;
        
        // Create battle state
        let state = BattleState::new(
            format,
            FormatRules::default(),
            vec![side1, side2],
            Some("test-seed".to_string()),
        )?;
        
        Ok(Self::new(state, dex))
    }
    
    /// Create a battle from team descriptions (species names and moves)
    /// Example: Battle::from_teams(dex, &[("pikachu", &["thunderbolt"])], &[("charizard", &["flamethrower"])])
    pub fn from_teams(
        dex: Box<dyn Dex>,
        team1_desc: &[(&str, &[&str])], // (species, moves)
        team2_desc: &[(&str, &[&str])],
        format: Option<crate::format::BattleFormat>,
    ) -> BattleResult<Self> {
        use crate::pokemon::Pokemon;
        
        // Create team 1
        let mut team1 = Vec::new();
        for (species, moves) in team1_desc {
            let pokemon = Pokemon::from_dex(dex.as_ref(), species, 50, moves, None, None, None, None)?;
            team1.push(pokemon);
        }
        
        // Create team 2
        let mut team2 = Vec::new();
        for (species, moves) in team2_desc {
            let pokemon = Pokemon::from_dex(dex.as_ref(), species, 50, moves, None, None, None, None)?;
            team2.push(pokemon);
        }
        
        Self::test_battle_with_teams(dex, Some(team1), Some(team2), format)
    }
    
    /// Create a fallback Pokemon when test_pokemon fails (no dex data available)
    fn create_fallback_pokemon() -> crate::pokemon::Pokemon {
        use crate::pokemon::{Pokemon, SpeciesData, MoveData, AbilityData};
        use crate::types::*;
        use crate::events::EventHandlerRegistry;
        
        let species = SpeciesData {
            id: "testmon".to_string(),
            name: "Test Pokemon".to_string(),
            types: [Type::Normal, Type::Normal],
            base_stats: StatsTable {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            abilities: vec!["testability".to_string()],
            height: 1.0,
            weight: 10.0,
            gender_ratio: crate::pokemon::GenderRatio::Genderless,
        };
        
        let moves = [
            MoveData {
                id: "test-move".to_string(),
                name: "Test Move".to_string(),
                type_: Type::Normal,
                category: MoveCategory::Physical,
                base_power: 80,
                accuracy: Some(100),
                pp: 20,
                target: MoveTarget::Normal,
                priority: 0,
                flags: Default::default(),
                crit_ratio: 1,
                multihit: None,
                drain: None,
                recoil: None,
                secondary_effect: None,
            },
            MoveData {
                id: "test-move-2".to_string(),
                name: "Test Move 2".to_string(),
                type_: Type::Normal,
                category: MoveCategory::Special,
                base_power: 80,
                accuracy: Some(100),
                pp: 20,
                target: MoveTarget::Normal,
                priority: 0,
                flags: Default::default(),
                crit_ratio: 1,
                multihit: None,
                drain: None,
                recoil: None,
                secondary_effect: None,
            },
            MoveData {
                id: "test-move-3".to_string(),
                name: "Test Move 3".to_string(),
                type_: Type::Normal,
                category: MoveCategory::Status,
                base_power: 0,
                accuracy: Some(100),
                pp: 20,
                target: MoveTarget::Self_,
                priority: 0,
                flags: Default::default(),
                crit_ratio: 1,
                multihit: None,
                drain: None,
                recoil: None,
                secondary_effect: None,
            },
            MoveData {
                id: "test-move-4".to_string(),
                name: "Test Move 4".to_string(),
                type_: Type::Normal,
                category: MoveCategory::Physical,
                base_power: 60,
                accuracy: Some(100),
                pp: 25,
                target: MoveTarget::Normal,
                priority: 1,
                flags: Default::default(),
                crit_ratio: 1,
                multihit: None,
                drain: None,
                recoil: None,
                secondary_effect: None,
            },
        ];
        
        let ability = AbilityData {
            id: "test-ability".to_string(),
            name: "Test Ability".to_string(),
            description: "A test ability".to_string(),
            event_handlers: crate::events::EventHandlerRegistry::default(),
        };
        
        Pokemon::new(
            species,
            50,
            moves,
            ability,
            None,
            Nature::Hardy,
            crate::types::StatsTable::max(),
            StatsTable::default(),
            Gender::Genderless,
        )
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
    
    /// Execute an action
    fn execute_action(&mut self, action: crate::action_queue::Action) -> BattleResult<()> {
        match action.choice {
            crate::action_queue::ActionChoice::Move { 
                move_id, 
                move_data,
                target_location: _,
                original_target: _,
                mega: _,
                z_move: _,
                max_move: _,
            } => {
                if let Some(pokemon_ref) = action.pokemon {
                    self.execute_move(&pokemon_ref, &move_id, move_data.as_ref())?;
                }
            },
            crate::action_queue::ActionChoice::Switch { target: _ } => {
                // TODO: Implement switch logic
            },
            _ => {
                // TODO: Implement other action types
            }
        }
        Ok(())
    }
    
    /// Execute a move
    fn execute_move(&mut self, user_ref: &crate::pokemon::PokemonRef, move_id: &str, move_data: Option<&crate::pokemon::MoveData>) -> BattleResult<()> {
        // 1. Get move data if not provided
        let move_data = if let Some(data) = move_data {
            data.clone()
        } else {
            // Look up move in dex
            self.dex.get_move(move_id)
                .ok_or_else(|| crate::errors::BattleError::InvalidMove(move_id.to_string()))?
                .clone()
        };
        
        // 2. Determine targets (for now, just target the opponent's first active Pokemon)
        let targets = self.get_move_targets(user_ref, &move_data)?;
        
        // 3. Calculate damage using the damage formula
        if move_data.base_power > 0 {
            for target_ref in &targets {
                let damage = self.calculate_move_damage(user_ref, target_ref, &move_data)?;
                self.apply_damage(target_ref, damage)?;
            }
        }
        
        // 4. TODO: Apply secondary effects (status, stat changes, etc.)
        
        Ok(())
    }
    
    /// Get targets for a move (simplified for now)
    fn get_move_targets(&self, user_ref: &crate::pokemon::PokemonRef, _move_data: &crate::pokemon::MoveData) -> BattleResult<Vec<crate::pokemon::PokemonRef>> {
        // For now, just target the opponent's first active Pokemon
        let opponent_side_id = if user_ref.side == crate::side::SideId::P1 { 
            crate::side::SideId::P2 
        } else { 
            crate::side::SideId::P1 
        };
        
        let opponent_side = self.state.sides.iter()
            .find(|s| s.id == opponent_side_id)
            .ok_or_else(|| crate::errors::BattleError::InvalidPokemon("No opponent side found".to_string()))?;
            
        if let Some(Some(target_index)) = opponent_side.active.get(0) {
            Ok(vec![crate::pokemon::PokemonRef {
                side: opponent_side_id,
                position: 0,
            }])
        } else {
            Err(crate::errors::BattleError::InvalidPokemon("No active opponent Pokemon".to_string()))
        }
    }
    
    /// Calculate damage for a move
    fn calculate_move_damage(&self, user_ref: &crate::pokemon::PokemonRef, target_ref: &crate::pokemon::PokemonRef, move_data: &crate::pokemon::MoveData) -> BattleResult<u16> {
        let user = self.state.get_pokemon(*user_ref)
            .ok_or_else(|| crate::errors::BattleError::InvalidPokemon("User not found".to_string()))?;
        let target = self.state.get_pokemon(*target_ref)
            .ok_or_else(|| crate::errors::BattleError::InvalidPokemon("Target not found".to_string()))?;
            
        // Use the damage calculation from moves::damage module
        let damage = crate::moves::damage::calculate_damage(
            user,
            target, 
            move_data,
            false, // no critical hit for now
            &mut self.state.random.clone(), // TODO: proper PRNG access
            self.dex.as_ref(),
        )?;
        
        Ok(damage)
    }
    
    /// Apply damage to a Pokemon
    fn apply_damage(&mut self, target_ref: &crate::pokemon::PokemonRef, damage: u16) -> BattleResult<()> {
        if let Some(target) = self.state.get_pokemon_mut(*target_ref) {
            let actual_damage = std::cmp::min(damage, target.hp);
            target.hp = target.hp.saturating_sub(actual_damage);
            
            // Check if Pokemon fainted
            if target.hp == 0 {
                target.fainted = true;
                // TODO: Handle faint consequences
            }
            
            println!("Applied {} damage to Pokemon at {:?}. HP: {}/{}", 
                actual_damage, target_ref, target.hp, target.max_hp);
        }
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