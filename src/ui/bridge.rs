//! # Engine Bridge
//! 
//! This module provides the bridge between the web UI and the tapu-simu engine.
//! It handles state serialization, instruction generation, and engine integration.

use crate::core::state::State;
use crate::core::move_choice::MoveChoice;
use crate::core::battle_format::{BattleFormat, BattlePosition};
use crate::core::battle_format::SideReference;
use crate::core::instruction::{StateInstructions, Instruction};
use crate::core::state::{Pokemon, Move, MoveCategory, Gender};
use crate::data::types::Stats;
use crate::core::move_choice::{MoveIndex, PokemonIndex};
use crate::data::ps_types::PSMoveTarget;
use crate::generation::Generation;
use crate::core::battle_format::FormatType;
use crate::engine::turn::instruction_generator::GenerationXInstructionGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simplified Pokemon representation for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPokemon {
    pub species: String,
    pub level: u8,
    pub hp: i16,
    pub max_hp: i16,
    pub stats: UIPokemonStats,
    pub moves: Vec<UIMove>,
    pub ability: String,
    pub item: Option<String>,
    pub types: Vec<String>,
    pub gender: String,
    pub nature: Option<String>,
    pub ivs: Option<Vec<u8>>,
    pub evs: Option<Vec<u8>>,
    pub tera_type: Option<String>,
    pub is_terastallized: bool,
}

/// Simplified stats for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPokemonStats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

/// Simplified move representation for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMove {
    pub name: String,
    pub move_type: String,
    pub category: String,
    pub base_power: u8,
    pub accuracy: u8,
    pub pp: u8,
    pub max_pp: u8,
    pub priority: i8,
    pub target: String,
}

/// Battle state for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIBattleState {
    pub format: UIBattleFormat,
    pub side_one: UIBattleSide,
    pub side_two: UIBattleSide,
    pub weather: String,
    pub weather_turns_remaining: Option<u8>,
    pub terrain: String,
    pub terrain_turns_remaining: Option<u8>,
    pub turn: u32,
    pub trick_room_active: bool,
    pub trick_room_turns_remaining: Option<u8>,
}

/// Battle format for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIBattleFormat {
    pub name: String,
    pub format_type: String,
    pub generation: String,
    pub active_pokemon_count: usize,
}

/// Battle side for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIBattleSide {
    pub pokemon: Vec<UIPokemon>,
    pub active_pokemon_indices: Vec<Option<usize>>,
    pub side_conditions: HashMap<String, u8>,
}

/// Move choice for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMoveChoice {
    pub choice_type: String, // "move", "switch", "none"
    pub move_index: Option<usize>,
    pub target_positions: Vec<UIBattlePosition>,
    pub pokemon_index: Option<usize>,
    pub tera_type: Option<String>,
}

/// Battle position for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIBattlePosition {
    pub side: String, // "one" or "two"
    pub slot: usize,
}

/// Instruction for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIInstruction {
    pub instruction_type: String,
    pub description: String,
    pub target_position: Option<UIBattlePosition>,
    pub affected_positions: Vec<UIBattlePosition>,
    pub details: HashMap<String, serde_json::Value>,
}

/// State instructions for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStateInstructions {
    pub percentage: f32,
    pub instructions: Vec<UIInstruction>,
    pub affected_positions: Vec<UIBattlePosition>,
}

/// Response containing instruction generation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionGenerationResponse {
    pub success: bool,
    pub error: Option<String>,
    pub instructions: Vec<UIStateInstructions>,
    pub updated_state: Option<UIBattleState>,
}

/// Legal option for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UILegalOption {
    pub choice_type: String,
    pub display_name: String,
    pub move_choice: UIMoveChoice,
    pub is_disabled: bool,
    pub disabled_reason: Option<String>,
}

/// Engine bridge for UI integration
#[derive(Clone, Debug)]
pub struct EngineBridge {
    format: BattleFormat,
    last_instructions: Option<Vec<StateInstructions>>,
}

impl EngineBridge {
    /// Create a new engine bridge with the specified format
    pub fn new(format: BattleFormat) -> Self {
        Self { 
            format,
            last_instructions: None,
        }
    }

    /// Get a generator for this bridge's format
    fn get_generator(&self) -> GenerationXInstructionGenerator {
        GenerationXInstructionGenerator::new(self.format.clone())
    }

    /// Convert internal state to UI state
    pub fn state_to_ui(&self, state: &State) -> UIBattleState {
        UIBattleState {
            format: UIBattleFormat {
                name: state.format.name.clone(),
                format_type: format!("{:?}", state.format.format_type),
                generation: format!("{:?}", state.format.generation),
                active_pokemon_count: state.format.active_pokemon_count(),
            },
            side_one: self.battle_side_to_ui(&state.side_one),
            side_two: self.battle_side_to_ui(&state.side_two),
            weather: format!("{:?}", state.weather),
            weather_turns_remaining: state.weather_turns_remaining,
            terrain: format!("{:?}", state.terrain),
            terrain_turns_remaining: state.terrain_turns_remaining,
            turn: state.turn,
            trick_room_active: state.trick_room_active,
            trick_room_turns_remaining: state.trick_room_turns_remaining,
        }
    }

    /// Convert battle side to UI representation
    fn battle_side_to_ui(&self, side: &crate::core::state::BattleSide) -> UIBattleSide {
        UIBattleSide {
            pokemon: side.pokemon.iter().map(|p| self.pokemon_to_ui(p)).collect(),
            active_pokemon_indices: side.active_pokemon_indices.clone(),
            side_conditions: side.side_conditions
                .iter()
                .map(|(k, v)| (format!("{:?}", k), *v))
                .collect(),
        }
    }

    /// Convert Pokemon to UI representation
    fn pokemon_to_ui(&self, pokemon: &Pokemon) -> UIPokemon {
        UIPokemon {
            species: pokemon.species.clone(),
            level: pokemon.level,
            hp: pokemon.hp,
            max_hp: pokemon.max_hp,
            stats: UIPokemonStats {
                hp: pokemon.max_hp,
                attack: pokemon.stats.attack,
                defense: pokemon.stats.defense,
                special_attack: pokemon.stats.special_attack,
                special_defense: pokemon.stats.special_defense,
                speed: pokemon.stats.speed,
            },
            moves: pokemon.moves.iter().map(|(_, m)| self.move_to_ui(m)).collect(),
            ability: pokemon.ability.clone(),
            item: pokemon.item.clone(),
            types: pokemon.types.clone(),
            gender: format!("{:?}", pokemon.gender),
            nature: None, // Note: Engine Pokemon doesn't track nature currently
            ivs: None,    // Note: Engine Pokemon doesn't track IVs currently
            evs: None,    // Note: Engine Pokemon doesn't track EVs currently
            tera_type: pokemon.tera_type.map(|t| format!("{:?}", t)),
            is_terastallized: pokemon.is_terastallized,
        }
    }

    /// Convert Move to UI representation
    fn move_to_ui(&self, move_data: &Move) -> UIMove {
        UIMove {
            name: move_data.name.clone(),
            move_type: move_data.move_type.clone(),
            category: format!("{:?}", move_data.category),
            base_power: move_data.base_power,
            accuracy: move_data.accuracy,
            pp: move_data.pp,
            max_pp: move_data.max_pp,
            priority: move_data.priority,
            target: format!("{:?}", move_data.target),
        }
    }

    /// Convert UI move choice to internal representation
    pub fn ui_move_choice_to_internal(&self, ui_choice: &UIMoveChoice) -> Result<MoveChoice, String> {
        match ui_choice.choice_type.as_str() {
            "move" => {
                let move_index = ui_choice.move_index
                    .and_then(|i| MoveIndex::from_index(i))
                    .ok_or("Invalid move index")?;
                
                let target_positions: Result<Vec<BattlePosition>, String> = ui_choice.target_positions
                    .iter()
                    .map(|pos| self.ui_position_to_internal(pos))
                    .collect();

                let target_positions = target_positions?;

                if let Some(_tera_type) = &ui_choice.tera_type {
                    // For now, just use regular moves - can expand Tera support later
                    Ok(MoveChoice::new_move(move_index, target_positions))
                } else {
                    Ok(MoveChoice::new_move(move_index, target_positions))
                }
            }
            "switch" => {
                let pokemon_index = ui_choice.pokemon_index
                    .and_then(|i| PokemonIndex::from_index(i))
                    .ok_or("Invalid pokemon index")?;
                Ok(MoveChoice::new_switch(pokemon_index))
            }
            "none" => Ok(MoveChoice::None),
            _ => Err(format!("Unknown choice type: {}", ui_choice.choice_type)),
        }
    }

    /// Convert UI position to internal representation
    fn ui_position_to_internal(&self, ui_pos: &UIBattlePosition) -> Result<BattlePosition, String> {
        let side = match ui_pos.side.as_str() {
            "one" => SideReference::SideOne,
            "two" => SideReference::SideTwo,
            _ => return Err(format!("Invalid side: {}", ui_pos.side)),
        };
        
        Ok(BattlePosition::new(side, ui_pos.slot))
    }

    /// Convert internal position to UI representation
    fn internal_position_to_ui(&self, pos: &BattlePosition) -> UIBattlePosition {
        UIBattlePosition {
            side: match pos.side {
                SideReference::SideOne => "one".to_string(),
                SideReference::SideTwo => "two".to_string(),
            },
            slot: pos.slot,
        }
    }

    /// Convert internal instructions to UI representation
    fn instructions_to_ui(&self, instructions: &[StateInstructions]) -> Vec<UIStateInstructions> {
        instructions.iter().map(|instr| {
            UIStateInstructions {
                percentage: instr.percentage,
                instructions: instr.instruction_list.iter().map(|i| self.instruction_to_ui(i)).collect(),
                affected_positions: instr.affected_positions.iter().map(|p| self.internal_position_to_ui(p)).collect(),
            }
        }).collect()
    }

    /// Convert single instruction to UI representation
    fn instruction_to_ui(&self, instruction: &Instruction) -> UIInstruction {
        let (instruction_type, description, target_position, details) = match instruction {
            Instruction::PositionDamage(instr) => (
                "PositionDamage".to_string(),
                format!("Deal {} damage to {:?}", instr.damage_amount, instr.target_position),
                Some(self.internal_position_to_ui(&instr.target_position)),
                {
                    let mut details = HashMap::new();
                    details.insert("damage_amount".to_string(), serde_json::Value::Number(instr.damage_amount.into()));
                    details
                }
            ),
            Instruction::PositionHeal(instr) => (
                "PositionHeal".to_string(),
                format!("Heal {} HP at {:?}", instr.heal_amount, instr.target_position),
                Some(self.internal_position_to_ui(&instr.target_position)),
                {
                    let mut details = HashMap::new();
                    details.insert("heal_amount".to_string(), serde_json::Value::Number(instr.heal_amount.into()));
                    details
                }
            ),
            Instruction::ApplyStatus(instr) => (
                "ApplyStatus".to_string(),
                format!("Apply {:?} status to {:?}", instr.status, instr.target_position),
                Some(self.internal_position_to_ui(&instr.target_position)),
                {
                    let mut details = HashMap::new();
                    details.insert("status".to_string(), serde_json::Value::String(format!("{:?}", instr.status)));
                    details
                }
            ),
            Instruction::BoostStats(instr) => (
                "BoostStats".to_string(),
                format!("Boost stats at {:?}: {:?}", instr.target_position, instr.stat_boosts),
                Some(self.internal_position_to_ui(&instr.target_position)),
                {
                    let mut details = HashMap::new();
                    for (stat, boost) in &instr.stat_boosts {
                        details.insert(format!("{:?}", stat), serde_json::Value::Number((*boost).into()));
                    }
                    details
                }
            ),
            Instruction::ChangeWeather(instr) => (
                "ChangeWeather".to_string(),
                format!("Change weather to {:?} for {:?} turns", instr.weather, instr.duration),
                None,
                {
                    let mut details = HashMap::new();
                    details.insert("weather".to_string(), serde_json::Value::String(format!("{:?}", instr.weather)));
                    if let Some(duration) = instr.duration {
                        details.insert("duration".to_string(), serde_json::Value::Number(duration.into()));
                    }
                    details
                }
            ),
            _ => (
                "Other".to_string(),
                format!("Instruction: {:?}", instruction),
                None,
                HashMap::new(),
            ),
        };

        UIInstruction {
            instruction_type,
            description,
            target_position,
            affected_positions: instruction.affected_positions().iter().map(|p| self.internal_position_to_ui(p)).collect(),
            details,
        }
    }

    /// Generate instructions from move choices
    pub fn generate_instructions(
        &mut self,
        state: &mut State,
        side_one_choice: &UIMoveChoice,
        side_two_choice: &UIMoveChoice,
    ) -> InstructionGenerationResponse {
        // Convert UI choices to internal format
        let internal_choice_one = match self.ui_move_choice_to_internal(side_one_choice) {
            Ok(choice) => choice,
            Err(e) => return InstructionGenerationResponse {
                success: false,
                error: Some(format!("Failed to convert side one choice: {}", e)),
                instructions: vec![],
                updated_state: None,
            }
        };

        let internal_choice_two = match self.ui_move_choice_to_internal(side_two_choice) {
            Ok(choice) => choice,
            Err(e) => return InstructionGenerationResponse {
                success: false,
                error: Some(format!("Failed to convert side two choice: {}", e)),
                instructions: vec![],
                updated_state: None,
            }
        };

        // Generate instructions using the engine
        let generator = self.get_generator();
        let instructions = generator.generate_instructions(state, &internal_choice_one, &internal_choice_two);

        // Store instructions for later application
        self.last_instructions = Some(instructions.clone());

        // Convert to UI format
        let ui_instructions = self.instructions_to_ui(&instructions);

        // Don't auto-apply instructions - let the frontend choose which set to apply
        // The updated_state should remain None until a specific instruction set is selected
        
        InstructionGenerationResponse {
            success: true,
            error: None,
            instructions: ui_instructions,
            updated_state: None, // No auto-applied state - frontend will request specific applications
        }
    }

    /// Apply a specific instruction set to the battle state
    pub fn apply_instruction_set(
        &self,
        state: &mut State,
        instruction_set_index: usize,
        expected_turn_number: Option<u32>,
    ) -> Result<UIBattleState, String> {
        let instructions = self.last_instructions.as_ref()
            .ok_or("No instructions available. Generate instructions first.")?;

        if instruction_set_index >= instructions.len() {
            return Err(format!("Invalid instruction set index: {}. Available sets: 0-{}", instruction_set_index, instructions.len() - 1));
        }

        // Apply the selected instruction set
        let selected_instruction_set = &instructions[instruction_set_index];
        state.apply_instructions(&selected_instruction_set.instruction_list);

        // Set turn number based on expected turn or increment by 1
        if let Some(expected_turn) = expected_turn_number {
            state.turn = expected_turn;
        } else {
            state.turn += 1;
        }

        // Return the updated state
        Ok(self.state_to_ui(state))
    }

    /// Create a default Pokemon for testing
    pub fn create_default_pokemon(species: &str) -> UIPokemon {
        UIPokemon {
            species: species.to_string(),
            level: 50,
            hp: 150,
            max_hp: 150,
            stats: UIPokemonStats {
                hp: 150,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            moves: vec![
                UIMove {
                    name: "Tackle".to_string(),
                    move_type: "Normal".to_string(),
                    category: "Physical".to_string(),
                    base_power: 40,
                    accuracy: 100,
                    pp: 35,
                    max_pp: 35,
                    priority: 0,
                    target: "Normal".to_string(),
                },
                UIMove {
                    name: "Thunder Wave".to_string(),
                    move_type: "Electric".to_string(),
                    category: "Status".to_string(),
                    base_power: 0,
                    accuracy: 90,
                    pp: 20,
                    max_pp: 20,
                    priority: 0,
                    target: "Normal".to_string(),
                },
            ],
            ability: "Static".to_string(),
            item: None,
            types: vec!["Normal".to_string()],
            gender: "Unknown".to_string(),
            nature: Some("Hardy".to_string()),
            ivs: Some(vec![31, 31, 31, 31, 31, 31]),
            evs: Some(vec![0, 0, 0, 0, 0, 0]),
            tera_type: None,
            is_terastallized: false,
        }
    }

    /// Convert UI Pokemon to internal Pokemon
    pub fn ui_pokemon_to_internal(&self, ui_pokemon: &UIPokemon) -> Pokemon {
        let mut pokemon = Pokemon::new(ui_pokemon.species.clone());
        
        pokemon.level = ui_pokemon.level;
        pokemon.hp = ui_pokemon.hp;
        pokemon.max_hp = ui_pokemon.max_hp;
        pokemon.stats = Stats {
            hp: ui_pokemon.hp,
            attack: ui_pokemon.stats.attack,
            defense: ui_pokemon.stats.defense,
            special_attack: ui_pokemon.stats.special_attack,
            special_defense: ui_pokemon.stats.special_defense,
            speed: ui_pokemon.stats.speed,
        };
        pokemon.ability = ui_pokemon.ability.clone();
        pokemon.item = ui_pokemon.item.clone();
        pokemon.types = ui_pokemon.types.clone();
        
        // Convert gender
        pokemon.gender = match ui_pokemon.gender.as_str() {
            "Male" => Gender::Male,
            "Female" => Gender::Female,
            _ => Gender::Unknown,
        };

        // Add moves
        for (i, ui_move) in ui_pokemon.moves.iter().enumerate() {
            if let Some(move_index) = MoveIndex::from_index(i) {
                let move_category = match ui_move.category.as_str() {
                    "Physical" => MoveCategory::Physical,
                    "Special" => MoveCategory::Special,
                    _ => MoveCategory::Status,
                };

                let target = match ui_move.target.as_str() {
                    "Self_" => PSMoveTarget::Self_,
                    "AllAdjacent" => PSMoveTarget::AllAdjacent,
                    "AllAdjacentFoes" => PSMoveTarget::AllAdjacentFoes,
                    _ => PSMoveTarget::Normal,
                };

                let move_data = Move::new_with_details(
                    ui_move.name.clone(),
                    ui_move.base_power,
                    ui_move.accuracy,
                    ui_move.move_type.clone(),
                    ui_move.pp,
                    target,
                    move_category,
                    ui_move.priority,
                );
                
                pokemon.add_move(move_index, move_data);
            }
        }

        // Terastallization fields (Gen 9+ only)
        pokemon.is_terastallized = ui_pokemon.is_terastallized;
        // Note: tera_type conversion would need more work to match the enum

        pokemon
    }

    /// Get all legal options for both sides
    pub fn get_all_legal_options(&self, state: &State) -> Result<(Vec<UILegalOption>, Vec<UILegalOption>), String> {
        let (side_one_choices, side_two_choices) = state.get_all_options();
        
        let side_one_options = side_one_choices.into_iter().enumerate().map(|(index, choice)| {
            self.move_choice_to_legal_option(choice, index, state, SideReference::SideOne)
        }).collect();
        
        let side_two_options = side_two_choices.into_iter().enumerate().map(|(index, choice)| {
            self.move_choice_to_legal_option(choice, index, state, SideReference::SideTwo)
        }).collect();
        
        Ok((side_one_options, side_two_options))
    }
    
    /// Get the move name for a specific side and move index
    fn get_move_name_for_side(&self, state: &State, side: SideReference, move_index: MoveIndex) -> Option<String> {
        let battle_side = match side {
            SideReference::SideOne => &state.side_one,
            SideReference::SideTwo => &state.side_two,
        };
        
        // Get the active Pokemon for this side
        if let Some(active_index) = battle_side.active_pokemon_indices.get(0) {
            if let Some(active_index) = active_index {
                if let Some(pokemon) = battle_side.pokemon.get(*active_index) {
                    if let Some(move_data) = pokemon.moves.get(&move_index) {
                        return Some(move_data.name.clone());
                    }
                }
            }
        }
        
        None
    }
    
    /// Get the Pokemon name for a specific side and Pokemon index
    fn get_pokemon_name_for_side(&self, state: &State, side: SideReference, pokemon_index: PokemonIndex) -> Option<String> {
        let battle_side = match side {
            SideReference::SideOne => &state.side_one,
            SideReference::SideTwo => &state.side_two,
        };
        
        let index = pokemon_index.to_index();
        if index < battle_side.pokemon.len() {
            let pokemon = &battle_side.pokemon[index];
            Some(pokemon.species.clone())
        } else {
            None
        }
    }
    
    /// Convert a move choice to a legal option for display
    fn move_choice_to_legal_option(&self, choice: MoveChoice, _index: usize, state: &State, side: SideReference) -> UILegalOption {
        let (display_name, ui_choice, is_disabled, disabled_reason) = match &choice {
            MoveChoice::Move { move_index, target_positions } => {
                let targets_str = if target_positions.len() > 1 {
                    format!(" → {} targets", target_positions.len())
                } else if let Some(pos) = target_positions.first() {
                    format!(" → {}{}", 
                        match pos.side {
                            SideReference::SideOne => "Side 1",
                            SideReference::SideTwo => "Side 2",
                        }, 
                        pos.slot + 1
                    )
                } else {
                    String::new()
                };
                
                // Get the actual move name from the Pokemon's moveset
                let move_name = self.get_move_name_for_side(state, side, *move_index)
                    .unwrap_or_else(|| format!("Move {}", *move_index as u8));
                
                (
                    format!("{}{}", move_name, targets_str),
                    UIMoveChoice {
                        choice_type: "move".to_string(),
                        move_index: Some(*move_index as usize),
                        target_positions: target_positions.iter().map(|p| self.internal_position_to_ui(p)).collect(),
                        pokemon_index: None,
                        tera_type: None,
                    },
                    false,
                    None
                )
            },
            MoveChoice::Switch(pokemon_index) => {
                // Get the Pokemon name from the team
                let pokemon_name = self.get_pokemon_name_for_side(state, side, *pokemon_index)
                    .unwrap_or_else(|| format!("Pokemon #{}", *pokemon_index as u8));
                
                (
                    format!("Switch to {}", pokemon_name),
                    UIMoveChoice {
                        choice_type: "switch".to_string(),
                        move_index: None,
                        target_positions: vec![], // Switches don't have targets
                        pokemon_index: Some(*pokemon_index as usize),
                        tera_type: None,
                    },
                    false,
                    None
                )
            },
            MoveChoice::None => (
                "No Action".to_string(),
                UIMoveChoice {
                    choice_type: "none".to_string(),
                    move_index: None,
                    target_positions: vec![],
                    pokemon_index: None,
                    tera_type: None,
                },
                false,
                None
            ),
            MoveChoice::MoveTera { move_index, target_positions, tera_type } => {
                let targets_str = if target_positions.len() > 1 {
                    format!(" → {} targets", target_positions.len())
                } else if let Some(pos) = target_positions.first() {
                    format!(" → {}{}", 
                        match pos.side {
                            SideReference::SideOne => "Side 1",
                            SideReference::SideTwo => "Side 2",
                        }, 
                        pos.slot + 1
                    )
                } else {
                    String::new()
                };
                
                (
                    format!("Tera {:?} Move {}{}", tera_type, *move_index as u8, targets_str),
                    UIMoveChoice {
                        choice_type: "move".to_string(),
                        move_index: Some(*move_index as usize),
                        target_positions: target_positions.iter().map(|p| self.internal_position_to_ui(p)).collect(),
                        pokemon_index: None,
                        tera_type: Some(format!("{:?}", tera_type)),
                    },
                    false,
                    None
                )
            },
        };
        
        UILegalOption {
            choice_type: ui_choice.choice_type.clone(),
            display_name,
            move_choice: ui_choice,
            is_disabled,
            disabled_reason,
        }
    }

    /// Create a battle state from UI components
    pub fn create_battle_state(&self, format: &UIBattleFormat, side_one: &UIBattleSide, side_two: &UIBattleSide) -> Result<State, String> {
        // Convert format
        let format_type = match format.format_type.as_str() {
            "Singles" => FormatType::Singles,
            "Doubles" => FormatType::Doubles,
            "Vgc" => FormatType::Vgc,
            "Triples" => FormatType::Triples,
            _ => return Err(format!("Unknown format type: {}", format.format_type)),
        };

        let generation = match format.generation.as_str() {
            "Gen9" => Generation::Gen9,
            "Gen8" => Generation::Gen8,
            "Gen7" => Generation::Gen7,
            "Gen6" => Generation::Gen6,
            "Gen5" => Generation::Gen5,
            "Gen4" => Generation::Gen4,
            _ => Generation::Gen9, // Default to Gen9
        };

        let battle_format = BattleFormat::new(format.name.clone(), generation, format_type);
        let mut state = State::new(battle_format);

        // Add Pokemon to sides
        for (i, ui_pokemon) in side_one.pokemon.iter().enumerate() {
            let pokemon = self.ui_pokemon_to_internal(ui_pokemon);
            state.side_one.add_pokemon(pokemon);
        }

        for (i, ui_pokemon) in side_two.pokemon.iter().enumerate() {
            let pokemon = self.ui_pokemon_to_internal(ui_pokemon);
            state.side_two.add_pokemon(pokemon);
        }

        // Set active Pokemon
        state.side_one.active_pokemon_indices = side_one.active_pokemon_indices.clone();
        state.side_two.active_pokemon_indices = side_two.active_pokemon_indices.clone();

        Ok(state)
    }

    /// Convert UI state back to internal state
    pub fn ui_to_state(&self, ui_state: &UIBattleState) -> Result<State, String> {
        // Start with the basic state creation from format and sides
        let mut state = self.create_battle_state(&ui_state.format, &ui_state.side_one, &ui_state.side_two)?;
        
        // Convert and set weather
        state.weather = match ui_state.weather.as_str() {
            "None" => crate::core::instruction::Weather::None,
            "Sun" => crate::core::instruction::Weather::Sun,
            "Rain" => crate::core::instruction::Weather::Rain,
            "Sandstorm" => crate::core::instruction::Weather::Sand,
            "Hail" => crate::core::instruction::Weather::Hail,
            "Snow" => crate::core::instruction::Weather::Snow,
            _ => crate::core::instruction::Weather::None,
        };
        state.weather_turns_remaining = ui_state.weather_turns_remaining;
        
        // Convert and set terrain
        state.terrain = match ui_state.terrain.as_str() {
            "None" => crate::core::instruction::Terrain::None,
            "Electric" => crate::core::instruction::Terrain::ElectricTerrain,
            "Grassy" => crate::core::instruction::Terrain::GrassyTerrain,
            "Misty" => crate::core::instruction::Terrain::MistyTerrain,
            "Psychic" => crate::core::instruction::Terrain::PsychicTerrain,
            _ => crate::core::instruction::Terrain::None,
        };
        state.terrain_turns_remaining = ui_state.terrain_turns_remaining;
        
        // Set turn counter
        state.turn = ui_state.turn;
        
        // Set Trick Room status
        state.trick_room_active = ui_state.trick_room_active;
        state.trick_room_turns_remaining = ui_state.trick_room_turns_remaining;
        
        Ok(state)
    }
}