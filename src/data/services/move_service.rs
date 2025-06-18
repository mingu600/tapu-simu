//! # Pokemon Showdown Move Service
//! 
//! This module provides a move data service based on Pokemon Showdown data,
//! replacing the rustemon-based move service with local PS data.

use std::collections::HashMap;
use std::sync::OnceLock;
use crate::data::loader::PSDataRepository;
use crate::data::ps_types::PSMoveData;
use crate::data::conversion::ps_target_from_string;
use crate::core::state::{Move, MoveCategory};

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
    fn get_repo(&self) -> Option<&PSDataRepository> {
        PS_REPO.get()
    }
    
    /// Check if PS repository is initialized
    pub fn is_initialized(&self) -> bool {
        PS_REPO.get().is_some()
    }

    /// Get move data by name (replaces rustemon async calls)
    pub fn get_move_by_name(&self, name: &str) -> Option<Move> {
        let repo = self.get_repo()?;
        let ps_move = repo.get_move_by_name(name)?;
        Some(self.ps_move_to_engine_move(ps_move))
    }

    /// Get move data by ID
    pub fn get_move_by_id(&self, id: &str) -> Option<Move> {
        let repo = self.get_repo()?;
        let ps_move = repo.get_move(id)?;
        Some(self.ps_move_to_engine_move(ps_move))
    }

    /// Convert PS move data to engine move with enhancements
    pub fn ps_move_to_engine_move(&self, ps_move: &PSMoveData) -> Move {
        // PS data already includes drain and recoil data - no need for manual enhancements
        
        Move {
            name: ps_move.name.clone(),
            base_power: ps_move.base_power as u8,
            accuracy: ps_move.accuracy as u8,
            move_type: ps_move.move_type.clone(),
            pp: ps_move.pp,
            max_pp: ps_move.max_pp,
            target: ps_target_from_string(&ps_move.target),
            category: self.convert_ps_category_to_engine(&ps_move.category),
            priority: ps_move.priority,
        }
    }

    /// Get drain ratio for a move (from PS data)
    pub fn get_drain_ratio(&self, move_name: &str) -> Option<f32> {
        let repo = self.get_repo()?;
        let ps_move = repo.get_move_by_name(move_name)?;
        if let Some(drain) = &ps_move.drain {
            Some(drain[0] as f32 / drain[1] as f32)
        } else {
            None
        }
    }

    /// Get recoil ratio for a move (from PS data)
    pub fn get_recoil_ratio(&self, move_name: &str) -> Option<f32> {
        let repo = self.get_repo()?;
        let ps_move = repo.get_move_by_name(move_name)?;
        if let Some(recoil) = &ps_move.recoil {
            Some(recoil[0] as f32 / recoil[1] as f32)
        } else {
            None
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
        if let Some(repo) = self.get_repo() {
            repo.get_all_moves()
                .values()
                .map(|ps_move| self.ps_move_to_engine_move(ps_move))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a move exists
    pub fn move_exists(&self, name: &str) -> bool {
        if let Some(repo) = self.get_repo() {
            repo.get_move_by_name(name).is_some()
        } else {
            false
        }
    }

    /// Get move statistics
    pub fn get_stats(&self) -> PSMoveServiceStats {
        if let Some(repo) = self.get_repo() {
            let repo_stats = repo.stats();
            PSMoveServiceStats {
                total_moves: repo_stats.move_count,
                enhanced_moves: self.engine_enhancements.len(),
            }
        } else {
            PSMoveServiceStats {
                total_moves: 0,
                enhanced_moves: self.engine_enhancements.len(),
            }
        }
    }

    /// Get PS move data directly (for advanced usage)
    pub fn get_ps_move(&self, name: &str) -> Option<&PSMoveData> {
        let repo = self.get_repo()?;
        repo.get_move_by_name(name)
    }
}

impl Default for PSMoveService {
    fn default() -> Self {
        // In tests or when PS data isn't available, create a service with empty data
        Self::new().unwrap_or_else(|_| {
            PSMoveService {
                engine_enhancements: HashMap::new(),
            }
        })
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
    fn test_drain_recoil_ratios() {
        if let Ok(service) = PSMoveService::new() {
            // Test drain ratio parsing
            if let Some(absorb_drain) = service.get_drain_ratio("absorb") {
                assert!((absorb_drain - 0.5).abs() < 0.01); // Should be 1/2 = 0.5
            }
            
            // Test recoil ratio parsing  
            if let Some(doubleedge_recoil) = service.get_recoil_ratio("doubleedge") {
                assert!((doubleedge_recoil - 0.33).abs() < 0.01); // Should be 33/100 = 0.33
            }
        }
    }
}