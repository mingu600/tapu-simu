//! # Generation-Specific Damage Mechanics
//!
//! This module contains generation-specific damage calculation implementations.
//! Each generation has unique mechanics and formulas that require separate handling.

pub mod gen1;
pub mod gen2;
pub mod gen3;
pub mod gen4;
pub mod gen56;
pub mod modern;

pub use gen1::*;
pub use gen2::*;
pub use gen3::*;
pub use gen4::*;
pub use gen56::*;
pub use modern::*;