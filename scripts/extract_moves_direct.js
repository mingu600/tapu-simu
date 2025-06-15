#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('Extracting Pokemon Showdown moves data...');

// Create a temporary Node.js script that can import the PS data
const tempScript = `
const fs = require('fs');
const path = require('path');

// Change to Pokemon Showdown directory and try to load the module
process.chdir('${path.join(__dirname, '../../pokemon-showdown')}');

try {
    // Try to compile and load the TypeScript moves file
    const { execSync } = require('child_process');
    
    // First try to transpile the TypeScript to JavaScript
    const movesPath = path.join(process.cwd(), 'data/moves.ts');
    const content = fs.readFileSync(movesPath, 'utf8');
    
    // Simple TypeScript to JavaScript conversion
    let jsContent = content
        .replace(/import\\s+.*?from\\s+['"][^'"]*['"];?\\s*/g, '')
        .replace(/export\\s+const\\s+(\\w+):\\s*[^=]+=\\s*/g, 'const $1 = ')
        .replace(/\\/\\/.*$/gm, '') // Remove comments
        .replace(/\\/\\*[\\s\\S]*?\\*\\//g, ''); // Remove multi-line comments
    
    // Wrap in a function that returns the data
    jsContent = jsContent + '\\nif (typeof Moves !== "undefined") console.log(JSON.stringify(Moves, null, 2));';
    
    // Write temporary JS file
    const tempJsFile = path.join(process.cwd(), 'temp_moves.js');
    fs.writeFileSync(tempJsFile, jsContent);
    
    // Execute it
    const result = execSync('node temp_moves.js', { encoding: 'utf8', maxBuffer: 50 * 1024 * 1024 });
    
    // Clean up
    fs.unlinkSync(tempJsFile);
    
    // Output the result
    console.log(result);
    
} catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
}
`;

// Write and execute the temporary script
const tempFile = path.join(__dirname, 'temp_extract.js');
try {
    fs.writeFileSync(tempFile, tempScript);
    
    const result = execSync(`node ${tempFile}`, { 
        encoding: 'utf8',
        maxBuffer: 50 * 1024 * 1024,
        cwd: __dirname
    });
    
    // Parse the JSON output
    const movesData = JSON.parse(result);
    
    // Validate
    const moveCount = Object.keys(movesData).length;
    if (moveCount === 0) {
        throw new Error('No moves found');
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
    console.error('❌ Extraction failed:', error.message);
    process.exit(1);
} finally {
    // Clean up temp file
    if (fs.existsSync(tempFile)) {
        fs.unlinkSync(tempFile);
    }
}