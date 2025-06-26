//! Specialized repository modules for different data types

pub mod move_repository;
pub mod pokemon_repository;
pub mod item_repository;

pub use move_repository::{MoveRepository, load_moves_data};
pub use pokemon_repository::{PokemonRepository, load_pokemon_data};
pub use item_repository::{ItemRepository, load_items_data};

use crate::types::DataResult;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

/// Composite repository containing all game data repositories
pub struct GameDataRepository {
    pub moves: MoveRepository,
    pub pokemon: PokemonRepository,
    pub items: ItemRepository,
}

impl GameDataRepository {
    /// Create new GameDataRepository from PS data directory
    pub fn from_path(path: impl AsRef<Path>) -> DataResult<Self> {
        let path = path.as_ref();
        
        // Load data from JSON files
        let moves_data = load_moves_data(&path.join("moves.json"))?;
        let pokemon_data = load_pokemon_data(&path.join("pokemon.json"))?;
        let items_data = load_items_data(&path.join("items.json"))?;
        
        // Create specialized repositories
        let moves = MoveRepository::new(moves_data);
        let pokemon = PokemonRepository::new(pokemon_data);
        let items = ItemRepository::new(items_data);
        
        Ok(Self {
            moves,
            pokemon,
            items,
        })
    }

    /// Get repository statistics
    pub fn stats(&self) -> RepositoryStats {
        RepositoryStats {
            move_count: self.moves.count(),
            pokemon_count: self.pokemon.count(),
            item_count: self.items.count(),
        }
    }
}

/// Repository statistics
#[derive(Debug)]
pub struct RepositoryStats {
    pub move_count: usize,
    pub pokemon_count: usize,
    pub item_count: usize,
}

// Global repository instance (singleton pattern)
static GLOBAL_REPOSITORY: OnceLock<Mutex<Option<Arc<GameDataRepository>>>> = OnceLock::new();

impl GameDataRepository {
    /// Get or create global repository instance (singleton pattern)
    pub fn global(path: impl AsRef<Path>) -> DataResult<Arc<Self>> {
        let mutex = GLOBAL_REPOSITORY.get_or_init(|| Mutex::new(None));
        let mut repo = mutex.lock().unwrap();
        
        if let Some(existing) = repo.as_ref() {
            return Ok(Arc::clone(existing));
        }
        
        let new_repo = Arc::new(Self::from_path(path)?);
        *repo = Some(Arc::clone(&new_repo));
        Ok(new_repo)
    }
}