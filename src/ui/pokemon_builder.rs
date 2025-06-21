//! # Pokemon Builder
//! 
//! This module provides utilities for building Pokemon from the PS data for the UI.

use crate::ui::bridge::{UIPokemon, UIPokemonStats, UIMove};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pokemon species data from PS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonData {
    pub name: String,
    pub types: Vec<String>,
    #[serde(rename = "baseStats")]
    pub base_stats: BaseStats,
    pub abilities: HashMap<String, String>,
    #[serde(default)]
    pub moves: Vec<String>,
    #[serde(rename = "weightkg")]
    pub weight: f32,
    #[serde(rename = "heightm")]
    pub height: f32,
}

/// Base stats from PS data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseStats {
    pub hp: i16,
    #[serde(rename = "atk")]
    pub attack: i16,
    #[serde(rename = "def")]
    pub defense: i16,
    #[serde(rename = "spa")]
    pub special_attack: i16,
    #[serde(rename = "spd")]
    pub special_defense: i16,
    #[serde(rename = "spe")]
    pub speed: i16,
}

/// Move data from PS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub name: String,
    #[serde(rename = "type")]
    pub move_type: String,
    pub category: String,
    #[serde(rename = "basePower")]
    pub base_power: u8,
    pub accuracy: u8,
    pub pp: u8,
    pub priority: i8,
    pub target: String,
    #[serde(default)]
    pub description: String,
}

/// Item data from PS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// Pokemon builder for creating Pokemon from PS data
pub struct PokemonBuilder {
    pokemon_data: HashMap<String, PokemonData>,
    move_data: HashMap<String, MoveData>,
    item_data: HashMap<String, ItemData>,
}

impl PokemonBuilder {
    /// Create a new Pokemon builder
    pub fn new() -> Self {
        Self {
            pokemon_data: HashMap::new(),
            move_data: HashMap::new(),
            item_data: HashMap::new(),
        }
    }

    /// Load Pokemon data from the PS extracted JSON files
    pub fn load_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load Pokemon data
        let pokemon_file = std::fs::read_to_string("data/ps-extracted/pokemon.json")
            .map_err(|e| format!("Failed to read pokemon.json: {}", e))?;
        let pokemon_map: HashMap<String, PokemonData> = serde_json::from_str(&pokemon_file)
            .map_err(|e| format!("Failed to parse pokemon.json: {}", e))?;
        self.pokemon_data = pokemon_map;

        // Load move data
        let moves_file = std::fs::read_to_string("data/ps-extracted/moves.json")
            .map_err(|e| format!("Failed to read moves.json: {}", e))?;
        let moves_map: HashMap<String, MoveData> = serde_json::from_str(&moves_file)
            .map_err(|e| format!("Failed to parse moves.json: {}", e))?;
        self.move_data = moves_map;

        // Load item data
        let items_file = std::fs::read_to_string("data/ps-extracted/items.json")
            .map_err(|e| format!("Failed to read items.json: {}", e))?;
        let items_map: HashMap<String, ItemData> = serde_json::from_str(&items_file)
            .map_err(|e| format!("Failed to parse items.json: {}", e))?;
        self.item_data = items_map;

        println!("Successfully loaded {} Pokemon, {} moves, {} items", 
                 self.pokemon_data.len(), self.move_data.len(), self.item_data.len());

        Ok(())
    }

    /// Get all available Pokemon species
    pub fn get_species_list(&self) -> Vec<String> {
        let mut species: Vec<String> = self.pokemon_data.keys().cloned().collect();
        species.sort();
        species
    }

    /// Get all available moves
    pub fn get_move_list(&self) -> Vec<String> {
        let mut moves: Vec<String> = self.move_data.keys().cloned().collect();
        moves.sort();
        moves
    }

    /// Get all available items
    pub fn get_item_list(&self) -> Vec<String> {
        let mut items: Vec<String> = self.item_data.keys().cloned().collect();
        items.sort();
        items
    }

    /// Get moves that a Pokemon can learn
    pub fn get_pokemon_moves(&self, species: &str) -> Vec<String> {
        if let Some(pokemon) = self.pokemon_data.get(species)
            .or_else(|| self.pokemon_data.get(&species.to_lowercase()))
            .or_else(|| self.pokemon_data.get(&self.normalize_name(species))) {
            pokemon.moves.clone()
        } else {
            vec![]
        }
    }

    /// Get Pokemon abilities
    pub fn get_pokemon_abilities(&self, species: &str) -> Vec<String> {
        if let Some(pokemon) = self.pokemon_data.get(species)
            .or_else(|| self.pokemon_data.get(&species.to_lowercase()))
            .or_else(|| self.pokemon_data.get(&self.normalize_name(species))) {
            pokemon.abilities.values().cloned().collect()
        } else {
            vec![]
        }
    }

    /// Create a Pokemon with default stats at level 50 with optimal IVs
    pub fn create_pokemon(&self, species: &str, level: u8) -> Result<UIPokemon, String> {
        self.create_pokemon_with_custom_stats(species, level, &[31; 6], &[0; 6], "Hardy")
    }

    /// Create a Pokemon with custom IVs, EVs, and nature
    pub fn create_pokemon_with_custom_stats(&self, species: &str, level: u8, ivs: &[u8; 6], evs: &[u8; 6], nature: &str) -> Result<UIPokemon, String> {
        self.create_fully_custom_pokemon(species, level, ivs, evs, nature, None, None, None, None)
    }

    /// Create a fully customized Pokemon with all options
    pub fn create_fully_custom_pokemon(
        &self,
        species: &str,
        level: u8,
        ivs: &[u8; 6],
        evs: &[u8; 6],
        nature: &str,
        ability: Option<&str>,
        item: Option<&str>,
        moves: Option<&[String]>,
        tera_type: Option<&str>,
    ) -> Result<UIPokemon, String> {
        // Try exact match first, then lowercase, then normalized
        let pokemon_data = self.pokemon_data.get(species)
            .or_else(|| self.pokemon_data.get(&species.to_lowercase()))
            .or_else(|| self.pokemon_data.get(&self.normalize_name(species)))
            .ok_or_else(|| format!("Pokemon {} not found", species))?;

        // Calculate stats with custom IVs, EVs, and nature
        let stats = self.calculate_stats_with_ivs_evs(&pokemon_data.base_stats, level, ivs, evs, nature);

        // Handle custom moves or use defaults
        let mut ui_moves = Vec::new();
        
        if let Some(custom_moves) = moves {
            // Use provided moves and validate them
            for move_name in custom_moves.iter().take(4) {
                if let Some(move_data) = self.move_data.get(move_name)
                    .or_else(|| self.move_data.get(&self.normalize_name(move_name))) {
                    ui_moves.push(UIMove {
                        name: move_data.name.clone(),
                        move_type: move_data.move_type.clone(),
                        category: move_data.category.clone(),
                        base_power: move_data.base_power,
                        accuracy: move_data.accuracy,
                        pp: move_data.pp,
                        max_pp: move_data.pp,
                        priority: move_data.priority,
                        target: move_data.target.clone(),
                    });
                } else {
                    // Log warning but don't fail - skip invalid moves
                    eprintln!("Warning: Move '{}' not found for {}, skipping", move_name, species);
                }
            }
        }
        
        // Fill remaining slots with default moves if needed
        if ui_moves.len() < 4 {
            let available_moves = self.get_pokemon_moves(species);
            for move_name in available_moves.iter() {
                if ui_moves.len() >= 4 { break; }
                
                // Don't add duplicates
                if ui_moves.iter().any(|m| m.name == *move_name) { continue; }
                
                if let Some(move_data) = self.move_data.get(move_name) {
                    ui_moves.push(UIMove {
                        name: move_data.name.clone(),
                        move_type: move_data.move_type.clone(),
                        category: move_data.category.clone(),
                        base_power: move_data.base_power,
                        accuracy: move_data.accuracy,
                        pp: move_data.pp,
                        max_pp: move_data.pp,
                        priority: move_data.priority,
                        target: move_data.target.clone(),
                    });
                }
            }
        }

        // Fill any remaining slots with Tackle if still needed
        while ui_moves.len() < 4 {
            ui_moves.push(UIMove {
                name: "Tackle".to_string(),
                move_type: "Normal".to_string(),
                category: "Physical".to_string(),
                base_power: 40,
                accuracy: 100,
                pp: 35,
                max_pp: 35,
                priority: 0,
                target: "Normal".to_string(),
            });
        }

        // Handle custom ability or use default
        let pokemon_ability = if let Some(custom_ability) = ability {
            // Validate the ability exists for this Pokemon
            if pokemon_data.abilities.values().any(|a| a == custom_ability) {
                custom_ability.to_string()
            } else {
                eprintln!("Warning: Ability '{}' not available for {}, using default", custom_ability, species);
                pokemon_data.abilities.get("0")
                    .or_else(|| pokemon_data.abilities.values().next())
                    .cloned()
                    .unwrap_or_else(|| "No Ability".to_string())
            }
        } else {
            // Default ability (prefer slot 0, then any other)
            pokemon_data.abilities.get("0")
                .or_else(|| pokemon_data.abilities.values().next())
                .cloned()
                .unwrap_or_else(|| "No Ability".to_string())
        };

        // Handle custom item
        let pokemon_item = if let Some(custom_item) = item {
            // Validate the item exists
            if self.item_data.contains_key(custom_item) || self.item_data.contains_key(&self.normalize_name(custom_item)) {
                Some(custom_item.to_string())
            } else {
                eprintln!("Warning: Item '{}' not found, ignoring", custom_item);
                None
            }
        } else {
            None
        };

        Ok(UIPokemon {
            species: species.to_string(),
            level,
            hp: stats.hp,
            max_hp: stats.hp,
            stats,
            moves: ui_moves,
            ability: pokemon_ability,
            item: pokemon_item,
            types: pokemon_data.types.clone(),
            gender: "Unknown".to_string(),
            nature: Some(nature.to_string()),
            ivs: Some(ivs.to_vec()),
            evs: Some(evs.to_vec()),
            tera_type: tera_type.map(|t| t.to_string()),
            is_terastallized: false,
        })
    }

    /// Calculate stats with IVs, EVs, and nature
    /// ivs and evs are arrays in order: [HP, Atk, Def, SpA, SpD, Spe]
    fn calculate_stats_with_ivs_evs(&self, base_stats: &BaseStats, level: u8, ivs: &[u8; 6], evs: &[u8; 6], nature: &str) -> UIPokemonStats {
        let level = level as f32;
        
        // Nature modifiers (1.1 for boosted, 0.9 for hindered, 1.0 for neutral)
        let (atk_mod, def_mod, spa_mod, spd_mod, spe_mod) = match nature {
            "Lonely" => (1.1, 0.9, 1.0, 1.0, 1.0),  // +Atk -Def
            "Brave" => (1.1, 1.0, 1.0, 1.0, 0.9),   // +Atk -Spe
            "Adamant" => (1.1, 1.0, 0.9, 1.0, 1.0), // +Atk -SpA
            "Naughty" => (1.1, 1.0, 1.0, 0.9, 1.0), // +Atk -SpD
            "Bold" => (0.9, 1.1, 1.0, 1.0, 1.0),    // +Def -Atk
            "Relaxed" => (1.0, 1.1, 1.0, 1.0, 0.9), // +Def -Spe
            "Impish" => (1.0, 1.1, 0.9, 1.0, 1.0),  // +Def -SpA
            "Lax" => (1.0, 1.1, 1.0, 0.9, 1.0),     // +Def -SpD
            "Timid" => (0.9, 1.0, 1.0, 1.0, 1.1),   // +Spe -Atk
            "Hasty" => (1.0, 0.9, 1.0, 1.0, 1.1),   // +Spe -Def
            "Jolly" => (1.0, 1.0, 0.9, 1.0, 1.1),   // +Spe -SpA
            "Naive" => (1.0, 1.0, 1.0, 0.9, 1.1),   // +Spe -SpD
            "Modest" => (0.9, 1.0, 1.1, 1.0, 1.0),  // +SpA -Atk
            "Mild" => (1.0, 0.9, 1.1, 1.0, 1.0),    // +SpA -Def
            "Quiet" => (1.0, 1.0, 1.1, 1.0, 0.9),   // +SpA -Spe
            "Rash" => (1.0, 1.0, 1.1, 0.9, 1.0),    // +SpA -SpD
            "Calm" => (0.9, 1.0, 1.0, 1.1, 1.0),    // +SpD -Atk
            "Gentle" => (1.0, 0.9, 1.0, 1.1, 1.0),  // +SpD -Def
            "Sassy" => (1.0, 1.0, 1.0, 1.1, 0.9),   // +SpD -Spe
            "Careful" => (1.0, 1.0, 0.9, 1.1, 1.0), // +SpD -SpA
            _ => (1.0, 1.0, 1.0, 1.0, 1.0),         // Hardy/Docile/Serious/Bashful/Quirky (neutral)
        };

        // Pokemon stat calculation formula
        // HP: floor(((2 * base + iv + floor(ev/4)) * level / 100) + level + 10)
        // Other: floor((floor(((2 * base + iv + floor(ev/4)) * level / 100) + 5) * nature))
        
        let hp = (((2.0 * base_stats.hp as f32 + ivs[0] as f32 + (evs[0] as f32 / 4.0).floor()) * level / 100.0) + level + 10.0).floor() as i16;
        
        let attack = ((((2.0 * base_stats.attack as f32 + ivs[1] as f32 + (evs[1] as f32 / 4.0).floor()) * level / 100.0) + 5.0).floor() * atk_mod).floor() as i16;
        
        let defense = ((((2.0 * base_stats.defense as f32 + ivs[2] as f32 + (evs[2] as f32 / 4.0).floor()) * level / 100.0) + 5.0).floor() * def_mod).floor() as i16;
        
        let special_attack = ((((2.0 * base_stats.special_attack as f32 + ivs[3] as f32 + (evs[3] as f32 / 4.0).floor()) * level / 100.0) + 5.0).floor() * spa_mod).floor() as i16;
        
        let special_defense = ((((2.0 * base_stats.special_defense as f32 + ivs[4] as f32 + (evs[4] as f32 / 4.0).floor()) * level / 100.0) + 5.0).floor() * spd_mod).floor() as i16;
        
        let speed = ((((2.0 * base_stats.speed as f32 + ivs[5] as f32 + (evs[5] as f32 / 4.0).floor()) * level / 100.0) + 5.0).floor() * spe_mod).floor() as i16;

        UIPokemonStats {
            hp,
            attack,
            defense,
            special_attack,
            special_defense,
            speed,
        }
    }

    /// Normalize a name for lookup (remove special characters, lowercase)
    fn normalize_name(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }

    /// Create a move by name
    pub fn create_move(&self, move_name: &str) -> Result<UIMove, String> {
        let move_data = self.move_data.get(move_name)
            .or_else(|| self.move_data.get(&self.normalize_name(move_name)))
            .ok_or_else(|| format!("Move {} not found", move_name))?;

        Ok(UIMove {
            name: move_data.name.clone(),
            move_type: move_data.move_type.clone(),
            category: move_data.category.clone(),
            base_power: move_data.base_power,
            accuracy: move_data.accuracy,
            pp: move_data.pp,
            max_pp: move_data.pp,
            priority: move_data.priority,
            target: move_data.target.clone(),
        })
    }

    /// Get suggested Pokemon for quick setup
    pub fn get_suggested_pokemon() -> Vec<(&'static str, &'static str)> {
        vec![
            // Classic starters
            ("Pikachu", "The classic electric mouse"),
            ("Charizard", "Fire/Flying starter evolution"),
            ("Blastoise", "Water starter evolution"),
            ("Venusaur", "Grass/Poison starter evolution"),
            
            // VGC 2024 Meta
            ("Flutter Mane", "Ghost/Fairy paradox Pokemon"),
            ("Chien-Pao", "Dark/Ice ruinous Pokemon"),
            ("Urshifu-Rapid-Strike", "Fighting/Water legendary"),
            ("Rillaboom", "Grass terrain setter"),
            ("Amoonguss", "Grass/Poison support"),
            ("Incineroar", "Fire/Dark intimidate user"),
            
            // OU Singles Meta
            ("Dragapult", "Dragon/Ghost pseudo-legendary"),
            ("Toxapex", "Poison/Water wall"),
            ("Garchomp", "Dragon/Ground pseudo-legendary"),
            ("Heatran", "Fire/Steel legendary"),
            ("Clefable", "Fairy magic guard user"),
            ("Rotom-Wash", "Electric/Water utility"),
            
            // Classic Powerhouses
            ("Metagross", "Steel/Psychic pseudo-legendary"),
            ("Tyranitar", "Rock/Dark pseudo-legendary"),
            ("Dragonite", "Dragon/Flying pseudo-legendary"),
        ]
    }

    /// Get available preset teams
    pub fn get_preset_teams() -> Vec<(&'static str, &'static str)> {
        vec![
            ("basic", "Simple team for testing basic mechanics"),
            ("vgc2024", "VGC 2024 regulation H meta team"),
            ("ou2024", "OU Singles meta team"),
            ("hyper_offense", "Fast-paced offensive team"),
            ("weather_teams", "Weather-based strategies showcase"),
        ]
    }

    /// Create a preset team for quick testing
    pub fn create_preset_team(&self, preset_name: &str) -> Result<Vec<UIPokemon>, String> {
        match preset_name {
            "basic" => {
                Ok(vec![
                    self.create_pokemon("Pikachu", 50)?,
                    self.create_pokemon("Charizard", 50)?,
                ])
            }
            "vgc2024" => {
                Ok(vec![
                    self.create_competitive_pokemon(
                        "Flutter Mane", 50, "Timid", "Protosynthesis", Some("Choice Specs"),
                        vec!["Shadow Ball", "Moonblast", "Mystical Fire", "Icy Wind"]
                    )?,
                    self.create_competitive_pokemon(
                        "Chien-Pao", 50, "Jolly", "Sword of Ruin", Some("Focus Sash"),
                        vec!["Ice Spinner", "Sucker Punch", "Sacred Sword", "Protect"]
                    )?,
                    self.create_competitive_pokemon(
                        "Urshifu-Rapid-Strike", 50, "Jolly", "Unseen Fist", Some("Choice Band"),
                        vec!["Surging Strikes", "Close Combat", "U-turn", "Aqua Jet"]
                    )?,
                    self.create_competitive_pokemon(
                        "Rillaboom", 50, "Adamant", "Grassy Surge", Some("Assault Vest"),
                        vec!["Grassy Glide", "Wood Hammer", "U-turn", "High Horsepower"]
                    )?,
                    self.create_competitive_pokemon(
                        "Amoonguss", 50, "Bold", "Regenerator", Some("Rocky Helmet"),
                        vec!["Spore", "Rage Powder", "Pollen Puff", "Protect"]
                    )?,
                    self.create_competitive_pokemon(
                        "Incineroar", 50, "Careful", "Intimidate", Some("Sitrus Berry"),
                        vec!["Fake Out", "Flare Blitz", "Knock Off", "Parting Shot"]
                    )?,
                ])
            }
            "ou2024" => {
                Ok(vec![
                    self.create_competitive_pokemon(
                        "Dragapult", 50, "Jolly", "Clear Body", Some("Choice Band"),
                        vec!["Dragon Darts", "Phantom Force", "U-turn", "Sucker Punch"]
                    )?,
                    self.create_competitive_pokemon(
                        "Toxapex", 50, "Bold", "Regenerator", Some("Black Sludge"),
                        vec!["Scald", "Recover", "Haze", "Toxic Spikes"]
                    )?,
                    self.create_competitive_pokemon(
                        "Garchomp", 50, "Jolly", "Rough Skin", Some("Rocky Helmet"),
                        vec!["Earthquake", "Dragon Claw", "Stone Edge", "Stealth Rock"]
                    )?,
                    self.create_competitive_pokemon(
                        "Heatran", 50, "Modest", "Flash Fire", Some("Air Balloon"),
                        vec!["Magma Storm", "Earth Power", "Flash Cannon", "Taunt"]
                    )?,
                    self.create_competitive_pokemon(
                        "Clefable", 50, "Bold", "Magic Guard", Some("Leftovers"),
                        vec!["Moonblast", "Soft-Boiled", "Stealth Rock", "Thunder Wave"]
                    )?,
                    self.create_competitive_pokemon(
                        "Rotom-Wash", 50, "Bold", "Levitate", Some("Leftovers"),
                        vec!["Hydro Pump", "Volt Switch", "Will-O-Wisp", "Defog"]
                    )?,
                ])
            }
            "hyper_offense" => {
                Ok(vec![
                    self.create_competitive_pokemon(
                        "Dragonite", 50, "Adamant", "Multiscale", Some("Weakness Policy"),
                        vec!["Dragon Dance", "Outrage", "Earthquake", "Extreme Speed"]
                    )?,
                    self.create_competitive_pokemon(
                        "Excadrill", 50, "Jolly", "Sand Rush", Some("Life Orb"),
                        vec!["Earthquake", "Iron Head", "Rock Slide", "Rapid Spin"]
                    )?,
                    self.create_competitive_pokemon(
                        "Tyranitar", 50, "Careful", "Sand Stream", Some("Smooth Rock"),
                        vec!["Stone Edge", "Crunch", "Stealth Rock", "Thunder Wave"]
                    )?,
                    self.create_competitive_pokemon(
                        "Hawlucha", 50, "Adamant", "Unburden", Some("Electric Seed"),
                        vec!["Acrobatics", "Close Combat", "Swords Dance", "Roost"]
                    )?,
                    self.create_competitive_pokemon(
                        "Azumarill", 50, "Adamant", "Huge Power", Some("Sitrus Berry"),
                        vec!["Aqua Jet", "Play Rough", "Belly Drum", "Knock Off"]
                    )?,
                    self.create_competitive_pokemon(
                        "Magnezone", 50, "Modest", "Magnet Pull", Some("Choice Scarf"),
                        vec!["Thunderbolt", "Flash Cannon", "Hidden Power Fire", "Volt Switch"]
                    )?,
                ])
            }
            "weather_teams" => {
                Ok(vec![
                    self.create_competitive_pokemon(
                        "Pelipper", 50, "Bold", "Drizzle", Some("Damp Rock"),
                        vec!["Hurricane", "Surf", "U-turn", "Roost"]
                    )?,
                    self.create_competitive_pokemon(
                        "Kingdra", 50, "Modest", "Swift Swim", Some("Life Orb"),
                        vec!["Hydro Pump", "Dragon Pulse", "Ice Beam", "Hurricane"]
                    )?,
                    self.create_competitive_pokemon(
                        "Torkoal", 50, "Quiet", "Drought", Some("Heat Rock"),
                        vec!["Eruption", "Solar Beam", "Earth Power", "Stealth Rock"]
                    )?,
                    self.create_competitive_pokemon(
                        "Venusaur", 50, "Modest", "Chlorophyll", Some("Life Orb"),
                        vec!["Solar Beam", "Sludge Bomb", "Hidden Power Fire", "Sleep Powder"]
                    )?,
                    self.create_competitive_pokemon(
                        "Alolan Ninetales", 50, "Timid", "Snow Warning", Some("Icy Rock"),
                        vec!["Blizzard", "Freeze-Dry", "Aurora Veil", "Encore"]
                    )?,
                    self.create_competitive_pokemon(
                        "Arctozolt", 50, "Naive", "Slush Rush", Some("Heavy-Duty Boots"),
                        vec!["Bolt Beak", "Blizzard", "Low Kick", "Substitute"]
                    )?,
                ])
            }
            _ => Err(format!("Unknown preset: {}", preset_name)),
        }
    }

    /// Create a competitive Pokemon with specific builds
    fn create_competitive_pokemon(
        &self,
        species: &str,
        level: u8,
        nature: &str,
        ability: &str,
        item: Option<&str>,
        moves: Vec<&str>,
    ) -> Result<UIPokemon, String> {
        // Get optimal IVs and EVs for competitive play
        let ivs = [31, 31, 31, 31, 31, 31]; // Perfect IVs
        let evs = self.get_optimal_evs_for_pokemon(species, nature);
        
        self.create_fully_custom_pokemon(
            species,
            level,
            &ivs,
            &evs,
            nature,
            Some(ability),
            item,
            Some(&moves.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
            None, // tera_type
        )
    }

    /// Get optimal EV spread for a Pokemon based on its typical competitive role
    fn get_optimal_evs_for_pokemon(&self, species: &str, nature: &str) -> [u8; 6] {
        match species {
            // Special Attackers
            "Flutter Mane" | "Heatran" | "Magnezone" | "Kingdra" | "Torkoal" => [0, 0, 0, 252, 4, 252], // HP/SpA/Spe
            "Toxapex" | "Clefable" | "Rotom-Wash" | "Amoonguss" | "Pelipper" => [252, 0, 252, 4, 0, 0], // HP/Def/SpA
            "Alolan Ninetales" => [0, 0, 4, 252, 0, 252], // SpA/Spe
            
            // Physical Attackers
            "Chien-Pao" | "Urshifu-Rapid-Strike" | "Dragapult" | "Excadrill" | "Hawlucha" => [0, 252, 4, 0, 0, 252], // Atk/Spe
            "Garchomp" | "Dragonite" | "Tyranitar" => [0, 252, 0, 0, 4, 252], // Atk/Spe
            "Rillaboom" | "Azumarill" => [4, 252, 0, 0, 0, 252], // HP/Atk/Spe
            
            // Mixed/Support
            "Incineroar" => [252, 4, 0, 0, 252, 0], // HP/SpD
            "Arctozolt" => [0, 252, 0, 4, 0, 252], // Atk/Spe
            
            // Default spread for unknown Pokemon
            _ => match nature {
                "Timid" | "Jolly" | "Naive" | "Hasty" => [0, 0, 4, 252, 0, 252], // Speed-boosting natures
                "Bold" | "Impish" | "Calm" | "Careful" => [252, 0, 252, 4, 0, 0], // Defensive natures
                "Modest" | "Quiet" => [0, 0, 0, 252, 4, 252], // Special Attack natures
                "Adamant" | "Brave" => [0, 252, 4, 0, 0, 252], // Attack natures
                _ => [85, 85, 85, 85, 85, 85], // Balanced spread
            }
        }
    }
}

impl Default for PokemonBuilder {
    fn default() -> Self {
        Self::new()
    }
}