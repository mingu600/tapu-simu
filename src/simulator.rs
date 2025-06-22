use crate::core::battle_format::BattleFormat;
use crate::core::battle_state::BattleState;
use crate::data::ps::repository::Repository;
use crate::types::errors::{BattleError, DataError, SimulatorError};
use crate::config::Config;
use crate::builders::{BattleBuilder, TeamBuilder};
use std::path::Path;

/// Main facade for the Tapu Simu battle simulator
/// 
/// Provides a clean, ergonomic API for creating and running Pokemon battles
/// with sensible defaults and fluent builder patterns.
pub struct Simulator {
    data: Repository,
    config: Config,
}

impl Simulator {
    /// Create a new simulator with default configuration
    pub fn new() -> Result<Self, SimulatorError> {
        let config = Config::default();
        let data = Repository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }

    /// Create a new simulator with custom configuration
    pub fn with_config(config: Config) -> Result<Self, SimulatorError> {
        let data = Repository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }

    /// Create a new simulator with data from a specific path
    pub fn with_data_path(path: impl AsRef<Path>) -> Result<Self, SimulatorError> {
        let mut config = Config::default();
        config.data_path = path.as_ref().to_path_buf();
        let data = Repository::from_path(&config.data_path)?;
        Ok(Self { data, config })
    }

    /// Get a battle builder for creating custom battles
    pub fn battle(&self) -> BattleBuilder<'_> {
        BattleBuilder::new(&self.data)
    }

    /// Quick API for common random battle
    pub fn quick_random_battle(&self, format: BattleFormat) -> Result<BattleResult, BattleError> {
        self.battle()
            .format(format)
            .random_teams()?
            .auto_players()
            .run()
    }

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

    /// Run battles in parallel for performance benchmarking
    pub fn benchmark_parallel(
        &self,
        format: BattleFormat,
        count: usize,
    ) -> Result<BenchmarkResult, BattleError> {
        // For now, implement as sequential - parallel execution would need threading
        self.benchmark_random_battles(format, count)
    }

    /// Get access to the underlying data repository
    pub fn data(&self) -> &Repository {
        &self.data
    }

    /// Get access to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Create a team builder for the current data
    pub fn team(&self) -> TeamBuilder<'_> {
        TeamBuilder::new(&self.data)
    }

    /// Create a format builder
    pub fn format() -> crate::builders::format::FormatBuilder {
        crate::builders::format::FormatBuilder::new()
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new().expect("Failed to create default simulator")
    }
}

/// Result of a single battle
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

impl BattleResult {
    pub fn new(
        winner: Option<usize>,
        turns: usize,
        final_state: BattleState,
        turn_limit_reached: bool,
    ) -> Self {
        Self {
            winner,
            turns,
            final_state,
            turn_limit_reached,
            duration: None,
        }
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = Some(duration);
        self
    }

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

/// Aggregated results from multiple battles
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

impl BenchmarkResult {
    pub fn from_results(battles: Vec<BattleResult>) -> Self {
        let total_battles = battles.len();
        let mut player_1_wins = 0;
        let mut player_2_wins = 0;
        let mut draws = 0;
        let mut total_turns = 0;
        let mut total_duration = std::time::Duration::ZERO;
        let mut has_durations = true;

        for battle in &battles {
            match battle.winner {
                Some(0) => player_1_wins += 1,
                Some(1) => player_2_wins += 1,
                None => draws += 1,
                Some(_) => {} // Invalid winner index
            }
            total_turns += battle.turns;
            
            if let Some(duration) = battle.duration {
                total_duration += duration;
            } else {
                has_durations = false;
            }
        }

        let average_turns = if total_battles > 0 {
            total_turns as f64 / total_battles as f64
        } else {
            0.0
        };

        Self {
            total_battles,
            player_1_wins,
            player_2_wins,
            draws,
            average_turns,
            total_duration: if has_durations { Some(total_duration) } else { None },
            battles,
        }
    }

    /// Get win rate for player 1
    pub fn player_1_win_rate(&self) -> f64 {
        if self.total_battles > 0 {
            self.player_1_wins as f64 / self.total_battles as f64
        } else {
            0.0
        }
    }

    /// Get win rate for player 2
    pub fn player_2_win_rate(&self) -> f64 {
        if self.total_battles > 0 {
            self.player_2_wins as f64 / self.total_battles as f64
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

/// Win rate tracking
#[derive(Debug, Clone, Copy)]
pub struct WinRate {
    pub wins: usize,
    pub total: usize,
}

impl WinRate {
    pub fn new(wins: usize, total: usize) -> Self {
        Self { wins, total }
    }

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

/// Trait for AI players
pub trait Player {
    /// Choose a move given the current battle state
    fn choose_move(&mut self, state: &BattleState, valid_moves: &[usize]) -> usize;

    /// Get a name for this player (for logging/debugging)
    fn name(&self) -> &str {
        "Unknown Player"
    }
}

/// Helper struct for creating players
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

/// Simple random player implementation
#[derive(Debug, Clone)]
pub struct RandomPlayer {
    name: String,
}

impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            name: "Random Player".to_string(),
        }
    }
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

/// Player that tries to maximize damage output
#[derive(Debug, Clone)]
pub struct DamageMaximizerPlayer {
    name: String,
}

impl DamageMaximizerPlayer {
    pub fn new() -> Self {
        Self {
            name: "Damage Maximizer".to_string(),
        }
    }
}

impl Player for DamageMaximizerPlayer {
    fn choose_move(&mut self, state: &BattleState, valid_moves: &[usize]) -> usize {
        // For now, just pick the first move (would need damage calculation integration)
        valid_moves.get(0).copied().unwrap_or(0)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

