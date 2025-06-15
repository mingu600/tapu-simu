//! Core battle state that can be serialized and restored

use serde::{Deserialize, Serialize};
use crate::side::{Side, SideId};
use crate::action_queue::ActionQueue;
use crate::prng::PRNGState;
use crate::format::{BattleFormat, FormatRules};
use crate::errors::{BattleError, BattleResult};
use crate::pokemon::StatType;
use std::collections::HashMap;

/// Complete battle state that can be serialized/deserialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    /// Current turn number
    pub turn: u32,
    
    /// All sides in the battle (up to 4 for FFA)
    pub sides: Vec<Side>,
    
    /// Field conditions and state
    pub field: FieldState,
    
    /// Action queue for current turn
    pub queue: ActionQueue,
    
    /// Random number generator state
    pub random: PRNGState,
    
    /// Battle format and rules
    pub format: BattleFormat,
    pub rules: FormatRules,
    
    /// Whether the battle has ended
    pub ended: bool,
    
    /// Winner of the battle (if ended)
    pub winner: Option<SideId>,
    
    /// Battle log for debugging/replay
    pub log: Vec<LogEntry>,
}

/// Field state (weather, terrain, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldState {
    /// Current weather
    pub weather: Option<Weather>,
    
    /// Weather turns remaining
    pub weather_turns: u8,
    
    /// Current terrain
    pub terrain: Option<Terrain>,
    
    /// Terrain turns remaining
    pub terrain_turns: u8,
    
    /// Trick Room turns remaining
    pub trick_room: u8,
    
    /// Magic Room turns remaining
    pub magic_room: u8,
    
    /// Wonder Room turns remaining
    pub wonder_room: u8,
    
    /// Gravity turns remaining
    pub gravity: u8,
    
    /// Other field effects
    pub effects: HashMap<String, FieldEffect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    Sun, Rain, Sandstorm, Hail, Snow,
    HarshSun, PrimordialSea, DeltaStream,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Terrain {
    Electric, Grassy, Misty, Psychic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEffect {
    pub id: String,
    pub duration: Option<u8>,
    pub data: HashMap<String, serde_json::Value>,
}

/// Battle log entry for debugging and replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub turn: u32,
    pub timestamp: u64, // Unix timestamp
    pub event: LogEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEvent {
    TurnStart(u32),
    Move { 
        side: SideId, 
        position: usize, 
        move_name: String, 
        target: Option<(SideId, usize)> 
    },
    Switch { 
        side: SideId, 
        position: usize, 
        pokemon_name: String 
    },
    Damage { 
        side: SideId, 
        position: usize, 
        damage: u16, 
        new_hp: u16 
    },
    Heal { 
        side: SideId, 
        position: usize, 
        heal: u16, 
        new_hp: u16 
    },
    StatusApply { 
        side: SideId, 
        position: usize, 
        status: String 
    },
    StatusRemove { 
        side: SideId, 
        position: usize, 
        status: String 
    },
    Faint { 
        side: SideId, 
        position: usize 
    },
    BattleEnd { 
        winner: Option<SideId> 
    },
    Debug(String),
}

impl BattleState {
    /// Create a new battle state (simple version for testing)
    pub fn new(format: BattleFormat) -> Self {
        Self {
            turn: 0,
            sides: Vec::new(),
            field: FieldState::new(),
            queue: ActionQueue::new(),
            random: PRNGState::generate_seed(),
            format,
            rules: FormatRules::default(),
            ended: false,
            winner: None,
            log: Vec::new(),
        }
    }
    
    /// Create a new battle state with full configuration
    pub fn new_with_sides(
        format: BattleFormat,
        rules: FormatRules,
        sides: Vec<Side>,
        seed: Option<String>,
    ) -> BattleResult<Self> {
        if sides.len() < 2 {
            return Err(BattleError::DataError("Need at least 2 sides".to_string()));
        }
        
        if sides.len() > format.max_sides() {
            return Err(BattleError::DataError(
                format!("Too many sides for format: {} > {}", sides.len(), format.max_sides())
            ));
        }
        
        let random = if let Some(seed) = seed {
            PRNGState::from_seed(&seed)?
        } else {
            PRNGState::generate_seed()
        };
        
        Ok(Self {
            turn: 0,
            sides,
            field: FieldState::new(),
            queue: ActionQueue::new(),
            random,
            format,
            rules,
            ended: false,
            winner: None,
            log: Vec::new(),
        })
    }
    
    /// Serialize to binary format
    pub fn to_bytes(&self) -> BattleResult<Vec<u8>> {
        bincode::serialize(self).map_err(BattleError::from)
    }
    
    /// Deserialize from binary format
    pub fn from_bytes(bytes: &[u8]) -> BattleResult<Self> {
        bincode::deserialize(bytes).map_err(BattleError::from)
    }
    
    /// Serialize to JSON format (human-readable)
    pub fn to_json(&self) -> BattleResult<String> {
        serde_json::to_string_pretty(self).map_err(BattleError::from)
    }
    
    /// Deserialize from JSON format
    pub fn from_json(json: &str) -> BattleResult<Self> {
        serde_json::from_str(json).map_err(BattleError::from)
    }
    
    /// Get side by ID
    pub fn get_side(&self, side_id: SideId) -> BattleResult<&Side> {
        self.sides.iter().find(|s| s.id == side_id)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid side ID: {:?}", side_id)))
    }
    
    /// Get mutable side by ID
    pub fn get_side_mut(&mut self, side_id: SideId) -> BattleResult<&mut Side> {
        self.sides.iter_mut().find(|s| s.id == side_id)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid side ID: {:?}", side_id)))
    }
    
    /// Get side by ID (option version for backwards compatibility)
    pub fn get_side_opt(&self, side_id: SideId) -> Option<&Side> {
        self.sides.iter().find(|s| s.id == side_id)
    }
    
    /// Get mutable side by ID (option version for backwards compatibility)
    pub fn get_side_mut_opt(&mut self, side_id: SideId) -> Option<&mut Side> {
        self.sides.iter_mut().find(|s| s.id == side_id)
    }
    
    /// Check if battle should end
    pub fn check_battle_end(&mut self) -> bool {
        if self.ended {
            return true;
        }
        
        // Count sides with Pokemon remaining
        let sides_alive: Vec<_> = self.sides.iter()
            .filter(|side| side.has_pokemon_left())
            .collect();
        
        if sides_alive.len() <= 1 {
            self.ended = true;
            self.winner = sides_alive.first().map(|s| s.id);
            
            self.add_log(LogEvent::BattleEnd { winner: self.winner });
            return true;
        }
        
        false
    }
    
    /// Add an entry to the battle log
    pub fn add_log(&mut self, event: LogEvent) {
        let entry = LogEntry {
            turn: self.turn,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event,
        };
        self.log.push(entry);
    }
    
    /// Start a new turn
    pub fn start_turn(&mut self) {
        self.turn += 1;
        
        // Clear previous choices
        for side in &mut self.sides {
            side.clear_choice();
        }
        
        self.add_log(LogEvent::TurnStart(self.turn));
        
        // Add start-of-turn effects to queue
        self.queue.add_start_turn();
    }
    
    /// Get all Pokemon speeds for priority calculation
    pub fn get_pokemon_speeds(&self) -> Vec<(SideId, usize, u16)> {
        let mut speeds = Vec::new();
        
        for side in &self.sides {
            for (pos, pokemon_opt) in side.active.iter().enumerate() {
                if let Some(pokemon_idx) = pokemon_opt {
                    if let Some(pokemon) = side.pokemon.get(*pokemon_idx) {
                        if !pokemon.is_fainted() {
                            let speed = pokemon.effective_stat(StatType::Speed);
                            speeds.push((side.id, pos, speed));
                        }
                    }
                }
            }
        }
        
        speeds
    }
    
    /// Check if all sides have made their choices
    pub fn all_choices_made(&self) -> bool {
        self.sides.iter().all(|side| side.is_choice_done())
    }
    
    /// Get a Pokemon by reference
    pub fn get_pokemon(&self, pokemon_ref: crate::pokemon::PokemonRef) -> BattleResult<&crate::pokemon::Pokemon> {
        let side = self.get_side(pokemon_ref.side)?;
        side.pokemon.get(pokemon_ref.position)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid Pokemon position: {}", pokemon_ref.position)))
    }
    
    /// Get a mutable Pokemon by reference
    pub fn get_pokemon_mut(&mut self, pokemon_ref: crate::pokemon::PokemonRef) -> BattleResult<&mut crate::pokemon::Pokemon> {
        let side = self.get_side_mut(pokemon_ref.side)?;
        side.pokemon.get_mut(pokemon_ref.position)
            .ok_or_else(|| BattleError::InvalidMove(format!("Invalid Pokemon position: {}", pokemon_ref.position)))
    }
    
    /// Get a Pokemon by reference (option version for backwards compatibility)
    pub fn get_pokemon_opt(&self, pokemon_ref: crate::pokemon::PokemonRef) -> Option<&crate::pokemon::Pokemon> {
        let side = self.sides.iter().find(|s| s.id == pokemon_ref.side)?;
        side.pokemon.get(pokemon_ref.position)
    }
    
    /// Get a mutable Pokemon by reference (option version for backwards compatibility)
    pub fn get_pokemon_mut_opt(&mut self, pokemon_ref: crate::pokemon::PokemonRef) -> Option<&mut crate::pokemon::Pokemon> {
        let side = self.sides.iter_mut().find(|s| s.id == pokemon_ref.side)?;
        side.pokemon.get_mut(pokemon_ref.position)
    }
    
    /// Get battle summary for AI/RL
    pub fn get_summary(&self) -> BattleSummary {
        BattleSummary {
            turn: self.turn,
            ended: self.ended,
            winner: self.winner,
            active_pokemon: self.sides.iter().map(|side| {
                side.get_all_active().into_iter().map(|p| PokemonSummary {
                    species: p.species.name.clone(),
                    hp: p.hp,
                    max_hp: p.max_hp,
                    status: p.status.map(|s| format!("{:?}", s)),
                    fainted: p.is_fainted(),
                }).collect()
            }).collect(),
        }
    }
}

/// Simplified battle state summary for AI/RL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSummary {
    pub turn: u32,
    pub ended: bool,
    pub winner: Option<SideId>,
    pub active_pokemon: Vec<Vec<PokemonSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSummary {
    pub species: String,
    pub hp: u16,
    pub max_hp: u16,
    pub status: Option<String>,
    pub fainted: bool,
}

impl FieldState {
    pub fn new() -> Self {
        Self {
            weather: None,
            weather_turns: 0,
            terrain: None,
            terrain_turns: 0,
            trick_room: 0,
            magic_room: 0,
            wonder_room: 0,
            gravity: 0,
            effects: HashMap::new(),
        }
    }
    
    /// Check if weather is active
    pub fn has_weather(&self, weather: Weather) -> bool {
        self.weather.as_ref() == Some(&weather) && self.weather_turns > 0
    }
    
    /// Check if terrain is active
    pub fn has_terrain(&self, terrain: Terrain) -> bool {
        self.terrain.as_ref() == Some(&terrain) && self.terrain_turns > 0
    }
    
    /// Set weather
    pub fn set_weather(&mut self, weather: Weather, turns: u8) {
        self.weather = Some(weather);
        self.weather_turns = turns;
    }
    
    /// Set terrain
    pub fn set_terrain(&mut self, terrain: Terrain, turns: u8) {
        self.terrain = Some(terrain);
        self.terrain_turns = turns;
    }
    
    /// Process end-of-turn field effects
    pub fn process_turn_end(&mut self) {
        // Decrement weather
        if self.weather_turns > 0 {
            self.weather_turns -= 1;
            if self.weather_turns == 0 {
                self.weather = None;
            }
        }
        
        // Decrement terrain
        if self.terrain_turns > 0 {
            self.terrain_turns -= 1;
            if self.terrain_turns == 0 {
                self.terrain = None;
            }
        }
        
        // Decrement room effects
        self.trick_room = self.trick_room.saturating_sub(1);
        self.magic_room = self.magic_room.saturating_sub(1);
        self.wonder_room = self.wonder_room.saturating_sub(1);
        self.gravity = self.gravity.saturating_sub(1);
    }
}

impl Default for FieldState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pokemon::*;
    use crate::types::*;
    
    fn create_test_side(id: SideId) -> Side {
        let pokemon = crate::side::tests::create_test_pokemon();
        let team = vec![pokemon];
        Side::new(id, "Test Player".to_string(), team, &BattleFormat::Singles).unwrap()
    }
    
    #[test]
    fn test_battle_state_creation() {
        let sides = vec![
            create_test_side(SideId::P1),
            create_test_side(SideId::P2),
        ];
        
        let state = BattleState::new(
            BattleFormat::Singles,
            FormatRules::default(),
            sides,
            Some("test,seed".to_string()),
        ).unwrap();
        
        assert_eq!(state.turn, 0);
        assert_eq!(state.sides.len(), 2);
        assert!(!state.ended);
        assert_eq!(state.winner, None);
    }
    
    #[test]
    fn test_serialization() {
        let sides = vec![
            create_test_side(SideId::P1),
            create_test_side(SideId::P2),
        ];
        
        let original = BattleState::new(
            BattleFormat::Singles,
            FormatRules::default(),
            sides,
            Some("test,seed".to_string()),
        ).unwrap();
        
        // Test binary serialization
        let bytes = original.to_bytes().unwrap();
        let restored = BattleState::from_bytes(&bytes).unwrap();
        
        assert_eq!(original.turn, restored.turn);
        assert_eq!(original.sides.len(), restored.sides.len());
        
        // Test JSON serialization
        let json = original.to_json().unwrap();
        let restored_json = BattleState::from_json(&json).unwrap();
        
        assert_eq!(original.turn, restored_json.turn);
        assert_eq!(original.sides.len(), restored_json.sides.len());
    }
}