#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const esbuild = require('esbuild');

async function extractMovesWithEsbuild() {
    console.log('Using esbuild to extract Pokemon Showdown moves data...');
    
    try {
        // Path to Pokemon Showdown data
        const psDataDir = path.join(__dirname, '../../pokemon-showdown/data');
        const movesPath = path.join(psDataDir, 'moves.ts');
        
        if (!fs.existsSync(movesPath)) {
            throw new Error(`Moves file not found: ${movesPath}`);
        }
        
        // Create a temporary directory for building
        const tempDir = path.join(__dirname, '../temp');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }
        
        // Create a wrapper script that will export the moves data
        const wrapperScript = `
import { Moves } from '${movesPath}';

// Export as JSON to stdout
console.log(JSON.stringify(Moves, null, 2));
`;
        
        const wrapperPath = path.join(tempDir, 'extract_wrapper.ts');
        fs.writeFileSync(wrapperPath, wrapperScript);
        
        // Use esbuild to transpile and bundle
        const buildResult = await esbuild.build({
            entryPoints: [wrapperPath],
            bundle: true,
            format: 'cjs',
            platform: 'node',
            outfile: path.join(tempDir, 'extract_moves.js'),
            // Ignore external imports that aren't needed for data extraction
            external: [],
            write: true,
            logLevel: 'warning'
        });
        
        // Run the built script
        const { execSync } = require('child_process');
        const builtScript = path.join(tempDir, 'extract_moves.js');
        
        const result = execSync(`node ${builtScript}`, {
            encoding: 'utf8',
            maxBuffer: 50 * 1024 * 1024, // 50MB buffer
            cwd: tempDir
        });
        
        // Parse the result
        const movesData = JSON.parse(result);
        
        // Clean up temp files
        fs.rmSync(tempDir, { recursive: true, force: true });
        
        return movesData;
        
    } catch (error) {
        console.error('esbuild extraction failed:', error.message);
        
        // Fallback: Try to manually parse the simpler object structure
        console.log('Trying fallback manual parsing...');
        return await extractMovesManual();
    }
}

async function extractMovesManual() {
    const psDataDir = path.join(__dirname, '../../pokemon-showdown/data');
    const movesPath = path.join(psDataDir, 'moves.ts');
    
    const content = fs.readFileSync(movesPath, 'utf8');
    
    // Find the moves object more carefully
    const movesMatch = content.match(/export const Moves[^{]*= {([^}]+{[^}]*}[^}]*)*}/s);
    if (!movesMatch) {
        throw new Error('Could not find Moves export in file');
    }
    
    let movesObjectStr = movesMatch[0];
    
    // Remove the export statement and type annotation
    movesObjectStr = movesObjectStr.replace(/export const Moves[^=]*= /, '');
    
    // Convert TypeScript object to JavaScript object
    // This is a simplified approach for the specific structure of PS moves
    movesObjectStr = movesObjectStr
        .replace(/(\w+): {/g, '"$1": {')  // Quote property names
        .replace(/(\w+): ([^,\n}]+)/g, (match, key, value) => {
            // Handle different value types
            value = value.trim().replace(/,$/, '');
            
            if (value === 'true' || value === 'false') {
                return `"${key}": ${value}`;
            }
            if (value === 'null') {
                return `"${key}": null`;
            }
            if (value.match(/^\\d+$/)) {
                return `"${key}": ${value}`;
            }
            if (value.startsWith('[') && value.endsWith(']')) {
                return `"${key}": ${value}`;
            }
            if (value.startsWith('{') && value.endsWith('}')) {
                return `"${key}": ${value}`;
            }
            // Quote string values
            if (!value.startsWith('"') && !value.endsWith('"')) {
                value = `"${value.replace(/"/g, '\\\\"')}"`;
            }
            return `"${key}": ${value}`;
        });
    
    // Try to parse as JSON
    try {
        return JSON.parse(movesObjectStr);
    } catch (parseError) {
        console.error('Manual parsing also failed:', parseError.message);
        throw new Error('All extraction methods failed');
    }
}

async function main() {
    try {
        const movesData = await extractMovesWithEsbuild();
        
        // Validate
        if (typeof movesData !== 'object' || movesData === null) {
            throw new Error('Extracted data is not an object');
        }
        
        const moveCount = Object.keys(movesData).length;
        if (moveCount === 0) {
            throw new Error('No moves found in extracted data');
        }
        
        // Create output directory
        const outputDir = path.join(__dirname, '../data/ps-extracted');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }
        
        // Write output
        const outputPath = path.join(outputDir, 'moves.json');
        fs.writeFileSync(outputPath, JSON.stringify(movesData, null, 2));
        
        console.log(`✅ Successfully extracted ${moveCount} moves to ${outputPath}`);
        
        // Show sample
        const sampleMoves = Object.keys(movesData).slice(0, 5);
        console.log('\\nSample moves:');
        sampleMoves.forEach(moveId => {
            const move = movesData[moveId];
            console.log(`  - ${moveId}: ${move.name || 'No name'} (${move.type || 'No type'})`);
        });
        
    } catch (error) {
        console.error('❌ Final extraction failed:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}