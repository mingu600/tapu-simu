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
        options[rng.gen_range(0..options.len())].clone()
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
        options[0].clone()
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
                        // Use actual base power from move data
                        move_data.base_power as f32
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
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
        let mut best_move = options[0].clone();
        let mut best_damage = self.estimate_damage(state, side_ref, &options[0]);

        for option in options.iter().skip(1) {
            let damage = self.estimate_damage(state, side_ref, option);
            if damage > best_damage {
                best_damage = damage;
                best_move = option.clone();
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
            let showdown_export = self.format_showdown_export(&state);
            let team_stats = self.format_team_stats(&state);

            if let Some(ref mut file) = log_file {
                writeln!(file, "{}", start_msg).unwrap();
                writeln!(file, "{}", showdown_export).unwrap();
                writeln!(file, "{}", team_stats).unwrap();
                file.flush().unwrap();
            } else {
                println!("{}", start_msg);
                println!("{}", showdown_export);
                println!("{}", team_stats);
            }
        }

        // DEBUG: Check stats before initial instructions
        if self.verbose {
            for (i, pokemon) in state.side_one.pokemon.iter().enumerate() {
                if pokemon.species == "Gothitelle" {
                    if let Some(ref mut file) = log_file {
                        writeln!(file, "DEBUG: {} stats BEFORE initial instructions: ATK:{} DEF:{} SPA:{} SPD:{} SPE:{}", 
                                pokemon.species, pokemon.stats.attack, pokemon.stats.defense, 
                                pokemon.stats.special_attack, pokemon.stats.special_defense, pokemon.stats.speed).unwrap();
                    }
                    println!("DEBUG: {} stats BEFORE initial instructions: ATK:{} DEF:{} SPA:{} SPD:{} SPE:{}", 
                            pokemon.species, pokemon.stats.attack, pokemon.stats.defense, 
                            pokemon.stats.special_attack, pokemon.stats.special_defense, pokemon.stats.speed);
                }
            }
        }

        // Generate and apply initial switch-in instructions
        let initial_instructions = Self::generate_initial_instructions(&mut state);
        if !initial_instructions.is_empty() {
            if self.verbose {
                println!("DEBUG: Generated {} initial instruction sequences", initial_instructions.len());
                for (i, sequence) in initial_instructions.iter().enumerate() {
                    println!("  Sequence {}: {} instructions", i, sequence.instruction_list.len());
                    for (j, instruction) in sequence.instruction_list.iter().enumerate() {
                        println!("    {}: {:?}", j, instruction);
                    }
                }
            }
            let chosen_index = self.sample_instruction_index(&initial_instructions);
            if self.verbose {
                println!("DEBUG: Applying initial instruction sequence {}", chosen_index);
            }
            state.apply_instructions(&initial_instructions[chosen_index].instruction_list);
        } else if self.verbose {
            println!("DEBUG: No initial instructions generated");
        }
        
        // DEBUG: Check stats after initial instructions
        if self.verbose {
            for (i, pokemon) in state.side_one.pokemon.iter().enumerate() {
                if pokemon.species == "Gothitelle" {
                    if let Some(ref mut file) = log_file {
                        writeln!(file, "DEBUG: {} stats AFTER initial instructions: ATK:{} DEF:{} SPA:{} SPD:{} SPE:{}", 
                                pokemon.species, pokemon.stats.attack, pokemon.stats.defense, 
                                pokemon.stats.special_attack, pokemon.stats.special_defense, pokemon.stats.speed).unwrap();
                    }
                    println!("DEBUG: {} stats AFTER initial instructions: ATK:{} DEF:{} SPA:{} SPD:{} SPE:{}", 
                            pokemon.species, pokemon.stats.attack, pokemon.stats.defense, 
                            pokemon.stats.special_attack, pokemon.stats.special_defense, pokemon.stats.speed);
                }
            }
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

            // Log generated instructions if verbose
            if self.verbose {
                let instructions_msg = format!(
                    "\nInstructions Generated: {} possible sequences\n",
                    instructions.len()
                );

                if let Some(ref mut file) = log_file {
                    write!(file, "{}", instructions_msg).unwrap();
                    for (i, instruction_set) in instructions.iter().enumerate() {
                        writeln!(file, "  Sequence {} ({:.1}%): {} instructions", i, instruction_set.percentage, instruction_set.instruction_list.len()).unwrap();
                        for (j, instruction) in instruction_set.instruction_list.iter().enumerate() {
                            writeln!(file, "    {}: {:?}", j, instruction).unwrap();
                        }
                    }
                    writeln!(file, "").unwrap();
                    file.flush().unwrap();
                } else {
                    print!("{}", instructions_msg);
                    for (i, instruction_set) in instructions.iter().enumerate() {
                        println!("  Sequence {} ({:.1}%): {} instructions", i, instruction_set.percentage, instruction_set.instruction_list.len());
                        for (j, instruction) in instruction_set.instruction_list.iter().enumerate() {
                            println!("    {}: {:?}", j, instruction);
                        }
                    }
                    println!();
                }
            }

            // Apply the instructions (sampling from possibilities)
            if !instructions.is_empty() {
                let chosen_index = self.sample_instruction_index(&instructions);
                
                if self.verbose {
                    let chosen_msg = format!("Applying instruction sequence {}\n", chosen_index);
                    if let Some(ref mut file) = log_file {
                        write!(file, "{}", chosen_msg).unwrap();
                        file.flush().unwrap();
                    } else {
                        print!("{}", chosen_msg);
                    }
                }
                
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

    /// Format full team stats for battle logging
    fn format_team_stats(&self, state: &State) -> String {
        let mut output = String::new();
        
        output.push_str("\n=== Full Team Stats ===\n");
        
        // Side One stats
        output.push_str(&format!("Player 1 ({})\n", self.player_one.name()));
        for (i, pokemon) in state.side_one.pokemon.iter().enumerate() {
            output.push_str(&format!("  Pokemon {}: {}\n", i + 1, self.format_pokemon_full_stats(pokemon)));
        }
        
        output.push_str("\n");
        
        // Side Two stats
        output.push_str(&format!("Player 2 ({})\n", self.player_two.name()));
        for (i, pokemon) in state.side_two.pokemon.iter().enumerate() {
            output.push_str(&format!("  Pokemon {}: {}\n", i + 1, self.format_pokemon_full_stats(pokemon)));
        }
        
        output.push_str("========================\n");
        output
    }

    /// Format individual Pokemon's full stats
    fn format_pokemon_full_stats(&self, pokemon: &crate::core::state::Pokemon) -> String {
        format!(
            "{} (Lv. {}) - HP: {}/{} | ATK: {} | DEF: {} | SPA: {} | SPD: {} | SPE: {} | Ability: {} | Item: {} | Type(s): {}",
            pokemon.species,
            pokemon.level,
            pokemon.hp,
            pokemon.max_hp,
            pokemon.stats.attack,
            pokemon.stats.defense,
            pokemon.stats.special_attack,
            pokemon.stats.special_defense,
            pokemon.stats.speed,
            pokemon.ability,
            pokemon.item.as_ref().unwrap_or(&"None".to_string()),
            pokemon.types.join("/")
        )
    }

    /// Generate Showdown paste export for both teams
    fn format_showdown_export(&self, state: &State) -> String {
        let mut output = String::new();
        
        output.push_str("\n=== Showdown Team Export ===\n");
        
        // Side One export
        output.push_str(&format!("Player 1 ({})\n", self.player_one.name()));
        for pokemon in &state.side_one.pokemon {
            output.push_str(&self.format_pokemon_showdown_paste(pokemon));
            output.push_str("\n");
        }
        
        output.push_str("\n");
        
        // Side Two export
        output.push_str(&format!("Player 2 ({})\n", self.player_two.name()));
        for pokemon in &state.side_two.pokemon {
            output.push_str(&self.format_pokemon_showdown_paste(pokemon));
            output.push_str("\n");
        }
        
        output.push_str("=============================\n");
        output
    }

    /// Format individual Pokemon as Showdown paste format
    fn format_pokemon_showdown_paste(&self, pokemon: &crate::core::state::Pokemon) -> String {
        let mut paste = String::new();
        
        // Species line with item and gender
        let gender_str = match pokemon.gender {
            crate::core::state::Gender::Male => " (M)",
            crate::core::state::Gender::Female => " (F)",
            crate::core::state::Gender::Unknown => "",
        };
        
        if let Some(ref item) = pokemon.item {
            paste.push_str(&format!("{}{} @ {}\n", pokemon.species, gender_str, item));
        } else {
            paste.push_str(&format!("{}{}\n", pokemon.species, gender_str));
        }
        
        // Ability
        paste.push_str(&format!("Ability: {}\n", pokemon.ability));
        
        // Level (only if not 50/100)
        if pokemon.level != 50 && pokemon.level != 100 {
            paste.push_str(&format!("Level: {}\n", pokemon.level));
        }
        
        // Tera Type (Gen 9+)
        if let Some(ref tera_type) = pokemon.tera_type {
            paste.push_str(&format!("Tera Type: {:?}\n", tera_type));
        }
        
        // Determine IVs and EVs based on moveset
        let (ivs, evs) = self.determine_ivs_evs_for_pokemon(pokemon);
        
        // EVs (only show if not all zero - Random Battles always have EVs)
        if !evs.is_all_zero() {
            paste.push_str(&format!("EVs: {}\n", evs.format_showdown()));
        }
        
        // Nature (neutral for Random Battle)
        paste.push_str("Nature: Hardy\n");
        
        // IVs (only show if not all 31)
        if !ivs.is_all_31() {
            paste.push_str(&format!("IVs: {}\n", ivs.format_showdown()));
        }
        
        // Moves
        let mut move_names: Vec<String> = pokemon.moves.values().map(|m| m.name.clone()).collect();
        move_names.sort(); // Sort for consistent output
        for move_name in move_names {
            paste.push_str(&format!("- {}\n", move_name));
        }
        
        paste
    }

    /// Determine IVs and EVs based on Pokemon's moveset following Smogon Random Battle rules
    fn determine_ivs_evs_for_pokemon(&self, pokemon: &crate::core::state::Pokemon) -> (PokemonIVs, PokemonEVs) {
        // Check if Pokemon has any physical moves
        let has_physical_moves = pokemon.moves.values()
            .any(|m| matches!(m.category, crate::core::instruction::MoveCategory::Physical));
        
        // Check if Pokemon has Trick Room or Gyro Ball
        let has_speed_dependent_moves = pokemon.moves.values()
            .any(|m| {
                let name_lower = m.name.to_lowercase();
                name_lower == "trick room" || name_lower == "gyro ball"
            });
        
        let mut ivs = PokemonIVs::default(); // 31 in all stats
        let mut evs = PokemonEVs::default(); // 85 in all stats
        
        // No physical attacks: Attack IV/EV = 0
        if !has_physical_moves {
            ivs.attack = 0;
            evs.attack = 0;
        }
        
        // Has Trick Room or Gyro Ball: Speed IV/EV = 0
        if has_speed_dependent_moves {
            ivs.speed = 0;
            evs.speed = 0;
        }
        
        (ivs, evs)
    }
}

/// Pokemon IVs for Showdown export
#[derive(Debug, Clone)]
struct PokemonIVs {
    hp: u8,
    attack: u8,
    defense: u8,
    special_attack: u8,
    special_defense: u8,
    speed: u8,
}

impl Default for PokemonIVs {
    fn default() -> Self {
        Self {
            hp: 31,
            attack: 31,
            defense: 31,
            special_attack: 31,
            special_defense: 31,
            speed: 31,
        }
    }
}

impl PokemonIVs {
    fn is_all_31(&self) -> bool {
        self.hp == 31 && self.attack == 31 && self.defense == 31 && 
        self.special_attack == 31 && self.special_defense == 31 && self.speed == 31
    }
    
    fn format_showdown(&self) -> String {
        let mut parts = Vec::new();
        if self.hp != 31 { parts.push(format!("{} HP", self.hp)); }
        if self.attack != 31 { parts.push(format!("{} Atk", self.attack)); }
        if self.defense != 31 { parts.push(format!("{} Def", self.defense)); }
        if self.special_attack != 31 { parts.push(format!("{} SpA", self.special_attack)); }
        if self.special_defense != 31 { parts.push(format!("{} SpD", self.special_defense)); }
        if self.speed != 31 { parts.push(format!("{} Spe", self.speed)); }
        parts.join(" / ")
    }
}

/// Pokemon EVs for Showdown export
#[derive(Debug, Clone)]
struct PokemonEVs {
    hp: u8,
    attack: u8,
    defense: u8,
    special_attack: u8,
    special_defense: u8,
    speed: u8,
}

impl Default for PokemonEVs {
    fn default() -> Self {
        Self {
            hp: 85,
            attack: 85,
            defense: 85,
            special_attack: 85,
            special_defense: 85,
            speed: 85,
        }
    }
}

impl PokemonEVs {
    fn is_all_zero(&self) -> bool {
        self.hp == 0 && self.attack == 0 && self.defense == 0 && 
        self.special_attack == 0 && self.special_defense == 0 && self.speed == 0
    }
    
    fn format_showdown(&self) -> String {
        let mut parts = Vec::new();
        if self.hp > 0 { parts.push(format!("{} HP", self.hp)); }
        if self.attack > 0 { parts.push(format!("{} Atk", self.attack)); }
        if self.defense > 0 { parts.push(format!("{} Def", self.defense)); }
        if self.special_attack > 0 { parts.push(format!("{} SpA", self.special_attack)); }
        if self.special_defense > 0 { parts.push(format!("{} SpD", self.special_defense)); }
        if self.speed > 0 { parts.push(format!("{} Spe", self.speed)); }
        parts.join(" / ")
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