# Testing Framework Architecture

The testing framework provides comprehensive battle mechanics validation with generation-aware testing, precise instruction verification, and probabilistic outcome analysis. It serves as the validation layer ensuring mechanical accuracy across all Pokemon generations and battle formats.

## System Architecture

### Framework Components

```
tests/
├── utils/
│   ├── mod.rs              // Centralized test utilities export
│   ├── framework.rs        // Core test execution engine with data integration
│   ├── builders.rs         // Fluent API for battle scenario construction
│   └── assertions.rs       // Specialized battle state validation system
├── basic_damage.rs         // Critical hit mechanics and damage calculations (773 lines)
├── move_categories.rs      // Multi-hit, variable power, and special mechanics (1,536 lines)
├── accuracy_miss.rs        // Accuracy modifiers and miss interactions (643 lines)
└── secondary_effects.rs    // Status conditions and secondary mechanics (498 lines)
```

## Core Framework (`tests/utils/framework.rs`)

### TapuTestFramework

**Generation-Aware Test Engine**: Multi-generational battle execution with proper mechanic differentiation:

```rust
pub struct TapuTestFramework {
    repository: Arc<GameDataRepository>,           // Pokemon Showdown data integration
    generation_repository: Arc<GenerationRepository>,  // Generation-specific mechanics
    format: BattleFormat,                         // Battle format configuration
}
```

**Generation Support**: Complete mechanic coverage across all generations:
- **Gen 1-2**: DV-based stat calculations, unique critical hit mechanics
- **Gen 3+**: IV/EV stat systems, modern critical hit rates
- **Format Variants**: Singles, Doubles, VGC, Triples with proper position targeting

**Data Integration**: Real Pokemon Showdown data with authentic mechanics:
```rust
impl TapuTestFramework {
    pub fn with_generation(gen: Generation) -> DataResult<Self> {
        let format = match gen {
            Generation::Gen1 => BattleFormat::gen1_ou(),
            Generation::Gen9 => BattleFormat::gen9_ou(),
            // ... complete generation coverage
        };
    }
}
```

### Battle Test Specifications

**BattleTest Structure**: Comprehensive test definition with setup and validation:
```rust
pub struct BattleTest {
    pub name: String,
    pub team_one: TeamSpec,      // Team composition specification
    pub team_two: TeamSpec,      // Opponent team specification
    pub setup: Vec<SetupAction>, // Pre-battle state modifications
    pub moves: Vec<MoveChoice>,  // Move execution sequence
    pub expected: Vec<ExpectedOutcome>, // Validation requirements
}
```

**PokemonSpec System**: Flexible Pokemon configuration with generation-aware defaults:
```rust
pub struct PokemonSpec {
    pub species: PokemonName,
    pub level: u8,
    pub ability: Option<Abilities>,
    pub item: Option<Items>,
    pub moves: Vec<Moves>,
    pub nature: Option<Nature>,
    pub evs: Option<Stats>,      // Gen 3+ EV system
    pub ivs: Option<Stats>,      // Gen 3+ IV system  
    pub dvs: Option<DVs>,        // Gen 1-2 DV system
    pub status: Option<PokemonStatus>,
    pub hp: Option<u16>,         // Raw HP value for precise control
    pub stat_boosts: HashMap<Stat, i8>, // Stat stage modifications
}
```

## Builder API (`tests/utils/builders.rs`)

### Fluent Test Construction

**TestBuilder Pattern**: Readable test construction with method chaining:
```rust
pub struct TestBuilder {
    framework: TapuTestFramework,
    pub test: BattleTest,
}

impl TestBuilder {
    pub fn new(name: &str) -> DataResult<Self>
    pub fn new_with_generation(name: &str, gen: Generation) -> DataResult<Self>
    pub fn new_with_format(name: &str, format: BattleFormat) -> DataResult<Self>
}
```

**Team Configuration Methods**:
```rust
impl TestBuilder {
    pub fn team_one(mut self, spec: PokemonSpec) -> Self
    pub fn team_one_multi(mut self, specs: Vec<PokemonSpec>) -> Self
    pub fn team_two(mut self, spec: PokemonSpec) -> Self
    pub fn team_two_multi(mut self, specs: Vec<PokemonSpec>) -> Self
}
```

**Battle Setup Methods**:
```rust
impl TestBuilder {
    pub fn with_weather(self, weather: Weather) -> Self
    pub fn with_terrain(self, terrain: Terrain) -> Self
    pub fn with_status(self, position: BattlePosition, status: PokemonStatus) -> Self
    pub fn with_stat_changes(self, position: BattlePosition, changes: HashMap<Stat, i8>) -> Self
    pub fn with_hp(self, position: BattlePosition, hp: u16) -> Self
}
```

### Pokemon Specification Builder

**Comprehensive Pokemon Configuration**:
```rust
impl PokemonSpec {
    pub fn new(species: PokemonName) -> Self  // Default Level 50 configuration
    pub fn level(mut self, level: u8) -> Self
    pub fn ability(mut self, ability: Abilities) -> Self
    pub fn item(mut self, item: Items) -> Self
    pub fn moves(mut self, moves: Vec<Moves>) -> Self
    pub fn nature(mut self, nature: Nature) -> Self
    pub fn evs(mut self, evs: Stats) -> Self
    pub fn ivs(mut self, ivs: Stats) -> Self
    pub fn status(mut self, status: PokemonStatus) -> Self
    pub fn hp(mut self, hp: u16) -> Self
    pub fn stat_boost(mut self, stat: Stat, boost: i8) -> Self
}
```

**Convenience Builders**:
```rust
// Optimized stat spreads
pub fn max_attack_ev_spread() -> Stats
pub fn max_speed_ev_spread() -> Stats  
pub fn balanced_ev_spread() -> Stats

// Common configurations
pub fn physical_attacker(species: PokemonName) -> PokemonSpec
pub fn special_attacker(species: PokemonName) -> PokemonSpec
pub fn tank(species: PokemonName) -> PokemonSpec
```

## Assertion System (`tests/utils/assertions.rs`)

### Specialized Battle Validation

**BattleAssertions Structure**: Domain-specific assertion methods:
```rust
pub struct BattleAssertions;

impl BattleAssertions {
    pub fn assert_damage(
        state: &BattleState,
        position: BattlePosition,
        expected_damage: u16,
    ) -> Result<(), String>
    
    pub fn assert_status(
        state: &BattleState,
        position: BattlePosition,
        expected_status: PokemonStatus,
    ) -> Result<(), String>
    
    pub fn assert_stat_changes(
        state: &BattleState,
        position: BattlePosition,
        expected_changes: &HashMap<Stat, i8>,
    ) -> Result<(), String>
}
```

### Instruction Verification

**Exact Instruction Matching**: Precise battle engine output validation:
```rust
pub fn assert_instructions_match(
    actual: &[BattleInstructions],
    expected: &[BattleInstructions]
) -> Result<(), String> {
    // Validates instruction count, probabilities, affected positions, and content
    for (actual_inst, expected_inst) in actual.iter().zip(expected.iter()) {
        assert_probability_match(actual_inst.percentage, expected_inst.percentage)?;
        assert_positions_match(&actual_inst.affected_positions, &expected_inst.affected_positions)?;
        assert_instruction_content_match(&actual_inst.instruction_list, &expected_inst.instruction_list)?;
    }
    Ok(())
}
```

**Probability Validation**: Statistical accuracy verification for RNG-dependent mechanics:
```rust
pub fn assert_probability_within_tolerance(
    actual: f64,
    expected: f64,
    tolerance: f64
) -> Result<(), String> {
    let diff = (actual - expected).abs();
    if diff > tolerance {
        return Err(format!(
            "Probability mismatch: expected {:.4}%, got {:.4}% (tolerance: {:.4}%)",
            expected, actual, tolerance
        ));
    }
    Ok(())
}
```

### Field State Validation

**Environmental Condition Verification**:
```rust
impl BattleAssertions {
    pub fn assert_weather(
        state: &BattleState,
        expected_weather: Option<Weather>
    ) -> Result<(), String>
    
    pub fn assert_terrain(
        state: &BattleState,
        expected_terrain: Option<Terrain>
    ) -> Result<(), String>
    
    pub fn assert_side_condition(
        state: &BattleState,
        side: SideReference,
        condition: SideCondition
    ) -> Result<(), String>
}
```

## Test Categories and Coverage

### Damage Calculation Tests (`basic_damage.rs`)

**Critical Hit Mechanics**: Generation-specific critical hit probability and damage:
```rust
#[test]
fn test_gen9_base_critical_hit_probability() {
    let instructions = TestBuilder::new("Gen 9 Critical Hit Base Rate")
        .new_with_generation(Generation::Gen9)
        .team_one(PokemonSpec::new(PokemonName::Pikachu).moves(vec![Moves::Tackle]))
        .team_two(PokemonSpec::new(PokemonName::Charmander))
        .use_move(Positions::SIDE_ONE_0, Moves::Tackle)
        .get_instructions();
        
    // Gen 9: 4.17% base critical hit rate (1/24)
    assert_eq!(instructions.len(), 2);
    assert_probability_within_tolerance(instructions[0].percentage, 95.83, 0.01);
    assert_probability_within_tolerance(instructions[1].percentage, 4.17, 0.01);
}
```

**Generation Differentiation**: Proper mechanics across all generations:
- **Gen 1**: 6.25% base crit rate, different damage formula
- **Gen 2**: 6.25% base crit rate, speed-based high crit moves  
- **Gen 3-6**: 4.17% base crit rate, standard mechanics
- **Gen 7-9**: 4.17% base crit rate, Z-move interactions

### Multi-Hit and Variable Power Tests (`move_categories.rs`)

**Multi-Hit Move Verification**: Hit-by-hit instruction tracking:
```rust
#[test]
fn test_double_kick_two_hits_exact_instructions() {
    let instructions = TestBuilder::new("Double Kick Two Hits")
        .team_one(PokemonSpec::new(PokemonName::Hitmonlee).moves(vec![Moves::DoubleKick]))
        .team_two(PokemonSpec::new(PokemonName::Machamp))
        .use_move(Positions::SIDE_ONE_0, Moves::DoubleKick)
        .get_instructions();
        
    // Verify two separate damage instructions
    let damage_instructions: Vec<_> = instructions[0].instruction_list.iter()
        .filter_map(|inst| match inst {
            BattleInstruction::Pokemon(PokemonInstruction::Damage { amount, .. }) => Some(*amount),
            _ => None,
        }).collect();
        
    assert_eq!(damage_instructions.len(), 2);
    assert_eq!(damage_instructions[0], 45); // First hit
    assert_eq!(damage_instructions[1], 45); // Second hit
}
```

**Variable Power Mechanics**: Context-dependent damage calculations:
- **Weight-based**: Heavy Slam, Heat Crash power calculations
- **HP-based**: Eruption, Water Spout power scaling  
- **Speed-based**: Gyro Ball reverse speed mechanics
- **Special conditions**: Body Press (uses Defense for Attack calculation)

### Accuracy and Miss Mechanics Tests (`accuracy_miss.rs`)

**Compound Eyes Interaction**: Ability-based accuracy modifications:
```rust
#[test]
fn test_compound_eyes_accuracy_boost() {
    let instructions = TestBuilder::new("Compound Eyes Accuracy")
        .team_one(PokemonSpec::new(PokemonName::Butterfree)
            .ability(Abilities::CompoundEyes)
            .moves(vec![Moves::Thunder])) // 70% base accuracy
        .team_two(PokemonSpec::new(PokemonName::Pikachu))
        .use_move(Positions::SIDE_ONE_0, Moves::Thunder)
        .get_instructions();
        
    // Thunder with Compound Eyes: 70% * 1.3 = 91% accuracy
    let hit_instructions = instructions.iter()
        .filter(|inst| !inst.instruction_list.is_empty())
        .collect::<Vec<_>>();
    let miss_instructions = instructions.iter()
        .filter(|inst| inst.instruction_list.is_empty())
        .collect::<Vec<_>>();
        
    assert_probability_within_tolerance(hit_instructions[0].percentage, 91.0, 0.1);
    assert_probability_within_tolerance(miss_instructions[0].percentage, 9.0, 0.1);
}
```

### Status Effect Tests (`secondary_effects.rs`)

**Secondary Effect Validation**: Status infliction with probability tracking:
```rust
#[test]
fn test_ice_fang_secondary_effects() {
    let instructions = TestBuilder::new("Ice Fang Secondary Effects")
        .team_one(PokemonSpec::new(PokemonName::Garchomp).moves(vec![Moves::IceFang]))
        .team_two(PokemonSpec::new(PokemonName::Dragonite))
        .use_move(Positions::SIDE_ONE_0, Moves::IceFang)
        .get_instructions();
        
    // Ice Fang: 10% freeze, 10% flinch (independent probabilities)
    // Expected branches: No effect (81%), Freeze only (9%), Flinch only (9%), Both (1%)
    assert_eq!(instructions.len(), 4);
    
    let probabilities: Vec<f64> = instructions.iter().map(|i| i.percentage).collect();
    assert_probability_within_tolerance(probabilities[0], 81.0, 0.1); // No secondary effects
    assert_probability_within_tolerance(probabilities[1], 9.0, 0.1);  // Freeze only
    assert_probability_within_tolerance(probabilities[2], 9.0, 0.1);  // Flinch only  
    assert_probability_within_tolerance(probabilities[3], 1.0, 0.1);  // Both effects
}
```

## Advanced Testing Patterns

### Probability Branching Validation

**Complex RNG Interaction Testing**: Multi-factor probability calculations:
```rust
pub fn validate_probability_branching(
    instructions: &[BattleInstructions],
    expected_branches: &[(f64, &str)] // (probability, description)
) -> Result<(), String> {
    if instructions.len() != expected_branches.len() {
        return Err(format!(
            "Branch count mismatch: expected {}, got {}",
            expected_branches.len(),
            instructions.len()
        ));
    }
    
    let total_probability: f64 = instructions.iter().map(|i| i.percentage).sum();
    if (total_probability - 100.0).abs() > 0.01 {
        return Err(format!("Probabilities don't sum to 100%: {:.4}%", total_probability));
    }
    
    for (instruction, (expected_prob, description)) in instructions.iter().zip(expected_branches.iter()) {
        assert_probability_within_tolerance(instruction.percentage, *expected_prob, 0.01)?;
    }
    
    Ok(())
}
```

### Generation-Specific Mechanic Testing

**Stat Calculation Validation**: DV vs IV/EV system differences:
```rust
pub fn calculate_gen1_stat(base: u16, dv: u8, level: u8, stat_type: StatType) -> u16 {
    // Gen 1 DV-based calculation
    match stat_type {
        StatType::HP => ((base + dv as u16) * 2 * level as u16) / 100 + level as u16 + 10,
        _ => ((base + dv as u16) * 2 * level as u16) / 100 + 5,
    }
}

pub fn calculate_modern_stat(base: u16, iv: u8, ev: u8, level: u8, nature_mod: f64, stat_type: StatType) -> u16 {
    // Gen 3+ IV/EV calculation with nature modifiers
    let base_calc = match stat_type {
        StatType::HP => ((2 * base as u32 + iv as u32 + ev as u32 / 4) * level as u32) / 100 + level as u32 + 10,
        _ => (((2 * base as u32 + iv as u32 + ev as u32 / 4) * level as u32) / 100 + 5) as f64 * nature_mod,
    };
    base_calc as u16
}
```

### Format-Specific Position Testing

**Multi-Format Target Validation**: Position-based targeting across formats:
```rust
#[test]
fn test_doubles_spread_move_targeting() {
    let instructions = TestBuilder::new("Doubles Spread Move")
        .new_with_format(BattleFormat::gen9_vgc())
        .team_one_multi(vec![
            PokemonSpec::new(PokemonName::Charizard).moves(vec![Moves::HeatWave]),
            PokemonSpec::new(PokemonName::Venusaur)
        ])
        .team_two_multi(vec![
            PokemonSpec::new(PokemonName::Blastoise),
            PokemonSpec::new(PokemonName::Pikachu)
        ])
        .use_move(Positions::SIDE_ONE_0, Moves::HeatWave)
        .get_instructions();
        
    // Heat Wave hits both opposing Pokemon in doubles
    let affected_positions = &instructions[0].affected_positions;
    assert!(affected_positions.contains(&Positions::SIDE_TWO_0));
    assert!(affected_positions.contains(&Positions::SIDE_TWO_1));
    assert_eq!(affected_positions.len(), 2);
}
```