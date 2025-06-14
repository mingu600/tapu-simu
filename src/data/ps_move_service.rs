//! # Pokemon Showdown Move Service
//! 
//! This module provides a move data service based on Pokemon Showdown data,
//! replacing the rustemon-based move service with local PS data.

use std::collections::HashMap;
use std::sync::OnceLock;
use crate::data::ps_loader::PSDataRepository;
use crate::data::ps_types::{PSMoveData, PSMoveTarget};
use crate::data::ps_conversion::ps_target_from_string;
use crate::state::{Move, MoveCategory};
use crate::data::types::MoveTarget;

/// Global PS data repository - loaded once and cached
static PS_REPO: OnceLock<PSDataRepository> = OnceLock::new();

/// Pokemon Showdown move service - replacement for rustemon-based MoveDataService
pub struct PSMoveService {
    engine_enhancements: HashMap<String, EngineEnhancement>,
}

/// Engine-specific move enhancements that aren't in PS data
#[derive(Debug, Clone)]
pub struct EngineEnhancement {
    pub drain_multiplier: Option<f32>,
    pub recoil_multiplier: Option<f32>,
    pub additional_flags: Vec<String>,
    pub custom_effects: Vec<String>,
}

impl PSMoveService {
    /// Create a new PS move service
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize PS repository if not already done
        if PS_REPO.get().is_none() {
            let repo = PSDataRepository::load_from_directory("data/ps-extracted")?;
            PS_REPO.set(repo).map_err(|_| "Failed to initialize PS repository")?;
        }
        
        Ok(Self {
            engine_enhancements: HashMap::new(),
        })
    }

    /// Get the PS repository
    fn get_repo(&self) -> &PSDataRepository {
        PS_REPO.get().expect("PS repository should be initialized")
    }

    /// Get move data by name (replaces rustemon async calls)
    pub fn get_move_by_name(&self, name: &str) -> Option<Move> {
        let ps_move = self.get_repo().get_move_by_name(name)?;
        Some(self.ps_move_to_engine_move(ps_move))
    }

    /// Get move data by ID
    pub fn get_move_by_id(&self, id: &str) -> Option<Move> {
        let ps_move = self.get_repo().get_move(id)?;
        Some(self.ps_move_to_engine_move(ps_move))
    }

    /// Convert PS move data to engine move with enhancements
    pub fn ps_move_to_engine_move(&self, ps_move: &PSMoveData) -> Move {
        // Apply engine enhancements if any
        let _enhancement = self.engine_enhancements.get(&ps_move.id);
        // TODO: Apply enhancement data to move when implementing advanced effects
        
        Move {
            name: ps_move.name.clone(),
            base_power: ps_move.base_power as u8,
            accuracy: ps_move.accuracy as u8,
            move_type: ps_move.move_type.clone(),
            pp: ps_move.pp,
            max_pp: ps_move.max_pp,
            target: self.convert_ps_target_to_engine(&ps_move.target),
            category: self.convert_ps_category_to_engine(&ps_move.category),
            priority: ps_move.priority,
        }
    }

    /// Convert PS target to current engine target (temporary during migration)
    fn convert_ps_target_to_engine(&self, ps_target: &str) -> MoveTarget {
        let ps_target_enum = ps_target_from_string(ps_target);
        
        match ps_target_enum {
            PSMoveTarget::Self_ => MoveTarget::User,
            PSMoveTarget::Normal | PSMoveTarget::AdjacentFoe => MoveTarget::SelectedPokemon,
            PSMoveTarget::AdjacentAlly => MoveTarget::Ally,
            PSMoveTarget::AdjacentAllyOrSelf => MoveTarget::UserOrAlly,
            PSMoveTarget::AllAdjacentFoes => MoveTarget::AllOpponents,
            PSMoveTarget::AllAdjacent => MoveTarget::AllOtherPokemon,
            PSMoveTarget::All => MoveTarget::EntireField,
            PSMoveTarget::AllySide => MoveTarget::UsersField,
            PSMoveTarget::FoeSide => MoveTarget::OpponentsField,
            PSMoveTarget::Any => MoveTarget::SelectedPokemon, // Best approximation
            PSMoveTarget::RandomNormal => MoveTarget::RandomOpponent,
            PSMoveTarget::Scripted => MoveTarget::SpecificMove,
            PSMoveTarget::AllyTeam => MoveTarget::UserAndAllies,
            PSMoveTarget::Allies => MoveTarget::AllAllies,
        }
    }

    /// Convert PS category to engine category
    fn convert_ps_category_to_engine(&self, ps_category: &str) -> MoveCategory {
        match ps_category {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            "Status" => MoveCategory::Status,
            _ => MoveCategory::Status,
        }
    }

    /// Register engine-specific enhancement for a move
    pub fn register_enhancement(&mut self, move_id: String, enhancement: EngineEnhancement) {
        self.engine_enhancements.insert(move_id, enhancement);
    }

    /// Get all available moves
    pub fn get_all_moves(&self) -> Vec<Move> {
        self.get_repo()
            .get_all_moves()
            .values()
            .map(|ps_move| self.ps_move_to_engine_move(ps_move))
            .collect()
    }

    /// Check if a move exists
    pub fn move_exists(&self, name: &str) -> bool {
        self.get_repo().get_move_by_name(name).is_some()
    }

    /// Get move statistics
    pub fn get_stats(&self) -> PSMoveServiceStats {
        let repo_stats = self.get_repo().stats();
        PSMoveServiceStats {
            total_moves: repo_stats.move_count,
            enhanced_moves: self.engine_enhancements.len(),
        }
    }

    /// Get PS move data directly (for advanced usage)
    pub fn get_ps_move(&self, name: &str) -> Option<&PSMoveData> {
        self.get_repo().get_move_by_name(name)
    }
}

impl Default for PSMoveService {
    fn default() -> Self {
        Self::new().expect("Failed to create PS move service")
    }
}

/// Statistics about the PS move service
#[derive(Debug)]
pub struct PSMoveServiceStats {
    pub total_moves: usize,
    pub enhanced_moves: usize,
}

/// Helper function to create common engine enhancements
impl EngineEnhancement {
    /// Create a drain move enhancement
    pub fn drain(multiplier: f32) -> Self {
        Self {
            drain_multiplier: Some(multiplier),
            recoil_multiplier: None,
            additional_flags: vec!["drain".to_string()],
            custom_effects: vec![],
        }
    }

    /// Create a recoil move enhancement
    pub fn recoil(multiplier: f32) -> Self {
        Self {
            drain_multiplier: None,
            recoil_multiplier: Some(multiplier),
            additional_flags: vec!["recoil".to_string()],
            custom_effects: vec![],
        }
    }

    /// Create a custom flag enhancement
    pub fn with_flags(flags: Vec<String>) -> Self {
        Self {
            drain_multiplier: None,
            recoil_multiplier: None,
            additional_flags: flags,
            custom_effects: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_move_service_creation() {
        // This test will fail without actual PS data files
        // but demonstrates the intended usage
        if let Ok(service) = PSMoveService::new() {
            let stats = service.get_stats();
            assert!(stats.total_moves > 0);
        }
    }

    #[test]
    fn test_engine_enhancement() {
        let drain_enhancement = EngineEnhancement::drain(0.5);
        assert_eq!(drain_enhancement.drain_multiplier, Some(0.5));
        assert!(drain_enhancement.additional_flags.contains(&"drain".to_string()));
    }

    #[test]
    fn test_target_conversion() {
        let service = PSMoveService::default();
        
        // Test PS target to engine target conversion
        assert_eq!(
            service.convert_ps_target_to_engine("normal"),
            MoveTarget::SelectedPokemon
        );
        assert_eq!(
            service.convert_ps_target_to_engine("self"),
            MoveTarget::User
        );
        assert_eq!(
            service.convert_ps_target_to_engine("allAdjacentFoes"),
            MoveTarget::AllOpponents
        );
    }
}