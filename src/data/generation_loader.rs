//! # Pokemon Showdown Generation-Specific Data Loader
//!
//! This module provides generation-aware data loading for Pokemon Showdown data,
//! enabling accurate battle simulation across different Pokemon generations.

use crate::core::battle_state::{Move, MoveCategory};
use crate::data::showdown_types::{ItemData, MoveData, PokemonData};
use crate::types::identifiers::{ItemId, MoveId, SpeciesId};
use crate::utils::normalize_name;
use crate::utils::target_from_string;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Generation configuration
#[derive(Debug, Clone)]
pub struct Generation {
    pub num: u8,
    pub id: String,
    pub name: String,
}

/// Generation-specific data repository
pub struct GenerationRepository {
    generations: Vec<Generation>,
    generation_move_data: HashMap<String, GenerationMoveData>,
    generation_item_data: HashMap<String, GenerationItemData>,
    generation_pokemon_data: HashMap<String, GenerationPokemonData>,
    move_changes: HashMap<MoveId, MoveChangeHistory>,
    item_changes: HashMap<ItemId, ItemChangeHistory>,
    pokemon_changes: HashMap<SpeciesId, PokemonChangeHistory>,
}

/// Move data for a specific generation
#[derive(Debug)]
pub struct GenerationMoveData {
    pub generation: u8,
    pub name: String,
    pub move_count: usize,
    pub moves: HashMap<String, MoveData>,
}

/// Item data for a specific generation
#[derive(Debug)]
pub struct GenerationItemData {
    pub generation: u8,
    pub name: String,
    pub item_count: usize,
    pub items: HashMap<String, ItemData>,
}

/// Pokemon data for a specific generation
#[derive(Debug)]
pub struct GenerationPokemonData {
    pub generation: u8,
    pub name: String,
    pub pokemon_count: usize,
    pub pokemon: HashMap<String, PokemonData>,
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

/// History of changes to a Pokemon across generations
#[derive(Debug)]
pub struct PokemonChangeHistory {
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

impl GenerationRepository {
    /// Helper method to get generation ID string
    fn gen_id(generation: u8) -> String {
        format!("gen{}", generation)
    }

    /// Helper function to load generation-specific data from JSON
    fn load_generation_data<T, G>(
        data_dir: &str,
        filename: &str,
        generations: &[Generation],
        data_key: &str,
        constructor: impl Fn(u8, String, usize, HashMap<String, T>) -> G,
    ) -> Result<HashMap<String, G>, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let file_path = Path::new(data_dir).join(filename);
        let mut result = HashMap::with_capacity(generations.len());

        if file_path.exists() {
            let file_content = fs::read_to_string(&file_path)?;
            let raw_data: serde_json::Value = serde_json::from_str(&file_content)?;

            for generation in generations {
                if let Some(gen_data) = raw_data.get(&generation.id) {
                    let gen_items_raw = gen_data.get(data_key).unwrap().as_object().unwrap();
                    let mut gen_items = HashMap::with_capacity(gen_items_raw.len());

                    for (item_id, item_value) in gen_items_raw {
                        let item_data: T = serde_json::from_value(item_value.clone())?;
                        gen_items.insert(item_id.clone(), item_data);
                    }

                    let count = gen_items.len();
                    result.insert(
                        generation.id.clone(),
                        constructor(generation.num, generation.name.clone(), count, gen_items),
                    );
                }
            }
        }

        Ok(result)
    }

    /// Helper function to load change history data from JSON
    fn load_change_history<C>(
        data_dir: &str,
        filename: &str,
        constructor: impl Fn(String, Vec<GenerationChange>) -> C,
    ) -> Result<HashMap<String, C>, Box<dyn std::error::Error>> {
        let file_path = Path::new(data_dir).join(filename);
        let mut result = HashMap::with_capacity(16); // Reasonable initial capacity for change history

        if file_path.exists() {
            let file_content = fs::read_to_string(&file_path)?;
            let raw_data: serde_json::Value = serde_json::from_str(&file_content)?;

            if let Some(changes_obj) = raw_data.as_object() {
                for (item_id, change_data) in changes_obj {
                    let name = change_data
                        .get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();
                    let changes_array = change_data.get("changes").unwrap().as_array().unwrap();

                    let mut changes = Vec::with_capacity(changes_array.len());
                    for change in changes_array {
                        let generation = change.get("generation").unwrap().as_u64().unwrap() as u8;
                        let field_changes_array = change.get("changes").unwrap().as_array().unwrap();

                        let mut field_changes = Vec::with_capacity(field_changes_array.len());
                        for field_change in field_changes_array {
                            field_changes.push(FieldChange {
                                field: field_change
                                    .get("field")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                from: field_change.get("from").unwrap().clone(),
                                to: field_change.get("to").unwrap().clone(),
                            });
                        }

                        changes.push(GenerationChange {
                            generation,
                            changes: field_changes,
                        });
                    }

                    result.insert(item_id.clone(), constructor(name, changes));
                }
            }
        }

        Ok(result)
    }

    /// Load generation-specific data from directory
    pub fn load_from_directory(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let generations = vec![
            Generation {
                num: 1,
                id: "gen1".to_string(),
                name: "RBY".to_string(),
            },
            Generation {
                num: 2,
                id: "gen2".to_string(),
                name: "GSC".to_string(),
            },
            Generation {
                num: 3,
                id: "gen3".to_string(),
                name: "RSE".to_string(),
            },
            Generation {
                num: 4,
                id: "gen4".to_string(),
                name: "DPPt".to_string(),
            },
            Generation {
                num: 5,
                id: "gen5".to_string(),
                name: "BW".to_string(),
            },
            Generation {
                num: 6,
                id: "gen6".to_string(),
                name: "XY".to_string(),
            },
            Generation {
                num: 7,
                id: "gen7".to_string(),
                name: "SM".to_string(),
            },
            Generation {
                num: 8,
                id: "gen8".to_string(),
                name: "SWSH".to_string(),
            },
            Generation {
                num: 9,
                id: "gen9".to_string(),
                name: "SV".to_string(),
            },
        ];

        // Load generation move data
        let generation_move_data = Self::load_generation_data(
            data_dir,
            "moves-by-generation.json",
            &generations,
            "moves",
            |generation, name, count, moves| GenerationMoveData {
                generation,
                name,
                move_count: count,
                moves,
            },
        )?;

        // Load generation item data
        let generation_item_data = Self::load_generation_data(
            data_dir,
            "items-by-generation.json",
            &generations,
            "items",
            |generation, name, count, items| GenerationItemData {
                generation,
                name,
                item_count: count,
                items,
            },
        )?;

        // Load move changes data
        let move_changes_raw = Self::load_change_history(
            data_dir,
            "move-changes.json",
            |name, changes| MoveChangeHistory { name, changes },
        )?;
        let move_changes: HashMap<MoveId, MoveChangeHistory> = move_changes_raw
            .into_iter()
            .map(|(key, value)| (MoveId::new(key), value))
            .collect();

        // Load generation Pokemon data (if available)
        let mut generation_pokemon_data = Self::load_generation_data(
            data_dir,
            "pokemon-by-generation.json",
            &generations,
            "pokemon",
            |generation, name, count, pokemon| GenerationPokemonData {
                generation,
                name,
                pokemon_count: count,
                pokemon,
            },
        )?;

        // If no generation-specific Pokemon data exists, create from base data
        if generation_pokemon_data.is_empty() {
            let base_pokemon_path = Path::new(data_dir).join("pokemon.json");
            if base_pokemon_path.exists() {
                let base_pokemon_content = fs::read_to_string(&base_pokemon_path)?;
                let base_pokemon: HashMap<String, PokemonData> =
                    serde_json::from_str(&base_pokemon_content)?;

                for generation in &generations {
                    let gen_pokemon = base_pokemon.clone();
                    generation_pokemon_data.insert(
                        generation.id.clone(),
                        GenerationPokemonData {
                            generation: generation.num,
                            name: generation.name.clone(),
                            pokemon_count: gen_pokemon.len(),
                            pokemon: gen_pokemon,
                        },
                    );
                }
            }
        }

        // Load item changes data
        let item_changes_raw = Self::load_change_history(
            data_dir,
            "item-changes.json",
            |name, changes| ItemChangeHistory { name, changes },
        )?;
        let item_changes: HashMap<ItemId, ItemChangeHistory> = item_changes_raw
            .into_iter()
            .map(|(key, value)| (ItemId::new(key), value))
            .collect();

        // Load Pokemon changes data
        let pokemon_changes_raw = Self::load_change_history(
            data_dir,
            "pokemon-changes.json",
            |name, changes| PokemonChangeHistory { name, changes },
        )?;
        let pokemon_changes: HashMap<SpeciesId, PokemonChangeHistory> = pokemon_changes_raw
            .into_iter()
            .map(|(key, value)| (SpeciesId::new(key), value))
            .collect();

        Ok(Self {
            generations,
            generation_move_data,
            generation_item_data,
            generation_pokemon_data,
            move_changes,
            item_changes,
            pokemon_changes,
        })
    }

    /// Get move data for a specific generation
    pub fn get_move_for_generation(&self, move_name: &str, generation: u8) -> Option<&MoveData> {
        let gen_id = Self::gen_id(generation);
        self.generation_move_data.get(&gen_id)?.moves.get(move_name)
    }

    /// Find move data by name (handles both display names and IDs) for a specific generation
    /// Falls back to earlier generations if move doesn't exist in target generation
    pub fn find_move_by_name_for_generation(
        &self,
        move_name: &str,
        generation: u8,
    ) -> Option<&MoveData> {
        // Normalize the search name
        let normalized_name = crate::utils::normalize_name(move_name);

        // Search from target generation backward to gen 1
        for gen in (1..=generation).rev() {
            let gen_id = Self::gen_id(gen);
            if let Some(generation_data) = self.generation_move_data.get(&gen_id) {
                // First try direct ID lookup
                if let Some(move_data) = generation_data.moves.get(&normalized_name) {
                    return Some(move_data);
                }

                // Fallback to searching by normalized display name
                if let Some(move_data) = generation_data.moves.values().find(|move_data| {
                    crate::utils::normalize_name(&move_data.name) == normalized_name
                }) {
                    return Some(move_data);
                }
            }
        }

        None
    }

    /// Get item data for a specific generation
    pub fn get_item_for_generation(&self, item_name: &str, generation: u8) -> Option<&ItemData> {
        let gen_id = Self::gen_id(generation);
        self.generation_item_data.get(&gen_id)?.items.get(item_name)
    }

    /// Find item data by name (handles both display names and IDs) for a specific generation
    /// Falls back to earlier generations if item doesn't exist in target generation
    pub fn find_item_by_name_for_generation(
        &self,
        item_name: &str,
        generation: u8,
    ) -> Option<&ItemData> {
        // Normalize the search name
        let normalized_name = normalize_name(item_name);

        // Search from target generation backward to gen 1
        for gen in (1..=generation).rev() {
            let gen_id = Self::gen_id(gen);
            if let Some(generation_data) = self.generation_item_data.get(&gen_id) {
                // First try direct ID lookup
                if let Some(item_data) = generation_data.items.get(&normalized_name) {
                    return Some(item_data);
                }

                // Fallback to searching by normalized display name
                if let Some(item_data) = generation_data
                    .items
                    .values()
                    .find(|item_data| normalize_name(&item_data.name) == normalized_name)
                {
                    return Some(item_data);
                }
            }
        }

        None
    }

    /// Get Pokemon data for a specific generation
    pub fn get_pokemon_for_generation(
        &self,
        pokemon_name: &str,
        generation: u8,
    ) -> Option<&PokemonData> {
        let gen_id = Self::gen_id(generation);
        self.generation_pokemon_data
            .get(&gen_id)?
            .pokemon
            .get(pokemon_name)
    }

    /// Find Pokemon data by name (handles both display names and IDs) for a specific generation
    /// Falls back to earlier generations if Pokemon doesn't exist in target generation
    pub fn find_pokemon_by_name_for_generation(
        &self,
        pokemon_name: &str,
        generation: u8,
    ) -> Option<&PokemonData> {
        // Normalize the search name
        let normalized_name = normalize_name(pokemon_name);

        // Search from target generation backward to gen 1
        for gen in (1..=generation).rev() {
            let gen_id = Self::gen_id(gen);
            if let Some(generation_data) = self.generation_pokemon_data.get(&gen_id) {
                // First try direct ID lookup
                if let Some(pokemon_data) = generation_data.pokemon.get(&normalized_name) {
                    return Some(pokemon_data);
                }

                // Fallback to searching by normalized display name
                if let Some(pokemon_data) = generation_data
                    .pokemon
                    .values()
                    .find(|pokemon_data| normalize_name(&pokemon_data.name) == normalized_name)
                {
                    return Some(pokemon_data);
                }
            }
        }

        None
    }

    /// Get current generation move data (Gen 9) - convenience method
    pub fn get_move(&self, move_name: &str) -> Option<&MoveData> {
        self.get_move_for_generation(move_name, 9)
    }

    /// Get current generation item data (Gen 9) - convenience method
    pub fn get_item(&self, item_name: &str) -> Option<&ItemData> {
        self.get_item_for_generation(item_name, 9)
    }

    /// Get all generations where a move exists (optimized single pass)
    pub fn get_move_generations(&self, move_name: &str) -> Vec<u8> {
        self.generations
            .iter()
            .filter_map(|gen| {
                let gen_id = Self::gen_id(gen.num);
                if self
                    .generation_move_data
                    .get(&gen_id)?
                    .moves
                    .contains_key(move_name)
                {
                    Some(gen.num)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all generations where an item exists (optimized single pass)
    pub fn get_item_generations(&self, item_name: &str) -> Vec<u8> {
        self.generations
            .iter()
            .filter_map(|gen| {
                let gen_id = Self::gen_id(gen.num);
                if self
                    .generation_item_data
                    .get(&gen_id)?
                    .items
                    .contains_key(item_name)
                {
                    Some(gen.num)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a move exists in a specific generation (optimized)
    pub fn move_exists_in_generation(&self, move_name: &str, generation: u8) -> bool {
        let gen_id = Self::gen_id(generation);
        self.generation_move_data
            .get(&gen_id)
            .map(|data| data.moves.contains_key(move_name))
            .unwrap_or(false)
    }

    /// Check if an item exists in a specific generation (optimized)
    pub fn item_exists_in_generation(&self, item_name: &str, generation: u8) -> bool {
        let gen_id = Self::gen_id(generation);
        self.generation_item_data
            .get(&gen_id)
            .map(|data| data.items.contains_key(item_name))
            .unwrap_or(false)
    }

    /// Get change history for a move
    pub fn get_move_changes(&self, move_id: &MoveId) -> Option<&MoveChangeHistory> {
        self.move_changes.get(move_id)
    }

    /// Get change history for an item
    pub fn get_item_changes(&self, item_id: &ItemId) -> Option<&ItemChangeHistory> {
        self.item_changes.get(item_id)
    }

    /// Get change history for a Pokemon
    pub fn get_pokemon_changes(&self, species_id: &SpeciesId) -> Option<&PokemonChangeHistory> {
        self.pokemon_changes.get(species_id)
    }

    /// Get all moves for a specific generation
    pub fn get_all_moves_for_generation(
        &self,
        generation: u8,
    ) -> Option<&HashMap<String, MoveData>> {
        let gen_id = Self::gen_id(generation);
        Some(&self.generation_move_data.get(&gen_id)?.moves)
    }

    /// Get all items for a specific generation
    pub fn get_all_items_for_generation(
        &self,
        generation: u8,
    ) -> Option<&HashMap<String, ItemData>> {
        let gen_id = Self::gen_id(generation);
        Some(&self.generation_item_data.get(&gen_id)?.items)
    }

    /// Convert move data to engine move (with generation awareness)
    pub fn move_to_engine_move(&self, move_data: &MoveData) -> Move {
        // Use the MoveData's direct conversion method
        move_data.to_engine_move()
    }

    /// Get generation statistics
    pub fn get_generation_stats(&self) -> Vec<GenerationStats> {
        self.generations
            .iter()
            .map(|gen| {
                let move_data = self.generation_move_data.get(&gen.id);
                let item_data = self.generation_item_data.get(&gen.id);
                GenerationStats {
                    generation: gen.num,
                    name: gen.name.clone(),
                    move_count: move_data.map(|d| d.move_count).unwrap_or(0),
                    item_count: item_data.map(|d| d.item_count).unwrap_or(0),
                    has_data: move_data.is_some(),
                }
            })
            .collect()
    }

    /// Get total change statistics
    pub fn get_change_stats(&self) -> ChangeStats {
        ChangeStats {
            total_moves_with_changes: self.move_changes.len(),
            total_items_with_changes: self.item_changes.len(),
            total_pokemon_with_changes: self.pokemon_changes.len(),
            total_move_changes: self
                .move_changes
                .values()
                .map(|history| history.changes.len())
                .sum(),
            total_item_changes: self
                .item_changes
                .values()
                .map(|history| history.changes.len())
                .sum(),
            total_pokemon_changes: self
                .pokemon_changes
                .values()
                .map(|history| history.changes.len())
                .sum(),
        }
    }

    /// Find moves that changed in a specific generation
    pub fn get_moves_changed_in_generation(&self, generation: u8) -> Vec<&MoveChangeHistory> {
        self.move_changes
            .values()
            .filter(|history| {
                history
                    .changes
                    .iter()
                    .any(|change| change.generation == generation)
            })
            .collect()
    }

    /// Find items that changed in a specific generation
    pub fn get_items_changed_in_generation(&self, generation: u8) -> Vec<&ItemChangeHistory> {
        self.item_changes
            .values()
            .filter(|history| {
                history
                    .changes
                    .iter()
                    .any(|change| change.generation == generation)
            })
            .collect()
    }

    /// Find Pokemon that changed in a specific generation
    pub fn get_pokemon_changed_in_generation(&self, generation: u8) -> Vec<&PokemonChangeHistory> {
        self.pokemon_changes
            .values()
            .filter(|history| {
                history
                    .changes
                    .iter()
                    .any(|change| change.generation == generation)
            })
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
    pub total_pokemon_with_changes: usize,
    pub total_move_changes: usize,
    pub total_item_changes: usize,
    pub total_pokemon_changes: usize,
}
