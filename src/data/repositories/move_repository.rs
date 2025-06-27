use crate::types::{DataError, DataResult, MoveId};
use crate::utils::normalize_name;
use crate::data::showdown_types::MoveData;
use std::collections::HashMap;
use std::path::Path;

/// Repository for move-related data operations
pub struct MoveRepository {
    data: HashMap<MoveId, MoveData>,
    name_index: HashMap<String, MoveId>,
}

impl MoveRepository {
    /// Create new MoveRepository from data
    pub fn new(data: HashMap<MoveId, MoveData>) -> Self {
        // Get capacity before moving data
        let capacity = data.len();
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
        for (move_id, move_data) in &self.data {
            let normalized_name = normalize_name(&move_data.name);
            // Use reference to avoid cloning ID
            self.name_index.insert(normalized_name, move_id.clone());
        }
    }

    /// Get move data by ID
    pub fn find_by_id(&self, id: &MoveId) -> DataResult<&MoveData> {
        self.data.get(id).ok_or_else(|| DataError::MoveNotFound { 
            move_id: id.clone() 
        })
    }

    /// Find move data by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&MoveData> {
        let normalized_name = normalize_name(name);
        
        // Try index lookup first (O(1) with pre-computed normalized names)
        if let Some(move_id) = self.name_index.get(&normalized_name) {
            return self.data.get(move_id);
        }
        
        // Fallback to linear search should rarely be needed with proper indexing
        self.data.values().find(|move_data| normalize_name(&move_data.name) == normalized_name)
    }

    /// Check if move exists
    pub fn has_move(&self, id: &MoveId) -> bool {
        self.data.contains_key(id)
    }

    /// Convert move data to engine Move type when needed
    pub fn create_move(&self, id: &MoveId) -> DataResult<crate::core::battle_state::Move> {
        let data = self.find_by_id(id)?;
        Ok(data.to_engine_move())
    }

    /// Get all available move IDs
    pub fn move_ids(&self) -> impl Iterator<Item = &MoveId> {
        self.data.keys()
    }

    /// Get move count
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Get name index size for performance monitoring
    pub fn index_size(&self) -> usize {
        self.name_index.len()
    }
}

/// Load moves data from JSON file
pub fn load_moves_data(path: &Path) -> DataResult<HashMap<MoveId, MoveData>> {
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
    
    // Pre-allocate capacity based on raw data size
    let mut moves = HashMap::with_capacity(raw_data.len());
    let mut parse_errors = Vec::with_capacity(raw_data.len() / 10); // Estimate ~10% parse errors
    
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