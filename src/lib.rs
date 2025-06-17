//! # Tapu Simu
//! 
//! A format-aware Pokemon battle simulator designed for multi-format support.
//! This simulator supports Singles, Doubles, and VGC formats with position-based
//! targeting and comprehensive battle mechanics.
//! 
//! ## Key Features
//! 
//! - **Multi-Format Support**: Singles, Doubles, VGC formats
//! - **Position-Based Targeting**: All moves use explicit position targeting
//! - **Format-Aware Architecture**: Battle logic adapts to the active format
//! - **Modern Design**: Built from the ground up with V2 principles
//! - **No Legacy Compatibility**: Clean, focused implementation
//! 
//! ## Architecture Overview
//! 
//! - `battle_format`: Format definitions and position management
//! - `engine`: Core battle mechanics (generation-specific)
//! - `instruction`: Battle instruction system with position tracking
//! - `move_choice`: Format-aware move choice system
//! - `state`: Battle state representation
//! - `data`: Pokemon data integration with rustemon/PokeAPI
//! 
//! ## Example Usage
//! 
//! ```rust
//! use tapu_simu::{BattleFormat, State, MoveChoice, InstructionGenerator};
//! use tapu_simu::move_choice::{MoveIndex, PokemonIndex};
//! use tapu_simu::battle_format::{BattlePosition, SideReference};
//! 
//! // Create a new singles battle
//! let mut state = State::new(BattleFormat::gen9_ou());
//! 
//! // Create move choices
//! let move1 = MoveChoice::new_move(
//!     MoveIndex::M0, 
//!     vec![BattlePosition::new(SideReference::SideTwo, 0)]
//! );
//! let move2 = MoveChoice::new_move(
//!     MoveIndex::M0, 
//!     vec![BattlePosition::new(SideReference::SideOne, 0)]
//! );
//! 
//! // Generate instructions for moves
//! let generator = InstructionGenerator::new(BattleFormat::gen9_ou());
//! let instructions = generator.generate_instructions(&mut state, &move1, &move2);
//! ```

// Generation-specific modules
#[cfg(feature = "gen1")]
#[path = "gen1/mod.rs"]
pub mod engine;

#[cfg(feature = "gen2")]
#[path = "gen2/mod.rs"]
pub mod engine;

#[cfg(feature = "gen3")]
#[path = "gen3/mod.rs"]
pub mod engine;

// Default generation (gen4-9)
#[cfg(not(any(feature = "gen1", feature = "gen2", feature = "gen3")))]
#[path = "genx/mod.rs"]
pub mod engine;

// Core V2 modules
pub mod battle_format;
pub mod instruction;
pub mod move_choice;
pub mod state;
pub mod data;
pub mod io;
pub mod generation;

// Test framework (available for integration tests)
pub mod test_framework;

// Re-exports for convenience
pub use battle_format::{BattleFormat, BattlePosition};
pub use instruction::{Instruction, StateInstructions, InstructionGenerator};
pub use move_choice::MoveChoice;
pub use state::State;
pub use generation::{Generation, GenerationMechanics, GenerationBattleMechanics};

// Ensure only one generation is enabled
#[cfg(all(feature = "gen1", feature = "gen2"))]
compile_error!("Features 'gen1' and 'gen2' cannot be used together");

#[cfg(all(feature = "gen1", feature = "gen3"))]
compile_error!("Features 'gen1' and 'gen3' cannot be used together");

#[cfg(all(feature = "gen2", feature = "gen3"))]
compile_error!("Features 'gen2' and 'gen3' cannot be used together");

// Add more generation mutual exclusion checks as needed...

// Terastallization requires Gen 9
#[cfg(all(feature = "terastallization", not(feature = "gen9")))]
compile_error!("Feature 'terastallization' requires 'gen9'");

// Macro for enum generation (copied from V1)
#[macro_export]
macro_rules! define_enum_with_from_str {
    // Case when a default variant is provided
    (
        #[repr($repr:ident)]
        $(#[$meta:meta])*
        $name:ident {
            $($variant:ident),+ $(,)?
        },
        default = $default_variant:ident
    ) => {
        #[repr($repr)]
        $(#[$meta])*
        pub enum $name {
            $($variant),+
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                match input.to_uppercase().as_str() {
                    $(
                        stringify!($variant) => Ok($name::$variant),
                    )+
                    _ => Ok($name::$default_variant),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl From<$repr> for $name {
            fn from(value: $repr) -> $name {
                match value {
                    $(
                        x if x == $name::$variant as $repr => $name::$variant,
                    )+
                    _ => $name::$default_variant,
                }
            }
        }
        impl Into<$repr> for $name {
            fn into(self) -> $repr {
                self as $repr
            }
        }
    };

    // Case when no default variant is provided
    (
        #[repr($repr:ident)]
        $(#[$meta:meta])*
        $name:ident {
            $($variant:ident),+ $(,)?
        }
    ) => {
        #[repr($repr)]
        $(#[$meta])*
        pub enum $name {
            $($variant),+
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                match input.to_uppercase().as_str() {
                    $(
                        stringify!($variant) => Ok($name::$variant),
                    )+
                    _ => panic!("Invalid {}: {}", stringify!($name), input.to_uppercase().as_str()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl From<$repr> for $name {
            fn from(value: $repr) -> $name {
                match value {
                    $(
                        x if x == $name::$variant as $repr => $name::$variant,
                    )+
                    _ => panic!("Invalid {}: {}", stringify!($name), value),
                }
            }
        }
        impl Into<$repr> for $name {
            fn into(self) -> $repr {
                self as $repr
            }
        }
    };
}