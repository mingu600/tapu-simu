# Simulator Module Documentation

The simulator module provides the primary facade and entry point for Tapu Simu, offering ergonomic APIs for battle execution, benchmarking, and performance analysis with built-in player implementations and result tracking.

## Architecture Overview

The simulator module consists of four main components:
- **Simulator**: Main facade with configuration and data management
- **Battle Results**: Comprehensive battle outcome tracking
- **Player System**: AI implementations and player trait
- **Benchmarking**: Performance analysis and statistical tools

## Primary Interface

Main entry point providing ergonomic APIs for battle execution and analysis.

### Core Capabilities

**Primary Functions:**
- Battle creation and execution with customizable parameters
- Team generation and management with random battle support
- Performance benchmarking and statistical analysis
- Multi-format support (Singles, Doubles, VGC, Triples)

**Key Features:**
- Centralized data and configuration management
- Integration with builder patterns for flexible setup
- Built-in player implementations for testing and evaluation
- Comprehensive result tracking and analysis

### Initialization

**Default Initialization:**
```rust
impl Simulator {
    /// Create a new simulator with default configuration
    pub fn new() -> Result<Self, SimulatorError> {
        let config = Config::default();
        let data = GameDataRepository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }
}
```

**Custom Configuration:**
```rust
impl Simulator {
    /// Create a new simulator with custom configuration
    pub fn with_config(config: Config) -> Result<Self, SimulatorError> {
        let data = GameDataRepository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }

    /// Create a new simulator with data from a specific path
    pub fn with_data_path(path: impl AsRef<Path>) -> Result<Self, SimulatorError> {
        let mut config = Config::default();
        config.data_path = path.as_ref().to_path_buf();
        let data = GameDataRepository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }
}
```

### Battle Creation APIs

**Builder Integration:**
```rust
impl Simulator {
    /// Get a battle builder for creating custom battles
    pub fn battle(&self) -> BattleBuilder<'_> {
        BattleBuilder::new(&self.data)
    }

    /// Create a team builder for the current data
    pub fn team(&self) -> TeamBuilder<'_> {
        TeamBuilder::new(&self.data)
    }

    /// Create a format builder
    pub fn format() -> FormatBuilder {
        FormatBuilder::new()
    }
}
```

**Quick Battle APIs:**
```rust
impl Simulator {
    /// Quick API for common random battle
    pub fn quick_random_battle(&self, format: BattleFormat) -> Result<BattleResult, BattleError> {
        self.battle()
            .format(format)
            .random_teams()?
            .auto_players()
            .run()
    }
}
```

### Benchmarking APIs

**Batch Execution:**
```rust
impl Simulator {
    /// Run multiple random battles and return aggregated results
    pub fn benchmark_random_battles(
        &self,
        format: BattleFormat,
        count: usize,
    ) -> Result<BenchmarkResult, BattleError> {
        let mut results = Vec::with_capacity(count);
        
        for _ in 0..count {
            let result = self.quick_random_battle(format.clone())?;
            results.push(result);
        }

        Ok(BenchmarkResult::from_results(results))
    }
}
```

**Player Evaluation:**
```rust
impl Simulator {
    /// Benchmark a specific player against random opponents
    pub fn benchmark_player<P>(
        &self,
        player: P,
        format: BattleFormat,
        games: usize,
    ) -> Result<WinRate, BattleError>
    where
        P: Player + Clone + 'static,
    {
        let mut wins = 0;
        let mut total = 0;

        for _ in 0..games {
            let result = self.battle()
                .format(format.clone())
                .random_teams()?
                .players(player.clone(), Players::random())
                .run()?;

            total += 1;
            if result.winner == Some(0) {
                wins += 1;
            }
        }

        Ok(WinRate::new(wins, total))
    }
}
```

## Battle Results

### BattleResult Structure

**Comprehensive Battle Outcome:**
```rust
#[derive(Debug, Clone)]
pub struct BattleResult {
    /// Winner of the battle (0 for player 1, 1 for player 2, None for draw)
    pub winner: Option<usize>,
    /// Total number of turns
    pub turns: usize,
    /// Final battle state
    pub final_state: BattleState,
    /// Whether the battle ended due to turn limit
    pub turn_limit_reached: bool,
    /// Time taken to complete the battle (if measured)
    pub duration: Option<std::time::Duration>,
}
```

**Result Analysis Methods:**
```rust
impl BattleResult {
    /// Returns true if player 1 won
    pub fn player_1_won(&self) -> bool {
        self.winner == Some(0)
    }

    /// Returns true if player 2 won
    pub fn player_2_won(&self) -> bool {
        self.winner == Some(1)
    }

    /// Returns true if the battle was a draw
    pub fn is_draw(&self) -> bool {
        self.winner.is_none()
    }
}
```

### BenchmarkResult Structure

**Aggregated Statistics:**
```rust
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Total number of battles
    pub total_battles: usize,
    /// Number of battles won by player 1
    pub player_1_wins: usize,
    /// Number of battles won by player 2
    pub player_2_wins: usize,
    /// Number of draws
    pub draws: usize,
    /// Average number of turns per battle
    pub average_turns: f64,
    /// Total time taken for all battles
    pub total_duration: Option<std::time::Duration>,
    /// Individual battle results
    pub battles: Vec<BattleResult>,
}
```

**Statistical Analysis:**
```rust
impl BenchmarkResult {
    /// Get win rate for player 1
    pub fn player_1_win_rate(&self) -> f64 {
        if self.total_battles > 0 {
            self.player_1_wins as f64 / self.total_battles as f64
        } else {
            0.0
        }
    }

    /// Get draw rate
    pub fn draw_rate(&self) -> f64 {
        if self.total_battles > 0 {
            self.draws as f64 / self.total_battles as f64
        } else {
            0.0
        }
    }

    /// Get average battles per second (if duration available)
    pub fn battles_per_second(&self) -> Option<f64> {
        self.total_duration.map(|duration| {
            if duration.as_secs_f64() > 0.0 {
                self.total_battles as f64 / duration.as_secs_f64()
            } else {
                0.0
            }
        })
    }
}
```

### WinRate Tracking

**Simple Win Rate Structure:**
```rust
#[derive(Debug, Clone, Copy)]
pub struct WinRate {
    pub wins: usize,
    pub total: usize,
}

impl WinRate {
    pub fn rate(&self) -> f64 {
        if self.total > 0 {
            self.wins as f64 / self.total as f64
        } else {
            0.0
        }
    }

    pub fn percentage(&self) -> f64 {
        self.rate() * 100.0
    }
}

impl std::fmt::Display for WinRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} ({:.1}%)", self.wins, self.total, self.percentage())
    }
}
```

## Player System

### Player Trait

**AI Interface:**
```rust
pub trait Player {
    /// Choose a move given the current battle state
    fn choose_move(&mut self, state: &BattleState, valid_moves: &[usize]) -> usize;

    /// Get a name for this player (for logging/debugging)
    fn name(&self) -> &str {
        "Unknown Player"
    }
}
```

**Design Features:**
- Stateful players with mutable state
- Access to complete battle state for decision making
- Valid move constraint enforcement
- Optional naming for debugging and logging

### Built-in Player Implementations

#### RandomPlayer

**Random Move Selection:**
```rust
#[derive(Debug, Clone)]
pub struct RandomPlayer {
    name: String,
}

impl Player for RandomPlayer {
    fn choose_move(&mut self, _state: &BattleState, valid_moves: &[usize]) -> usize {
        if valid_moves.is_empty() {
            0
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            valid_moves[rng.gen_range(0..valid_moves.len())]
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

**Use Cases:**
- Baseline comparison for AI development
- Testing battle mechanics across random scenarios
- Default player for quick battles

#### DamageMaximizerPlayer

**Damage-Focused Strategy:**
```rust
#[derive(Debug, Clone)]
pub struct DamageMaximizerPlayer {
    name: String,
}

impl Player for DamageMaximizerPlayer {
    fn choose_move(&mut self, state: &BattleState, valid_moves: &[usize]) -> usize {
        // For now, just pick the first move (would need damage calculation integration)
        valid_moves.get(0).copied().unwrap_or(0)
    }
}
```

**Future Enhancements:**
- Integration with damage calculation system
- Move power and type effectiveness evaluation
- Multi-target damage optimization

### Player Factory

**Convenient Player Creation:**
```rust
pub struct Players;

impl Players {
    /// Create a random player that chooses moves randomly
    pub fn random() -> RandomPlayer {
        RandomPlayer::new()
    }

    /// Create a damage maximizer player
    pub fn damage_maximizer() -> DamageMaximizerPlayer {
        DamageMaximizerPlayer::new()
    }
}
```

## Usage Patterns

### Basic Battle Execution

**Simple Random Battle:**
```rust
let simulator = Simulator::new()?;
let result = simulator.quick_random_battle(BattleFormat::singles())?;

println!("Winner: {:?}", result.winner);
println!("Turns: {}", result.turns);
```

**Custom Battle Setup:**
```rust
let simulator = Simulator::new()?;
let result = simulator.battle()
    .format(BattleFormat::doubles())
    .side_one_pokemon(
        PokemonSpec::new("Pikachu")
            .level(50)
            .moves(vec!["Thunderbolt", "Quick Attack"])
    )
    .side_two_pokemon(
        PokemonSpec::new("Charizard")
            .level(50)
            .moves(vec!["Flamethrower", "Dragon Pulse"])
    )
    .players(Players::random(), Players::damage_maximizer())
    .run()?;
```

### Performance Benchmarking

**Format Performance Analysis:**
```rust
let simulator = Simulator::new()?;
let benchmark = simulator.benchmark_random_battles(
    BattleFormat::singles(),
    1000  // Run 1000 battles
)?;

println!("Player 1 win rate: {:.1}%", benchmark.player_1_win_rate() * 100.0);
println!("Average turns: {:.1}", benchmark.average_turns);
if let Some(bps) = benchmark.battles_per_second() {
    println!("Battles per second: {:.1}", bps);
}
```

**Player Evaluation:**
```rust
let simulator = Simulator::new()?;
let win_rate = simulator.benchmark_player(
    Players::damage_maximizer(),
    BattleFormat::singles(),
    500  // Play 500 games
)?;

println!("Win rate: {}", win_rate);  // Displays "250/500 (50.0%)"
```

### Configuration-Based Setup

**Custom Configuration:**
```rust
let config = Config::builder()
    .data_path("/custom/data")
    .debug(true)
    .parallel_battles(true)
    .thread_count(8)
    .random_seed(12345)  // Deterministic battles
    .build()?;

let simulator = Simulator::with_config(config)?;
```

**Environment Configuration:**
```rust
// Set environment variables:
// TAPU_SIMU_DEBUG=true
// TAPU_SIMU_THREADS=4
// TAPU_SIMU_SEED=98765

let config = Config::from_env()?;
let simulator = Simulator::with_config(config)?;
```

## Integration Points

The simulator module integrates with:
- **Core Module**: Battle state and format management
- **Engine Module**: Battle execution and mechanics
- **Data Module**: Pokemon and move data access
- **Builders Module**: Fluent APIs for battle construction
- **Config Module**: Configuration management and validation