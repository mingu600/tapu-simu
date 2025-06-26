//! Format-specific damage modifiers
//!
//! This module handles damage modifications based on battle format,
//! such as spread move penalties in multi-Pokemon formats.

use crate::core::battle_format::BattleFormat;

/// Calculate spread move damage modifier based on format and target count
pub fn get_spread_move_modifier(
    format: &BattleFormat,
    target_count: usize,
) -> f32 {
    // Spread moves only have damage reduction in multi-Pokemon formats
    // and only when actually hitting multiple targets
    if format.supports_spread_moves() && target_count > 1 {
        0.75 // 25% damage reduction for spread moves hitting multiple targets
    } else {
        1.0
    }
}