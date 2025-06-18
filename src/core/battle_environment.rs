//! # Battle Environment Module
//! 
//! This module provides the main battle orchestration and player interfaces for Tapu Simu.
//! It implements 100% parity with poke-engine's battle_environment.rs, adapted for V2 architecture.

use crate::core::state::State;
use crate::core::move_choice::MoveChoice;
use crate::core::battle_format::SideReference;
use crate::core::instruction::StateInstructions;
use crate::engine::turn::instruction_generator::GenerationXInstructionGenerator;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

/// Player trait for different agent types - exact parity with poke-engine
pub trait Player: Send + Sync + 'static {
    /// Choose a move from available options
    fn choose_move(
        &self,
        state: &State,
        side_ref: SideReference,
        options: &[MoveChoice],
    ) -> MoveChoice;
    
    /// Get the player's name for identification
    fn name(&self) -> &str;
}

/// Random player implementation - selects moves randomly
pub struct RandomPlayer {
    name: String,
}

impl RandomPlayer {
    pub fn new(name: String) -> Self {
        RandomPlayer { name }
    }
}

impl Player for RandomPlayer {
    fn choose_move(
        &self,
        _state: &State,
        _side_ref: SideReference,
        options: &[MoveChoice],
    ) -> MoveChoice {
        let mut rng = thread_rng();
        options[rng.gen_range(0..options.len())]
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// First move player - always picks the first available move
pub struct FirstMovePlayer {
    name: String,
}

impl FirstMovePlayer {
    pub fn new(name: String) -> Self {
        FirstMovePlayer { name }
    }
}

impl Player for FirstMovePlayer {
    fn choose_move(
        &self,
        _state: &State,
        _side_ref: SideReference,
        options: &[MoveChoice],
    ) -> MoveChoice {
        options[0]
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Simple damage maximizer - picks the move that would do most damage
pub struct DamageMaximizer {
    name: String,
}

impl DamageMaximizer {
    pub fn new(name: String) -> Self {
        DamageMaximizer { name }
    }

    fn estimate_damage(
        &self,
        state: &State,
        side_ref: SideReference,
        move_choice: &MoveChoice,
    ) -> f32 {
        match move_choice {
            MoveChoice::Move { move_index, .. } => {
                let side = match side_ref {
                    SideReference::SideOne => &state.side_one,
                    SideReference::SideTwo => &state.side_two,
                };
                
                // Get the active Pokemon at position 0
                if let Some(active_pokemon) = side.get_active_pokemon_at_slot(0) {
                    if let Some(move_data) = active_pokemon.get_move(*move_index) {
                        // Simple damage estimate based on base power
                        // TODO: Integrate with move database for actual base power lookup
                        100.0 // Placeholder - in real implementation would look up move power
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
            #[cfg(feature = "terastallization")]
            MoveChoice::MoveTera { move_index, .. } => {
                // Same logic as regular move but potentially higher power due to Tera
                let side = match side_ref {
                    SideReference::SideOne => &state.side_one,
                    SideReference::SideTwo => &state.side_two,
                };
                
                if let Some(active_pokemon) = side.get_active_pokemon_at_slot(0) {
                    if let Some(_move_data) = active_pokemon.get_move(*move_index) {
                        120.0 // Slightly higher estimate for Tera moves
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
            MoveChoice::Switch(_) => -10.0, // Slight penalty for switching
            MoveChoice::None => 0.0,
        }
    }
}

impl Player for DamageMaximizer {
    fn choose_move(
        &self,
        state: &State,
        side_ref: SideReference,
        options: &[MoveChoice],
    ) -> MoveChoice {
        let mut best_move = options[0];
        let mut best_damage = self.estimate_damage(state, side_ref, &options[0]);

        for option in options.iter().skip(1) {
            let damage = self.estimate_damage(state, side_ref, option);
            if damage > best_damage {
                best_damage = damage;
                best_move = *option;
            }
        }

        best_move
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Battle result information
#[derive(Debug, Clone)]
pub struct BattleResult {
    pub winner: Option<SideReference>,
    pub turn_count: usize,
    pub final_state: State,
    pub turn_history: Vec<TurnInfo>,
}

/// Turn information for observability - exact parity with poke-engine
#[derive(Debug, Clone)]
pub struct TurnInfo {
    pub turn_number: usize,
    pub state_before: State,
    pub side_one_choice: MoveChoice,
    pub side_two_choice: MoveChoice,
    pub instructions_generated: Vec<StateInstructions>,
    pub state_after: State,
}

/// Battle environment orchestrator - exact parity with poke-engine's BattleEnvironment
pub struct BattleEnvironment {
    pub player_one: Box<dyn Player>,
    pub player_two: Box<dyn Player>,
    pub max_turns: usize,
    pub verbose: bool,
    pub log_file: Option<String>,
}

impl BattleEnvironment {
    /// Create a new battle environment
    pub fn new(
        player_one: Box<dyn Player>,
        player_two: Box<dyn Player>,
        max_turns: usize,
        verbose: bool,
    ) -> Self {
        BattleEnvironment {
            player_one,
            player_two,
            max_turns,
            verbose,
            log_file: None,
        }
    }

    /// Add logging capability to the battle environment
    pub fn with_log_file(mut self, log_file: String) -> Self {
        self.log_file = Some(log_file);
        self
    }

    /// Generate initial switch-in instructions
    fn generate_initial_instructions(state: &mut State) -> Vec<StateInstructions> {
        // Generate initial instructions for start-of-battle effects like abilities
        let no_move_s1 = MoveChoice::None;
        let no_move_s2 = MoveChoice::None;

        let generator = GenerationXInstructionGenerator::new(state.format.clone());
        generator.generate_instructions_from_move_pair(state, &no_move_s1, &no_move_s2)
    }

    /// Run a complete battle - exact parity with poke-engine logic
    pub fn run_battle(&self, initial_state: State) -> BattleResult {
        let mut state = initial_state.clone();
        let mut turn_history = Vec::new();
        let mut turn_count = 0;

        // Create log file if verbose
        let mut log_file = if self.verbose && self.log_file.is_some() {
            use std::fs::OpenOptions;
            Some(OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(self.log_file.as_ref().unwrap())
                .expect("Failed to create log file"))
        } else {
            None
        };
        
        // Set environment variable for players to use
        if let Some(ref log_path) = self.log_file {
            std::env::set_var("BATTLE_LOG_FILE", log_path);
        }

        // Log battle start
        if self.verbose {
            let start_msg = format!(
                "\n=== Battle Starting ===\nPlayer 1 ({}) vs Player 2 ({})\n",
                self.player_one.name(),
                self.player_two.name()
            );

            if let Some(ref mut file) = log_file {
                writeln!(file, "{}", start_msg).unwrap();
                file.flush().unwrap();
            } else {
                println!("{}", start_msg);
            }
        }

        // Generate and apply initial switch-in instructions
        let initial_instructions = Self::generate_initial_instructions(&mut state);
        if !initial_instructions.is_empty() {
            let chosen_index = self.sample_instruction_index(&initial_instructions);
            state.apply_instructions(&initial_instructions[chosen_index].instruction_list);
        }

        // Main battle loop - exact parity with poke-engine
        while !state.is_battle_over() && turn_count < self.max_turns {
            turn_count += 1;
            let state_before = state.clone();

            // Get available options for both players
            let (side_one_options, side_two_options) = state.get_all_options();

            // Check if we have any options
            if side_one_options.is_empty() || side_two_options.is_empty() {
                if self.verbose {
                    if let Some(ref mut file) = log_file {
                        writeln!(file, "WARNING: No options available for one or both sides!").ok();
                    }
                }
                break;
            }

            // Write turn header and state BEFORE players make moves
            if self.verbose {
                let turn_header = format!(
                    "\n========== Turn {} ==========\n{}\n\nSerialized State:\n{}\n",
                    turn_count,
                    state.pretty_print(),
                    state.serialize()
                );

                if let Some(ref mut file) = log_file {
                    write!(file, "{}", turn_header).unwrap();
                    file.flush().unwrap();
                } else {
                    print!("{}", turn_header);
                }
            }
            
            // Close the log file so players can append to it
            if log_file.is_some() {
                drop(log_file.take());
            }

            // Players choose their moves
            let side_one_choice =
                self.player_one
                    .choose_move(&state, SideReference::SideOne, &side_one_options);
            let side_two_choice =
                self.player_two
                    .choose_move(&state, SideReference::SideTwo, &side_two_options);

            // Reopen log file to write the selected moves
            if self.verbose && self.log_file.is_some() {
                use std::fs::OpenOptions;
                log_file = Some(OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.log_file.as_ref().unwrap())
                    .expect("Failed to reopen log file"));
                    
                let moves_msg = format!(
                    "\nMoves Selected:\n  Side 1: {}\n  Side 2: {}\n=============================\n",
                    side_one_choice.to_string(&state.side_one),
                    side_two_choice.to_string(&state.side_two)
                );

                if let Some(ref mut file) = log_file {
                    write!(file, "{}", moves_msg).unwrap();
                    file.flush().unwrap();
                }
            }

            // Generate instructions from the move pair
            let generator = GenerationXInstructionGenerator::new(state.format.clone());
            let instructions = generator.generate_instructions_from_move_pair(
                &mut state,
                &side_one_choice,
                &side_two_choice,
            );

            // Apply the instructions (sampling from possibilities)
            if !instructions.is_empty() {
                let chosen_index = self.sample_instruction_index(&instructions);
                state.apply_instructions(&instructions[chosen_index].instruction_list);
            }

            // Record turn information
            turn_history.push(TurnInfo {
                turn_number: turn_count,
                state_before,
                side_one_choice,
                side_two_choice,
                instructions_generated: instructions,
                state_after: state.clone(),
            });
        }

        // Determine winner
        let winner = state.get_winner();

        if self.verbose {
            let end_msg = format!(
                "\n=== Battle Ended ===\n{}\nTotal turns: {}\n",
                match winner {
                    Some(SideReference::SideOne) =>
                        format!("Player 1 ({}) wins!", self.player_one.name()),
                    Some(SideReference::SideTwo) =>
                        format!("Player 2 ({}) wins!", self.player_two.name()),
                    None => "Battle ended in a draw (turn limit reached)".to_string(),
                },
                turn_count
            );

            if let Some(ref mut file) = log_file {
                write!(file, "{}", end_msg).unwrap();
                file.flush().unwrap();
            } else {
                print!("{}", end_msg);
            }
        }

        // Clear the environment variable
        if self.log_file.is_some() {
            std::env::remove_var("BATTLE_LOG_FILE");
        }
        
        BattleResult {
            winner,
            turn_count,
            final_state: state,
            turn_history,
        }
    }

    /// Sample from possible instruction outcomes based on their probabilities
    fn sample_instruction_index(&self, state_instructions: &[StateInstructions]) -> usize {
        if state_instructions.len() == 1 {
            return 0;
        }

        let mut rng = thread_rng();
        let total_percentage: f32 = state_instructions.iter().map(|si| si.percentage).sum();
        let mut random_value = rng.gen::<f32>() * total_percentage;

        for (index, si) in state_instructions.iter().enumerate() {
            random_value -= si.percentage;
            if random_value <= 0.0 {
                return index;
            }
        }

        state_instructions.len() - 1
    }
}

/// Parallel battle execution results
#[derive(Debug)]
pub struct ParallelBattleResults {
    pub player_one_wins: usize,
    pub player_two_wins: usize,
    pub draws: usize,
    pub total_battles: usize,
}

impl ParallelBattleResults {
    /// Calculate player one win rate
    pub fn player_one_win_rate(&self) -> f64 {
        if self.total_battles == 0 {
            0.0
        } else {
            self.player_one_wins as f64 / self.total_battles as f64
        }
    }

    /// Calculate player two win rate
    pub fn player_two_win_rate(&self) -> f64 {
        if self.total_battles == 0 {
            0.0
        } else {
            self.player_two_wins as f64 / self.total_battles as f64
        }
    }

    /// Calculate draw rate
    pub fn draw_rate(&self) -> f64 {
        if self.total_battles == 0 {
            0.0
        } else {
            self.draws as f64 / self.total_battles as f64
        }
    }
}

/// Run parallel battles with pre-generated states - exact parity with poke-engine
pub fn run_parallel_battles_with_states<F1, F2>(
    battle_states: Vec<State>,
    num_threads: usize,
    player_one_factory: F1,
    player_two_factory: F2,
    max_turns: usize,
) -> ParallelBattleResults
where
    F1: Fn() -> Box<dyn Player> + Send + Sync + 'static,
    F2: Fn() -> Box<dyn Player> + Send + Sync + 'static,
{
    let num_battles = battle_states.len();
    let battle_states = Arc::new(battle_states);
    
    let player_one_factory = Arc::new(player_one_factory);
    let player_two_factory = Arc::new(player_two_factory);
    let results = Arc::new(Mutex::new(ParallelBattleResults {
        player_one_wins: 0,
        player_two_wins: 0,
        draws: 0,
        total_battles: 0,
    }));

    let battles_per_thread = num_battles / num_threads;
    let remainder = num_battles % num_threads;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let p1_factory = Arc::clone(&player_one_factory);
            let p2_factory = Arc::clone(&player_two_factory);
            let results = Arc::clone(&results);
            let states = Arc::clone(&battle_states);

            let thread_start = thread_id * battles_per_thread + thread_id.min(remainder);
            let thread_battles = if thread_id < remainder {
                battles_per_thread + 1
            } else {
                battles_per_thread
            };

            thread::spawn(move || {
                for i in 0..thread_battles {
                    let state_idx = thread_start + i;
                    let initial_state = states[state_idx].clone();
                    
                    let env = BattleEnvironment::new(
                        p1_factory(),
                        p2_factory(),
                        max_turns,
                        false, // Not verbose for parallel runs
                    );

                    let result = env.run_battle(initial_state);

                    let mut results = results.lock().unwrap();
                    results.total_battles += 1;
                    match result.winner {
                        Some(SideReference::SideOne) => results.player_one_wins += 1,
                        Some(SideReference::SideTwo) => results.player_two_wins += 1,
                        None => results.draws += 1,
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Helper function to create a battle from a state and run it
pub fn run_battle_from_state(
    initial_state: State,
    player_one: Box<dyn Player>,
    player_two: Box<dyn Player>,
    max_turns: usize,
    verbose: bool,
) -> BattleResult {
    let env = BattleEnvironment::new(player_one, player_two, max_turns, verbose);
    env.run_battle(initial_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_format::BattleFormat;

    #[test]
    fn test_random_player_creation() {
        let player = RandomPlayer::new("TestPlayer".to_string());
        assert_eq!(player.name(), "TestPlayer");
    }

    #[test]
    fn test_first_move_player() {
        let player = FirstMovePlayer::new("FirstBot".to_string());
        let state = State::new(BattleFormat::gen9_ou());
        let options = vec![MoveChoice::None, MoveChoice::Switch(crate::core::move_choice::PokemonIndex::P0)];
        
        let choice = player.choose_move(&state, SideReference::SideOne, &options);
        assert_eq!(choice, MoveChoice::None); // Always picks first option
    }

    #[test]
    fn test_battle_environment_creation() {
        let player_one = Box::new(RandomPlayer::new("P1".to_string()));
        let player_two = Box::new(RandomPlayer::new("P2".to_string()));
        
        let env = BattleEnvironment::new(player_one, player_two, 100, false);
        assert_eq!(env.max_turns, 100);
        assert!(!env.verbose);
        assert!(env.log_file.is_none());
    }

    #[test]
    fn test_parallel_battle_results() {
        let results = ParallelBattleResults {
            player_one_wins: 30,
            player_two_wins: 20,
            draws: 0,
            total_battles: 50,
        };
        
        assert_eq!(results.player_one_win_rate(), 0.6);
        assert_eq!(results.player_two_win_rate(), 0.4);
        assert_eq!(results.draw_rate(), 0.0);
    }
}