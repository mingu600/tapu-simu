#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read the moves.ts file
const movesPath = path.join(__dirname, '../../pokemon-showdown/data/moves.ts');
const movesContent = fs.readFileSync(movesPath, 'utf8');

console.log('Parsing Pokemon Showdown moves.ts...');

// Create a more sophisticated parser that handles TypeScript syntax
function parseMovesData(content) {
    // Remove imports and type annotations more carefully
    let processed = content
        // Remove import statements
        .replace(/import\s+.*?from\s+['"][^'"]*['"];\s*/g, '')
        // Remove export and type annotation
        .replace(/export\s+const\s+Moves:\s*import\([^)]+\)\.[^=]+=\s*/, 'const Moves = ')
        // Handle object properties that might have complex syntax
        .replace(/(\w+):\s*{\s*([^}]*)\s*}/g, (match, key, content) => {
            // Clean up property syntax within each move object
            const cleanContent = content
                .replace(/(\w+):\s*([^,\n}]+)/g, (propMatch, propKey, propValue) => {
                    // Handle different value types
                    if (propValue.trim() === 'true' || propValue.trim() === 'false') {
                        return `"${propKey}": ${propValue.trim()}`;
                    }
                    if (propValue.trim().match(/^\d+$/)) {
                        return `"${propKey}": ${propValue.trim()}`;
                    }
                    if (propValue.trim().match(/^\[.*\]$/)) {
                        return `"${propKey}": ${propValue.trim()}`;
                    }
                    if (propValue.trim().match(/^{.*}$/)) {
                        return `"${propKey}": ${propValue.trim()}`;
                    }
                    // String values
                    const cleanValue = propValue.trim().replace(/^["']|["']$/g, '');
                    return `"${propKey}": "${cleanValue}"`;
                });
            return `"${key}": { ${cleanContent} }`;
        });

    // Try to create valid JSON structure
    try {
        // Find the main object boundaries
        const startBrace = processed.indexOf('{');
        const lastBrace = processed.lastIndexOf('}');
        
        if (startBrace === -1 || lastBrace === -1) {
            throw new Error('Could not find object boundaries');
        }

        let objectContent = processed.substring(startBrace + 1, lastBrace).trim();
        
        // Remove trailing commas and clean up
        objectContent = objectContent.replace(/,\s*}/g, '}').replace(/,\s*$/, '');
        
        // Wrap in proper JSON object
        const jsonString = `{${objectContent}}`;
        
        // Parse as JSON
        return JSON.parse(jsonString);
    } catch (error) {
        console.error('JSON parsing failed, trying eval approach...');
        
        // Fallback: Use more aggressive preprocessing for eval
        let evalContent = content
            .replace(/import\s+.*?from\s+['"][^'"]*['"];\s*/g, '')
            .replace(/export\s+const\s+Moves:\s*[^=]+=\s*/, '')
            .replace(/\/\/.*$/gm, '') // Remove single-line comments
            .replace(/\/\*[\s\S]*?\*\//g, '') // Remove multi-line comments
            .trim();
        
        // Remove any trailing semicolon
        if (evalContent.endsWith(';')) {
            evalContent = evalContent.slice(0, -1);
        }
        
        // Use eval as last resort
        return eval('(' + evalContent + ')');
    }
}

try {
    const movesData = parseMovesData(movesContent);
    
    // Validate the data
    if (typeof movesData !== 'object' || movesData === null) {
        throw new Error('Parsed data is not an object');
    }
    
    const moveCount = Object.keys(movesData).length;
    if (moveCount === 0) {
        throw new Error('No moves found in parsed data');
    }
    
    // Create output directory if it doesn't exist
    const outputDir = path.join(__dirname, '../data/ps-extracted');
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // Write to output file
    const outputPath = path.join(outputDir, 'moves.json');
    fs.writeFileSync(outputPath, JSON.stringify(movesData, null, 2));
    
    console.log(`✅ Successfully extracted ${moveCount} moves to ${outputPath}`);
    
    // Show a sample of the data for verification
    const sampleMoves = Object.keys(movesData).slice(0, 5);
    console.log('\nSample moves extracted:');
    sampleMoves.forEach(moveId => {
        const move = movesData[moveId];
        console.log(`  - ${moveId}: ${move.name || 'No name'} (${move.type || 'No type'})`);
    });
    
} catch (error) {
    console.error('❌ Error extracting moves data:', error.message);
    console.error('Full error:', error);
    process.exit(1);
}