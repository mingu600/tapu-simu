//! # Test Builder API
//!
//! This module provides fluent builder APIs for creating battle tests
//! in a readable and maintainable way.

use super::framework::{
    BattleTest, ExpectedOutcome, PokemonSpec, SetupAction, TapuTestFramework, TeamSpec, TestResult,
};
use std::collections::HashMap;
use tapu_simu::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use tapu_simu::core::instructions::{
    BattleInstructions, PokemonStatus, SideCondition, Stat, Terrain, VolatileStatus, Weather,
};
use tapu_simu::core::move_choice::{MoveChoice, MoveIndex};
use tapu_simu::data::types::BaseStats;
use tapu_simu::generation::Generation;
use tapu_simu::types::DataResult;

/// Fluent builder for creating battle tests
pub struct TestBuilder {
    framework: TapuTestFramework,
    pub test: BattleTest,
}

impl TestBuilder {
    /// Create a new test builder with default Gen 9 Singles format
    pub fn new(name: &str) -> DataResult<Self> {
        let framework = TapuTestFramework::new()?;
        let test = BattleTest::new(name.to_string());

        Ok(Self { framework, test })
    }

    /// Create a test builder for a specific generation
    pub fn new_with_generation(name: &str, gen: Generation) -> DataResult<Self> {
        let framework = TapuTestFramework::with_generation(gen)?;
        let test = BattleTest::new(name.to_string());

        Ok(Self { framework, test })
    }

    /// Create a test builder with a specific format
    pub fn new_with_format(name: &str, format: BattleFormat) -> DataResult<Self> {
        let framework = TapuTestFramework::with_format(format)?;
        let test = BattleTest::new(name.to_string());

        Ok(Self { framework, test })
    }

    /// Set the first team
    pub fn team_one(mut self, spec: PokemonSpec) -> Self {
        self.test.team_one = TeamSpec::Pokemon(spec);
        self
    }

    /// Set the first team with multiple Pokemon
    pub fn team_one_multi(mut self, specs: Vec<PokemonSpec>) -> Self {
        self.test.team_one = TeamSpec::MultiPokemon(specs);
        self
    }

    /// Set the second team
    pub fn team_two(mut self, spec: PokemonSpec) -> Self {
        self.test.team_two = TeamSpec::Pokemon(spec);
        self
    }

    /// Set the second team with multiple Pokemon
    pub fn team_two_multi(mut self, specs: Vec<PokemonSpec>) -> Self {
        self.test.team_two = TeamSpec::MultiPokemon(specs);
        self
    }

    /// Add a setup action
    pub fn setup(mut self, action: SetupAction) -> Self {
        self.test.setup.push(action);
        self
    }

    /// Set the weather before battle
    pub fn with_weather(self, weather: Weather) -> Self {
        self.setup(SetupAction::SetWeather(weather))
    }

    /// Set the terrain before battle
    pub fn with_terrain(self, terrain: Terrain) -> Self {
        self.setup(SetupAction::SetTerrain(terrain))
    }

    /// Apply a status condition to a Pokemon
    pub fn with_status(self, position: BattlePosition, status: PokemonStatus) -> Self {
        self.setup(SetupAction::ApplyStatus(position, status))
    }

    /// Modify stats for a Pokemon
    pub fn with_stat_changes(self, position: BattlePosition, changes: HashMap<Stat, i8>) -> Self {
        self.setup(SetupAction::ModifyStats(position, changes))
    }

    /// Set HP for a Pokemon
    pub fn with_hp(self, position: BattlePosition, hp: u16) -> Self {
        self.setup(SetupAction::SetHP(position, hp))
    }

    /// Add a side condition
    pub fn with_side_condition(self, side: SideReference, condition: SideCondition) -> Self {
        self.setup(SetupAction::AddSideCondition(side, condition))
    }

    /// Add a substitute with specific health for a Pokemon
    pub fn with_substitute(self, position: BattlePosition, health: i16) -> Self {
        self.setup(SetupAction::AddSubstitute(position, health))
    }

    /// Add a turn with move choices
    pub fn turn(mut self, move_one: MoveChoice, move_two: MoveChoice) -> Self {
        self.test.moves.push((move_one, move_two));
        self
    }

    /// Add a turn with move names (auto-resolves targeting)
    pub fn turn_with_moves(self, move_one: &str, move_two: &str) -> Self {
        let choice_one = self.create_move_choice(move_one, SideReference::SideOne);
        let choice_two = self.create_move_choice(move_two, SideReference::SideTwo);
        self.turn(choice_one, choice_two)
    }

    /// Add a turn where side one uses a move and side two does nothing
    pub fn turn_one_move(self, move_one: &str) -> Self {
        let choice_one = self.create_move_choice(move_one, SideReference::SideOne);
        self.turn(choice_one, MoveChoice::None)
    }

    /// Add a turn where side two uses a move and side one does nothing
    pub fn turn_two_move(self, move_two: &str) -> Self {
        let choice_two = self.create_move_choice(move_two, SideReference::SideTwo);
        self.turn(MoveChoice::None, choice_two)
    }

    /// Add a turn where both sides use moves (convenient string API)
    pub fn turn_moves(self, move_one: &str, move_two: &str) -> Self {
        let choice_one = self.create_move_choice(move_one, SideReference::SideOne);
        let choice_two = self.create_move_choice(move_two, SideReference::SideTwo);
        self.turn(choice_one, choice_two)
    }

    /// Add an expected outcome
    pub fn expect(mut self, outcome: ExpectedOutcome) -> Self {
        self.test.expected_outcomes.push(outcome);
        self
    }

    /// Expect damage to a specific position
    pub fn expect_damage(self, position: BattlePosition, damage: u16) -> Self {
        self.expect(ExpectedOutcome::Damage(position, damage))
    }

    /// Expect a status condition on a Pokemon
    pub fn expect_status(self, position: BattlePosition, status: PokemonStatus) -> Self {
        self.expect(ExpectedOutcome::Status(position, status))
    }

    /// Expect a stat change on a Pokemon
    pub fn expect_stat_change(self, position: BattlePosition, stat: Stat, change: i8) -> Self {
        self.expect(ExpectedOutcome::StatChange(position, stat, change))
    }

    /// Expect weather to be set
    pub fn expect_weather(self, weather: Weather) -> Self {
        self.expect(ExpectedOutcome::WeatherSet(weather))
    }

    /// Expect terrain to be set
    pub fn expect_terrain(self, terrain: Terrain) -> Self {
        self.expect(ExpectedOutcome::TerrainSet(terrain))
    }

    /// Expect a side condition
    pub fn expect_side_condition(self, side: SideReference, condition: SideCondition) -> Self {
        self.expect(ExpectedOutcome::SideCondition(side, condition))
    }

    /// Expect a Pokemon to faint
    pub fn expect_faint(self, position: BattlePosition) -> Self {
        self.expect(ExpectedOutcome::Faint(position))
    }

    /// Expect no effect at a position
    pub fn expect_no_effect(self, position: BattlePosition) -> Self {
        self.expect(ExpectedOutcome::NoEffect(position))
    }

    /// Expect substitute health at a position
    pub fn expect_substitute_health(self, position: BattlePosition, health: i16) -> Self {
        self.expect(ExpectedOutcome::SubstituteHealth(position, health))
    }

    /// Expect a volatile status at a position
    pub fn expect_volatile_status(self, position: BattlePosition, status: VolatileStatus) -> Self {
        self.expect(ExpectedOutcome::VolatileStatus(position, status))
    }

    /// Expect specific instructions
    pub fn expect_instructions(mut self, instructions: Vec<BattleInstructions>) -> Self {
        self.test.expected_instructions.push(instructions);
        self
    }

    /// Set whether to branch on damage (critical hits, damage rolls)
    pub fn branch_on_damage(mut self, branch: bool) -> Self {
        self.test.branch_on_damage = branch;
        self
    }

    /// Execute the test and return the result
    pub fn run(self) -> TestResult {
        self.framework.execute_test(self.test)
    }

    /// Execute the test and assert it succeeds
    pub fn assert_success(self) {
        let result = self.run();
        result.assert_success();
    }

    /// Create a move choice from move name and side
    fn create_move_choice(&self, move_name: &str, side: SideReference) -> MoveChoice {
        // For test builder, we need to determine the move index based on the team spec
        let move_index = self.find_move_index_in_team(move_name, side);

        // Determine target based on the opposing side
        let target_position = match side {
            SideReference::SideOne => BattlePosition::new(SideReference::SideTwo, 0),
            SideReference::SideTwo => BattlePosition::new(SideReference::SideOne, 0),
        };

        MoveChoice::Move {
            move_index,
            target_positions: vec![target_position],
        }
    }

    /// Find the move index in a team's first Pokemon
    fn find_move_index_in_team(&self, move_name: &str, side: SideReference) -> MoveIndex {
        let team_spec = match side {
            SideReference::SideOne => &self.test.team_one,
            SideReference::SideTwo => &self.test.team_two,
        };

        let default_spec = PokemonSpec::default();
        let pokemon_spec = match team_spec {
            TeamSpec::Pokemon(spec) => spec,
            TeamSpec::MultiPokemon(specs) => specs.first().unwrap_or(&default_spec),
        };

        // Find the move in the Pokemon's move list
        pokemon_spec
            .moves
            .iter()
            .position(|&mv| mv.to_lowercase() == move_name.to_lowercase())
            .map(|pos| match pos {
                0 => MoveIndex::M0,
                1 => MoveIndex::M1,
                2 => MoveIndex::M2,
                3 => MoveIndex::M3,
                _ => MoveIndex::M0,
            })
            .unwrap_or(MoveIndex::M0) // Default to first move if not found
    }
}

/// Builder for Pokemon specifications
impl PokemonSpec {
    /// Create a new Pokemon spec with the given species
    pub fn new(species: &'static str) -> Self {
        Self {
            species,
            level: 100,
            ability: None,
            item: None,
            moves: vec!["Tackle"],
            nature: None,
            evs: None,
            ivs: None,
            status: None,
            hp: None,
        }
    }

    /// Set the Pokemon's level
    pub fn level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Set the Pokemon's ability
    pub fn ability(mut self, ability: &'static str) -> Self {
        self.ability = Some(ability);
        self
    }

    /// Set the Pokemon's held item
    pub fn item(mut self, item: &'static str) -> Self {
        self.item = Some(item);
        self
    }

    /// Set the Pokemon's moves
    pub fn moves(mut self, moves: Vec<&'static str>) -> Self {
        self.moves = moves;
        self
    }

    /// Add a single move to the Pokemon's moveset
    pub fn move_slot(mut self, move_name: &'static str) -> Self {
        if self.moves.len() < 4 {
            self.moves.push(move_name);
        }
        self
    }

    /// Set the Pokemon's nature
    pub fn nature(mut self, nature: &'static str) -> Self {
        self.nature = Some(nature);
        self
    }

    /// Set the Pokemon's EVs
    pub fn evs(mut self, evs: BaseStats) -> Self {
        self.evs = Some(evs);
        self
    }

    /// Set specific EV values
    pub fn ev_spread(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        self.evs = Some(BaseStats {
            hp: hp as i16,
            attack: atk as i16,
            defense: def as i16,
            special_attack: spa as i16,
            special_defense: spd as i16,
            speed: spe as i16,
        });
        self
    }

    /// Set the Pokemon's IVs
    pub fn ivs(mut self, ivs: BaseStats) -> Self {
        self.ivs = Some(ivs);
        self
    }

    /// Set perfect IVs (31 in all stats)
    pub fn perfect_ivs(mut self) -> Self {
        self.ivs = Some(BaseStats {
            hp: 31,
            attack: 31,
            defense: 31,
            special_attack: 31,
            special_defense: 31,
            speed: 31,
        });
        self
    }

    /// Set the Pokemon's status condition
    pub fn status(mut self, status: PokemonStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the Pokemon's HP to a specific raw value
    pub fn hp(mut self, hp: u16) -> Self {
        self.hp = Some(hp);
        self
    }

    /// Set all base stats to the same value (for dummy Pokemon)
    pub fn base_stats_all(mut self, stat_value: u8) -> Self {
        self.evs = Some(BaseStats {
            hp: stat_value as i16,
            attack: stat_value as i16,
            defense: stat_value as i16,
            special_attack: stat_value as i16,
            special_defense: stat_value as i16,
            speed: stat_value as i16,
        });
        self
    }

    /// Set custom types (this will be applied after Pokemon creation)
    pub fn types(mut self, type1: &'static str, type2: Option<&'static str>) -> Self {
        // We'll store this in a custom field and apply it later
        // For now, we'll modify the species name to indicate types
        if let Some(t2) = type2 {
            self.species = Box::leak(format!("{}|{}|{}", self.species, type1, t2).into_boxed_str());
        } else {
            self.species = Box::leak(format!("{}|{}", self.species, type1).into_boxed_str());
        }
        self
    }

    /// Set custom weight
    pub fn weight(mut self, _weight_kg: f32) -> Self {
        // We'll handle this in the framework when creating the Pokemon
        // For now, store in species name
        let current = self.species;
        self.species = Box::leak(format!("{}|weight={}", current, _weight_kg).into_boxed_str());
        self
    }

    /// Set max HP directly
    pub fn max_hp(mut self, max_hp: i16) -> Self {
        // Store in EVs for now, we'll handle this specially
        let mut evs = self.evs.unwrap_or(BaseStats {
            hp: 85,
            attack: 85,
            defense: 85,
            special_attack: 85,
            special_defense: 85,
            speed: 85,
        });
        evs.hp = max_hp; // We'll interpret this as max HP in framework
        self.evs = Some(evs);
        self
    }
}

/// Helper functions for creating common test scenarios
pub struct TestScenarios;

impl TestScenarios {
    /// Create a basic damage test scenario
    pub fn damage_test(
        attacker_species: &'static str,
        attacker_move: &'static str,
        defender_species: &'static str,
        expected_damage: u16,
    ) -> DataResult<TestBuilder> {
        Ok(TestBuilder::new("damage test")?
            .team_one(PokemonSpec::new(attacker_species).move_slot(attacker_move))
            .team_two(PokemonSpec::new(defender_species))
            .turn_with_moves(attacker_move, "Tackle")
            .expect_damage(
                BattlePosition::new(SideReference::SideTwo, 0),
                expected_damage,
            ))
    }

    /// Create a status effect test scenario
    pub fn status_test(
        attacker_species: &'static str,
        status_move: &'static str,
        defender_species: &'static str,
        expected_status: PokemonStatus,
    ) -> DataResult<TestBuilder> {
        Ok(TestBuilder::new("status test")?
            .team_one(PokemonSpec::new(attacker_species).move_slot(status_move))
            .team_two(PokemonSpec::new(defender_species))
            .turn_with_moves(status_move, "Tackle")
            .expect_status(
                BattlePosition::new(SideReference::SideTwo, 0),
                expected_status,
            ))
    }

    /// Create an ability test scenario
    pub fn ability_test(
        pokemon_species: &'static str,
        ability: &'static str,
        opponent_species: &'static str,
        opponent_move: &'static str,
    ) -> DataResult<TestBuilder> {
        Ok(TestBuilder::new("ability test")?
            .team_one(PokemonSpec::new(pokemon_species).ability(ability))
            .team_two(PokemonSpec::new(opponent_species).move_slot(opponent_move))
            .turn_with_moves("Tackle", opponent_move))
    }

    /// Create an item test scenario
    pub fn item_test(
        pokemon_species: &'static str,
        item: &'static str,
        opponent_species: &'static str,
        opponent_move: &'static str,
    ) -> DataResult<TestBuilder> {
        Ok(TestBuilder::new("item test")?
            .team_one(PokemonSpec::new(pokemon_species).item(item))
            .team_two(PokemonSpec::new(opponent_species).move_slot(opponent_move))
            .turn_with_moves("Tackle", opponent_move))
    }
}

/// Convenience macros for common test patterns
#[macro_export]
macro_rules! damage_test {
    ($attacker:expr, $move:expr, $defender:expr, $damage:expr) => {
        TestBuilder::new("damage test")?
            .team_one(PokemonSpec::new($attacker).move_slot($move))
            .team_two(PokemonSpec::new($defender))
            .turn_with_moves($move, "Tackle")
            .expect_damage(BattlePosition::new(SideReference::SideTwo, 0), $damage)
    };
}

#[macro_export]
macro_rules! status_test {
    ($attacker:expr, $move:expr, $defender:expr, $status:expr) => {
        TestBuilder::new("status test")?
            .team_one(PokemonSpec::new($attacker).move_slot($move))
            .team_two(PokemonSpec::new($defender))
            .turn_with_moves($move, "Tackle")
            .expect_status(BattlePosition::new(SideReference::SideTwo, 0), $status)
    };
}

#[macro_export]
macro_rules! ability_test {
    ($pokemon:expr, $ability:expr, $opponent:expr, $move:expr) => {
        TestBuilder::new("ability test")?
            .team_one(PokemonSpec::new($pokemon).ability($ability))
            .team_two(PokemonSpec::new($opponent).move_slot($move))
            .turn_with_moves("Tackle", $move)
    };
}

/// Helper for creating common battle positions
pub struct Positions;

impl Positions {
    /// Side one, slot 0 (first Pokemon on side one)
    pub const SIDE_ONE_0: BattlePosition = BattlePosition {
        side: SideReference::SideOne,
        slot: 0,
    };

    /// Side one, slot 1 (second Pokemon on side one, for doubles)
    pub const SIDE_ONE_1: BattlePosition = BattlePosition {
        side: SideReference::SideOne,
        slot: 1,
    };

    /// Side two, slot 0 (first Pokemon on side two)
    pub const SIDE_TWO_0: BattlePosition = BattlePosition {
        side: SideReference::SideTwo,
        slot: 0,
    };

    /// Side two, slot 1 (second Pokemon on side two, for doubles)
    pub const SIDE_TWO_1: BattlePosition = BattlePosition {
        side: SideReference::SideTwo,
        slot: 1,
    };
}

/// Helper for creating common stat change maps
pub struct StatChanges;

impl StatChanges {
    /// Create a single stat change
    pub fn single(stat: Stat, change: i8) -> HashMap<Stat, i8> {
        let mut map = HashMap::new();
        map.insert(stat, change);
        map
    }

    /// Create multiple stat changes
    pub fn multiple(changes: &[(Stat, i8)]) -> HashMap<Stat, i8> {
        changes.iter().cloned().collect()
    }

    /// Attack boost
    pub fn attack_boost(change: i8) -> HashMap<Stat, i8> {
        Self::single(Stat::Attack, change)
    }

    /// Defense boost
    pub fn defense_boost(change: i8) -> HashMap<Stat, i8> {
        Self::single(Stat::Defense, change)
    }

    /// Speed boost
    pub fn speed_boost(change: i8) -> HashMap<Stat, i8> {
        Self::single(Stat::Speed, change)
    }
}
