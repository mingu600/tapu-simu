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
    _player_one: &str,
    _player_two: &str,
    _max_turns: u32,
    runs: u32,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running {} battle(s) in {} format", runs, format);
    
    if verbose {
        println!("Format details:");
        println!("  Active Pokemon per side: {}", format.active_pokemon_count());
        println!("  Supports spread moves: {}", format.supports_spread_moves());
        println!("  Allows ally damage: {}", format.allows_ally_damage());
        println!();
    }

    for run in 1..=runs {
        if verbose {
            println!("=== Battle {} ===", run);
        }
        
        let mut state = State::new(format.clone());
        
        if verbose {
            println!("Initialized battle state with format: {}", state.format);
            println!("Turn: {}", state.turn);
            println!("Weather: {:?}", state.weather);
            println!("Terrain: {:?}", state.terrain);
        }
        
        // TODO: Implement actual battle simulation
        println!("Battle {} completed (placeholder)", run);
        
        if verbose {
            println!();
        }
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
