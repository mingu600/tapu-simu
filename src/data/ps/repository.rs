use crate::types::{AbilityId, DataError, DataResult, ItemId, MoveId, SpeciesId, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Direct data access repository without factory or service layers
pub struct Repository {
    moves: HashMap<MoveId, MoveData>,
    pokemon: HashMap<SpeciesId, PokemonData>,
    items: HashMap<ItemId, ItemData>,
    abilities: HashMap<AbilityId, AbilityData>,
}

impl Repository {
    /// Load repository from PS data directory
    pub fn from_path(path: impl AsRef<Path>) -> DataResult<Self> {
        let path = path.as_ref();
        
        // Load each data type directly from JSON files
        let moves = load_moves_data(&path.join("moves.json"))?;
        let pokemon = load_pokemon_data(&path.join("pokemon.json"))?;
        let items = load_items_data(&path.join("items.json"))?;
        let abilities = load_abilities_data(&path.join("abilities.json"))?;
        
        Ok(Self {
            moves,
            pokemon, 
            items,
            abilities,
        })
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
    
    /// Get Pokemon weight in kilograms
    pub fn get_pokemon_weight(&self, species_name: &str) -> Option<f32> {
        // Normalize the species name for lookup
        let normalized_name = species_name.to_lowercase()
            .replace(" ", "")
            .replace("-", "")
            .replace("'", "");
            
        // Try different name variations for lookup
        for (pokemon_id, pokemon_data) in &self.pokemon {
            let normalized_id = pokemon_id.as_str().to_lowercase()
                .replace(" ", "")
                .replace("-", "")
                .replace("'", "");
                
            if normalized_id == normalized_name || 
               pokemon_data.name.to_lowercase() == species_name.to_lowercase() {
                return Some(pokemon_data.weight_kg);
            }
        }
        
        None
    }
    
    /// Get item fling power
    pub fn get_item_fling_power(&self, item_name: &str) -> Option<u8> {
        let normalized_name = item_name.to_lowercase().replace(" ", "").replace("-", "");
        
        for (item_id, item_data) in &self.items {
            let normalized_id = item_id.as_str().to_lowercase().replace(" ", "").replace("-", "");
            
            if normalized_id == normalized_name {
                return item_data.fling_power;
            }
        }
        
        None
    }
    
    /// Check if item can be flung
    pub fn can_item_be_flung(&self, item_name: &str) -> bool {
        let normalized_name = item_name.to_lowercase().replace(" ", "").replace("-", "");
        
        for (item_id, item_data) in &self.items {
            let normalized_id = item_id.as_str().to_lowercase().replace(" ", "").replace("-", "");
            
            if normalized_id == normalized_name {
                return item_data.can_be_flung;
            }
        }
        
        false // Default to not flungable if not found
    }
}

/// Simplified move data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub name: String,
    pub base_power: u8,
    pub accuracy: u8,
    pub move_type: TypeId,
    pub pp: u8,
    pub max_pp: u8,
    pub target: String,
    pub category: String,
    pub priority: i8,
    pub drain: Option<[u8; 2]>,  // [numerator, denominator]
    pub recoil: Option<[u8; 2]>, // [numerator, denominator]
    pub flags: Vec<String>,
}

impl MoveData {
    /// Convert to engine Move type
    pub fn to_engine_move(&self) -> crate::core::battle_state::Move {
        use crate::core::battle_state::{Move, MoveCategory};
        use crate::data::conversion::target_from_string;
        
        Move {
            name: self.name.clone(),
            base_power: self.base_power,
            accuracy: self.accuracy,
            move_type: self.move_type.as_str().to_string(),
            pp: self.pp,
            max_pp: self.max_pp,
            target: target_from_string(&self.target),
            category: match self.category.as_str() {
                "Physical" => MoveCategory::Physical,
                "Special" => MoveCategory::Special,
                "Status" => MoveCategory::Status,
                _ => MoveCategory::Status,
            },
            priority: self.priority,
        }
    }
    
    /// Get drain ratio if move has drain
    pub fn drain_ratio(&self) -> Option<f32> {
        self.drain.map(|[num, denom]| num as f32 / denom as f32)
    }
    
    /// Get recoil ratio if move has recoil
    pub fn recoil_ratio(&self) -> Option<f32> {
        self.recoil.map(|[num, denom]| num as f32 / denom as f32)
    }
}

/// Simplified pokemon data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonData {
    pub name: String,
    pub base_stats: BaseStats,
    pub types: Vec<TypeId>,
    pub abilities: HashMap<String, AbilityId>, // slot -> ability
    #[serde(default = "default_weight")]
    pub weight_kg: f32,  // Weight in kilograms
}

fn default_weight() -> f32 {
    50.0
}

/// Base stats structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseStats {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}

impl BaseStats {
    /// Convert to engine stats format
    pub fn to_engine_stats(&self) -> crate::data::types::EngineBaseStats {
        crate::data::types::EngineBaseStats {
            hp: self.hp as i16,
            attack: self.attack as i16,
            defense: self.defense as i16,
            special_attack: self.special_attack as i16,
            special_defense: self.special_defense as i16,
            speed: self.speed as i16,
        }
    }
}

/// Simplified item data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub name: String,
    pub description: String,
    pub fling_power: Option<u8>,
    pub fling_effect: Option<String>,
    #[serde(default = "default_can_be_flung")]
    pub can_be_flung: bool,  // Whether item can be flung
}

fn default_can_be_flung() -> bool {
    true
}

/// Simplified ability data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityData {
    pub name: String,
    pub description: String,
    pub short_desc: String,
}

/// Repository statistics
#[derive(Debug)]
pub struct RepositoryStats {
    pub move_count: usize,
    pub pokemon_count: usize,
    pub item_count: usize,
    pub ability_count: usize,
}

// Helper functions for loading data from JSON files
fn load_moves_data(path: &Path) -> DataResult<HashMap<MoveId, MoveData>> {
    if !path.exists() {
        return Ok(HashMap::new()); // Return empty if file doesn't exist
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
    for (id, value) in raw_data {
        if let Ok(move_data) = serde_json::from_value::<MoveData>(value) {
            moves.insert(MoveId::from(id), move_data);
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
    for (id, value) in raw_data {
        // Parse manually to handle weight extraction
        if let Ok(mut pokemon_data) = serde_json::from_value::<PokemonData>(value.clone()) {
            // Extract weight from PS data if available
            pokemon_data.weight_kg = value
                .get("weightkg")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(50.0); // Default to 50kg if missing
                
            pokemon.insert(SpeciesId::from(id), pokemon_data);
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
    for (id, value) in raw_data {
        // Parse manually to handle fling data extraction
        if let Ok(mut item_data) = serde_json::from_value::<ItemData>(value.clone()) {
            // Extract fling power from PS data
            item_data.fling_power = value
                .get("fling")
                .and_then(|fling| fling.get("basePower"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u8);
            
            // Determine if item can be flung - default to true unless marked as key item or unobtainable
            let is_key_item = value.get("isNonstandard")
                .and_then(|v| v.as_str())
                .map(|s| s == "Unobtainable" || s == "Past")
                .unwrap_or(false);
            
            // Specific unflingable items (orbs, etc.)
            let is_unflingable_orb = id.contains("orb") && 
                (id.contains("red") || id.contains("blue") || id.contains("adamant") || 
                 id.contains("lustrous") || id.contains("griseous"));
            
            item_data.can_be_flung = !is_key_item && !is_unflingable_orb;
            
            items.insert(ItemId::from(id), item_data);
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
    for (id, value) in raw_data {
        if let Ok(ability_data) = serde_json::from_value::<AbilityData>(value) {
            abilities.insert(AbilityId::from(id), ability_data);
        }
    }
    
    Ok(abilities)
}