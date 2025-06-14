//! Data management system for Pokemon, moves, abilities, and items

pub mod showdown_data;

use crate::pokemon::{SpeciesData, MoveData, AbilityData, ItemData};
use crate::types::Type;
use crate::errors::BattleResult;

/// Trait for accessing Pokemon data
pub trait Dex {
    fn get_move(&self, id: &str) -> Option<&MoveData>;
    fn get_species(&self, id: &str) -> Option<&SpeciesData>;
    fn get_ability(&self, id: &str) -> Option<&AbilityData>;
    fn get_item(&self, id: &str) -> Option<&ItemData>;
    fn get_type_effectiveness(&self, attacking: Type, defending: Type) -> f32;
}

/// Implementation will come later
pub struct ShowdownDex {
    // Will be implemented in future phases
}

impl Dex for ShowdownDex {
    fn get_move(&self, _id: &str) -> Option<&MoveData> {
        None // TODO: Implement
    }
    
    fn get_species(&self, _id: &str) -> Option<&SpeciesData> {
        None // TODO: Implement
    }
    
    fn get_ability(&self, _id: &str) -> Option<&AbilityData> {
        None // TODO: Implement
    }
    
    fn get_item(&self, _id: &str) -> Option<&ItemData> {
        None // TODO: Implement
    }
    
    fn get_type_effectiveness(&self, _attacking: Type, _defending: Type) -> f32 {
        1.0 // TODO: Implement type chart
    }
}