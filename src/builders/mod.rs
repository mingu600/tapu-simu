pub mod format;
pub mod battle;
pub mod team;
pub mod traits;

// Primary builders (simplified names)
pub use battle::{BattleBuilder, BattleConfig, Battle};
pub use team::{TeamBuilder, PokemonBuilder, EVsConfig, IVsConfig};
pub use format::FormatBuilder;
pub use traits::{Builder, BuilderError, ValidationContext};