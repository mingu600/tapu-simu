//! # Pokemon Showdown Generation-Specific Data Loader
//! 
//! This module provides generation-aware data loading for Pokemon Showdown data,
//! enabling accurate battle simulation across different Pokemon generations.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use crate::data::ps_types::{PSMoveData, PSMoveTarget};
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
    generation_data: HashMap<String, GenerationMoveData>,
    move_changes: HashMap<String, MoveChangeHistory>,
}

/// Move data for a specific generation
#[derive(Debug)]
pub struct GenerationMoveData {
    pub generation: u8,
    pub name: String,
    pub move_count: usize,
    pub moves: HashMap<String, PSMoveData>,
}

/// History of changes to a move across generations
#[derive(Debug)]
pub struct MoveChangeHistory {
    pub name: String,
    pub changes: Vec<GenerationChange>,
}

/// A change made to a move in a specific generation
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

        // Load generation data
        let generation_data_path = Path::new(data_dir).join("moves-by-generation.json");
        let generation_data_content = fs::read_to_string(&generation_data_path)?;
        let raw_generation_data: serde_json::Value = serde_json::from_str(&generation_data_content)?;

        let mut generation_data = HashMap::new();
        
        for generation in &generations {
            if let Some(gen_data) = raw_generation_data.get(&generation.id) {
                let gen_moves_raw = gen_data.get("moves").unwrap().as_object().unwrap();
                let mut gen_moves = HashMap::new();

                for (move_id, move_value) in gen_moves_raw {
                    let move_data: PSMoveData = serde_json::from_value(move_value.clone())?;
                    gen_moves.insert(move_id.clone(), move_data);
                }

                generation_data.insert(generation.id.clone(), GenerationMoveData {
                    generation: generation.num,
                    name: generation.name.clone(),
                    move_count: gen_moves.len(),
                    moves: gen_moves,
                });
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

        Ok(Self {
            generations,
            generation_data,
            move_changes,
        })
    }

    /// Get move data for a specific generation
    pub fn get_move_for_generation(&self, move_name: &str, generation: u8) -> Option<&PSMoveData> {
        let gen_id = format!("gen{}", generation);
        self.generation_data.get(&gen_id)?.moves.get(move_name)
    }

    /// Get current generation move data (Gen 9)
    pub fn get_move(&self, move_name: &str) -> Option<&PSMoveData> {
        self.get_move_for_generation(move_name, 9)
    }

    /// Check if a move exists in a specific generation
    pub fn move_exists_in_generation(&self, move_name: &str, generation: u8) -> bool {
        self.get_move_for_generation(move_name, generation).is_some()
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

    /// Get change history for a move
    pub fn get_move_changes(&self, move_name: &str) -> Option<&MoveChangeHistory> {
        self.move_changes.get(move_name)
    }

    /// Get all moves for a specific generation
    pub fn get_all_moves_for_generation(&self, generation: u8) -> Option<&HashMap<String, PSMoveData>> {
        let gen_id = format!("gen{}", generation);
        Some(&self.generation_data.get(&gen_id)?.moves)
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
            let data = self.generation_data.get(&gen.id);
            GenerationStats {
                generation: gen.num,
                name: gen.name.clone(),
                move_count: data.map(|d| d.move_count).unwrap_or(0),
                has_data: data.is_some(),
            }
        }).collect()
    }

    /// Get total change statistics
    pub fn get_change_stats(&self) -> ChangeStats {
        ChangeStats {
            total_moves_with_changes: self.move_changes.len(),
            total_changes: self.move_changes.values()
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
}

/// Statistics about a generation
#[derive(Debug)]
pub struct GenerationStats {
    pub generation: u8,
    pub name: String,
    pub move_count: usize,
    pub has_data: bool,
}

/// Statistics about move changes
#[derive(Debug)]
pub struct ChangeStats {
    pub total_moves_with_changes: usize,
    pub total_changes: usize,
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