//! # Tapu Simu CLI
//!
//! Command-line interface for Tapu Simu.

use clap::Parser;
use tapu_simu::data::RandomTeamLoader;
use tapu_simu::io::{parse_battle_format, print_engine_info, Cli, Commands};
use tapu_simu::{BattleFormat, BattleState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Battle {
            format,
            player_one,
            player_two,
            max_turns,
            runs,
            verbose,
            log_file,
            team_index,
            config,
            seed,
        } => {
            let battle_format = parse_battle_format(&format)?;
            run_battle(
                battle_format,
                &player_one,
                &player_two,
                max_turns,
                runs,
                verbose,
                log_file,
                team_index,
                config,
                seed,
            )?;
        }

        Commands::ValidateFormat { format } => {
            let battle_format = parse_battle_format(&format)?;
            validate_format(battle_format);
        }

        Commands::Info => {
            print_engine_info();
        }
    }

    Ok(())
}

/// Run a battle with the specified parameters
fn run_battle(
    format: BattleFormat,
    player_one: &str,
    player_two: &str,
    max_turns: u32,
    runs: u32,
    verbose: bool,
    log_file: Option<String>,
    team_index: Option<usize>,
    config_file: Option<String>,
    seed: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tapu_simu::{BattleEnvironment, DamageMaximizer, FirstMovePlayer, RandomPlayer};
    use rand::{SeedableRng, thread_rng};
    use rand::rngs::StdRng;

    // Set random seed if provided
    if let Some(seed_value) = seed {
        let mut rng = StdRng::seed_from_u64(seed_value);
        if verbose {
            println!("Using random seed: {}", seed_value);
        }
    }

    // Load configuration if provided
    if let Some(config_path) = config_file {
        if verbose {
            println!("Loading configuration from: {}", config_path);
        }
        // TODO: Add configuration loading when Config::load is available
        // let _config = tapu_simu::Config::load(&config_path)?;
    }

    println!("Running {} battle(s) in {} format", runs, format);

    if verbose {
        println!("Format details:");
        println!(
            "  Active Pokemon per side: {}",
            format.active_pokemon_count()
        );
        println!(
            "  Supports spread moves: {}",
            format.supports_spread_moves()
        );
        println!("  Allows ally damage: {}", format.allows_ally_damage());
        println!();
    }

    // Create players based on type strings
    let create_player = |player_type: &str, name: String| -> Box<dyn tapu_simu::Player> {
        match player_type.to_lowercase().as_str() {
            "random" => Box::new(RandomPlayer::new(name)),
            "first" | "firstmove" => Box::new(FirstMovePlayer::new(name)),
            "damage" | "damagemax" => Box::new(DamageMaximizer::new(name)),
            _ => {
                eprintln!(
                    "Unknown player type '{}', defaulting to random",
                    player_type
                );
                Box::new(RandomPlayer::new(name))
            }
        }
    };

    let mut results = (0usize, 0usize, 0usize); // (p1_wins, p2_wins, draws)

    for run in 1..=runs {
        if verbose {
            println!("=== Battle {} ===", run);
        }

        // Load teams for the battle
        let mut team_loader = RandomTeamLoader::new();

        let (team_one, team_two) = if let Some(index) = team_index {
            // Use specific team indices for consistent team selection
            let team_one_index = index;
            let team_two_index = (index + 1) % team_loader.get_team_count(&format).unwrap_or(1);

            if verbose && run == 1 {
                println!(
                    "Using team indices: {} and {} for consistent team selection",
                    team_one_index, team_two_index
                );
            }

            let team_one = team_loader
                .get_team_by_index(&format, team_one_index)
                .map_err(|e| {
                    format!(
                        "Failed to load team one at index {} for format {}: {}\nTry using --team-index with a value between 0 and {}",
                        team_one_index, 
                        format,
                        e,
                        team_loader.get_team_count(&format).unwrap_or(0).saturating_sub(1)
                    )
                })?;
            let team_two = team_loader
                .get_team_by_index(&format, team_two_index)
                .map_err(|e| {
                    format!(
                        "Failed to load team two at index {} for format {}: {}\nTry using --team-index with a value between 0 and {}",
                        team_two_index,
                        format, 
                        e,
                        team_loader.get_team_count(&format).unwrap_or(0).saturating_sub(1)
                    )
                })?;

            (team_one, team_two)
        } else {
            // Use random team selection as before
            let team_one = team_loader
                .get_random_team(&format)
                .map_err(|e| format!("Failed to load random team one for format {}: {}\nMake sure the data files for this format are available", format, e))?;
            let team_two = team_loader
                .get_random_team(&format)
                .map_err(|e| format!("Failed to load random team two for format {}: {}\nMake sure the data files for this format are available", format, e))?;

            (team_one, team_two)
        };

        if verbose && run == 1 {
            println!("Team One:");
            for (i, pokemon) in team_one.iter().enumerate() {
                println!("  {}: {} (Level {})", i + 1, pokemon.species, pokemon.level);
            }
            println!("Team Two:");
            for (i, pokemon) in team_two.iter().enumerate() {
                println!("  {}: {} (Level {})", i + 1, pokemon.species, pokemon.level);
            }
            println!();
        }

        // Create battle state using modern API
        let mut state = BattleState::new_with_teams(format.clone(), team_one, team_two);

        if verbose && run == 1 {
            println!("Initialized battle state with format: {}", state.format);
            println!("Turn: {}", state.turn_info.number);
            println!("Weather: {:?}", state.weather());
            println!("Terrain: {:?}", state.terrain());
            println!("Side one team: {} Pokemon", state.get_side(0).map(|s| s.pokemon.len()).unwrap_or(0));
            println!("Side two team: {} Pokemon", state.get_side(1).map(|s| s.pokemon.len()).unwrap_or(0));
            println!();
        }

        // Create players for this battle
        let p1 = create_player(player_one, format!("Player1_{}", run));
        let p2 = create_player(player_two, format!("Player2_{}", run));

        // Create battle environment with log file support using modern pattern
        let mut env = BattleEnvironment::new(p1, p2, max_turns as usize, verbose && runs == 1);
        if let Some(ref log_path) = log_file {
            let battle_log_path = if runs > 1 {
                format!("{}.battle_{}", log_path, run)
            } else {
                log_path.clone()
            };
            env = env.with_log_file(battle_log_path);
        }

        // Run the battle
        let result = env.run_battle(state);

        // Track results
        match result.winner {
            Some(tapu_simu::SideReference::SideOne) => {
                results.0 += 1;
                if verbose || runs == 1 {
                    println!(
                        "Battle {}: Player 1 wins! (Turn {})",
                        run, result.turn_count
                    );
                }
            }
            Some(tapu_simu::SideReference::SideTwo) => {
                results.1 += 1;
                if verbose || runs == 1 {
                    println!(
                        "Battle {}: Player 2 wins! (Turn {})",
                        run, result.turn_count
                    );
                }
            }
            None => {
                results.2 += 1;
                if verbose || runs == 1 {
                    println!(
                        "Battle {}: Draw (Turn limit reached at {})",
                        run, result.turn_count
                    );
                }
            }
        }

        if verbose && runs > 1 {
            println!();
        }
    }

    // Print summary if multiple battles
    if runs > 1 {
        println!("\n=== Battle Summary ===");
        println!("Total battles: {}", runs);
        println!(
            "Player 1 ({}) wins: {} ({:.1}%)",
            player_one,
            results.0,
            (results.0 as f64 / runs as f64) * 100.0
        );
        println!(
            "Player 2 ({}) wins: {} ({:.1}%)",
            player_two,
            results.1,
            (results.1 as f64 / runs as f64) * 100.0
        );
        println!(
            "Draws: {} ({:.1}%)",
            results.2,
            (results.2 as f64 / runs as f64) * 100.0
        );
    }

    Ok(())
}

/// Validate a battle format
fn validate_format(format: BattleFormat) {
    println!("Validating format: {}", format);
    println!(
        "  Active Pokemon per side: {}",
        format.active_pokemon_count()
    );
    println!(
        "  Supports spread moves: {}",
        format.supports_spread_moves()
    );
    println!(
        "  Spread damage multiplier: {:.2}x",
        format.spread_damage_multiplier()
    );
    println!("  Allows ally damage: {}", format.allows_ally_damage());
    println!("  Valid slots: {:?}", format.valid_slots());
    println!("Format validation: ✅ Valid");
}
