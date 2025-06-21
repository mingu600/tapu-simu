//! # Modern Battle Builder
//! 
//! Standardized battle builder implementing the common Builder trait
//! with comprehensive validation and error handling.

use crate::core::battle_format::BattleFormat;
use crate::core::battle_state::BattleState;
use crate::data::ps::repository::Repository;
use crate::data::RandomPokemonSet;
use crate::simulator::Player;
use crate::types::errors::BattleError;
use super::traits::{Builder, BuilderError, ValidationContext, ValidatingBuilder};

/// Modern battle builder with standardized interface
pub struct ModernBattleBuilder<'a> {
    /// Data repository for Pokemon/move/ability data
    data: &'a Repository,
    /// Battle format configuration
    format: Option<BattleFormat>,
    /// Teams for the battle
    teams: Option<(Vec<RandomPokemonSet>, Vec<RandomPokemonSet>)>,
    /// Players for the battle
    players: Option<(Box<dyn Player>, Box<dyn Player>)>,
    /// Battle configuration options
    config: BattleConfig,
    /// Validation context
    validation_context: ValidationContext,
}

/// Configuration options for battles
#[derive(Debug, Clone)]
pub struct BattleConfig {
    /// Maximum number of turns before declaring a draw
    pub max_turns: u32,
    /// Random seed for reproducible battles
    pub seed: Option<u64>,
    /// Whether to measure execution time
    pub measure_time: bool,
    /// Whether to enable detailed logging
    pub detailed_logging: bool,
    /// Timeout per turn in milliseconds
    pub turn_timeout_ms: Option<u32>,
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            max_turns: 1000,
            seed: None,
            measure_time: false,
            detailed_logging: false,
            turn_timeout_ms: None,
        }
    }
}

/// Result of building a battle
pub struct Battle {
    /// The battle state
    pub state: BattleState,
    /// Player 1
    pub player1: Box<dyn Player>,
    /// Player 2
    pub player2: Box<dyn Player>,
    /// Battle configuration
    pub config: BattleConfig,
}

impl<'a> ModernBattleBuilder<'a> {
    /// Create a new modern battle builder
    pub fn new(data: &'a Repository) -> Self {
        Self {
            data,
            format: None,
            teams: None,
            players: None,
            config: BattleConfig::default(),
            validation_context: ValidationContext::default(),
        }
    }

    /// Set the battle format
    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set both teams
    pub fn teams(mut self, team1: Vec<RandomPokemonSet>, team2: Vec<RandomPokemonSet>) -> Self {
        self.teams = Some((team1, team2));
        self
    }

    /// Generate random teams for the current format
    pub fn random_teams(mut self) -> Result<Self, BuilderError> {
        let _format = self.format.as_ref().ok_or_else(|| BuilderError::MissingRequired {
            field: "format".to_string(),
        })?;

        // For demonstration purposes, create simple placeholder teams
        // In a real implementation, this would use the actual random team loader
        let team1 = vec![
            RandomPokemonSet {
                name: "Charizard".to_string(),
                species: "Charizard".to_string(),
                level: 50,
                gender: None,
                shiny: None,
                ability: Some("Blaze".to_string()),
                item: None,
                moves: vec!["Flamethrower".to_string()],
                nature: None,
                evs: None,
                ivs: None,
                gigantamax: None,
                role: None,
                tera_type: None,
            }
        ];
        
        let team2 = vec![
            RandomPokemonSet {
                name: "Blastoise".to_string(),
                species: "Blastoise".to_string(),
                level: 50,
                gender: None,
                shiny: None,
                ability: Some("Torrent".to_string()),
                item: None,
                moves: vec!["Surf".to_string()],
                nature: None,
                evs: None,
                ivs: None,
                gigantamax: None,
                role: None,
                tera_type: None,
            }
        ];

        self.teams = Some((team1, team2));
        Ok(self)
    }

    /// Set both players
    pub fn players<P1, P2>(mut self, player1: P1, player2: P2) -> Self 
    where 
        P1: Player + 'static,
        P2: Player + 'static,
    {
        self.players = Some((Box::new(player1), Box::new(player2)));
        self
    }

    /// Set player 1
    pub fn player1<P>(mut self, player: P) -> Self 
    where 
        P: Player + 'static,
    {
        match self.players.take() {
            Some((_, player2)) => {
                self.players = Some((Box::new(player), player2));
            }
            None => {
                // Create a default player2 for now
                let default_player2 = Box::new(crate::simulator::RandomPlayer::new());
                self.players = Some((Box::new(player), default_player2));
            }
        }
        self
    }

    /// Set player 2
    pub fn player2<P>(mut self, player: P) -> Self 
    where 
        P: Player + 'static,
    {
        match self.players.take() {
            Some((player1, _)) => {
                self.players = Some((player1, Box::new(player)));
            }
            None => {
                // Create a default player1 for now
                let default_player1 = Box::new(crate::simulator::RandomPlayer::new());
                self.players = Some((default_player1, Box::new(player)));
            }
        }
        self
    }

    /// Configure battle options
    pub fn config(mut self, config: BattleConfig) -> Self {
        self.config = config;
        self
    }

    /// Set maximum turns
    pub fn max_turns(mut self, max_turns: u32) -> Self {
        self.config.max_turns = max_turns;
        self
    }

    /// Set random seed
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = Some(seed);
        self
    }

    /// Enable time measurement
    pub fn measure_time(mut self, enable: bool) -> Self {
        self.config.measure_time = enable;
        self
    }

    /// Set validation context
    pub fn validation_context(mut self, context: ValidationContext) -> Self {
        self.validation_context = context;
        self
    }

    /// Validate teams are compatible with format
    fn validate_teams(&self, teams: &(Vec<RandomPokemonSet>, Vec<RandomPokemonSet>), format: &BattleFormat) -> Result<(), BuilderError> {
        let (team1, team2) = teams;

        // Check team sizes
        let expected_size = match format.format_type {
            crate::core::battle_format::FormatType::Singles => 6,
            crate::core::battle_format::FormatType::Doubles => 6,
            crate::core::battle_format::FormatType::Vgc => 6,
            crate::core::battle_format::FormatType::Triples => 6,
        };

        if team1.len() != expected_size {
            return Err(BuilderError::InvalidValue {
                field: "team1".to_string(),
                value: team1.len().to_string(),
                reason: format!("Expected {} Pokemon, got {}", expected_size, team1.len()),
            });
        }

        if team2.len() != expected_size {
            return Err(BuilderError::InvalidValue {
                field: "team2".to_string(),
                value: team2.len().to_string(),
                reason: format!("Expected {} Pokemon, got {}", expected_size, team2.len()),
            });
        }

        // Additional format-specific validation could go here
        Ok(())
    }
}

impl<'a> Builder<Battle> for ModernBattleBuilder<'a> {
    type Error = BuilderError;

    fn build(self) -> Result<Battle, Self::Error> {
        // Validate first
        self.validate()?;

        let format = self.format.unwrap(); // Safe because validate() checks this
        let (team1, team2) = self.teams.unwrap(); // Safe because validate() checks this
        let (player1, player2) = self.players.unwrap_or_else(|| {
            (
                Box::new(crate::simulator::RandomPlayer::new()),
                Box::new(crate::simulator::RandomPlayer::new()),
            )
        });

        // Create battle state using the modern decomposed system
        let mut battle_state = BattleState::new(format);
        
        // Convert RandomPokemonSet to battle Pokemon and add to sides
        for pokemon_set in team1 {
            let battle_pokemon = pokemon_set.to_battle_pokemon(self.data);
            battle_state.sides[0].add_pokemon(battle_pokemon);
        }
        
        for pokemon_set in team2 {
            let battle_pokemon = pokemon_set.to_battle_pokemon(self.data);
            battle_state.sides[1].add_pokemon(battle_pokemon);
        }

        Ok(Battle {
            state: battle_state,
            player1,
            player2,
            config: self.config,
        })
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Check required fields
        let format = self.format.as_ref().ok_or_else(|| BuilderError::MissingRequired {
            field: "format".to_string(),
        })?;

        let teams = self.teams.as_ref().ok_or_else(|| BuilderError::MissingRequired {
            field: "teams".to_string(),
        })?;

        // Validate teams compatibility with format
        self.validate_teams(teams, format)?;

        // Validate config
        if self.config.max_turns == 0 {
            return Err(BuilderError::InvalidValue {
                field: "max_turns".to_string(),
                value: "0".to_string(),
                reason: "Max turns must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

impl<'a> ValidatingBuilder<Battle> for ModernBattleBuilder<'a> {
    type Context = ValidationContext;

    fn validate_aspect(&self, context: &Self::Context) -> Result<(), Self::Error> {
        if context.strict_mode {
            // Strict validation
            self.validate()?;
            
            // Additional strict checks
            if self.players.is_none() && context.collect_warnings {
                eprintln!("Warning: No players set, will use default RandomPlayers");
            }
        } else {
            // Lenient validation - only check critical requirements
            if self.format.is_none() {
                return Err(BuilderError::MissingRequired {
                    field: "format".to_string(),
                });
            }
        }

        Ok(())
    }

    fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.players.is_none() {
            warnings.push("No players specified, will use RandomPlayers".to_string());
        }

        if self.config.seed.is_none() {
            warnings.push("No seed specified, battles will not be reproducible".to_string());
        }

        warnings
    }
}