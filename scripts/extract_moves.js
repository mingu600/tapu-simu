#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read the moves.ts file
const movesPath = path.join(__dirname, '../../pokemon-showdown/data/moves.ts');
const movesContent = fs.readFileSync(movesPath, 'utf8');

// Extract the data object by finding the opening and closing braces
const startMatch = movesContent.match(/export const Moves[^{]*= {/);
if (!startMatch) {
    console.error('Could not find moves export');
    process.exit(1);
}

const startIndex = movesContent.indexOf('{', startMatch.index);
let braceCount = 0;
let endIndex = startIndex;

for (let i = startIndex; i < movesContent.length; i++) {
    if (movesContent[i] === '{') braceCount++;
    if (movesContent[i] === '}') braceCount--;
    if (braceCount === 0) {
        endIndex = i;
        break;
    }
}

// Extract the data object content
const dataContent = movesContent.substring(startIndex, endIndex + 1);

// Create a simple eval context
try {
    const movesData = eval('(' + dataContent + ')');
    
    // Write to output file
    const outputPath = path.join(__dirname, '../data/ps-extracted/moves.json');
    fs.writeFileSync(outputPath, JSON.stringify(movesData, null, 2));
    
    console.log(`Extracted ${Object.keys(movesData).length} moves to ${outputPath}`);
} catch (error) {
    console.error('Error parsing moves data:', error.message);
    process.exit(1);
}