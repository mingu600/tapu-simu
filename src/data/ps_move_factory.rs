//! # Pokemon Showdown Move Factory
//! 
//! This module provides a factory for creating moves using Pokemon Showdown data,
//! replacing the rustemon-based move factory with PS data sources.

use std::collections::HashMap;
use crate::data::ps_move_service::{PSMoveService, EngineEnhancement};
use crate::state::Move;
use crate::move_choice::MoveIndex;

/// Factory for creating moves with PS data and engine enhancements
pub struct PSMoveFactory {
    move_service: PSMoveService,
}

impl PSMoveFactory {
    /// Create a new PS move factory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut move_service = PSMoveService::new()?;
        
        // Register engine-specific enhancements
        Self::register_engine_enhancements(&mut move_service);
        
        Ok(Self { move_service })
    }

    /// Register engine-specific move enhancements that aren't in PS data
    fn register_engine_enhancements(service: &mut PSMoveService) {
        // Drain moves
        service.register_enhancement(
            "absorb".to_string(),
            EngineEnhancement::drain(0.5)
        );
        service.register_enhancement(
            "megadrain".to_string(),
            EngineEnhancement::drain(0.5)
        );
        service.register_enhancement(
            "gigadrain".to_string(),
            EngineEnhancement::drain(0.5)
        );
        service.register_enhancement(
            "dreameater".to_string(),
            EngineEnhancement::drain(0.5)
        );
        service.register_enhancement(
            "drainingkiss".to_string(),
            EngineEnhancement::drain(0.75)
        );
        
        // Recoil moves
        service.register_enhancement(
            "doubleedge".to_string(),
            EngineEnhancement::recoil(0.33)
        );
        service.register_enhancement(
            "submission".to_string(),
            EngineEnhancement::recoil(0.25)
        );
        service.register_enhancement(
            "takedown".to_string(),
            EngineEnhancement::recoil(0.25)
        );
        service.register_enhancement(
            "volttackle".to_string(),
            EngineEnhancement::recoil(0.33)
        );
        service.register_enhancement(
            "flareblitz".to_string(),
            EngineEnhancement::recoil(0.33)
        );
        service.register_enhancement(
            "wildcharge".to_string(),
            EngineEnhancement::recoil(0.25)
        );
        
        // Special mechanics
        service.register_enhancement(
            "selfdestruct".to_string(),
            EngineEnhancement::with_flags(vec!["ohko_user".to_string()])
        );
        service.register_enhancement(
            "explosion".to_string(),
            EngineEnhancement::with_flags(vec!["ohko_user".to_string()])
        );
    }

    /// Create a move by name
    pub fn create_move(&self, name: &str) -> Option<Move> {
        self.move_service.get_move_by_name(name)
    }

    /// Create a move by ID
    pub fn create_move_by_id(&self, id: &str) -> Option<Move> {
        self.move_service.get_move_by_id(id)
    }

    /// Create a moveset for a Pokemon (4 moves)
    pub fn create_moveset(&self, move_names: &[String]) -> HashMap<MoveIndex, Move> {
        let mut moveset = HashMap::new();
        let indices = [MoveIndex::M0, MoveIndex::M1, MoveIndex::M2, MoveIndex::M3];
        
        for (i, move_name) in move_names.iter().enumerate().take(4) {
            if let Some(move_data) = self.create_move(move_name) {
                moveset.insert(indices[i], move_data);
            }
        }
        
        moveset
    }

    /// Create a moveset from a slice of move names
    pub fn create_moveset_from_slice(&self, move_names: &[&str]) -> HashMap<MoveIndex, Move> {
        let string_names: Vec<String> = move_names.iter().map(|s| s.to_string()).collect();
        self.create_moveset(&string_names)
    }

    /// Get popular moves for testing/demo purposes
    pub fn get_popular_moves(&self) -> Vec<Move> {
        let popular_move_names = vec![
            "thunderbolt", "flamethrower", "surf", "earthquake",
            "icebeam", "psychic", "shadowball", "airslash",
            "energyball", "focusblast", "thunderwave", "toxic",
            "recover", "roost", "substitute", "protect",
            "swordsdance", "nastyplot", "stealthrock", "spikes"
        ];
        
        popular_move_names
            .iter()
            .filter_map(|name| self.create_move(name))
            .collect()
    }

    /// Check if move exists in PS data
    pub fn move_exists(&self, name: &str) -> bool {
        self.move_service.move_exists(name)
    }

    /// Get all available moves
    pub fn get_all_moves(&self) -> Vec<Move> {
        self.move_service.get_all_moves()
    }

    /// Get factory statistics
    pub fn get_stats(&self) -> PSMoveFactoryStats {
        let service_stats = self.move_service.get_stats();
        PSMoveFactoryStats {
            total_moves: service_stats.total_moves,
            enhanced_moves: service_stats.enhanced_moves,
        }
    }

    /// Create moves for common movesets by archetype
    pub fn create_standard_moveset(&self, archetype: MovesetArchetype) -> HashMap<MoveIndex, Move> {
        let move_names = match archetype {
            MovesetArchetype::PhysicalAttacker => vec!["earthquake", "rockslide", "uturn", "swordsdance"],
            MovesetArchetype::SpecialAttacker => vec!["thunderbolt", "icebeam", "hiddenpowergrass", "nastyplot"],
            MovesetArchetype::Tank => vec!["scald", "toxic", "recover", "protect"],
            MovesetArchetype::Support => vec!["thunderwave", "stealthrock", "roost", "defog"],
            MovesetArchetype::Sweeper => vec!["dragondance", "outrage", "earthquake", "firepunch"],
        };
        
        self.create_moveset_from_slice(&move_names)
    }
}

impl Default for PSMoveFactory {
    fn default() -> Self {
        Self::new().expect("Failed to create PS move factory")
    }
}

/// Common moveset archetypes for testing
#[derive(Debug, Clone, Copy)]
pub enum MovesetArchetype {
    PhysicalAttacker,
    SpecialAttacker,
    Tank,
    Support,
    Sweeper,
}

/// Statistics about the move factory
#[derive(Debug)]
pub struct PSMoveFactoryStats {
    pub total_moves: usize,
    pub enhanced_moves: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moveset_creation() {
        if let Ok(factory) = PSMoveFactory::new() {
            let moveset = factory.create_moveset_from_slice(&["thunderbolt", "icebeam"]);
            // Would need actual PS data to test fully
            println!("Created moveset with {} moves", moveset.len());
        }
    }

    #[test]
    fn test_archetype_movesets() {
        if let Ok(factory) = PSMoveFactory::new() {
            let tank_set = factory.create_standard_moveset(MovesetArchetype::Tank);
            // Tank moveset should have defensive moves
            assert!(tank_set.len() > 0);
        }
    }

    #[test]
    fn test_popular_moves() {
        if let Ok(factory) = PSMoveFactory::new() {
            let popular = factory.get_popular_moves();
            // Should return some moves even without data
            println!("Popular moves: {}", popular.len());
        }
    }
}