//! # Pokemon Showdown Data Conversion
//! 
//! This module provides conversion between rustemon/legacy formats and 
//! Pokemon Showdown data formats.

use crate::data::types::MoveTarget;
use crate::data::ps_types::PSMoveTarget;

/// Convert rustemon/PokeAPI move target to Pokemon Showdown target
pub fn rustemon_to_ps_target(rustemon_target: MoveTarget) -> PSMoveTarget {
    match rustemon_target {
        // Direct mappings
        MoveTarget::User => PSMoveTarget::Self_,
        MoveTarget::Ally => PSMoveTarget::AdjacentAlly,
        MoveTarget::UserOrAlly => PSMoveTarget::AdjacentAllyOrSelf,
        MoveTarget::RandomOpponent => PSMoveTarget::RandomNormal,
        MoveTarget::AllOpponents => PSMoveTarget::AllAdjacentFoes,
        MoveTarget::EntireField => PSMoveTarget::All,
        MoveTarget::UsersField => PSMoveTarget::AllySide,
        MoveTarget::OpponentsField => PSMoveTarget::FoeSide,
        MoveTarget::SpecificMove => PSMoveTarget::Scripted,
        
        // Complex mappings
        MoveTarget::SelectedPokemon | MoveTarget::SelectedPokemonMeFirst => PSMoveTarget::Normal,
        MoveTarget::AllOtherPokemon => PSMoveTarget::AllAdjacent, // Closest match
        MoveTarget::UserAndAllies => PSMoveTarget::Allies, // Not exact but closest
        MoveTarget::AllPokemon => PSMoveTarget::All, // Field effect is closest
        MoveTarget::AllAllies => PSMoveTarget::Allies,
        MoveTarget::FaintingPokemon => PSMoveTarget::Scripted, // Special handling needed
    }
}

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
    fn test_rustemon_to_ps_conversion() {
        assert_eq!(rustemon_to_ps_target(MoveTarget::User), PSMoveTarget::Self_);
        assert_eq!(rustemon_to_ps_target(MoveTarget::SelectedPokemon), PSMoveTarget::Normal);
        assert_eq!(rustemon_to_ps_target(MoveTarget::AllOpponents), PSMoveTarget::AllAdjacentFoes);
        assert_eq!(rustemon_to_ps_target(MoveTarget::EntireField), PSMoveTarget::All);
    }

    #[test]
    fn test_ps_target_from_string() {
        assert_eq!(ps_target_from_string("normal"), PSMoveTarget::Normal);
        assert_eq!(ps_target_from_string("self"), PSMoveTarget::Self_);
        assert_eq!(ps_target_from_string("allAdjacentFoes"), PSMoveTarget::AllAdjacentFoes);
        assert_eq!(ps_target_from_string("unknown"), PSMoveTarget::Normal); // Fallback
    }
}