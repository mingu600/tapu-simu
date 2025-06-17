//! # Pokemon Showdown Data Conversion
//! 
//! This module provides conversion utilities for Pokemon Showdown data formats.

use crate::data::ps_types::PSMoveTarget;

// This module previously handled conversion from legacy MoveTarget to PSMoveTarget
// Since we now use PSMoveTarget throughout, only string conversion functions remain

/// Convert Pokemon Showdown target string to enum
pub fn ps_target_from_string(target: &str) -> PSMoveTarget {
    match target {
        "normal" => PSMoveTarget::Normal,
        "self" => PSMoveTarget::Self_,
        "adjacentAlly" => PSMoveTarget::AdjacentAlly,
        "adjacentAllyOrSelf" => PSMoveTarget::AdjacentAllyOrSelf,
        "adjacentFoe" => PSMoveTarget::AdjacentFoe,
        "allAdjacentFoes" => PSMoveTarget::AllAdjacentFoes,
        "allAdjacent" => PSMoveTarget::AllAdjacent,
        "all" => PSMoveTarget::All,
        "allyTeam" => PSMoveTarget::AllyTeam,
        "allySide" => PSMoveTarget::AllySide,
        "foeSide" => PSMoveTarget::FoeSide,
        "any" => PSMoveTarget::Any,
        "randomNormal" => PSMoveTarget::RandomNormal,
        "scripted" => PSMoveTarget::Scripted,
        "allies" => PSMoveTarget::Allies,
        _ => PSMoveTarget::Normal, // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_target_from_string() {
        assert_eq!(ps_target_from_string("normal"), PSMoveTarget::Normal);
        assert_eq!(ps_target_from_string("self"), PSMoveTarget::Self_);
        assert_eq!(ps_target_from_string("allAdjacentFoes"), PSMoveTarget::AllAdjacentFoes);
        assert_eq!(ps_target_from_string("unknown"), PSMoveTarget::Normal); // Fallback
    }
}