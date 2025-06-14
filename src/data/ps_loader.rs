//! # Pokemon Showdown Data Loader
//! 
//! This module loads extracted Pokemon Showdown data and provides
//! it in a format suitable for the battle engine.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use crate::data::ps_types::{PSMoveData, PSMoveTarget};
use crate::data::ps_conversion::ps_target_from_string;

/// Pokemon Showdown data repository
pub struct PSDataRepository {
    moves: HashMap<String, PSMoveData>,
    items: HashMap<String, serde_json::Value>, // Items not fully typed yet
}

impl PSDataRepository {
    /// Load PS data from extracted JSON files
    pub fn load_from_directory<P: AsRef<Path>>(data_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = data_dir.as_ref();
        
        // Load moves
        let moves_path = data_dir.join("moves.json");
        let moves_json = fs::read_to_string(moves_path)?;
        let moves_raw: HashMap<String, serde_json::Value> = serde_json::from_str(&moves_json)?;
        
        let mut moves = HashMap::new();
        for (id, move_value) in moves_raw {
            // Parse the move data and convert target string to enum
            let mut move_data: PSMoveData = serde_json::from_value(move_value)?;
            moves.insert(id, move_data);
        }
        
        // Load items (simplified for now)
        let items_path = data_dir.join("items.json");
        let items = if items_path.exists() {
            let items_json = fs::read_to_string(items_path)?;
            serde_json::from_str(&items_json)?
        } else {
            HashMap::new()
        };
        
        Ok(Self { moves, items })
    }

    /// Get move data by ID
    pub fn get_move(&self, move_id: &str) -> Option<&PSMoveData> {
        self.moves.get(move_id)
    }

    /// Get move data by name (case insensitive, space normalized)
    pub fn get_move_by_name(&self, name: &str) -> Option<&PSMoveData> {
        let normalized = normalize_name(name);
        self.get_move(&normalized)
    }

    /// Get all moves
    pub fn get_all_moves(&self) -> &HashMap<String, PSMoveData> {
        &self.moves
    }

    /// Convert PS move data to engine move for compatibility
    pub fn ps_move_to_engine_move(&self, ps_move: &PSMoveData) -> crate::state::Move {
        crate::state::Move {
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

    /// Convert PS target string to engine MoveTarget (temporary during migration)
    fn convert_ps_target_to_engine(&self, ps_target: &str) -> crate::data::types::MoveTarget {
        // This is a temporary bridge during migration
        use crate::data::ps_conversion::ps_target_from_string;
        use crate::data::ps_conversion::rustemon_to_ps_target;
        
        // For now, map PS targets back to our current engine targets
        // In the final implementation, we'd use PS targets directly
        let ps_target_enum = ps_target_from_string(ps_target);
        
        // Reverse mapping (this will be removed when we fully migrate)
        match ps_target_enum {
            PSMoveTarget::Self_ => crate::data::types::MoveTarget::User,
            PSMoveTarget::Normal => crate::data::types::MoveTarget::SelectedPokemon,
            PSMoveTarget::AdjacentAlly => crate::data::types::MoveTarget::Ally,
            PSMoveTarget::AdjacentAllyOrSelf => crate::data::types::MoveTarget::UserOrAlly,
            PSMoveTarget::AllAdjacentFoes => crate::data::types::MoveTarget::AllOpponents,
            PSMoveTarget::All => crate::data::types::MoveTarget::EntireField,
            PSMoveTarget::AllySide => crate::data::types::MoveTarget::UsersField,
            PSMoveTarget::FoeSide => crate::data::types::MoveTarget::OpponentsField,
            PSMoveTarget::RandomNormal => crate::data::types::MoveTarget::RandomOpponent,
            PSMoveTarget::Scripted => crate::data::types::MoveTarget::SpecificMove,
            _ => crate::data::types::MoveTarget::SelectedPokemon, // Default
        }
    }

    /// Convert PS category to engine category
    fn convert_ps_category_to_engine(&self, ps_category: &str) -> crate::state::MoveCategory {
        match ps_category {
            "Physical" => crate::state::MoveCategory::Physical,
            "Special" => crate::state::MoveCategory::Special,
            "Status" => crate::state::MoveCategory::Status,
            _ => crate::state::MoveCategory::Status,
        }
    }

    /// Get statistics about loaded data
    pub fn stats(&self) -> PSDataStats {
        PSDataStats {
            move_count: self.moves.len(),
            item_count: self.items.len(),
        }
    }
}

/// Statistics about loaded PS data
#[derive(Debug)]
pub struct PSDataStats {
    pub move_count: usize,
    pub item_count: usize,
}

/// Normalize a Pokemon/move name to PS ID format
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Thunderbolt"), "thunderbolt");
        assert_eq!(normalize_name("U-turn"), "uturn");
        assert_eq!(normalize_name("King's Shield"), "kingsshield");
        assert_eq!(normalize_name("10,000,000 Volt Thunderbolt"), "10000000voltthunderbolt");
    }

    // Note: Integration tests would require actual PS data files
}