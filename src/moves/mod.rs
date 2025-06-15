//! Move execution system
//! 
//! This module implements Pokemon Showdown's move execution pipeline
//! with exact fidelity to the original implementation.

pub mod execution;
pub mod damage;
pub mod targeting;

pub use execution::*;
pub use damage::*;
pub use targeting::*;