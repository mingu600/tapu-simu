//! # Pokemon Showdown Generation-Specific Data Loader
//! 
//! This module provides generation-aware data loading for Pokemon Showdown data,
//! enabling accurate battle simulation across different Pokemon generations.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use crate::data::ps_types::{PSMoveData, PSMoveTarget, PSItemData};
use crate::data::ps_conversion::ps_target_from_string;
use crate::state::{Move, MoveCategory};

/// Generation configuration
#[derive(Debug, Clone)]
pub struct Generation {
    pub num: u8,
    pub id: String,
    pub name: String,
}

/// Generation-specific data repository
pub struct PSGenerationRepository {
    generations: Vec<Generation>,
    generation_move_data: HashMap<String, GenerationMoveData>,
    generation_item_data: HashMap<String, GenerationItemData>,
    move_changes: HashMap<String, MoveChangeHistory>,
    item_changes: HashMap<String, ItemChangeHistory>,
}

/// Move data for a specific generation
#[derive(Debug)]
pub struct GenerationMoveData {
    pub generation: u8,
    pub name: String,
    pub move_count: usize,
    pub moves: HashMap<String, PSMoveData>,
}

/// Item data for a specific generation
#[derive(Debug)]
pub struct GenerationItemData {
    pub generation: u8,
    pub name: String,
    pub item_count: usize,
    pub items: HashMap<String, PSItemData>,
}

/// History of changes to a move across generations
#[derive(Debug)]
pub struct MoveChangeHistory {
    pub name: String,
    pub changes: Vec<GenerationChange>,
}

/// History of changes to an item across generations
#[derive(Debug)]
pub struct ItemChangeHistory {
    pub name: String,
    pub changes: Vec<GenerationChange>,
}

/// A change made to a move/item in a specific generation
#[derive(Debug)]
pub struct GenerationChange {
    pub generation: u8,
    pub changes: Vec<FieldChange>,
}

/// A specific field change
#[derive(Debug)]
pub struct FieldChange {
    pub field: String,
    pub from: serde_json::Value,
    pub to: serde_json::Value,
}

impl PSGenerationRepository {
    /// Load generation-specific data from directory
    pub fn load_from_directory(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let generations = vec![
            Generation { num: 1, id: "gen1".to_string(), name: "RBY".to_string() },
            Generation { num: 2, id: "gen2".to_string(), name: "GSC".to_string() },
            Generation { num: 3, id: "gen3".to_string(), name: "RSE".to_string() },
            Generation { num: 4, id: "gen4".to_string(), name: "DPPt".to_string() },
            Generation { num: 5, id: "gen5".to_string(), name: "BW".to_string() },
            Generation { num: 6, id: "gen6".to_string(), name: "XY".to_string() },
            Generation { num: 7, id: "gen7".to_string(), name: "SM".to_string() },
            Generation { num: 8, id: "gen8".to_string(), name: "SWSH".to_string() },
            Generation { num: 9, id: "gen9".to_string(), name: "SV".to_string() },
        ];

        // Load generation move data
        let generation_move_data_path = Path::new(data_dir).join("moves-by-generation.json");
        let generation_move_data_content = fs::read_to_string(&generation_move_data_path)?;
        let raw_generation_move_data: serde_json::Value = serde_json::from_str(&generation_move_data_content)?;

        let mut generation_move_data = HashMap::new();
        
        for generation in &generations {
            if let Some(gen_data) = raw_generation_move_data.get(&generation.id) {
                let gen_moves_raw = gen_data.get("moves").unwrap().as_object().unwrap();
                let mut gen_moves = HashMap::new();

                for (move_id, move_value) in gen_moves_raw {
                    let move_data: PSMoveData = serde_json::from_value(move_value.clone())?;
                    gen_moves.insert(move_id.clone(), move_data);
                }

                generation_move_data.insert(generation.id.clone(), GenerationMoveData {
                    generation: generation.num,
                    name: generation.name.clone(),
                    move_count: gen_moves.len(),
                    moves: gen_moves,
                });
            }
        }

        // Load generation item data
        let generation_item_data_path = Path::new(data_dir).join("items-by-generation.json");
        let mut generation_item_data = HashMap::new();
        
        if generation_item_data_path.exists() {
            let generation_item_data_content = fs::read_to_string(&generation_item_data_path)?;
            let raw_generation_item_data: serde_json::Value = serde_json::from_str(&generation_item_data_content)?;
            
            for generation in &generations {
                if let Some(gen_data) = raw_generation_item_data.get(&generation.id) {
                    let gen_items_raw = gen_data.get("items").unwrap().as_object().unwrap();
                    let mut gen_items = HashMap::new();

                    for (item_id, item_value) in gen_items_raw {
                        let item_data: PSItemData = serde_json::from_value(item_value.clone())?;
                        gen_items.insert(item_id.clone(), item_data);
                    }

                    generation_item_data.insert(generation.id.clone(), GenerationItemData {
                        generation: generation.num,
                        name: generation.name.clone(),
                        item_count: gen_items.len(),
                        items: gen_items,
                    });
                }
            }
        }

        // Load move changes data
        let move_changes_path = Path::new(data_dir).join("move-changes.json");
        let move_changes_content = fs::read_to_string(&move_changes_path)?;
        let raw_move_changes: serde_json::Value = serde_json::from_str(&move_changes_content)?;

        let mut move_changes = HashMap::new();
        
        if let Some(changes_obj) = raw_move_changes.as_object() {
            for (move_id, change_data) in changes_obj {
                let name = change_data.get("name").unwrap().as_str().unwrap().to_string();
                let changes_array = change_data.get("changes").unwrap().as_array().unwrap();
                
                let mut changes = Vec::new();
                for change in changes_array {
                    let generation = change.get("generation").unwrap().as_u64().unwrap() as u8;
                    let field_changes_array = change.get("changes").unwrap().as_array().unwrap();
                    
                    let mut field_changes = Vec::new();
                    for field_change in field_changes_array {
                        field_changes.push(FieldChange {
                            field: field_change.get("field").unwrap().as_str().unwrap().to_string(),
                            from: field_change.get("from").unwrap().clone(),
                            to: field_change.get("to").unwrap().clone(),
                        });
                    }
                    
                    changes.push(GenerationChange {
                        generation,
                        changes: field_changes,
                    });
                }
                
                move_changes.insert(move_id.clone(), MoveChangeHistory {
                    name,
                    changes,
                });
            }
        }

        // Load item changes data
        let item_changes_path = Path::new(data_dir).join("item-changes.json");
        let mut item_changes = HashMap::new();
        
        if item_changes_path.exists() {
            let item_changes_content = fs::read_to_string(&item_changes_path)?;
            let raw_item_changes: serde_json::Value = serde_json::from_str(&item_changes_content)?;
            
            if let Some(changes_obj) = raw_item_changes.as_object() {
                for (item_id, change_data) in changes_obj {
                    let name = change_data.get("name").unwrap().as_str().unwrap().to_string();
                    let changes_array = change_data.get("changes").unwrap().as_array().unwrap();
                    
                    let mut changes = Vec::new();
                    for change in changes_array {
                        let generation = change.get("generation").unwrap().as_u64().unwrap() as u8;
                        let field_changes_array = change.get("changes").unwrap().as_array().unwrap();
                        
                        let mut field_changes = Vec::new();
                        for field_change in field_changes_array {
                            field_changes.push(FieldChange {
                                field: field_change.get("field").unwrap().as_str().unwrap().to_string(),
                                from: field_change.get("from").unwrap().clone(),
                                to: field_change.get("to").unwrap().clone(),
                            });
                        }
                        
                        changes.push(GenerationChange {
                            generation,
                            changes: field_changes,
                        });
                    }
                    
                    item_changes.insert(item_id.clone(), ItemChangeHistory {
                        name,
                        changes,
                    });
                }
            }
        }

        Ok(Self {
            generations,
            generation_move_data,
            generation_item_data,
            move_changes,
            item_changes,
        })
    }

    /// Get move data for a specific generation
    pub fn get_move_for_generation(&self, move_name: &str, generation: u8) -> Option<&PSMoveData> {
        let gen_id = format!("gen{}", generation);
        self.generation_move_data.get(&gen_id)?.moves.get(move_name)
    }

    /// Get item data for a specific generation
    pub fn get_item_for_generation(&self, item_name: &str, generation: u8) -> Option<&PSItemData> {
        let gen_id = format!("gen{}", generation);
        self.generation_item_data.get(&gen_id)?.items.get(item_name)
    }

    /// Get current generation move data (Gen 9)
    pub fn get_move(&self, move_name: &str) -> Option<&PSMoveData> {
        self.get_move_for_generation(move_name, 9)
    }

    /// Get current generation item data (Gen 9)
    pub fn get_item(&self, item_name: &str) -> Option<&PSItemData> {
        self.get_item_for_generation(item_name, 9)
    }

    /// Check if a move exists in a specific generation
    pub fn move_exists_in_generation(&self, move_name: &str, generation: u8) -> bool {
        self.get_move_for_generation(move_name, generation).is_some()
    }

    /// Check if an item exists in a specific generation
    pub fn item_exists_in_generation(&self, item_name: &str, generation: u8) -> bool {
        self.get_item_for_generation(item_name, generation).is_some()
    }

    /// Get all generations where a move exists
    pub fn get_move_generations(&self, move_name: &str) -> Vec<u8> {
        let mut generations = Vec::new();
        for generation in &self.generations {
            if self.move_exists_in_generation(move_name, generation.num) {
                generations.push(generation.num);
            }
        }
        generations
    }

    /// Get all generations where an item exists
    pub fn get_item_generations(&self, item_name: &str) -> Vec<u8> {
        let mut generations = Vec::new();
        for generation in &self.generations {
            if self.item_exists_in_generation(item_name, generation.num) {
                generations.push(generation.num);
            }
        }
        generations
    }

    /// Get change history for a move
    pub fn get_move_changes(&self, move_name: &str) -> Option<&MoveChangeHistory> {
        self.move_changes.get(move_name)
    }

    /// Get change history for an item
    pub fn get_item_changes(&self, item_name: &str) -> Option<&ItemChangeHistory> {
        self.item_changes.get(item_name)
    }

    /// Get all moves for a specific generation
    pub fn get_all_moves_for_generation(&self, generation: u8) -> Option<&HashMap<String, PSMoveData>> {
        let gen_id = format!("gen{}", generation);
        Some(&self.generation_move_data.get(&gen_id)?.moves)
    }

    /// Get all items for a specific generation
    pub fn get_all_items_for_generation(&self, generation: u8) -> Option<&HashMap<String, PSItemData>> {
        let gen_id = format!("gen{}", generation);
        Some(&self.generation_item_data.get(&gen_id)?.items)
    }

    /// Convert PS move data to engine move (with generation awareness)
    pub fn ps_move_to_engine_move(&self, ps_move: &PSMoveData) -> Move {
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

    /// Convert PS category to engine category
    fn convert_ps_category_to_engine(&self, ps_category: &str) -> MoveCategory {
        match ps_category {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            "Status" => MoveCategory::Status,
            _ => MoveCategory::Status,
        }
    }

    /// Get generation statistics
    pub fn get_generation_stats(&self) -> Vec<GenerationStats> {
        self.generations.iter().map(|gen| {
            let move_data = self.generation_move_data.get(&gen.id);
            let item_data = self.generation_item_data.get(&gen.id);
            GenerationStats {
                generation: gen.num,
                name: gen.name.clone(),
                move_count: move_data.map(|d| d.move_count).unwrap_or(0),
                item_count: item_data.map(|d| d.item_count).unwrap_or(0),
                has_data: move_data.is_some(),
            }
        }).collect()
    }

    /// Get total change statistics
    pub fn get_change_stats(&self) -> ChangeStats {
        ChangeStats {
            total_moves_with_changes: self.move_changes.len(),
            total_items_with_changes: self.item_changes.len(),
            total_move_changes: self.move_changes.values()
                .map(|history| history.changes.len())
                .sum(),
            total_item_changes: self.item_changes.values()
                .map(|history| history.changes.len())
                .sum(),
        }
    }

    /// Find moves that changed in a specific generation
    pub fn get_moves_changed_in_generation(&self, generation: u8) -> Vec<&MoveChangeHistory> {
        self.move_changes.values()
            .filter(|history| history.changes.iter().any(|change| change.generation == generation))
            .collect()
    }

    /// Find items that changed in a specific generation
    pub fn get_items_changed_in_generation(&self, generation: u8) -> Vec<&ItemChangeHistory> {
        self.item_changes.values()
            .filter(|history| history.changes.iter().any(|change| change.generation == generation))
            .collect()
    }
}

/// Statistics about a generation
#[derive(Debug)]
pub struct GenerationStats {
    pub generation: u8,
    pub name: String,
    pub move_count: usize,
    pub item_count: usize,
    pub has_data: bool,
}

/// Statistics about changes
#[derive(Debug)]
pub struct ChangeStats {
    pub total_moves_with_changes: usize,
    pub total_items_with_changes: usize,
    pub total_move_changes: usize,
    pub total_item_changes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_repository_creation() {
        // This test will fail without actual PS generation data files
        // but demonstrates the intended usage
        if let Ok(repo) = PSGenerationRepository::load_from_directory("data/ps-extracted") {
            let stats = repo.get_generation_stats();
            assert!(!stats.is_empty());
            
            // Test specific move lookups
            if let Some(bite_gen1) = repo.get_move_for_generation("bite", 1) {
                assert_eq!(bite_gen1.move_type, "Normal"); // Bite was Normal in Gen 1
            }
            
            if let Some(bite_gen9) = repo.get_move_for_generation("bite", 9) {
                assert_eq!(bite_gen9.move_type, "Dark"); // Bite is Dark in modern gens
            }
        }
    }

    #[test]
    fn test_move_generation_tracking() {
        if let Ok(repo) = PSGenerationRepository::load_from_directory("data/ps-extracted") {
            // Test that a Gen 1 move exists in multiple generations
            let absorb_gens = repo.get_move_generations("absorb");
            assert!(!absorb_gens.is_empty());
            assert!(absorb_gens.contains(&1)); // Should exist in Gen 1
            
            // Test that a modern move doesn't exist in Gen 1
            assert!(!repo.move_exists_in_generation("accelerock", 1));
        }
    }
}