# Testing Module Documentation

The testing module provides a comprehensive framework for verifying battle mechanics, move effects, and system integration in Tapu Simu. It offers precise assertion capabilities, battle simulation utilities, and generation-aware testing patterns.

## Architecture Overview

The testing framework consists of four main components:
- **Test Framework**: Central battle simulation and execution engine
- **Test Builders**: Fluent API for constructing battle scenarios
- **Assertion System**: Precise verification of battle outcomes and state
- **Test Utilities**: Helper functions and data management for tests

## Test Framework (`testing.rs`)

Core testing framework that integrates with the battle engine to provide controlled testing environments.

### Framework Structure

**TapuTestFramework:**
```rust
pub struct TestFramework {
    data_repository: Arc<GameDataRepository>,
    generation_repository: Arc<GenerationRepository>,
}

impl TestFramework {
    pub fn new() -> Self {
        Self {
            data_repository: Arc::new(GameDataRepository::new().unwrap()),
            generation_repository: Arc::new(GenerationRepository::new().unwrap()),
        }
    }

    pub fn test_battle_scenario(&self, scenario: BattleScenario) -> TestResult {
        // Executes complete battle scenarios with controlled conditions
    }
}
```

**Key Features:**
- Integrates with production `GameDataRepository` for authentic data
- Supports all generations (Gen 1-9) with accurate mechanics
- Provides controlled battle execution with precise state tracking
- Enables complex scenario testing with multiple turns and conditions

### Contact Status Results

**ContactStatusResult System:**
```rust
pub struct ContactStatusResult {
    pub damage_dealt: u16,
    pub status_applied: Option<String>,
    pub volatile_status_applied: Vec<String>,
    pub stat_changes: HashMap<String, i8>,
    pub secondary_effects: Vec<SecondaryEffect>,
}
```

This system captures comprehensive battle interaction results for detailed verification.

## Test Builders (`tests/utils/builders.rs`)

Fluent API builder pattern for constructing complex battle scenarios with ease.

### TestBuilder System

**Core Builder Pattern:**
```rust
pub struct TestBuilder {
    name: String,
    generation: Generation,
    format: BattleFormat,
    side_one: Vec<PokemonSpec>,
    side_two: Vec<PokemonSpec>,
    setup_actions: Vec<SetupAction>,
    moves: Vec<MoveAction>,
    expectations: Vec<Expectation>,
}

impl TestBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            generation: Generation::Gen9,
            format: BattleFormat::singles(),
            side_one: Vec::new(),
            side_two: Vec::new(),
            setup_actions: Vec::new(),
            moves: Vec::new(),
            expectations: Vec::new(),
        }
    }
}
```

### Fluent API Methods

**Team and Pokemon Setup:**
```rust
impl TestBuilder {
    pub fn generation(mut self, gen: Generation) -> Self {
        self.generation = gen;
        self
    }

    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = format;
        self
    }

    pub fn side_one_pokemon(mut self, spec: PokemonSpec) -> Self {
        self.side_one.push(spec);
        self
    }

    pub fn side_two_pokemon(mut self, spec: PokemonSpec) -> Self {
        self.side_two.push(spec);
        self
    }
}
```

**Battle Actions:**
```rust
impl TestBuilder {
    pub fn use_move(mut self, user: BattlePosition, move_name: &str) -> Self {
        self.moves.push(MoveAction {
            user,
            move_name: move_name.to_string(),
            targets: vec![], // Auto-resolved
        });
        self
    }

    pub fn use_move_targeting(mut self, user: BattlePosition, move_name: &str, targets: Vec<BattlePosition>) -> Self {
        self.moves.push(MoveAction {
            user,
            move_name: move_name.to_string(),
            targets,
        });
        self
    }
}
```

**Expectations and Assertions:**
```rust
impl TestBuilder {
    pub fn expect_damage(mut self, target: BattlePosition, amount: u16) -> Self {
        self.expectations.push(Expectation::Damage { target, amount });
        self
    }

    pub fn expect_status(mut self, target: BattlePosition, status: &str) -> Self {
        self.expectations.push(Expectation::Status { 
            target, 
            status: status.to_string() 
        });
        self
    }

    pub fn expect_stat_change(mut self, target: BattlePosition, stat: &str, change: i8) -> Self {
        self.expectations.push(Expectation::StatChange { 
            target, 
            stat: stat.to_string(), 
            change 
        });
        self
    }

    pub fn expect_instructions(mut self, instructions: Vec<BattleInstructions>) -> Self {
        self.expectations.push(Expectation::Instructions { instructions });
        self
    }
}
```

### PokemonSpec Builder

**Pokemon Specification System:**
```rust
pub struct PokemonSpec {
    pub species: String,
    pub level: u8,
    pub ability: Option<String>,
    pub item: Option<String>,
    pub moves: Vec<String>,
    pub nature: Option<Nature>,
    pub evs: Option<StatSpread>,
    pub ivs: Option<StatSpread>,
    pub status: Option<String>,
    pub hp_percentage: Option<f32>,
    pub stat_boosts: HashMap<String, i8>,
}

impl PokemonSpec {
    pub fn new(species: &str) -> Self {
        Self {
            species: species.to_string(),
            level: 50,
            ability: None,
            item: None,
            moves: Vec::new(),
            nature: None,
            evs: None,
            ivs: None,
            status: None,
            hp_percentage: None,
            stat_boosts: HashMap::new(),
        }
    }

    pub fn level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    pub fn ability(mut self, ability: &str) -> Self {
        self.ability = Some(ability.to_string());
        self
    }

    pub fn moves(mut self, moves: Vec<&str>) -> Self {
        self.moves = moves.into_iter().map(String::from).collect();
        self
    }

    pub fn max_evs(mut self, stat: &str) -> Self {
        let mut evs = self.evs.unwrap_or_default();
        evs.set_stat(stat, 252);
        self.evs = Some(evs);
        self
    }
}
```

## Assertion System (`tests/utils/assertions.rs`)

Comprehensive assertion capabilities for verifying battle outcomes and state changes.

### Battle State Assertions

**Damage and HP Verification:**
```rust
pub fn assert_hp(state: &BattleState, position: BattlePosition, expected_hp: u16) {
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    assert_eq!(
        pokemon.current_hp, 
        expected_hp,
        "Pokemon at {} has {} HP, expected {}",
        position, pokemon.current_hp, expected_hp
    );
}

pub fn assert_damage_dealt(
    previous_state: &BattleState, 
    current_state: &BattleState, 
    position: BattlePosition, 
    expected_damage: u16
) {
    let prev_hp = previous_state.get_pokemon_at_position(position).unwrap().current_hp;
    let curr_hp = current_state.get_pokemon_at_position(position).unwrap().current_hp;
    let actual_damage = prev_hp.saturating_sub(curr_hp);
    
    assert_eq!(
        actual_damage, 
        expected_damage,
        "Expected {} damage to {}, but dealt {}",
        expected_damage, position, actual_damage
    );
}
```

**Status Condition Verification:**
```rust
pub fn assert_status(state: &BattleState, position: BattlePosition, expected_status: &str) {
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    match &pokemon.status {
        Some(status) => assert_eq!(
            status, expected_status,
            "Pokemon at {} has status '{}', expected '{}'",
            position, status, expected_status
        ),
        None => panic!(
            "Pokemon at {} has no status, expected '{}'",
            position, expected_status
        ),
    }
}

pub fn assert_volatile_status(state: &BattleState, position: BattlePosition, expected_status: &str) {
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    assert!(
        pokemon.volatile_status.contains_key(expected_status),
        "Pokemon at {} does not have volatile status '{}'",
        position, expected_status
    );
}
```

**Stat Modification Verification:**
```rust
pub fn assert_stat_boost(
    state: &BattleState, 
    position: BattlePosition, 
    stat: &str, 
    expected_boost: i8
) {
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    let actual_boost = pokemon.stat_boosts.get(stat).copied().unwrap_or(0);
    assert_eq!(
        actual_boost, 
        expected_boost,
        "Pokemon at {} has {} boost of {}, expected {}",
        position, stat, actual_boost, expected_boost
    );
}
```

### Instruction Verification

**Exact Instruction Matching:**
```rust
pub fn assert_instructions_match(
    actual: &[BattleInstructions],
    expected: &[BattleInstructions]
) {
    assert_eq!(
        actual.len(), 
        expected.len(),
        "Instruction count mismatch: got {} expected {}",
        actual.len(), expected.len()
    );

    for (i, (actual_inst, expected_inst)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_instructions_equal(actual_inst, expected_inst, i);
    }
}

fn assert_instructions_equal(
    actual: &BattleInstructions,
    expected: &BattleInstructions,
    index: usize
) {
    assert_eq!(
        actual.percentage, 
        expected.percentage,
        "Instruction {} probability mismatch: {} vs {}",
        index, actual.percentage, expected.percentage
    );

    assert_eq!(
        actual.affected_positions, 
        expected.affected_positions,
        "Instruction {} affected positions mismatch",
        index
    );

    assert_eq!(
        normalize_instructions(&actual.instruction_list),
        normalize_instructions(&expected.instruction_list),
        "Instruction {} content mismatch",
        index
    );
}
```

### Field State Verification

**Weather and Terrain Checking:**
```rust
pub fn assert_weather(state: &BattleState, expected_weather: Option<&str>) {
    match (&state.field.weather, expected_weather) {
        (Some(weather), Some(expected)) => {
            assert_eq!(
                weather.weather_type, expected,
                "Weather is '{}', expected '{}'",
                weather.weather_type, expected
            );
        }
        (None, None) => {}, // Both none, correct
        (Some(weather), None) => {
            panic!("Unexpected weather '{}', expected no weather", weather.weather_type);
        }
        (None, Some(expected)) => {
            panic!("No weather active, expected '{}'", expected);
        }
    }
}

pub fn assert_terrain(state: &BattleState, expected_terrain: Option<&str>) {
    match (&state.field.terrain, expected_terrain) {
        (Some(terrain), Some(expected)) => {
            assert_eq!(
                terrain.terrain_type, expected,
                "Terrain is '{}', expected '{}'",
                terrain.terrain_type, expected
            );
        }
        (None, None) => {},
        (Some(terrain), None) => {
            panic!("Unexpected terrain '{}', expected no terrain", terrain.terrain_type);
        }
        (None, Some(expected)) => {
            panic!("No terrain active, expected '{}'", expected);
        }
    }
}
```

## Test Utilities (`tests/utils/`)

Helper functions and utilities for test construction and data management.

### Position Constants

**Pre-defined Battle Positions:**
```rust
pub mod Positions {
    pub const SIDE_ONE_0: BattlePosition = BattlePosition::new(SideReference::SideOne, 0);
    pub const SIDE_ONE_1: BattlePosition = BattlePosition::new(SideReference::SideOne, 1);
    pub const SIDE_TWO_0: BattlePosition = BattlePosition::new(SideReference::SideTwo, 0);
    pub const SIDE_TWO_1: BattlePosition = BattlePosition::new(SideReference::SideTwo, 1);
}
```

### Test Data Helpers

**Stat Spread Creation:**
```rust
pub fn create_stat_spread(hp: u16, atk: u16, def: u16, spa: u16, spd: u16, spe: u16) -> StatSpread {
    StatSpread { hp, atk, def, spa, spd, spe }
}

pub fn max_evs_spread() -> StatSpread {
    StatSpread { hp: 252, atk: 252, def: 4, spa: 0, spd: 0, spe: 0 }
}
```

**Battle Setup Utilities:**
```rust
pub fn create_singles_battle(
    side_one: Vec<PokemonSpec>,
    side_two: Vec<PokemonSpec>
) -> BattleState {
    let format = BattleFormat::singles();
    let team_one = build_team(side_one);
    let team_two = build_team(side_two);
    BattleState::new(format, team_one, team_two).unwrap()
}
```

## Testing Patterns

### Basic Test Structure

```rust
#[test]
fn test_basic_damage() {
    let result = TestBuilder::new("Basic Damage Test")
        .generation(Generation::Gen9)
        .side_one_pokemon(
            PokemonSpec::new("Pikachu")
                .level(50)
                .moves(vec!["Thunderbolt"])
        )
        .side_two_pokemon(
            PokemonSpec::new("Charizard")
                .level(50)
        )
        .use_move(Positions::SIDE_ONE_0, "Thunderbolt")
        .expect_damage(Positions::SIDE_TWO_0, 85)
        .run();

    assert!(result.passed(), "Test failed: {}", result.error_message());
}
```

### Complex Scenario Testing

```rust
#[test]
fn test_status_interaction() {
    let result = TestBuilder::new("Status Interaction Test")
        .generation(Generation::Gen9)
        .side_one_pokemon(
            PokemonSpec::new("Gengar")
                .ability("Cursed Body")
                .moves(vec!["Will-O-Wisp", "Hex"])
        )
        .side_two_pokemon(
            PokemonSpec::new("Machamp")
                .level(50)
        )
        .use_move(Positions::SIDE_ONE_0, "Will-O-Wisp")
        .expect_status(Positions::SIDE_TWO_0, "Burn")
        .use_move(Positions::SIDE_ONE_0, "Hex")
        .expect_damage(Positions::SIDE_TWO_0, 130) // Double power vs burned target
        .run();

    assert!(result.passed());
}
```

### Probability Testing

```rust
#[test]
fn test_critical_hit_branching() {
    let instructions = TestBuilder::new("Critical Hit Probability")
        .generation(Generation::Gen9)
        .side_one_pokemon(
            PokemonSpec::new("Mewtwo")
                .moves(vec!["Psychic"])
        )
        .side_two_pokemon(PokemonSpec::new("Alakazam"))
        .use_move(Positions::SIDE_ONE_0, "Psychic")
        .get_instructions();

    // Should have two branches: normal hit (95.83%) and critical hit (4.17%)
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].percentage, 95.83);
    assert_eq!(instructions[1].percentage, 4.17);
}
```