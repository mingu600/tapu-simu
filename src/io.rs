//! # Input/Output Module
//!
//! This module provides CLI interface and subcommands for Tapu Simu.

use crate::core::battle_format::BattleFormat;
use clap::{Parser, Subcommand};

/// Tapu Simu CLI
#[derive(Parser)]
#[command(name = "tapu-simu")]
#[command(about = "A format-aware Pokemon battle simulator")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Run a battle between two AI players
    Battle {
        /// Battle format to use
        #[arg(short, long, default_value = "gen9randombattle")]
        format: String,

        /// Player 1 type
        #[arg(short = '1', long, default_value = "random")]
        player_one: String,

        /// Player 2 type
        #[arg(short = '2', long, default_value = "random")]
        player_two: String,

        /// Maximum number of turns
        #[arg(short, long, default_value_t = 100)]
        max_turns: u32,

        /// Number of battles to run
        #[arg(short, long, default_value_t = 1)]
        runs: u32,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Log file path for detailed battle output
        #[arg(short, long)]
        log_file: Option<String>,
    },

    /// Validate battle format configuration
    ValidateFormat {
        /// Format to validate
        format: String,
    },

    /// Show engine information
    Info,
}

/// Parse battle format from string
pub fn parse_battle_format(format_str: &str) -> Result<BattleFormat, String> {
    use crate::core::battle_format::FormatType;
    use crate::generation::Generation;

    match format_str.to_lowercase().as_str() {
        "singles" | "single" => Ok(BattleFormat::new(
            "Singles".to_string(),
            Generation::Gen9,
            FormatType::Singles,
        )),
        "doubles" | "double" => Ok(BattleFormat::new(
            "Doubles".to_string(),
            Generation::Gen9,
            FormatType::Doubles,
        )),
        "vgc" => Ok(BattleFormat::vgc2024()),
        "triples" | "triple" => Ok(BattleFormat::new(
            "Triples".to_string(),
            Generation::Gen9,
            FormatType::Triples,
        )),
        "gen9ou" | "gen9-ou" => Ok(BattleFormat::gen9_ou()),
        "gen4ou" | "gen4-ou" => Ok(BattleFormat::gen4_ou()),
        "gen9randombattle" | "gen9random" | "gen 9 random battle" => {
            Ok(BattleFormat::gen9_random_battle())
        }
        "gen9randomdoubles" | "gen9randomdoublesbattle" | "gen 9 random doubles" => {
            Ok(BattleFormat::gen9_random_doubles())
        }
        "gen8randombattle" | "gen8random" | "gen 8 random battle" => {
            Ok(BattleFormat::gen8_random_battle())
        }
        "gen8randomdoubles" | "gen8randomdoublesbattle" | "gen 8 random doubles" => {
            Ok(BattleFormat::gen8_random_doubles())
        }
        "gen7randombattle" | "gen7random" | "gen 7 random battle" => {
            Ok(BattleFormat::gen7_random_battle())
        }
        _ => Err(format!("Unknown format: {}", format_str)),
    }
}

/// Print simulator information
pub fn print_engine_info() {
    println!("Tapu Simu");
    println!("=========");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Format-aware Pokemon battle simulator");
    println!();
    println!("Supported formats:");
    println!("  - Singles (1v1)");
    println!("  - Doubles (2v2)");
    println!("  - VGC (2v2 with VGC rules)");
    println!("  - Triples (3v3)");
    println!();
    println!("Features:");
    println!("  - Position-based targeting");
    println!("  - Format-aware battle mechanics");
    println!("  - Pokemon Showdown data integration");

    println!("  - Terastallization support (Gen 9+)");

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            parse_battle_format("singles").unwrap(),
            BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles)
        );
        assert_eq!(
            parse_battle_format("doubles").unwrap(),
            BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles)
        );
        assert_eq!(parse_battle_format("vgc").unwrap(), BattleFormat::vgc2024());
        assert_eq!(
            parse_battle_format("triples").unwrap(),
            BattleFormat::new("Triples".to_string(), Generation::Gen9, FormatType::Triples)
        );

        assert!(parse_battle_format("invalid").is_err());
    }
}
