#!/usr/bin/env node

/**
 * Extract Pokemon Showdown data from TypeScript files to JSON
 * This script reads the PS data files and extracts them to JSON format
 * that can be easily consumed by the Rust implementation.
 */

const fs = require('fs');
const path = require('path');

// Pokemon Showdown data directory
const PS_DATA_DIR = path.join(__dirname, '../../pokemon-showdown/data');
const OUTPUT_DIR = path.join(__dirname, '../data/ps-extracted');

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

/**
 * Dynamically import and evaluate a TypeScript module
 */
async function importTSModule(filePath) {
    try {
        const content = fs.readFileSync(filePath, 'utf8');
        
        // Remove import statements and type annotations
        let processedContent = content
            .replace(/import\s+.*?from\s+['"].*?['"];?\s*\n/g, '')
            .replace(/export\s+const\s+(\w+):\s*[^=]+=/g, 'const $1 =')
            .replace(/:\s*import\([^)]+\)\.[^=]+=/g, ' =');
        
        // Create a simple module context
        const moduleContext = {
            exports: {},
            module: { exports: {} }
        };
        
        // Evaluate the processed content
        const func = new Function('exports', 'module', processedContent + '\nreturn ' + getExportName(content) + ';');
        const result = func(moduleContext.exports, moduleContext);
        
        return result;
    } catch (error) {
        console.error(`Error importing ${filePath}:`, error.message);
        return null;
    }
}

/**
 * Extract the export name from TypeScript content
 */
function getExportName(content) {
    const match = content.match(/export\s+const\s+(\w+):/);
    return match ? match[1] : null;
}

/**
 * Extract moves data
 */
async function extractMoves() {
    console.log('Extracting moves data...');
    const movesPath = path.join(PS_DATA_DIR, 'moves.ts');
    
    if (!fs.existsSync(movesPath)) {
        console.error('moves.ts not found');
        return;
    }
    
    const movesData = await importTSModule(movesPath);
    if (movesData) {
        const outputPath = path.join(OUTPUT_DIR, 'moves.json');
        fs.writeFileSync(outputPath, JSON.stringify(movesData, null, 2));
        console.log(`Extracted ${Object.keys(movesData).length} moves to ${outputPath}`);
    }
}

/**
 * Extract Pokedex data
 */
async function extractPokedex() {
    console.log('Extracting Pokedex data...');
    const pokedexPath = path.join(PS_DATA_DIR, 'pokedex.ts');
    
    if (!fs.existsSync(pokedexPath)) {
        console.error('pokedex.ts not found');
        return;
    }
    
    const pokedexData = await importTSModule(pokedexPath);
    if (pokedexData) {
        const outputPath = path.join(OUTPUT_DIR, 'pokedex.json');
        fs.writeFileSync(outputPath, JSON.stringify(pokedexData, null, 2));
        console.log(`Extracted ${Object.keys(pokedexData).length} species to ${outputPath}`);
    }
}

/**
 * Extract abilities data
 */
async function extractAbilities() {
    console.log('Extracting abilities data...');
    const abilitiesPath = path.join(PS_DATA_DIR, 'abilities.ts');
    
    if (!fs.existsSync(abilitiesPath)) {
        console.error('abilities.ts not found');
        return;
    }
    
    const abilitiesData = await importTSModule(abilitiesPath);
    if (abilitiesData) {
        const outputPath = path.join(OUTPUT_DIR, 'abilities.json');
        fs.writeFileSync(outputPath, JSON.stringify(abilitiesData, null, 2));
        console.log(`Extracted ${Object.keys(abilitiesData).length} abilities to ${outputPath}`);
    }
}

/**
 * Extract items data
 */
async function extractItems() {
    console.log('Extracting items data...');
    const itemsPath = path.join(PS_DATA_DIR, 'items.ts');
    
    if (!fs.existsSync(itemsPath)) {
        console.error('items.ts not found');
        return;
    }
    
    const itemsData = await importTSModule(itemsPath);
    if (itemsData) {
        const outputPath = path.join(OUTPUT_DIR, 'items.json');
        fs.writeFileSync(outputPath, JSON.stringify(itemsData, null, 2));
        console.log(`Extracted ${Object.keys(itemsData).length} items to ${outputPath}`);
    }
}

/**
 * Extract type chart data
 */
async function extractTypeChart() {
    console.log('Extracting type chart data...');
    const typechartPath = path.join(PS_DATA_DIR, 'typechart.ts');
    
    if (!fs.existsSync(typechartPath)) {
        console.error('typechart.ts not found');
        return;
    }
    
    const typechartData = await importTSModule(typechartPath);
    if (typechartData) {
        const outputPath = path.join(OUTPUT_DIR, 'typechart.json');
        fs.writeFileSync(outputPath, JSON.stringify(typechartData, null, 2));
        console.log(`Extracted type chart to ${outputPath}`);
    }
}

/**
 * Main extraction function
 */
async function main() {
    console.log('Pokemon Showdown Data Extractor');
    console.log('================================');
    
    if (!fs.existsSync(PS_DATA_DIR)) {
        console.error(`Pokemon Showdown data directory not found: ${PS_DATA_DIR}`);
        console.error('Please ensure Pokemon Showdown is cloned in the expected location.');
        process.exit(1);
    }
    
    try {
        await extractMoves();
        await extractPokedex();
        await extractAbilities();
        await extractItems();
        await extractTypeChart();
        
        console.log('\nData extraction completed successfully!');
    } catch (error) {
        console.error('Error during extraction:', error);
        process.exit(1);
    }
}

// Run the extraction if this script is executed directly
if (require.main === module) {
    main();
}