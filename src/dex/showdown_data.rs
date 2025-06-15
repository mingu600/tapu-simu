//! Pokemon Showdown data integration
//! 
//! This module handles loading and managing Pokemon Showdown data
//! by parsing TypeScript data files and converting them to Rust structures.

use crate::pokemon::{SpeciesData, MoveData, AbilityData, ItemData, MoveFlags, SecondaryEffect, SecondaryEffectType, MultihitData, NaturalGiftData};
use crate::types::*;
use crate::errors::{BattleError, BattleResult};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::{Value, Map};
use regex::Regex;

/// Parses Pokemon Showdown TypeScript data files
pub struct ShowdownDataParser {
    pub moves: HashMap<String, MoveData>,
    pub species: HashMap<String, SpeciesData>,
    pub abilities: HashMap<String, AbilityData>,
    pub items: HashMap<String, ItemData>,
    pub type_chart: HashMap<Type, HashMap<Type, f32>>,
}

impl ShowdownDataParser {
    /// Create a new parser and load all data
    pub fn new(data_dir: &Path) -> BattleResult<Self> {
        let mut parser = Self {
            moves: HashMap::new(),
            species: HashMap::new(),
            abilities: HashMap::new(),
            items: HashMap::new(),
            type_chart: HashMap::new(),
        };
        
        // Try to load from JSON files first, then fallback to TypeScript
        parser.load_from_json(data_dir).unwrap_or_else(|_| {
            parser.load_from_typescript(data_dir).unwrap_or_else(|e| {
                eprintln!("Warning: Could not load Pokemon Showdown data: {}", e);
            })
        });
        
        Ok(parser)
    }
    
    /// Load data from extracted JSON files
    pub fn load_from_json(&mut self, base_dir: &Path) -> BattleResult<()> {
        let json_dir = base_dir.join("data/ps-extracted");
        
        // Load all data from JSON files
        if let Err(e) = self.load_species_json(&json_dir) {
            eprintln!("Could not load species JSON: {}", e);
        }
        
        if let Err(e) = self.load_moves_json(&json_dir) {
            eprintln!("Could not load moves JSON: {}", e);
            // Fallback to minimal moves if JSON loading fails
            self.create_minimal_moves();
        }
        
        if let Err(e) = self.load_abilities_json(&json_dir) {
            eprintln!("Could not load abilities JSON: {}", e);
        }
        
        if let Err(e) = self.load_items_json(&json_dir) {
            eprintln!("Could not load items JSON: {}", e);
        }
        
        if let Err(e) = self.load_type_chart_json(&json_dir) {
            eprintln!("Could not load type chart JSON: {}", e);
        }
        
        Ok(())
    }
    
    /// Load data from TypeScript files (fallback)
    pub fn load_from_typescript(&mut self, data_dir: &Path) -> BattleResult<()> {
        self.load_moves(data_dir)?;
        self.load_species(data_dir)?;
        self.load_abilities(data_dir)?;
        self.load_items(data_dir)?;
        self.load_type_chart(data_dir)?;
        Ok(())
    }
    
    /// Load species from JSON file
    fn load_species_json(&mut self, json_dir: &Path) -> BattleResult<()> {
        let pokedex_path = json_dir.join("pokedex.json");
        if !pokedex_path.exists() {
            return Err(BattleError::DataError("pokedex.json not found".to_string()));
        }
        
        let content = fs::read_to_string(&pokedex_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read pokedex.json: {}", e)))?;
        
        let pokedex_data: Value = serde_json::from_str(&content)
            .map_err(|e| BattleError::DataError(format!("Failed to parse pokedex.json: {}", e)))?;
        
        if let Value::Object(species_obj) = pokedex_data {
            for (species_id, species_data) in species_obj {
                if let Ok(species) = self.parse_species_data(&species_id, &species_data) {
                    self.species.insert(species_id, species);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load type chart from JSON file
    fn load_type_chart_json(&mut self, json_dir: &Path) -> BattleResult<()> {
        let typechart_path = json_dir.join("typechart.json");
        if !typechart_path.exists() {
            return Err(BattleError::DataError("typechart.json not found".to_string()));
        }
        
        let content = fs::read_to_string(&typechart_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read typechart.json: {}", e)))?;
        
        let chart_data: Value = serde_json::from_str(&content)
            .map_err(|e| BattleError::DataError(format!("Failed to parse typechart.json: {}", e)))?;
        
        if let Value::Object(types_obj) = chart_data {
            for (type_name, type_data) in types_obj {
                if let Ok(defending_type) = self.parse_type(&type_name) {
                    if let Some(damage_taken_obj) = type_data.get("damageTaken").and_then(|v| v.as_object()) {
                        let mut effectiveness_map = HashMap::new();
                        
                        for (attacking_type_str, effectiveness_val) in damage_taken_obj {
                            if let Ok(attacking_type) = self.parse_type(attacking_type_str) {
                                let effectiveness = match effectiveness_val.as_u64().unwrap_or(0) {
                                    0 => 1.0,   // Normal effectiveness
                                    1 => 0.5,   // Not very effective
                                    2 => 2.0,   // Super effective
                                    3 => 0.0,   // No effect
                                    _ => 1.0,
                                };
                                effectiveness_map.insert(attacking_type, effectiveness);
                            }
                        }
                        
                        self.type_chart.insert(defending_type, effectiveness_map);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Load moves from JSON file
    fn load_moves_json(&mut self, json_dir: &Path) -> BattleResult<()> {
        let moves_path = json_dir.join("moves.json");
        if !moves_path.exists() {
            return Err(BattleError::DataError("moves.json not found".to_string()));
        }
        
        let content = fs::read_to_string(&moves_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read moves.json: {}", e)))?;
        
        let moves_data: Value = serde_json::from_str(&content)
            .map_err(|e| BattleError::DataError(format!("Failed to parse moves.json: {}", e)))?;
        
        if let Value::Object(moves_obj) = moves_data {
            for (move_id, move_data) in moves_obj {
                if let Ok(move_info) = self.parse_move_data(&move_id, &move_data) {
                    self.moves.insert(move_id, move_info);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load abilities from JSON file
    fn load_abilities_json(&mut self, json_dir: &Path) -> BattleResult<()> {
        let abilities_path = json_dir.join("abilities.json");
        if !abilities_path.exists() {
            return Err(BattleError::DataError("abilities.json not found".to_string()));
        }
        
        let content = fs::read_to_string(&abilities_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read abilities.json: {}", e)))?;
        
        let abilities_data: Value = serde_json::from_str(&content)
            .map_err(|e| BattleError::DataError(format!("Failed to parse abilities.json: {}", e)))?;
        
        if let Value::Object(abilities_obj) = abilities_data {
            for (ability_id, ability_data) in abilities_obj {
                if let Ok(ability_info) = self.parse_ability_data(&ability_id, &ability_data) {
                    self.abilities.insert(ability_id, ability_info);
                }
            }
        }
        
        Ok(())
    }
    
    /// Load items from JSON file
    fn load_items_json(&mut self, json_dir: &Path) -> BattleResult<()> {
        let items_path = json_dir.join("items.json");
        if !items_path.exists() {
            return Err(BattleError::DataError("items.json not found".to_string()));
        }
        
        let content = fs::read_to_string(&items_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read items.json: {}", e)))?;
        
        let items_data: Value = serde_json::from_str(&content)
            .map_err(|e| BattleError::DataError(format!("Failed to parse items.json: {}", e)))?;
        
        if let Value::Object(items_obj) = items_data {
            for (item_id, item_data) in items_obj {
                if let Ok(item_info) = self.parse_item_data(&item_id, &item_data) {
                    self.items.insert(item_id, item_info);
                }
            }
        }
        
        Ok(())
    }
    
    /// Create minimal move data for testing
    fn create_minimal_moves(&mut self) {
        // Add a few basic moves for testing
        let tackle = MoveData {
            id: "tackle".to_string(),
            name: "Tackle".to_string(),
            type_: Type::Normal,
            category: MoveCategory::Physical,
            base_power: 40,
            accuracy: Some(100),
            pp: 35,
            target: MoveTarget::Normal,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        };
        
        let thunderbolt = MoveData {
            id: "thunderbolt".to_string(),
            name: "Thunderbolt".to_string(),
            type_: Type::Electric,
            category: MoveCategory::Special,
            base_power: 90,
            accuracy: Some(100),
            pp: 15,
            target: MoveTarget::Normal,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        };
        
        self.moves.insert("tackle".to_string(), tackle);
        self.moves.insert("thunderbolt".to_string(), thunderbolt);
    }
    
    /// Parse species data from JSON
    fn parse_species_data(&self, id: &str, data: &Value) -> BattleResult<SpeciesData> {
        let obj = data.as_object()
            .ok_or_else(|| BattleError::DataError("Species data is not an object".to_string()))?;
        
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        
        // Parse types
        let types_array = obj.get("types")
            .and_then(|v| v.as_array())
            .ok_or_else(|| BattleError::DataError("Species missing types".to_string()))?;
        
        let primary_type = types_array.get(0)
            .and_then(|v| v.as_str())
            .map(|s| self.parse_type(s))
            .unwrap_or(Ok(Type::Normal))?;
        
        let secondary_type = types_array.get(1)
            .and_then(|v| v.as_str())
            .map(|s| self.parse_type(s))
            .unwrap_or(Ok(primary_type))?;
        
        // Parse base stats
        let base_stats_obj = obj.get("baseStats")
            .and_then(|v| v.as_object())
            .ok_or_else(|| BattleError::DataError("Species missing baseStats".to_string()))?;
        
        let base_stats = StatsTable {
            hp: base_stats_obj.get("hp").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
            attack: base_stats_obj.get("atk").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
            defense: base_stats_obj.get("def").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
            special_attack: base_stats_obj.get("spa").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
            special_defense: base_stats_obj.get("spd").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
            speed: base_stats_obj.get("spe").and_then(|v| v.as_u64()).unwrap_or(50) as u16,
        };
        
        // Parse abilities (simplified)
        let abilities = if let Some(abilities_obj) = obj.get("abilities").and_then(|v| v.as_object()) {
            let mut ability_list = Vec::new();
            if let Some(ability0) = abilities_obj.get("0").and_then(|v| v.as_str()) {
                ability_list.push(ability0.to_string());
            }
            if let Some(ability1) = abilities_obj.get("1").and_then(|v| v.as_str()) {
                ability_list.push(ability1.to_string());
            }
            if let Some(hidden) = abilities_obj.get("H").and_then(|v| v.as_str()) {
                ability_list.push(hidden.to_string());
            }
            ability_list
        } else {
            vec!["noability".to_string()]
        };
        
        // Parse other fields
        let height = obj.get("heightm").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let weight = obj.get("weightkg").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        
        // Gender ratio (simplified)
        let gender_ratio = if let Some(gender_obj) = obj.get("genderRatio").and_then(|v| v.as_object()) {
            let male = gender_obj.get("M").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
            let female = gender_obj.get("F").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
            crate::pokemon::GenderRatio::Ratio { male, female }
        } else {
            crate::pokemon::GenderRatio::Genderless
        };
        
        Ok(SpeciesData {
            id: id.to_string(),
            name,
            types: [primary_type, secondary_type],
            base_stats,
            abilities,
            height,
            weight,
            gender_ratio,
        })
    }
    
    /// Parse TypeScript export data to JSON (simplified version)
    fn parse_ts_export(content: &str, export_name: &str) -> BattleResult<Value> {
        // This is a simplified implementation - for complex files we rely on JSON extraction
        Err(BattleError::DataError(format!("TypeScript parsing not fully implemented for {}", export_name)))
    }
    
    /// Convert JavaScript object notation to valid JSON
    fn js_object_to_json(js_obj: &str) -> BattleResult<String> {
        let mut json = js_obj.to_string();
        
        // Replace unquoted object keys
        let key_re = Regex::new(r"(\s*)([a-zA-Z_][a-zA-Z0-9_]*)(\s*):(\s*)").unwrap();
        json = key_re.replace_all(&json, r#"$1"$2"$3:$4"#).to_string();
        
        // Replace single quotes with double quotes
        json = json.replace("'", "\"");
        
        // Handle boolean true/false
        let bool_re = Regex::new(r"\btrue\b").unwrap();
        json = bool_re.replace_all(&json, "true").to_string();
        let bool_re = Regex::new(r"\bfalse\b").unwrap();
        json = bool_re.replace_all(&json, "false").to_string();
        
        // Handle null
        let null_re = Regex::new(r"\bnull\b").unwrap();
        json = null_re.replace_all(&json, "null").to_string();
        
        // Handle trailing commas
        let comma_re = Regex::new(r",\s*([}\]])").unwrap();
        json = comma_re.replace_all(&json, "$1").to_string();
        
        Ok(json)
    }
    
    /// Load moves data
    fn load_moves(&mut self, data_dir: &Path) -> BattleResult<()> {
        let moves_path = data_dir.join("moves.ts");
        let content = fs::read_to_string(&moves_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read moves.ts: {}", e)))?;
            
        let moves_json = Self::parse_ts_export(&content, "Moves")?;
        
        if let Value::Object(moves_obj) = moves_json {
            for (move_id, move_data) in moves_obj {
                if let Ok(move_data) = self.parse_move_data(&move_id, &move_data) {
                    self.moves.insert(move_id, move_data);
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse individual move data
    fn parse_move_data(&self, id: &str, data: &Value) -> BattleResult<MoveData> {
        let obj = data.as_object()
            .ok_or_else(|| BattleError::DataError("Move data is not an object".to_string()))?;
            
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
            
        let type_str = obj.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BattleError::DataError("Move missing type".to_string()))?;
        let type_ = self.parse_type(type_str)?;
        
        let category_str = obj.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("Status");
        let category = match category_str {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            _ => MoveCategory::Status,
        };
        
        let base_power = obj.get("basePower")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
            
        let accuracy = obj.get("accuracy")
            .and_then(|v| {
                if v.as_bool() == Some(true) {
                    None // true means always hits
                } else {
                    v.as_u64().map(|a| a as u8)
                }
            });
            
        let pp = obj.get("pp")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as u8;
            
        let priority = obj.get("priority")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i8;
            
        let target_str = obj.get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        let target = self.parse_move_target(target_str);
        
        let flags = obj.get("flags")
            .and_then(|v| v.as_object())
            .map(|f| self.parse_move_flags(f))
            .unwrap_or_default();
            
        let secondary_effect = obj.get("secondary")
            .and_then(|v| self.parse_secondary_effect(v).ok());
            
        let crit_ratio = obj.get("critRatio")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u8;
            
        let multihit = obj.get("multihit")
            .and_then(|v| self.parse_multihit(v).ok());
            
        let drain = obj.get("drain")
            .and_then(|v| self.parse_fraction_array(v));
            
        let recoil = obj.get("recoil")
            .and_then(|v| self.parse_fraction_array(v));
        
        Ok(MoveData {
            id: id.to_string(),
            name,
            type_,
            category,
            base_power,
            accuracy,
            pp,
            target,
            priority,
            flags,
            secondary_effect,
            crit_ratio,
            multihit,
            drain,
            recoil,
        })
    }
    
    /// Parse ability data from JSON
    fn parse_ability_data(&self, id: &str, data: &Value) -> BattleResult<AbilityData> {
        let obj = data.as_object()
            .ok_or_else(|| BattleError::DataError("Ability data is not an object".to_string()))?;
            
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
            
        let description = obj.get("shortDesc")
            .or_else(|| obj.get("desc"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(AbilityData {
            id: id.to_string(),
            name,
            description,
            event_handlers: crate::events::EventHandlerRegistry::new(),
        })
    }
    
    /// Parse item data from JSON
    fn parse_item_data(&self, id: &str, data: &Value) -> BattleResult<ItemData> {
        let obj = data.as_object()
            .ok_or_else(|| BattleError::DataError("Item data is not an object".to_string()))?;
            
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
            
        let description = obj.get("shortDesc")
            .or_else(|| obj.get("desc"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        // Parse Natural Gift data if present
        let natural_gift = obj.get("naturalGift")
            .and_then(|v| v.as_object())
            .and_then(|ng_obj| {
                let type_str = ng_obj.get("type")?.as_str()?;
                let base_power = ng_obj.get("basePower")?.as_u64()? as u16;
                let type_ = self.parse_type(type_str).ok()?;
                
                Some(NaturalGiftData {
                    type_,
                    base_power,
                })
            });
        
        Ok(ItemData {
            id: id.to_string(),
            name,
            description,
            natural_gift,
            event_handlers: crate::events::EventHandlerRegistry::new(),
        })
    }
    
    /// Parse Pokemon type from string (case-insensitive)
    fn parse_type(&self, type_str: &str) -> BattleResult<Type> {
        match type_str.to_lowercase().as_str() {
            "normal" => Ok(Type::Normal),
            "fire" => Ok(Type::Fire),
            "water" => Ok(Type::Water),
            "electric" => Ok(Type::Electric),
            "grass" => Ok(Type::Grass),
            "ice" => Ok(Type::Ice),
            "fighting" => Ok(Type::Fighting),
            "poison" => Ok(Type::Poison),
            "ground" => Ok(Type::Ground),
            "flying" => Ok(Type::Flying),
            "psychic" => Ok(Type::Psychic),
            "bug" => Ok(Type::Bug),
            "rock" => Ok(Type::Rock),
            "ghost" => Ok(Type::Ghost),
            "dragon" => Ok(Type::Dragon),
            "dark" => Ok(Type::Dark),
            "steel" => Ok(Type::Steel),
            "fairy" => Ok(Type::Fairy),
            _ => Err(BattleError::DataError(format!("Unknown type: {}", type_str))),
        }
    }
    
    /// Parse move target from string
    fn parse_move_target(&self, target_str: &str) -> MoveTarget {
        match target_str {
            "normal" => MoveTarget::Normal,
            "any" => MoveTarget::Any,
            "adjacentAlly" => MoveTarget::AdjacantAlly,
            "adjacentFoe" => MoveTarget::AdjacantFoe,
            "adjacentAllyOrSelf" => MoveTarget::AdjacantAllyOrSelf,
            "self" => MoveTarget::Self_,
            "allAdjacent" => MoveTarget::AllAdjacent,
            "allAdjacentFoes" => MoveTarget::AllAdjacentFoes,
            "allAllies" => MoveTarget::AllAllies,
            "allEnemies" => MoveTarget::AllFoes,
            "all" => MoveTarget::All,
            "foeSide" => MoveTarget::FoeSide,
            "allySide" => MoveTarget::AllySide,
            "randomNormal" => MoveTarget::RandomNormal,
            _ => MoveTarget::Normal,
        }
    }
    
    /// Parse move flags
    fn parse_move_flags(&self, flags_obj: &Map<String, Value>) -> MoveFlags {
        MoveFlags {
            contact: flags_obj.contains_key("contact"),
            sound: flags_obj.contains_key("sound"),
            bullet: flags_obj.contains_key("bullet"),
            pulse: flags_obj.contains_key("pulse"),
            bite: flags_obj.contains_key("bite"),
            punch: flags_obj.contains_key("punch"),
            powder: flags_obj.contains_key("powder"),
            reflectable: flags_obj.contains_key("reflectable"),
            charge: flags_obj.contains_key("charge"),
            recharge: flags_obj.contains_key("recharge"),
            gravity: flags_obj.contains_key("gravity"),
            defrost: flags_obj.contains_key("defrost"),
            distance: flags_obj.contains_key("distance"),
            heal: flags_obj.contains_key("heal"),
            authentic: flags_obj.contains_key("authentic"),
        }
    }
    
    /// Parse secondary effect
    fn parse_secondary_effect(&self, data: &Value) -> BattleResult<SecondaryEffect> {
        if data.is_null() {
            return Err(BattleError::DataError("No secondary effect".to_string()));
        }
        
        let obj = data.as_object()
            .ok_or_else(|| BattleError::DataError("Secondary effect not an object".to_string()))?;
            
        let chance = obj.get("chance")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as u8;
            
        // Parse the effect type - this is simplified, PS has more complex effects
        let effect = if obj.contains_key("status") {
            let status_str = obj.get("status").unwrap().as_str().unwrap_or("");
            match status_str {
                "par" => SecondaryEffectType::Paralyze,
                "brn" => SecondaryEffectType::Burn,
                "frz" => SecondaryEffectType::Freeze,
                "psn" => SecondaryEffectType::Poison,
                "tox" => SecondaryEffectType::BadlyPoison,
                "slp" => SecondaryEffectType::Sleep,
                _ => SecondaryEffectType::Flinch,
            }
        } else {
            SecondaryEffectType::Flinch
        };
        
        Ok(SecondaryEffect { chance, effect })
    }
    
    /// Parse multihit data
    fn parse_multihit(&self, data: &Value) -> BattleResult<MultihitData> {
        if let Some(array) = data.as_array() {
            if array.len() >= 2 {
                let min_hits = array[0].as_u64().unwrap_or(1) as u8;
                let max_hits = array[1].as_u64().unwrap_or(1) as u8;
                return Ok(MultihitData { min_hits, max_hits });
            }
        }
        
        if let Some(num) = data.as_u64() {
            let hits = num as u8;
            return Ok(MultihitData { min_hits: hits, max_hits: hits });
        }
        
        Err(BattleError::DataError("Invalid multihit data".to_string()))
    }
    
    /// Parse fraction array like [1, 2] for drain/recoil
    fn parse_fraction_array(&self, data: &Value) -> Option<[u8; 2]> {
        if let Some(array) = data.as_array() {
            if array.len() >= 2 {
                let num = array[0].as_u64()? as u8;
                let den = array[1].as_u64()? as u8;
                return Some([num, den]);
            }
        }
        None
    }
    
    /// Load species data (simplified implementation)
    fn load_species(&mut self, _data_dir: &Path) -> BattleResult<()> {
        // TODO: Implement species parsing
        Ok(())
    }
    
    /// Load abilities data (simplified implementation)
    fn load_abilities(&mut self, _data_dir: &Path) -> BattleResult<()> {
        // TODO: Implement abilities parsing
        Ok(())
    }
    
    /// Load items data (simplified implementation)
    fn load_items(&mut self, _data_dir: &Path) -> BattleResult<()> {
        // TODO: Implement items parsing
        Ok(())
    }
    
    /// Load type effectiveness chart
    fn load_type_chart(&mut self, data_dir: &Path) -> BattleResult<()> {
        let typechart_path = data_dir.join("typechart.ts");
        let content = fs::read_to_string(&typechart_path)
            .map_err(|e| BattleError::DataError(format!("Failed to read typechart.ts: {}", e)))?;
            
        let chart_json = Self::parse_ts_export(&content, "TypeChart")?;
        
        if let Value::Object(types_obj) = chart_json {
            for (type_name, type_data) in types_obj {
                if let Ok(defending_type) = self.parse_type(&type_name) {
                    if let Some(damage_taken_obj) = type_data.get("damageTaken").and_then(|v| v.as_object()) {
                        let mut effectiveness_map = HashMap::new();
                        
                        for (attacking_type_str, effectiveness_val) in damage_taken_obj {
                            if let Ok(attacking_type) = self.parse_type(attacking_type_str) {
                                let effectiveness = match effectiveness_val.as_u64().unwrap_or(0) {
                                    0 => 1.0,   // Normal effectiveness
                                    1 => 0.5,   // Not very effective
                                    2 => 2.0,   // Super effective
                                    3 => 0.0,   // No effect
                                    _ => 1.0,
                                };
                                effectiveness_map.insert(attacking_type, effectiveness);
                            }
                        }
                        
                        self.type_chart.insert(defending_type, effectiveness_map);
                    }
                }
            }
        }
        
        Ok(())
    }
}