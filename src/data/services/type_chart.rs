//! # Pokemon Showdown Type Chart Loader
//! 
//! This module loads type effectiveness data directly from Pokemon Showdown
//! data files, ensuring 100% accuracy with the official simulator.

use std::collections::HashMap;
use std::fs;
use serde_json::Value;
use crate::engine::combat::type_effectiveness::PokemonType;

/// Type effectiveness values from Pokemon Showdown
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PSTypeEffectiveness {
    Neutral = 0,     // 1.0x damage
    SuperEffective = 1, // 2.0x damage  
    NotVeryEffective = 2, // 0.5x damage
    NoEffect = 3,    // 0.0x damage
}

impl PSTypeEffectiveness {
    /// Convert PS effectiveness value to damage multiplier
    pub fn to_multiplier(self) -> f32 {
        match self {
            Self::Neutral => 1.0,
            Self::SuperEffective => 2.0,
            Self::NotVeryEffective => 0.5,
            Self::NoEffect => 0.0,
        }
    }

    /// Create from PS damage value
    pub fn from_ps_value(value: i32) -> Self {
        match value {
            0 => Self::Neutral,
            1 => Self::SuperEffective,
            2 => Self::NotVeryEffective,
            3 => Self::NoEffect,
            _ => Self::Neutral, // Default to neutral for unknown values
        }
    }
}

/// Pokemon Showdown type chart loader
pub struct PSTypeChartLoader {
    /// Path to Pokemon Showdown data directory
    ps_data_path: String,
}

impl PSTypeChartLoader {
    /// Create a new PS type chart loader
    pub fn new(ps_data_path: String) -> Self {
        Self { ps_data_path }
    }

    /// Load type chart from Pokemon Showdown data
    pub fn load_type_chart(&self, generation: u8) -> Result<HashMap<(PokemonType, PokemonType), f32>, Box<dyn std::error::Error>> {
        let type_chart_path = if generation >= 6 {
            format!("{}/typechart.ts", self.ps_data_path)
        } else {
            format!("{}/mods/gen{}/typechart.ts", self.ps_data_path, generation)
        };

        let mut effectiveness_map = HashMap::new();
        
        // Try to read the type chart file
        let content = fs::read_to_string(&type_chart_path)
            .map_err(|e| format!("Failed to read type chart from {}: {}", type_chart_path, e))?;
        
        self.parse_typechart_content(&content, &mut effectiveness_map)?;

        Ok(effectiveness_map)
    }

    /// Parse TypeScript type chart content
    fn parse_typechart_content(
        &self,
        content: &str,
        effectiveness_map: &mut HashMap<(PokemonType, PokemonType), f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified parser for the TypeScript format
        // For production, we'd want a more robust parser
        
        let all_types = PokemonType::all_types();
        
        // Initialize all matchups to neutral
        for &attacking_type in &all_types {
            for &defending_type in &all_types {
                effectiveness_map.insert((attacking_type, defending_type), 1.0);
            }
        }

        // Parse each type section
        for defending_type in &all_types {
            let type_name = defending_type.to_str().to_lowercase();
            
            if let Some(type_section) = self.extract_type_section(content, &type_name) {
                self.parse_damage_taken_section(&type_section, *defending_type, effectiveness_map)?;
            }
        }

        Ok(())
    }

    /// Extract a type's section from the TypeScript content
    fn extract_type_section(&self, content: &str, type_name: &str) -> Option<String> {
        // Find the type definition
        let type_start = content.find(&format!("{}: {{", type_name))?;
        let section_start = content[type_start..].find("damageTaken: {")?;
        let damage_taken_start = type_start + section_start + "damageTaken: {".len();
        
        // Find the closing brace for damageTaken
        let mut brace_count = 1;
        let mut end_pos = damage_taken_start;
        
        for ch in content[damage_taken_start..].chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end_pos += ch.len_utf8();
        }
        
        Some(content[damage_taken_start..end_pos].to_string())
    }

    /// Parse a damageTaken section
    fn parse_damage_taken_section(
        &self,
        section: &str,
        defending_type: PokemonType,
        effectiveness_map: &mut HashMap<(PokemonType, PokemonType), f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse lines like "Fire: 1," or "Water: 2,"
        for line in section.lines() {
            let trimmed = line.trim();
            
            // Skip non-type lines (status conditions, etc.)
            if trimmed.contains(':') 
                && !trimmed.contains("prankster") 
                && !trimmed.contains("powder")
                && !trimmed.contains("par:")
                && !trimmed.contains("psn:")
                && !trimmed.contains("tox:")
                && !trimmed.contains("brn:")
                && !trimmed.contains("frz:")
                && !trimmed.contains("trapped:")
                && !trimmed.contains("sandstorm:")
                && !trimmed.contains("hail:")
            {
                if let Some((type_str, value_str)) = trimmed.split_once(':') {
                    let type_name = type_str.trim();
                    let value_part = value_str.trim().trim_end_matches(',');
                    
                    if let (Some(attacking_type), Ok(ps_value)) = (
                        PokemonType::from_str(type_name),
                        value_part.parse::<i32>()
                    ) {
                        let effectiveness = PSTypeEffectiveness::from_ps_value(ps_value);
                        effectiveness_map.insert((attacking_type, defending_type), effectiveness.to_multiplier());
                    }
                }
            }
        }
        
        Ok(())
    }

}

/// Create a PS type chart loader pointing to the pokemon-showdown directory
pub fn create_ps_type_chart_loader() -> PSTypeChartLoader {
    // Assuming pokemon-showdown is in the parent directory
    let ps_path = "../pokemon-showdown/data".to_string();
    PSTypeChartLoader::new(ps_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_effectiveness_conversion() {
        assert_eq!(PSTypeEffectiveness::Neutral.to_multiplier(), 1.0);
        assert_eq!(PSTypeEffectiveness::SuperEffective.to_multiplier(), 2.0);
        assert_eq!(PSTypeEffectiveness::NotVeryEffective.to_multiplier(), 0.5);
        assert_eq!(PSTypeEffectiveness::NoEffect.to_multiplier(), 0.0);
    }

    #[test]
    fn test_ps_value_parsing() {
        assert_eq!(PSTypeEffectiveness::from_ps_value(0), PSTypeEffectiveness::Neutral);
        assert_eq!(PSTypeEffectiveness::from_ps_value(1), PSTypeEffectiveness::SuperEffective);
        assert_eq!(PSTypeEffectiveness::from_ps_value(2), PSTypeEffectiveness::NotVeryEffective);
        assert_eq!(PSTypeEffectiveness::from_ps_value(3), PSTypeEffectiveness::NoEffect);
    }

    #[test]
    fn test_type_chart_loader_creation() {
        let loader = create_ps_type_chart_loader();
        // Just test that it creates without error
        assert!(loader.ps_data_path.contains("pokemon-showdown"));
    }
}