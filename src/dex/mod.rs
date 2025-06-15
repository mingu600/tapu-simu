//! Data management system for Pokemon, moves, abilities, and items

pub mod showdown_data;

use crate::pokemon::{SpeciesData, MoveData, AbilityData, ItemData};
use crate::types::Type;
use crate::errors::{BattleResult, BattleError};
use self::showdown_data::ShowdownDataParser;
use std::path::Path;
use std::collections::HashMap;

/// Trait for accessing Pokemon data
pub trait Dex {
    fn get_move(&self, id: &str) -> Option<&MoveData>;
    fn get_species(&self, id: &str) -> Option<&SpeciesData>;
    fn get_ability(&self, id: &str) -> Option<&AbilityData>;
    fn get_item(&self, id: &str) -> Option<&ItemData>;
    fn get_type_effectiveness(&self, attacking: Type, defending: Type) -> f32;
}

/// Pokemon Showdown data implementation
pub struct ShowdownDex {
    pub parser: ShowdownDataParser,
}

impl ShowdownDex {
    /// Create a new ShowdownDex from Pokemon Showdown data directory
    pub fn new(data_dir: &Path) -> BattleResult<Self> {
        let parser = ShowdownDataParser::new(data_dir)?;
        Ok(Self { parser })
    }
    
    /// Create a ShowdownDex with the default Pokemon Showdown data location
    pub fn with_default_data() -> BattleResult<Self> {
        let default_path = Path::new("../pokemon-showdown/data");
        if default_path.exists() {
            Self::new(default_path)
        } else {
            Err(BattleError::DataError(
                "Pokemon Showdown data directory not found. Please specify the path manually.".to_string()
            ))
        }
    }
    
    /// Get the number of moves loaded
    pub fn moves_count(&self) -> usize {
        self.parser.moves.len()
    }
    
    /// Get the number of species loaded
    pub fn species_count(&self) -> usize {
        self.parser.species.len()
    }
    
    /// Get the number of abilities loaded
    pub fn abilities_count(&self) -> usize {
        self.parser.abilities.len()
    }
    
    /// Get the number of items loaded
    pub fn items_count(&self) -> usize {
        self.parser.items.len()
    }
}

impl Dex for ShowdownDex {
    fn get_move(&self, id: &str) -> Option<&MoveData> {
        self.parser.moves.get(id)
    }
    
    fn get_species(&self, id: &str) -> Option<&SpeciesData> {
        self.parser.species.get(id)
    }
    
    fn get_ability(&self, id: &str) -> Option<&AbilityData> {
        self.parser.abilities.get(id)
    }
    
    fn get_item(&self, id: &str) -> Option<&ItemData> {
        self.parser.items.get(id)
    }
    
    fn get_type_effectiveness(&self, attacking: Type, defending: Type) -> f32 {
        self.parser.type_chart
            .get(&defending)
            .and_then(|effectiveness_map| effectiveness_map.get(&attacking))
            .copied()
            .unwrap_or(1.0)
    }
}

/// Test dex factory methods
impl ShowdownDex {
    /// Create a test dex that tries to load real data but falls back gracefully
    pub fn test_dex() -> Box<dyn Dex> {
        // Try to load real data first - pass the project root, parser will append data/ps-extracted
        if let Ok(dex) = Self::new(Path::new(".")) {
            Box::new(dex)
        } else {
            // Fall back to minimal mock dex for testing
            Box::new(TestDex::new())
        }
    }
}

/// Minimal test dex for when Pokemon Showdown data isn't available
pub struct TestDex;

impl TestDex {
    pub fn new() -> Self {
        Self
    }
}

impl Dex for TestDex {
    fn get_move(&self, _id: &str) -> Option<&MoveData> {
        // Return None - Pokemon factory methods will handle fallback
        None
    }
    
    fn get_species(&self, _id: &str) -> Option<&SpeciesData> {
        // Return None - Pokemon factory methods will handle fallback
        None
    }
    
    fn get_ability(&self, _id: &str) -> Option<&AbilityData> {
        // Return None - Pokemon factory methods will handle fallback
        None
    }
    
    fn get_item(&self, _id: &str) -> Option<&ItemData> {
        // Return None - Pokemon factory methods will handle fallback
        None
    }
    
    fn get_type_effectiveness(&self, _attacking: Type, _defending: Type) -> f32 {
        // Return neutral effectiveness for testing
        1.0
    }
}