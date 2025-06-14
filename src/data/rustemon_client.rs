//! # Rustemon Client Wrapper
//! 
//! This module provides a wrapper around the rustemon client for fetching
//! Pokemon data from PokeAPI.

use rustemon::client::RustemonClient as BaseRustemonClient;
use rustemon::model::pokemon::Pokemon as RustemonPokemon;
use rustemon::model::moves::Move as RustemonMove;
use std::collections::HashMap;

/// Wrapper client for rustemon with caching and error handling
pub struct RustemonClient {
    client: BaseRustemonClient,
    pokemon_cache: HashMap<String, RustemonPokemon>,
    move_cache: HashMap<String, RustemonMove>,
}

impl RustemonClient {
    /// Create a new rustemon client
    pub fn new() -> Self {
        Self {
            client: BaseRustemonClient::default(),
            pokemon_cache: HashMap::new(),
            move_cache: HashMap::new(),
        }
    }

    /// Fetch Pokemon data by name or ID
    pub async fn get_pokemon(&mut self, name_or_id: &str) -> Result<RustemonPokemon, Box<dyn std::error::Error>> {
        if let Some(pokemon) = self.pokemon_cache.get(name_or_id) {
            return Ok(pokemon.clone());
        }

        let pokemon = rustemon::pokemon::pokemon::get_by_name(name_or_id, &self.client).await?;
        self.pokemon_cache.insert(name_or_id.to_string(), pokemon.clone());
        Ok(pokemon)
    }

    /// Fetch move data by name or ID
    pub async fn get_move(&mut self, name_or_id: &str) -> Result<RustemonMove, Box<dyn std::error::Error>> {
        if let Some(move_data) = self.move_cache.get(name_or_id) {
            return Ok(move_data.clone());
        }

        let move_data = rustemon::moves::move_::get_by_name(name_or_id, &self.client).await?;
        self.move_cache.insert(name_or_id.to_string(), move_data.clone());
        Ok(move_data)
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.pokemon_cache.clear();
        self.move_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.pokemon_cache.len(), self.move_cache.len())
    }
}

impl Default for RustemonClient {
    fn default() -> Self {
        Self::new()
    }
}