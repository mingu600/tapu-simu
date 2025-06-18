//! # Tapu Simu CLI
//! 
//! Command-line interface for Tapu Simu.

use clap::Parser;
use tapu_simu::{State, BattleFormat};
use tapu_simu::io::{Cli, Commands, parse_battle_format, print_engine_info};

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
        } => {
            let battle_format = parse_battle_format(&format)?;
            run_battle(battle_format, &player_one, &player_two, max_turns, runs, verbose)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    use tapu_simu::{RandomPlayer, FirstMovePlayer, DamageMaximizer, BattleEnvironment, run_battle_from_state};
    
    println!("Running {} battle(s) in {} format", runs, format);
    
    if verbose {
        println!("Format details:");
        println!("  Active Pokemon per side: {}", format.active_pokemon_count());
        println!("  Supports spread moves: {}", format.supports_spread_moves());
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
                eprintln!("Unknown player type '{}', defaulting to random", player_type);
                Box::new(RandomPlayer::new(name))
            }
        }
    };

    let mut results = (0usize, 0usize, 0usize); // (p1_wins, p2_wins, draws)

    for run in 1..=runs {
        if verbose {
            println!("=== Battle {} ===", run);
        }
        
        // Create initial battle state with the format
        let mut state = State::new(format.clone());
        
        // TODO: In a real implementation, we'd load teams from JSON or other sources
        // For now, create a minimal state that can at least test the battle flow
        if verbose && run == 1 {
            println!("Initialized battle state with format: {}", state.format);
            println!("Turn: {}", state.turn);
            println!("Weather: {:?}", state.weather);
            println!("Terrain: {:?}", state.terrain);
            println!();
        }
        
        // Create players for this battle
        let p1 = create_player(player_one, format!("Player1_{}", run));
        let p2 = create_player(player_two, format!("Player2_{}", run));
        
        // Run the battle
        let result = run_battle_from_state(
            state,
            p1,
            p2,
            max_turns as usize,
            verbose && runs == 1, // Only verbose for single runs
        );
        
        // Track results
        match result.winner {
            Some(tapu_simu::SideReference::SideOne) => {
                results.0 += 1;
                if verbose || runs == 1 {
                    println!("Battle {}: Player 1 wins! (Turn {})", run, result.turn_count);
                }
            }
            Some(tapu_simu::SideReference::SideTwo) => {
                results.1 += 1;
                if verbose || runs == 1 {
                    println!("Battle {}: Player 2 wins! (Turn {})", run, result.turn_count);
                }
            }
            None => {
                results.2 += 1;
                if verbose || runs == 1 {
                    println!("Battle {}: Draw (Turn limit reached at {})", run, result.turn_count);
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
        println!("Player 1 ({}) wins: {} ({:.1}%)", player_one, results.0, (results.0 as f64 / runs as f64) * 100.0);
        println!("Player 2 ({}) wins: {} ({:.1}%)", player_two, results.1, (results.1 as f64 / runs as f64) * 100.0);
        println!("Draws: {} ({:.1}%)", results.2, (results.2 as f64 / runs as f64) * 100.0);
    }

    Ok(())
}

/// Validate a battle format
fn validate_format(format: BattleFormat) {
    println!("Validating format: {}", format);
    println!("  Active Pokemon per side: {}", format.active_pokemon_count());
    println!("  Supports spread moves: {}", format.supports_spread_moves());
    println!("  Spread damage multiplier: {:.2}x", format.spread_damage_multiplier());
    println!("  Allows ally damage: {}", format.allows_ally_damage());
    println!("  Valid slots: {:?}", format.valid_slots());
    println!("Format validation: ✅ Valid");
}
