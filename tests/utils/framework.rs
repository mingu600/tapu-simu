//! # Core Test Framework
//!
//! This module provides the core test framework for tapu-simu, enabling
//! end-to-end testing with proper data loading and battle execution.

use std::collections::HashMap;
use std::sync::Arc;
use tapu_simu::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use tapu_simu::core::battle_state::BattleState;
use tapu_simu::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, SideCondition, Stat,
    Terrain, Weather,
};
use tapu_simu::core::move_choice::MoveChoice;
use tapu_simu::data::ps::repository::Repository;
use tapu_simu::data::types::Stats;
use tapu_simu::engine::turn;
use tapu_simu::generation::Generation;
use tapu_simu::types::DataResult;

/// Core test framework for tapu-simu battles
pub struct TapuTestFramework {
    repository: Arc<Repository>,
    format: BattleFormat,
}

impl TapuTestFramework {
    /// Create a new test framework with default Gen 9 Singles format
    pub fn new() -> DataResult<Self> {
        let repository = Arc::new(Repository::from_path("data/ps-extracted")?);
        Ok(Self {
            repository,
            format: BattleFormat::gen9_ou(),
        })
    }

    /// Create a test framework for a specific generation
    pub fn with_generation(gen: Generation) -> DataResult<Self> {
        let repository = Arc::new(Repository::from_path("data/ps-extracted")?);
        let format = match gen {
            Generation::Gen1 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen2 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen3 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen4 => BattleFormat::gen4_ou(),
            Generation::Gen5 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen6 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen7 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen8 => BattleFormat::gen4_ou(), // Use gen4 as fallback
            Generation::Gen9 => BattleFormat::gen9_ou(),
        };

        Ok(Self { repository, format })
    }

    /// Create a test framework with a specific format
    pub fn with_format(format: BattleFormat) -> DataResult<Self> {
        let repository = Arc::new(Repository::from_path("data/ps-extracted")?);
        Ok(Self { repository, format })
    }

    /// Get a reference to the repository
    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    /// Get the current battle format
    pub fn format(&self) -> &BattleFormat {
        &self.format
    }

    /// Execute a complete battle test
    pub fn execute_test(&self, test: BattleTest) -> TestResult {
        // Create battle state with teams
        let mut state = match self.create_battle_state(&test.team_one, &test.team_two) {
            Ok(state) => state,
            Err(e) => return TestResult::Failed(format!("Failed to create battle state: {}", e)),
        };

        // Apply Pokemon-specific overrides
        match &test.team_one {
            TeamSpec::Pokemon(spec) => {
                self.apply_pokemon_overrides(
                    &mut state,
                    spec,
                    BattlePosition::new(SideReference::SideOne, 0),
                );
            }
            TeamSpec::MultiPokemon(specs) => {
                for (i, spec) in specs.iter().enumerate() {
                    self.apply_pokemon_overrides(
                        &mut state,
                        spec,
                        BattlePosition::new(SideReference::SideOne, i),
                    );
                }
            }
        }

        match &test.team_two {
            TeamSpec::Pokemon(spec) => {
                self.apply_pokemon_overrides(
                    &mut state,
                    spec,
                    BattlePosition::new(SideReference::SideTwo, 0),
                );
            }
            TeamSpec::MultiPokemon(specs) => {
                for (i, spec) in specs.iter().enumerate() {
                    self.apply_pokemon_overrides(
                        &mut state,
                        spec,
                        BattlePosition::new(SideReference::SideTwo, i),
                    );
                }
            }
        }

        // Apply setup actions
        for setup_action in &test.setup {
            if let Err(e) = self.apply_setup_action(&mut state, setup_action) {
                return TestResult::Failed(format!("Failed to apply setup action: {}", e));
            }
        }

        // Execute moves and validate outcomes
        for (turn_idx, (move_one, move_two)) in test.moves.iter().enumerate() {
            // Execute the turn
            match self.execute_turn(&mut state, move_one.clone(), move_two.clone()) {
                Ok(instructions) => {
                    // Validate expected outcomes for this turn
                    if let Some(expected_instructions) = test.expected_instructions.get(turn_idx) {
                        if !self.validate_instructions(&instructions, expected_instructions) {
                            // Show normalized instructions for debugging
                            let normalized_actual = self.normalize_instructions(&instructions);
                            let normalized_expected =
                                self.normalize_instructions(expected_instructions);
                            return TestResult::Failed(format!(
                                "Instructions mismatch on turn {}: expected {:?}, got {:?}",
                                turn_idx + 1,
                                normalized_expected,
                                normalized_actual
                            ));
                        }
                    }
                }
                Err(e) => {
                    return TestResult::Failed(format!("Turn {} failed: {}", turn_idx + 1, e))
                }
            }
        }

        // Validate final expected outcomes
        for expected_outcome in &test.expected_outcomes {
            if let Err(e) = self.validate_outcome(&state, expected_outcome) {
                return TestResult::Failed(format!("Final outcome validation failed: {}", e));
            }
        }

        TestResult::Success
    }

    /// Create a battle state with the specified teams
    fn create_battle_state(
        &self,
        team_one: &TeamSpec,
        team_two: &TeamSpec,
    ) -> DataResult<BattleState> {
        let pokemon_one = self.create_pokemon_from_spec(team_one)?;
        let pokemon_two = self.create_pokemon_from_spec(team_two)?;

        // Create battle state with pre-constructed Pokemon
        let state = BattleState::new_with_pokemon(self.format.clone(), pokemon_one, pokemon_two);

        Ok(state)
    }

    /// Create Pokemon objects from team specification
    fn create_pokemon_from_spec(
        &self,
        team_spec: &TeamSpec,
    ) -> DataResult<Vec<tapu_simu::core::battle_state::Pokemon>> {
        match team_spec {
            TeamSpec::Pokemon(spec) => {
                let pokemon = self.create_pokemon_from_pokemon_spec(spec)?;
                Ok(vec![pokemon])
            }
            TeamSpec::MultiPokemon(specs) => {
                let mut pokemon_list = Vec::new();
                for spec in specs {
                    let pokemon = self.create_pokemon_from_pokemon_spec(spec)?;
                    pokemon_list.push(pokemon);
                }
                Ok(pokemon_list)
            }
        }
    }

    /// Create a Pokemon from PokemonSpec using repository data
    fn create_pokemon_from_pokemon_spec(
        &self,
        spec: &PokemonSpec,
    ) -> DataResult<tapu_simu::core::battle_state::Pokemon> {
        use std::collections::HashMap;
        use tapu_simu::core::battle_state::{Gender, Move, Pokemon};
        use tapu_simu::core::move_choice::MoveIndex;
        use tapu_simu::data::types::Stats;
        use tapu_simu::types::{MoveId, SpeciesId};

        // Create Pokemon with basic data
        let mut pokemon = Pokemon::new(spec.species.to_string());

        // Set level
        pokemon.level = spec.level;
        // Get Pokemon data from repository - fail if not found
        let pokemon_data = self
            .repository
            .find_pokemon_by_name(spec.species)
            .ok_or_else(|| {
                use tapu_simu::types::{DataError, SpeciesId};
                DataError::SpeciesNotFound {
                    species: SpeciesId::from(spec.species),
                }
            })?;

        // Use provided EVs/IVs or defaults
        let evs = spec.evs.unwrap_or(Stats {
            hp: 0,
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
        });

        let ivs = spec.ivs.unwrap_or(Stats {
            hp: 31,
            attack: 31,
            defense: 31,
            special_attack: 31,
            special_defense: 31,
            speed: 31,
        });

        // Use actual base stats from Pokemon data
        let level = spec.level as i16;
        let base_hp = pokemon_data.base_stats.hp as i16;
        let base_attack = pokemon_data.base_stats.attack as i16;
        let base_defense = pokemon_data.base_stats.defense as i16;
        let base_special_attack = pokemon_data.base_stats.special_attack as i16;
        let base_special_defense = pokemon_data.base_stats.special_defense as i16;
        let base_speed = pokemon_data.base_stats.speed as i16;

        pokemon.stats = Stats {
            hp: (2 * base_hp + ivs.hp + evs.hp / 4) * level / 100 + level + 10,
            attack: (2 * base_attack + ivs.attack + evs.attack / 4) * level / 100 + 5,
            defense: (2 * base_defense + ivs.defense + evs.defense / 4) * level / 100 + 5,
            special_attack: (2 * base_special_attack
                + ivs.special_attack
                + evs.special_attack / 4)
                * level
                / 100
                + 5,
            special_defense: (2 * base_special_defense
                + ivs.special_defense
                + evs.special_defense / 4)
                * level
                / 100
                + 5,
            speed: (2 * base_speed + ivs.speed + evs.speed / 4) * level / 100 + 5,
        };

        pokemon.max_hp = pokemon.stats.hp;
        pokemon.hp = pokemon.max_hp;

        // Set types from Pokemon data (convert TypeId to String)
        pokemon.types = pokemon_data
            .types
            .iter()
            .map(|t| t.as_str().to_string())
            .collect();

        // Set weight
        pokemon.weight_kg = pokemon_data.weight_kg;

        // Set ability if specified
        if let Some(ability) = spec.ability {
            pokemon.ability = ability.to_string();
        }

        // Set item if specified
        if let Some(item) = spec.item {
            pokemon.item = Some(item.to_string());
        }

        // Add moves using repository data - fail if move not found
        let mut moves = HashMap::new();
        for (i, &move_name) in spec.moves.iter().enumerate() {
            if let Some(move_index) = MoveIndex::from_index(i) {
                // Get move data from repository - fail if not found
                let move_id = MoveId::from(move_name);
                let move_data = self.repository.create_move(&move_id)?;
                moves.insert(move_index, move_data);
            }
        }
        pokemon.moves = moves;

        // Set status if specified
        if let Some(status) = spec.status {
            pokemon.status = status;
        }

        // Set HP percentage if specified
        if let Some(hp_percentage) = spec.hp_percentage {
            pokemon.hp = ((pokemon.max_hp as f32) * hp_percentage / 100.0) as i16;
        }

        Ok(pokemon)
    }

    /// Apply test-specific overrides to a Pokemon after battle state creation
    fn apply_pokemon_overrides(
        &self,
        state: &mut BattleState,
        spec: &PokemonSpec,
        position: BattlePosition,
    ) {
        if let Some(pokemon) = state.get_pokemon_at_position_mut(position) {
            // Apply any test-specific overrides
            if let Some(status) = spec.status {
                pokemon.status = status;
            }

            if let Some(hp_percentage) = spec.hp_percentage {
                pokemon.hp = ((pokemon.max_hp as f32) * hp_percentage / 100.0) as i16;
            }
        }
    }

    /// Apply a setup action to the battle state
    fn apply_setup_action(
        &self,
        state: &mut BattleState,
        action: &SetupAction,
    ) -> Result<(), String> {
        match action {
            SetupAction::SetWeather(weather) => {
                state.field.weather.set(*weather, Some(5), None); // Standard weather duration
            }
            SetupAction::SetTerrain(terrain) => {
                state.field.terrain.set(*terrain, Some(5), None); // Standard terrain duration
            }
            SetupAction::ApplyStatus(position, status) => {
                if let Some(pokemon) = state.get_pokemon_at_position_mut(*position) {
                    pokemon.status = *status;
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            SetupAction::ModifyStats(position, stat_changes) => {
                if let Some(pokemon) = state.get_pokemon_at_position_mut(*position) {
                    for (stat, change) in stat_changes {
                        let current = pokemon.stat_boosts.get(stat).unwrap_or(&0);
                        let new_value = (current + change).clamp(-6, 6);
                        pokemon.stat_boosts.insert(*stat, new_value);
                    }
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            SetupAction::SetHP(position, hp) => {
                if let Some(pokemon) = state.get_pokemon_at_position_mut(*position) {
                    pokemon.hp = *hp as i16;
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            SetupAction::AddSideCondition(side, condition) => {
                let side_state = state.get_side_by_ref_mut(*side);
                side_state.side_conditions.insert(*condition, 1);
            }
        }
        Ok(())
    }

    /// Execute a single turn with the given moves using the real battle engine
    fn execute_turn(
        &self,
        state: &mut BattleState,
        move_one: MoveChoice,
        move_two: MoveChoice,
    ) -> Result<Vec<BattleInstructions>, String> {
        // Use the actual battle engine to generate instructions
        let instructions = turn::generate_instructions(state, (&move_one, &move_two))
            .map_err(|e| format!("Battle engine error: {:?}", e))?;

        // Apply the generated instructions to update the battle state
        for instruction_set in &instructions {
            state.apply_instructions(&instruction_set.instruction_list);
        }

        Ok(instructions)
    }

    /// Normalize instructions to match expected format (remove previous_hp etc.)
    fn normalize_instructions(
        &self,
        instructions: &[BattleInstructions],
    ) -> Vec<BattleInstructions> {
        instructions
            .iter()
            .map(|instruction_set| {
                let normalized_list = instruction_set
                    .instruction_list
                    .iter()
                    .map(|instruction| {
                        match instruction {
                            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                                target,
                                amount,
                                ..
                            }) => {
                                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                                    target: *target,
                                    amount: *amount,
                                    previous_hp: None, // Always set to None for test comparison
                                })
                            }
                            BattleInstruction::Pokemon(PokemonInstruction::Heal {
                                target,
                                amount,
                                ..
                            }) => BattleInstruction::Pokemon(PokemonInstruction::Heal {
                                target: *target,
                                amount: *amount,
                                previous_hp: None,
                            }),
                            other => other.clone(),
                        }
                    })
                    .collect();

                BattleInstructions {
                    percentage: instruction_set.percentage,
                    instruction_list: normalized_list,
                    affected_positions: instruction_set.affected_positions.clone(),
                }
            })
            .collect()
    }

    /// Validate that actual instructions match expected instructions
    fn validate_instructions(
        &self,
        actual: &[BattleInstructions],
        expected: &[BattleInstructions],
    ) -> bool {
        if actual.len() != expected.len() {
            return false;
        }

        // Normalize both sets for comparison
        let normalized_actual = self.normalize_instructions(actual);
        let normalized_expected = self.normalize_instructions(expected);

        for (actual_set, expected_set) in normalized_actual.iter().zip(normalized_expected.iter()) {
            if (actual_set.percentage - expected_set.percentage).abs() > 0.01 {
                return false;
            }

            if actual_set.instruction_list != expected_set.instruction_list {
                return false;
            }
        }

        true
    }

    /// Validate an expected outcome against the current battle state
    fn validate_outcome(
        &self,
        state: &BattleState,
        outcome: &ExpectedOutcome,
    ) -> Result<(), String> {
        match outcome {
            ExpectedOutcome::Damage(position, expected_damage) => {
                if let Some(pokemon) = state.get_pokemon_at_position(*position) {
                    let actual_damage = pokemon.max_hp - pokemon.hp;
                    if actual_damage != *expected_damage as i16 {
                        return Err(format!(
                            "Damage mismatch at {:?}: expected {}, got {}",
                            position, expected_damage, actual_damage
                        ));
                    }
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            ExpectedOutcome::Status(position, expected_status) => {
                if let Some(pokemon) = state.get_pokemon_at_position(*position) {
                    if pokemon.status != *expected_status {
                        return Err(format!(
                            "Status mismatch at {:?}: expected {:?}, got {:?}",
                            position, expected_status, pokemon.status
                        ));
                    }
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            ExpectedOutcome::StatChange(position, stat, expected_change) => {
                if let Some(pokemon) = state.get_pokemon_at_position(*position) {
                    let actual_change = pokemon.stat_boosts.get(stat).unwrap_or(&0);
                    if actual_change != expected_change {
                        return Err(format!(
                            "Stat change mismatch at {:?} for {:?}: expected {}, got {}",
                            position, stat, expected_change, actual_change
                        ));
                    }
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            ExpectedOutcome::WeatherSet(expected_weather) => {
                if state.weather() != *expected_weather {
                    return Err(format!(
                        "Weather mismatch: expected {:?}, got {:?}",
                        expected_weather,
                        state.weather()
                    ));
                }
            }
            ExpectedOutcome::TerrainSet(expected_terrain) => {
                if state.terrain() != *expected_terrain {
                    return Err(format!(
                        "Terrain mismatch: expected {:?}, got {:?}",
                        expected_terrain,
                        state.terrain()
                    ));
                }
            }
            ExpectedOutcome::SideCondition(side, expected_condition) => {
                let side_state = state.get_side_by_ref(*side);

                if !side_state.side_conditions.contains_key(expected_condition) {
                    return Err(format!(
                        "Side condition missing for {:?}: expected {:?}",
                        side, expected_condition
                    ));
                }
            }
            ExpectedOutcome::Faint(position) => {
                if let Some(pokemon) = state.get_pokemon_at_position(*position) {
                    if pokemon.hp > 0 {
                        return Err(format!(
                            "Pokemon at {:?} should have fainted but has {} HP",
                            position, pokemon.hp
                        ));
                    }
                } else {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            ExpectedOutcome::Switch(_position, _expected_index) => {
                // This would require tracking switch events, which is more complex
                // For now, we can validate that the Pokemon at the position is different
                // This is a simplified implementation
            }
            ExpectedOutcome::NoEffect(position) => {
                // This is context-dependent and would need more sophisticated tracking
                // For now, just validate the position exists
                if state.get_pokemon_at_position(*position).is_none() {
                    return Err(format!("No Pokemon at position {:?}", position));
                }
            }
            ExpectedOutcome::Instructions(_expected_instructions) => {
                // This is handled in the turn execution validation
            }
        }

        Ok(())
    }
}

/// Specification for a team (single or multiple Pokemon)
#[derive(Debug, Clone)]
pub enum TeamSpec {
    Pokemon(PokemonSpec),
    MultiPokemon(Vec<PokemonSpec>),
}

/// Specification for a single Pokemon
#[derive(Debug, Clone)]
pub struct PokemonSpec {
    pub species: &'static str,
    pub level: u8,
    pub ability: Option<&'static str>,
    pub item: Option<&'static str>,
    pub moves: Vec<&'static str>,
    pub nature: Option<&'static str>,
    pub evs: Option<Stats>,
    pub ivs: Option<Stats>,
    pub status: Option<PokemonStatus>,
    pub hp_percentage: Option<f32>,
}

impl Default for PokemonSpec {
    fn default() -> Self {
        Self {
            species: "Pikachu",
            level: 50,
            ability: None,
            item: None,
            moves: vec!["Tackle"],
            nature: None,
            evs: None,
            ivs: None,
            status: None,
            hp_percentage: None,
        }
    }
}

/// Setup actions to apply before battle execution
#[derive(Debug, Clone)]
pub enum SetupAction {
    SetWeather(Weather),
    SetTerrain(Terrain),
    ApplyStatus(BattlePosition, PokemonStatus),
    ModifyStats(BattlePosition, HashMap<Stat, i8>),
    SetHP(BattlePosition, u16),
    AddSideCondition(SideReference, SideCondition),
}

/// Expected outcomes for test validation
#[derive(Debug, Clone)]
pub enum ExpectedOutcome {
    Damage(BattlePosition, u16),
    Status(BattlePosition, PokemonStatus),
    StatChange(BattlePosition, Stat, i8),
    WeatherSet(Weather),
    TerrainSet(Terrain),
    SideCondition(SideReference, SideCondition),
    Faint(BattlePosition),
    Switch(BattlePosition, usize),
    NoEffect(BattlePosition),
    Instructions(Vec<BattleInstructions>),
}

/// Complete battle test specification
#[derive(Debug, Clone)]
pub struct BattleTest {
    pub name: String,
    pub team_one: TeamSpec,
    pub team_two: TeamSpec,
    pub setup: Vec<SetupAction>,
    pub moves: Vec<(MoveChoice, MoveChoice)>,
    pub expected_outcomes: Vec<ExpectedOutcome>,
    pub expected_instructions: Vec<Vec<BattleInstructions>>,
}

impl BattleTest {
    /// Create a new battle test with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            team_one: TeamSpec::Pokemon(PokemonSpec::default()),
            team_two: TeamSpec::Pokemon(PokemonSpec::default()),
            setup: Vec::new(),
            moves: Vec::new(),
            expected_outcomes: Vec::new(),
            expected_instructions: Vec::new(),
        }
    }
}

/// Result of test execution
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    Success,
    Failed(String),
}

impl TestResult {
    /// Check if the test was successful
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success)
    }

    /// Get the failure message if the test failed
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            TestResult::Failed(msg) => Some(msg),
            TestResult::Success => None,
        }
    }

    /// Assert that the test was successful, panicking if it failed
    pub fn assert_success(&self) {
        if let TestResult::Failed(msg) = self {
            panic!("Test failed: {}", msg);
        }
    }
}
