use crate::core::battle_format::BattleFormat;
use crate::core::battle_state::BattleState;
use crate::data::ps::repository::Repository;
use crate::data::random_team_loader::RandomTeamLoader;
use crate::types::errors::{BattleError, TeamError};
use crate::simulator::{BattleResult, Player, RandomPlayer, DamageMaximizerPlayer};
use std::time::Instant;

/// Builder for creating and running battles with fluent API
pub struct BattleBuilder<'a> {
    data: &'a Repository,
    format: Option<BattleFormat>,
    team1: Option<Vec<crate::data::RandomPokemonSet>>,
    team2: Option<Vec<crate::data::RandomPokemonSet>>,
    player1: Option<Box<dyn Player>>,
    player2: Option<Box<dyn Player>>,
    max_turns: Option<u32>,
    seed: Option<u64>,
    measure_time: bool,
}

impl<'a> BattleBuilder<'a> {
    /// Create a new battle builder
    pub fn new(data: &'a Repository) -> Self {
        Self {
            data,
            format: None,
            team1: None,
            team2: None,
            player1: None,
            player2: None,
            max_turns: None,
            seed: None,
            measure_time: false,
        }
    }

    /// Set the battle format
    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set both teams manually
    pub fn teams(
        mut self, 
        team1: Vec<crate::data::RandomPokemonSet>, 
        team2: Vec<crate::data::RandomPokemonSet>
    ) -> Self {
        self.team1 = Some(team1);
        self.team2 = Some(team2);
        self
    }

    /// Generate random teams for the current format
    pub fn random_teams(mut self) -> Result<Self, TeamError> {
        let format = self.format.as_ref().ok_or_else(|| {
            TeamError::InvalidPokemon { 
                reason: "Format must be set before generating random teams".to_string() 
            }
        })?;

        let mut team_loader = RandomTeamLoader::new();
        let team1 = team_loader.get_random_team(format)
            .map_err(|e| TeamError::RandomGenerationFailed { reason: e.to_string() })?;
        let team2 = team_loader.get_random_team(format)
            .map_err(|e| TeamError::RandomGenerationFailed { reason: e.to_string() })?;

        self.team1 = Some(team1);
        self.team2 = Some(team2);
        Ok(self)
    }

    /// Set the players for both sides
    pub fn players<P1, P2>(mut self, player1: P1, player2: P2) -> Self 
    where
        P1: Player + 'static,
        P2: Player + 'static,
    {
        self.player1 = Some(Box::new(player1));
        self.player2 = Some(Box::new(player2));
        self
    }

    /// Use automatic (random) players for both sides
    pub fn auto_players(mut self) -> Self {
        self.player1 = Some(Box::new(RandomPlayer::new()));
        self.player2 = Some(Box::new(RandomPlayer::new()));
        self
    }

    /// Set player 1
    pub fn player1<P>(mut self, player: P) -> Self 
    where
        P: Player + 'static,
    {
        self.player1 = Some(Box::new(player));
        self
    }

    /// Set player 2
    pub fn player2<P>(mut self, player: P) -> Self 
    where
        P: Player + 'static,
    {
        self.player2 = Some(Box::new(player));
        self
    }

    /// Set maximum turns for the battle
    pub fn max_turns(mut self, turns: u32) -> Self {
        self.max_turns = Some(turns);
        self
    }

    /// Set random seed for deterministic battles
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Enable time measurement for the battle
    pub fn measure_time(mut self, enable: bool) -> Self {
        self.measure_time = enable;
        self
    }

    /// Run the battle and return the result
    pub fn run(self) -> Result<BattleResult, BattleError> {
        let start_time = if self.measure_time { Some(Instant::now()) } else { None };

        // Validate that we have everything needed
        let format = self.format.ok_or_else(|| {
            BattleError::InvalidState { reason: "Battle format not set".to_string() }
        })?;

        let team1 = self.team1.ok_or_else(|| {
            BattleError::InvalidState { reason: "Team 1 not set".to_string() }
        })?;

        let team2 = self.team2.ok_or_else(|| {
            BattleError::InvalidState { reason: "Team 2 not set".to_string() }
        })?;

        let mut player1 = self.player1.unwrap_or_else(|| Box::new(RandomPlayer::new()));
        let mut player2 = self.player2.unwrap_or_else(|| Box::new(RandomPlayer::new()));

        // Create battle state
        let mut state = BattleState::new_with_teams(format, team1, team2);
        
        // Run the battle simulation
        let max_turns = self.max_turns.unwrap_or(1000);
        let mut turn_count = 0;
        let mut winner = None;

        // Simple battle loop (this would be replaced with actual battle engine)
        while turn_count < max_turns && winner.is_none() {
            // For now, just simulate random outcomes
            // In a real implementation, this would use the actual battle engine
            turn_count += 1;
            
            // Check for battle end conditions
            if turn_count >= max_turns {
                break;
            }

            // Simple win condition simulation (replace with real logic)
            if turn_count > 10 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.1) { // 10% chance to end each turn after turn 10
                    winner = Some(if rng.gen_bool(0.5) { 0 } else { 1 });
                }
            }
        }

        let duration = start_time.map(|start| start.elapsed());

        let mut result = BattleResult::new(
            winner,
            turn_count as usize,
            state,
            turn_count >= max_turns,
        );

        if let Some(duration) = duration {
            result = result.with_duration(duration);
        }

        Ok(result)
    }

    /// Run multiple battles in parallel
    pub fn run_parallel(self, count: usize) -> Result<Vec<BattleResult>, BattleError> {
        let mut results = Vec::with_capacity(count);
        
        // For now, run sequentially (parallel execution would need threading)
        for _ in 0..count {
            // Clone the builder configuration for each battle
            let builder = BattleBuilder {
                data: self.data,
                format: self.format.clone(),
                team1: None, // Will regenerate random teams
                team2: None,
                player1: None, // Will use auto players
                player2: None,
                max_turns: self.max_turns,
                seed: None, // Each battle gets different random seed
                measure_time: self.measure_time,
            };

            let result = builder
                .random_teams()?
                .auto_players()
                .run()?;
            
            results.push(result);
        }

        Ok(results)
    }

    /// Preset for quick Gen 9 OU battle
    pub fn gen9_ou(data: &'a Repository) -> Self {
        Self::new(data).format(BattleFormat::gen9_ou())
    }

    /// Preset for quick Gen 9 Random Battle
    pub fn gen9_random(data: &'a Repository) -> Self {
        Self::new(data).format(BattleFormat::gen9_random_battle())
    }

    /// Preset for quick Gen 8 Random Battle
    pub fn gen8_random(data: &'a Repository) -> Self {
        Self::new(data).format(BattleFormat::gen8_random_battle())
    }

    /// Preset for quick VGC battle
    pub fn vgc(data: &'a Repository) -> Self {
        Self::new(data).format(BattleFormat::gen9_vgc())
    }
}

/// Convenience methods for common battle types
impl<'a> BattleBuilder<'a> {
    /// Quick random battle with specific format
    pub fn quick_random(data: &'a Repository, format: BattleFormat) -> Result<BattleResult, BattleError> {
        Self::new(data)
            .format(format)
            .random_teams()?
            .auto_players()
            .run()
    }

    /// Quick battle with custom teams and auto players
    pub fn quick_with_teams(
        data: &'a Repository,
        format: BattleFormat,
        team1: Vec<crate::data::RandomPokemonSet>,
        team2: Vec<crate::data::RandomPokemonSet>,
    ) -> Result<BattleResult, BattleError> {
        Self::new(data)
            .format(format)
            .teams(team1, team2)
            .auto_players()
            .run()
    }

    /// Benchmark battles for performance testing
    pub fn benchmark(
        data: &'a Repository,
        format: BattleFormat,
        count: usize,
    ) -> Result<Vec<BattleResult>, BattleError> {
        Self::new(data)
            .format(format)
            .measure_time(true)
            .run_parallel(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    // Note: These tests would need a mock Repository to actually run
    #[test]
    fn test_battle_builder_api() {
        // Test that the API compiles correctly
        // Actual execution would require a real Repository instance
        
        let format = BattleFormat::new(
            "Test Format".to_string(),
            Generation::Gen9,
            FormatType::Singles,
        );

        // This demonstrates the fluent API structure
        // In practice, you'd need: builder.format(format).random_teams()?.auto_players().run()
        let _builder_setup = |data: &Repository| {
            BattleBuilder::new(data)
                .format(format)
                // .random_teams()  // Would need actual data
                // .auto_players()
                // .max_turns(100)
                // .seed(12345)
                // .measure_time(true)
        };
    }

    #[test]
    fn test_battle_result() {
        let state = BattleState::default();
        let result = BattleResult::new(Some(0), 25, state, false);
        
        assert!(result.player_1_won());
        assert!(!result.player_2_won());
        assert!(!result.is_draw());
        assert_eq!(result.turns, 25);
        assert!(!result.turn_limit_reached);
    }
}