use crate::types::{DataError, DataResult, PokemonName};
use crate::utils::normalize_name;
use crate::data::showdown_types::PokemonData;
use std::collections::HashMap;
use std::path::Path;

/// Repository for pokemon-related data operations
pub struct PokemonRepository {
    data: HashMap<PokemonName, PokemonData>,
    name_index: HashMap<String, PokemonName>,
}

impl PokemonRepository {
    /// Create new PokemonRepository from data
    pub fn new(data: HashMap<PokemonName, PokemonData>) -> Self {
        // Get capacity before moving data
        let capacity = data.len() * 2; // Multiply by 2 to account for both name and ID indexing
        let mut repo = Self {
            data,
            // Pre-allocate capacity for name index to avoid rehashing
            name_index: HashMap::with_capacity(capacity),
        };
        repo.build_index();
        repo
    }

    /// Build performance index for fast name lookups
    fn build_index(&mut self) {
        for (species_id, pokemon_data) in &self.data {
            let normalized_name = normalize_name(&pokemon_data.name);
            self.name_index.insert(normalized_name, species_id.clone());
            // Also index by species ID string
            let normalized_id = normalize_name(&format!("{:?}", species_id));
            self.name_index.insert(normalized_id, species_id.clone());
        }
    }

    /// Get pokemon data by ID
    pub fn find_by_id(&self, id: &PokemonName) -> DataResult<&PokemonData> {
        self.data.get(id).ok_or_else(|| DataError::SpeciesNotFound { 
            species: id.clone() 
        })
    }

    /// Find Pokemon data by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&PokemonData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first
        if let Some(species_id) = self.name_index.get(&normalized_name) {
            return self.data.get(species_id);
        }
        
        // Fallback to linear search for edge cases
        self.data.values().find(|pokemon_data| normalize_name(&pokemon_data.name) == normalized_name)
    }

    /// Check if pokemon exists
    pub fn has_pokemon(&self, id: &PokemonName) -> bool {
        self.data.contains_key(id)
    }

    /// Get Pokemon weight in kilograms
    pub fn get_pokemon_weight(&self, species_name: &str) -> Option<f32> {
        let normalized_name = normalize_name(species_name);
        
        // Try index lookup first
        if let Some(species_id) = self.name_index.get(&normalized_name) {
            if let Some(pokemon_data) = self.data.get(species_id) {
                return Some(pokemon_data.weight_kg);
            }
        }
        
        // Fallback to linear search for edge cases
        for pokemon_data in self.data.values() {
            if normalize_name(&pokemon_data.name) == normalized_name {
                return Some(pokemon_data.weight_kg);
            }
        }
        
        None
    }

    /// Get all available species IDs
    pub fn species_ids(&self) -> impl Iterator<Item = &PokemonName> {
        self.data.keys()
    }

    /// Get pokemon count
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Get name index size for performance monitoring
    pub fn index_size(&self) -> usize {
        self.name_index.len()
    }
}

/// Load pokemon data from JSON file
pub fn load_pokemon_data(path: &Path) -> DataResult<HashMap<PokemonName, PokemonData>> {
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
    
    // Pre-allocate capacity based on raw data size
    let mut pokemon = HashMap::with_capacity(raw_data.len());
    let mut parse_errors = Vec::with_capacity(raw_data.len() / 10); // Estimate ~10% parse errors
    
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
                    
                pokemon.insert(crate::types::FromNormalizedString::from_normalized_str(&crate::utils::normalize_name(&id)).unwrap_or(PokemonName::NONE), pokemon_data);
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