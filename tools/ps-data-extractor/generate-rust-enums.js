#!/usr/bin/env node

/**
 * Generate Rust enum implementations with complete trait support
 * 
 * This script generates fully-featured Rust enums with:
 * - FromNormalizedString trait implementations
 * - Display trait implementations  
 * - as_str() methods
 * - Serde support
 * - Complete match arms for all Pokemon Showdown data
 */

import { Dex } from '@pkmn/sim';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Normalize names to match the normalize_name function in Rust
function normalizeName(name) {
    return name.toLowerCase()
        .replace(/[^a-z0-9]/g, '')
        .replace(/\s+/g, '');
}

// Convert normalized name to Rust enum variant (SCREAMING_SNAKE_CASE)
function toEnumVariant(name) {
    if (!name || name === '') return 'NONE';
    
    // Handle special cases
    const specialCases = {
        '10000000voltthunderbolt': 'ZMOVE10000000VOLTTHUNDERBOLT',
        '1000000voltthunderbolt': 'ZMOVE1000000VOLTTHUNDERBOLT', 
        'gmaxbefuddle': 'GMAXBEFUDDLE',
        'gmaxblast': 'GMAXBLAST',
        // Add more as needed
    };
    
    if (specialCases[name]) {
        return specialCases[name];
    }
    
    // Convert to SCREAMING_SNAKE_CASE
    return name.toUpperCase()
        .replace(/([0-9]+)/g, '_$1')  // Add underscore before numbers
        .replace(/^_/, '');           // Remove leading underscore
}

// Generate the Rust enum with all variants
function generateEnum(dataMap, enumName, moduleName) {
    const variants = new Set(['NONE']); // Always include NONE
    const normalizedToVariant = new Map();
    const variantToNormalized = new Map();
    
    // Collect all data entries
    for (const [id, entry] of Object.entries(dataMap)) {
        const normalized = normalizeName(id);
        const variant = toEnumVariant(normalized);
        variants.add(variant);
        normalizedToVariant.set(normalized, variant);
        variantToNormalized.set(variant, normalized);
    }
    
    const sortedVariants = Array.from(variants).sort();
    
    // Generate enum definition
    let enumDef = `use serde::{Deserialize, Serialize};
use crate::types::from_string::FromNormalizedString;
use std::fmt;

/// ${enumName} enum with complete Pokemon Showdown data coverage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ${enumName} {
`;
    
    for (const variant of sortedVariants) {
        enumDef += `    ${variant},\n`;
    }
    
    enumDef += `}

impl ${enumName} {
    /// Get the normalized string representation of this ${enumName.toLowerCase()}
    pub fn as_str(&self) -> &'static str {
        match self {
`;
    
    // Add as_str() method
    enumDef += `            ${enumName}::NONE => "",\n`;
    for (const variant of sortedVariants) {
        if (variant !== 'NONE') {
            const normalized = variantToNormalized.get(variant) || '';
            enumDef += `            ${enumName}::${variant} => "${normalized}",\n`;
        }
    }
    
    enumDef += `        }
    }
    
    /// Get the display name of this ${enumName.toLowerCase()}
    pub fn display_name(&self) -> &'static str {
        match self {
`;
    
    // Add display_name() method
    enumDef += `            ${enumName}::NONE => "None",\n`;
    for (const [id, entry] of Object.entries(dataMap)) {
        const normalized = normalizeName(id);
        const variant = normalizedToVariant.get(normalized);
        if (variant) {
            const displayName = entry.name || id;
            enumDef += `            ${enumName}::${variant} => "${displayName}",\n`;
        }
    }
    
    enumDef += `        }
    }
}

impl FromNormalizedString for ${enumName} {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s {
            "" => Some(${enumName}::NONE),
`;
    
    // Add FromNormalizedString implementation
    for (const [normalized, variant] of normalizedToVariant.entries()) {
        if (variant !== 'NONE') {
            enumDef += `            "${normalized}" => Some(${enumName}::${variant}),\n`;
        }
    }
    
    enumDef += `            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec![
`;
    
    for (const normalized of normalizedToVariant.keys()) {
        enumDef += `            "${normalized}",\n`;
    }
    
    enumDef += `        ]
    }
}

impl fmt::Display for ${enumName} {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for ${enumName} {
    fn default() -> Self {
        ${enumName}::NONE
    }
}
`;
    
    return enumDef;
}

// Generate all enum files
function generateAllEnums() {
    console.log('Generating Rust enums with complete trait implementations...');
    
    const outputDir = path.join(__dirname, '../../src/types/generated');
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // Generate Moves enum
    console.log('Generating Moves enum...');
    const movesEnum = generateEnum(Dex.data.Moves, 'Moves', 'moves');
    fs.writeFileSync(path.join(outputDir, 'moves.rs'), movesEnum);
    
    // Generate Items enum  
    console.log('Generating Items enum...');
    const itemsEnum = generateEnum(Dex.data.Items, 'Items', 'items');
    fs.writeFileSync(path.join(outputDir, 'items.rs'), itemsEnum);
    
    // Generate Abilities enum
    console.log('Generating Abilities enum...');
    const abilitiesEnum = generateEnum(Dex.data.Abilities, 'Abilities', 'abilities');
    fs.writeFileSync(path.join(outputDir, 'abilities.rs'), abilitiesEnum);
    
    // Generate PokemonName enum
    console.log('Generating PokemonName enum...');
    const pokemonEnum = generateEnum(Dex.data.Species, 'PokemonName', 'pokemon');
    fs.writeFileSync(path.join(outputDir, 'pokemon.rs'), pokemonEnum);
    
    console.log('Generated complete Rust enums in:', outputDir);
    console.log('');
    console.log('Next steps:');
    console.log('1. Replace the existing enum files with the generated ones');
    console.log('2. Update mod.rs to use the new generated enums');
    console.log('3. Test compilation and functionality');
}

generateAllEnums();