//! # Pokemon Showdown Data Conversion
//! 
//! This module provides conversion utilities for Pokemon Showdown data formats.

use crate::data::showdown_types::MoveTarget;

// This module previously handled conversion from legacy MoveTarget to MoveTarget
// Since we now use MoveTarget throughout, only string conversion functions remain

/// Convert Pokemon Showdown target string to enum
pub fn target_from_string(target: &str) -> MoveTarget {
    match target {
        "normal" => MoveTarget::Normal,
        "self" => MoveTarget::Self_,
        "adjacentAlly" => MoveTarget::AdjacentAlly,
        "adjacentAllyOrSelf" => MoveTarget::AdjacentAllyOrSelf,
        "adjacentFoe" => MoveTarget::AdjacentFoe,
        "allAdjacentFoes" => MoveTarget::AllAdjacentFoes,
        "allAdjacent" => MoveTarget::AllAdjacent,
        "all" => MoveTarget::All,
        "allyTeam" => MoveTarget::AllyTeam,
        "allySide" => MoveTarget::AllySide,
        "foeSide" => MoveTarget::FoeSide,
        "any" => MoveTarget::Any,
        "randomNormal" => MoveTarget::RandomNormal,
        "scripted" => MoveTarget::Scripted,
        "allies" => MoveTarget::Allies,
        _ => MoveTarget::Normal, // Default fallback
    }
}

