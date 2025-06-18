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
//! - `core`: Core types and abstractions (state, instructions, battle format)
//! - `engine`: Battle mechanics implementation
//! - `data`: Pokemon data integration with Pokemon Showdown
//! - `testing`: Testing utilities and framework
//! - `ui`: Testing UI interface
//! 
//! ## Example Usage
//! 
//! ```rust
//! use tapu_simu::{BattleFormat, State, MoveChoice};
//! use tapu_simu::core::move_choice::{MoveIndex, PokemonIndex};
//! use tapu_simu::core::battle_format::{BattlePosition, SideReference};
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
//! ```

// Core modules
pub mod core;
pub mod engine;
pub mod data;
pub mod testing;
pub mod ui;
pub mod generation;
pub mod io;

// Re-exports for convenience
pub use core::battle_format::{BattleFormat, BattlePosition, SideReference};
pub use core::battle_environment::{
    Player, RandomPlayer, FirstMovePlayer, DamageMaximizer,
    BattleEnvironment, BattleResult, TurnInfo, ParallelBattleResults,
    run_parallel_battles_with_states, run_battle_from_state
};
pub use core::instruction::{Instruction, StateInstructions};
pub use core::move_choice::MoveChoice;
pub use core::state::State;
pub use generation::{Generation, GenerationMechanics, GenerationBattleMechanics};

// Test framework re-export
pub use testing::framework::TestFramework;

// Engine module re-exports for testing
pub use engine::combat::damage_calc;

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