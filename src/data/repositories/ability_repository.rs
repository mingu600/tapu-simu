use crate::types::{DataError, DataResult, AbilityId};
use crate::utils::normalize_name;
use crate::data::showdown_types::AbilityData;
use std::collections::HashMap;
use std::path::Path;

/// Repository for ability-related data operations
pub struct AbilityRepository {
    data: HashMap<AbilityId, AbilityData>,
    name_index: HashMap<String, AbilityId>,
}

impl AbilityRepository {
    /// Create new AbilityRepository from data
    pub fn new(data: HashMap<AbilityId, AbilityData>) -> Self {
        let mut repo = Self {
            data,
            name_index: HashMap::new(),
        };
        repo.build_index();
        repo
    }

    /// Build performance index for fast name lookups
    fn build_index(&mut self) {
        for (ability_id, ability_data) in &self.data {
            let normalized_name = normalize_name(&ability_data.name);
            self.name_index.insert(normalized_name, ability_id.clone());
        }
    }

    /// Get ability data by ID
    pub fn find_by_id(&self, id: &AbilityId) -> DataResult<&AbilityData> {
        self.data.get(id).ok_or_else(|| DataError::AbilityNotFound { 
            ability: id.clone() 
        })
    }

    /// Find ability data by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&AbilityData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first
        if let Some(ability_id) = self.name_index.get(&normalized_name) {
            return self.data.get(ability_id);
        }
        
        // Fallback to linear search for edge cases
        self.data.values().find(|ability_data| normalize_name(&ability_data.name) == normalized_name)
    }

    /// Check if ability exists
    pub fn has_ability(&self, id: &AbilityId) -> bool {
        self.data.contains_key(id)
    }

    /// Get all available ability IDs
    pub fn ability_ids(&self) -> impl Iterator<Item = &AbilityId> {
        self.data.keys()
    }

    /// Get ability count
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

/// Load abilities data from JSON file
pub fn load_abilities_data(path: &Path) -> DataResult<HashMap<AbilityId, AbilityData>> {
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