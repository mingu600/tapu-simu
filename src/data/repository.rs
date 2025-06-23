use crate::types::{AbilityId, DataError, DataResult, ItemId, MoveId, SpeciesId, TypeId};
use crate::utils::normalize_name;
use crate::data::showdown_types::{MoveData, PokemonData, ItemData, AbilityData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Direct data access repository without factory or service layers
pub struct Repository {
    moves: HashMap<MoveId, MoveData>,
    pokemon: HashMap<SpeciesId, PokemonData>,
    items: HashMap<ItemId, ItemData>,
    abilities: HashMap<AbilityId, AbilityData>,
    // Performance indexes
    move_name_index: HashMap<String, MoveId>,
    pokemon_name_index: HashMap<String, SpeciesId>,
    item_name_index: HashMap<String, ItemId>,
    // ID counters for unique ID generation
    next_move_id: u32,
    next_pokemon_id: u32,
    next_item_id: u32,
    next_ability_id: u32,
}

// Global repository instance
use std::sync::OnceLock;
static GLOBAL_REPOSITORY: OnceLock<Mutex<Option<Arc<Repository>>>> = OnceLock::new();

impl Repository {
    /// Get or create global repository instance (singleton pattern)
    pub fn global(path: impl AsRef<Path>) -> DataResult<Arc<Self>> {
        let mutex = GLOBAL_REPOSITORY.get_or_init(|| Mutex::new(None));
        let mut repo = mutex.lock().unwrap();
        
        if let Some(existing) = repo.as_ref() {
            return Ok(Arc::clone(existing));
        }
        
        let new_repo = Arc::new(Self::from_path_internal(path)?);
        *repo = Some(Arc::clone(&new_repo));
        Ok(new_repo)
    }
    
    /// Load repository from PS data directory (internal method)
    fn from_path_internal(path: impl AsRef<Path>) -> DataResult<Self> {
        let path = path.as_ref();
        
        // Load each data type directly from JSON files
        let moves = load_moves_data(&path.join("moves.json"))?;
        let pokemon = load_pokemon_data(&path.join("pokemon.json"))?;
        let items = load_items_data(&path.join("items.json"))?;
        let abilities = load_abilities_data(&path.join("abilities.json"))?;
        
        let mut repo = Self {
            moves,
            pokemon, 
            items,
            abilities,
            move_name_index: HashMap::new(),
            pokemon_name_index: HashMap::new(),
            item_name_index: HashMap::new(),
            next_move_id: 1,
            next_pokemon_id: 1,
            next_item_id: 1,
            next_ability_id: 1,
        };
        
        // Build performance indexes
        repo.build_indexes();
        
        Ok(repo)
    }
    
    /// Load repository from PS data directory (kept for backward compatibility)
    pub fn from_path(path: impl AsRef<Path>) -> DataResult<Self> {
        Self::from_path_internal(path)
    }
    
    /// Build performance indexes for fast lookups
    fn build_indexes(&mut self) {
        // Build move name index
        for (move_id, move_data) in &self.moves {
            let normalized_name = normalize_name(&move_data.name);
            self.move_name_index.insert(normalized_name, move_id.clone());
        }
        
        // Build pokemon name index
        for (species_id, pokemon_data) in &self.pokemon {
            let normalized_name = normalize_name(&pokemon_data.name);
            self.pokemon_name_index.insert(normalized_name, species_id.clone());
            // Also index by species ID string
            let normalized_id = normalize_name(species_id.as_str());
            self.pokemon_name_index.insert(normalized_id, species_id.clone());
        }
        
        // Build item name index
        for (item_id, item_data) in &self.items {
            let normalized_name = normalize_name(&item_data.name);
            self.item_name_index.insert(normalized_name, item_id.clone());
            // Also index by item ID string
            let normalized_id = normalize_name(item_id.as_str());
            self.item_name_index.insert(normalized_id, item_id.clone());
        }
    }
    
    /// Direct access to move data
    pub fn move_data(&self, id: &MoveId) -> DataResult<&MoveData> {
        self.moves.get(id).ok_or_else(|| DataError::MoveNotFound { 
            move_id: id.clone() 
        })
    }
    
    /// Direct access to pokemon data
    pub fn pokemon_data(&self, id: &SpeciesId) -> DataResult<&PokemonData> {
        self.pokemon.get(id).ok_or_else(|| DataError::SpeciesNotFound { 
            species: id.clone() 
        })
    }
    
    /// Direct access to item data
    pub fn item_data(&self, id: &ItemId) -> DataResult<&ItemData> {
        self.items.get(id).ok_or_else(|| DataError::ItemNotFound { 
            item: id.clone() 
        })
    }
    
    /// Direct access to ability data
    pub fn ability_data(&self, id: &AbilityId) -> DataResult<&AbilityData> {
        self.abilities.get(id).ok_or_else(|| DataError::AbilityNotFound { 
            ability: id.clone() 
        })
    }
    
    /// Convert move data to engine Move type when needed
    pub fn create_move(&self, id: &MoveId) -> DataResult<crate::core::battle_state::Move> {
        let data = self.move_data(id)?;
        Ok(data.to_engine_move())
    }
    
    /// Check if move exists
    pub fn has_move(&self, id: &MoveId) -> bool {
        self.moves.contains_key(id)
    }
    
    /// Check if pokemon exists
    pub fn has_pokemon(&self, id: &SpeciesId) -> bool {
        self.pokemon.contains_key(id)
    }
    
    /// Check if item exists
    pub fn has_item(&self, id: &ItemId) -> bool {
        self.items.contains_key(id)
    }
    
    /// Check if ability exists
    pub fn has_ability(&self, id: &AbilityId) -> bool {
        self.abilities.contains_key(id)
    }
    
    /// Find move data by name (case-insensitive) - optimized with index
    pub fn find_move_by_name(&self, name: &str) -> Option<&MoveData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first
        if let Some(move_id) = self.move_name_index.get(&normalized_name) {
            return self.moves.get(move_id);
        }
        
        // Fallback to linear search for edge cases
        self.moves.values().find(|move_data| normalize_name(&move_data.name) == normalized_name)
    }
    
    /// Find Pokemon data by name (case-insensitive) - optimized with index
    pub fn find_pokemon_by_name(&self, name: &str) -> Option<&PokemonData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first
        if let Some(species_id) = self.pokemon_name_index.get(&normalized_name) {
            return self.pokemon.get(species_id);
        }
        
        // Fallback to linear search for edge cases
        self.pokemon.values().find(|pokemon_data| normalize_name(&pokemon_data.name) == normalized_name)
    }
    
    
    /// Get all available move IDs
    pub fn move_ids(&self) -> impl Iterator<Item = &MoveId> {
        self.moves.keys()
    }
    
    /// Get all available species IDs
    pub fn species_ids(&self) -> impl Iterator<Item = &SpeciesId> {
        self.pokemon.keys()
    }
    
    /// Get all available item IDs
    pub fn item_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.items.keys()
    }
    
    /// Get all available ability IDs
    pub fn ability_ids(&self) -> impl Iterator<Item = &AbilityId> {
        self.abilities.keys()
    }
    
    /// Get repository statistics
    pub fn stats(&self) -> RepositoryStats {
        RepositoryStats {
            move_count: self.moves.len(),
            pokemon_count: self.pokemon.len(),
            item_count: self.items.len(),
            ability_count: self.abilities.len(),
        }
    }
    
    /// Get Pokemon weight in kilograms - optimized with index
    pub fn get_pokemon_weight(&self, species_name: &str) -> Option<f32> {
        let normalized_name = normalize_name(species_name);
        
        // Try index lookup first
        if let Some(species_id) = self.pokemon_name_index.get(&normalized_name) {
            if let Some(pokemon_data) = self.pokemon.get(species_id) {
                return Some(pokemon_data.weight_kg);
            }
        }
        
        // Fallback to linear search for edge cases
        for pokemon_data in self.pokemon.values() {
            if normalize_name(&pokemon_data.name) == normalized_name {
                return Some(pokemon_data.weight_kg);
            }
        }
        
        None
    }
    
    /// Get item fling power - optimized with index  
    pub fn get_item_fling_power(&self, item_name: &str) -> Option<u8> {
        let normalized_name = normalize_name(item_name);
        
        // Try index lookup first
        if let Some(item_id) = self.item_name_index.get(&normalized_name) {
            if let Some(item_data) = self.items.get(item_id) {
                return item_data.fling.as_ref().map(|f| f.base_power);
            }
        }
        
        // Fallback to linear search for edge cases
        for item_data in self.items.values() {
            if normalize_name(&item_data.name) == normalized_name {
                return item_data.fling.as_ref().map(|f| f.base_power);
            }
        }
        
        None
    }
    
    /// Check if item can be flung - optimized with index
    pub fn can_item_be_flung(&self, item_name: &str) -> bool {
        let normalized_name = normalize_name(item_name);
        
        // Try index lookup first
        if let Some(item_id) = self.item_name_index.get(&normalized_name) {
            if let Some(item_data) = self.items.get(item_id) {
                return item_data.fling.is_some();
            }
        }
        
        // Fallback to linear search for edge cases
        for item_data in self.items.values() {
            if normalize_name(&item_data.name) == normalized_name {
                return item_data.fling.is_some();
            }
        }
        
        false // Default to not flungable if not found
    }
}

/// Repository statistics
#[derive(Debug)]
pub struct RepositoryStats {
    pub move_count: usize,
    pub pokemon_count: usize,
    pub item_count: usize,
    pub ability_count: usize,
}

/// Generate consistent ID from string using hash
fn generate_consistent_id(input: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    // Ensure we don't generate 0 as ID
    let hash = hasher.finish() as u32;
    if hash == 0 { 1 } else { hash }
}

// Helper functions for loading data from JSON files
fn load_moves_data(path: &Path) -> DataResult<HashMap<MoveId, MoveData>> {
    if !path.exists() {
        return Err(DataError::FileRead { 
            path: path.to_path_buf(), 
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Moves data file not found") 
        });
    }
    
    let contents = std::fs::read_to_string(path)
        .map_err(|e| DataError::FileRead { 
            path: path.to_path_buf(), 
            source: e 
        })?;
    
    let raw_data: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|e| DataError::JsonParse { 
            file: path.display().to_string(), 
            source: e 
        })?;
    
    let mut moves = HashMap::new();
    let mut parse_errors = Vec::new();
    
    for (id, value) in raw_data {
        match serde_json::from_value::<MoveData>(value) {
            Ok(move_data) => {
                moves.insert(MoveId::from(id), move_data);
            }
            Err(e) => {
                parse_errors.push(format!("Failed to parse move '{}': {}", id, e));
            }
        }
    }
    
    // Log parse errors if any (could be made configurable)
    if !parse_errors.is_empty() {
        eprintln!("Warning: {} move parsing errors occurred", parse_errors.len());
        for error in parse_errors.iter().take(5) { // Show first 5 errors
            eprintln!("  {}", error);
        }
        if parse_errors.len() > 5 {
            eprintln!("  ... and {} more", parse_errors.len() - 5);
        }
        
        // If more than 90% of moves failed to parse, this indicates a structural issue
        let total_count = moves.len() + parse_errors.len();
        if parse_errors.len() > (total_count * 9 / 10) {
            return Err(DataError::JsonParse {
                file: path.display().to_string(),
                source: serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Too many parsing errors ({}/{}). This indicates a structural issue with the JSON format or struct definition.",
                        parse_errors.len(), total_count
                    )
                ))
            });
        }
    }
    
    Ok(moves)
}

fn load_pokemon_data(path: &Path) -> DataResult<HashMap<SpeciesId, PokemonData>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    
    let contents = std::fs::read_to_string(path)
        .map_err(|e| DataError::FileRead { 
            path: path.to_path_buf(), 
            source: e 
        })?;
    
    let raw_data: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|e| DataError::JsonParse { 
            file: path.display().to_string(), 
            source: e 
        })?;
    
    let mut pokemon = HashMap::new();
    let mut parse_errors = Vec::new();
    
    for (id, value) in raw_data {
        // Parse manually to handle weight extraction
        match serde_json::from_value::<PokemonData>(value.clone()) {
            Ok(mut pokemon_data) => {
                // Extract weight from PS data if available
                pokemon_data.weight_kg = value
                    .get("weightkg")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(50.0); // Default to 50kg if missing
                    
                pokemon.insert(SpeciesId::from(id), pokemon_data);
            }
            Err(e) => {
                parse_errors.push(format!("Failed to parse pokemon '{}': {}", id, e));
            }
        }
    }
    
    // Log parse errors if any
    if !parse_errors.is_empty() {
        eprintln!("Warning: {} pokemon parsing errors occurred", parse_errors.len());
        for error in parse_errors.iter().take(5) {
            eprintln!("  {}", error);
        }
        if parse_errors.len() > 5 {
            eprintln!("  ... and {} more", parse_errors.len() - 5);
        }
    }
    
    Ok(pokemon)
}

fn load_items_data(path: &Path) -> DataResult<HashMap<ItemId, ItemData>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    
    let contents = std::fs::read_to_string(path)
        .map_err(|e| DataError::FileRead { 
            path: path.to_path_buf(), 
            source: e 
        })?;
    
    let raw_data: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|e| DataError::JsonParse { 
            file: path.display().to_string(), 
            source: e 
        })?;
    
    let mut items = HashMap::new();
    let mut parse_errors = Vec::new();
    
    for (id, value) in raw_data {
        match serde_json::from_value::<ItemData>(value) {
            Ok(item_data) => {
                items.insert(ItemId::from(id), item_data);
            }
            Err(e) => {
                parse_errors.push(format!("Failed to parse item '{}': {}", id, e));
            }
        }
    }
    
    // Log parse errors if any
    if !parse_errors.is_empty() {
        eprintln!("Warning: {} item parsing errors occurred", parse_errors.len());
        for error in parse_errors.iter().take(5) {
            eprintln!("  {}", error);
        }
        if parse_errors.len() > 5 {
            eprintln!("  ... and {} more", parse_errors.len() - 5);
        }
    }
    
    Ok(items)
}

fn load_abilities_data(path: &Path) -> DataResult<HashMap<AbilityId, AbilityData>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    
    let contents = std::fs::read_to_string(path)
        .map_err(|e| DataError::FileRead { 
            path: path.to_path_buf(), 
            source: e 
        })?;
    
    let raw_data: HashMap<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|e| DataError::JsonParse { 
            file: path.display().to_string(), 
            source: e 
        })?;
    
    let mut abilities = HashMap::new();
    let mut parse_errors = Vec::new();
    
    for (id, value) in raw_data {
        match serde_json::from_value::<AbilityData>(value) {
            Ok(ability_data) => {
                abilities.insert(AbilityId::from(id), ability_data);
            }
            Err(e) => {
                parse_errors.push(format!("Failed to parse ability '{}': {}", id, e));
            }
        }
    }
    
    // Log parse errors if any
    if !parse_errors.is_empty() {
        eprintln!("Warning: {} ability parsing errors occurred", parse_errors.len());
        for error in parse_errors.iter().take(5) {
            eprintln!("  {}", error);
        }
        if parse_errors.len() > 5 {
            eprintln!("  ... and {} more", parse_errors.len() - 5);
        }
    }
    
    Ok(abilities)
}