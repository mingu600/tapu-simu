use crate::types::{DataError, DataResult, ItemId};
use crate::utils::normalize_name;
use crate::data::showdown_types::ItemData;
use std::collections::HashMap;
use std::path::Path;

/// Repository for item-related data operations
pub struct ItemRepository {
    data: HashMap<ItemId, ItemData>,
    name_index: HashMap<String, ItemId>,
}

impl ItemRepository {
    /// Create new ItemRepository from data
    pub fn new(data: HashMap<ItemId, ItemData>) -> Self {
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
        for (item_id, item_data) in &self.data {
            let normalized_name = normalize_name(&item_data.name);
            self.name_index.insert(normalized_name, item_id.clone());
            // Also index by item ID string
            let normalized_id = normalize_name(item_id.as_str());
            self.name_index.insert(normalized_id, item_id.clone());
        }
    }

    /// Get item data by ID
    pub fn find_by_id(&self, id: &ItemId) -> DataResult<&ItemData> {
        self.data.get(id).ok_or_else(|| DataError::ItemNotFound { 
            item: id.clone() 
        })
    }

    /// Find item data by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&ItemData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first
        if let Some(item_id) = self.name_index.get(&normalized_name) {
            return self.data.get(item_id);
        }
        
        // Fallback to linear search for edge cases
        self.data.values().find(|item_data| normalize_name(&item_data.name) == normalized_name)
    }

    /// Check if item exists
    pub fn has_item(&self, id: &ItemId) -> bool {
        self.data.contains_key(id)
    }

    /// Get item fling power
    pub fn get_item_fling_power(&self, item_name: &str) -> Option<u8> {
        let normalized_name = normalize_name(item_name);
        
        // Try index lookup first
        if let Some(item_id) = self.name_index.get(&normalized_name) {
            if let Some(item_data) = self.data.get(item_id) {
                return item_data.fling.as_ref().map(|f| f.base_power);
            }
        }
        
        // Fallback to linear search for edge cases
        for item_data in self.data.values() {
            if normalize_name(&item_data.name) == normalized_name {
                return item_data.fling.as_ref().map(|f| f.base_power);
            }
        }
        
        None
    }

    /// Check if item can be flung
    pub fn can_item_be_flung(&self, item_name: &str) -> bool {
        let normalized_name = normalize_name(item_name);
        
        // Try index lookup first
        if let Some(item_id) = self.name_index.get(&normalized_name) {
            if let Some(item_data) = self.data.get(item_id) {
                return item_data.fling.is_some();
            }
        }
        
        // Fallback to linear search for edge cases
        for item_data in self.data.values() {
            if normalize_name(&item_data.name) == normalized_name {
                return item_data.fling.is_some();
            }
        }
        
        false // Default to not flungable if not found
    }

    /// Get all available item IDs
    pub fn item_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.data.keys()
    }

    /// Get item count
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Get name index size for performance monitoring
    pub fn index_size(&self) -> usize {
        self.name_index.len()
    }
}

/// Load items data from JSON file
pub fn load_items_data(path: &Path) -> DataResult<HashMap<ItemId, ItemData>> {
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
    let mut items = HashMap::with_capacity(raw_data.len());
    let mut parse_errors = Vec::with_capacity(raw_data.len() / 10); // Estimate ~10% parse errors
    
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