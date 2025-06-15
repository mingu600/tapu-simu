#!/usr/bin/env npx ts-node

import * as fs from 'fs';
import * as path from 'path';

// Import Pokemon Showdown moves data directly
const psDataPath = path.join(__dirname, '../../pokemon-showdown/data/moves.ts');

async function extractMoves() {
    console.log('Loading Pokemon Showdown moves data...');
    
    try {
        // Read the file content
        const movesContent = fs.readFileSync(psDataPath, 'utf8');
        
        // Create a temporary TypeScript file that we can compile and run
        const tempDir = path.join(__dirname, '../temp');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }
        
        // Create a simplified extraction script
        const extractorScript = `
// Import the moves data
${movesContent}

// Export the data for extraction
if (typeof Moves !== 'undefined') {
    console.log(JSON.stringify(Moves, null, 2));
} else {
    console.error('Moves not found');
    process.exit(1);
}
`;
        
        const tempFile = path.join(tempDir, 'extract_moves_temp.ts');
        fs.writeFileSync(tempFile, extractorScript);
        
        // Use ts-node to run the script and capture output
        const { exec } = require('child_process');
        
        return new Promise((resolve, reject) => {
            exec(`npx ts-node ${tempFile}`, { 
                cwd: path.join(__dirname, '../..'),
                maxBuffer: 50 * 1024 * 1024 // 50MB buffer for large data
            }, (error, stdout, stderr) => {
                // Clean up temp file
                if (fs.existsSync(tempFile)) {
                    fs.unlinkSync(tempFile);
                }
                
                if (error) {
                    console.error('TypeScript execution error:', error.message);
                    reject(error);
                    return;
                }
                
                if (stderr && !stderr.includes('ExperimentalWarning')) {
                    console.error('TypeScript stderr:', stderr);
                }
                
                try {
                    const movesData = JSON.parse(stdout);
                    resolve(movesData);
                } catch (parseError) {
                    console.error('Failed to parse extracted JSON:', parseError.message);
                    console.error('Raw output:', stdout.substring(0, 500) + '...');
                    reject(parseError);
                }
            });
        });
        
    } catch (error) {
        console.error('Error in extraction process:', error.message);
        throw error;
    }
}

async function main() {
    try {
        const movesData = await extractMoves();
        
        // Validate the data
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
        
        // Write to output file
        const outputPath = path.join(outputDir, 'moves.json');
        fs.writeFileSync(outputPath, JSON.stringify(movesData, null, 2));
        
        console.log(`✅ Successfully extracted ${moveCount} moves to ${outputPath}`);
        
        // Show sample data
        const sampleMoves = Object.keys(movesData).slice(0, 5);
        console.log('\nSample moves extracted:');
        sampleMoves.forEach(moveId => {
            const move = movesData[moveId];
            console.log(`  - ${moveId}: ${move.name || 'No name'} (${move.type || 'No type'})`);
        });
        
    } catch (error) {
        console.error('❌ Fatal error:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}