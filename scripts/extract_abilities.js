#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const esbuild = require('esbuild');

async function extractAbilities() {
    console.log('Using esbuild to extract Pokemon Showdown abilities data...');
    
    try {
        // Path to Pokemon Showdown data
        const psDataDir = path.join(__dirname, '../../pokemon-showdown/data');
        const abilitiesPath = path.join(psDataDir, 'abilities.ts');
        
        if (!fs.existsSync(abilitiesPath)) {
            throw new Error(`Abilities file not found: ${abilitiesPath}`);
        }
        
        // Create a temporary directory for building
        const tempDir = path.join(__dirname, '../temp');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }
        
        // Create a wrapper script that will export the abilities data
        const wrapperScript = `
import { Abilities } from '${abilitiesPath}';

// Export as JSON to stdout
console.log(JSON.stringify(Abilities, null, 2));
`;
        
        const wrapperPath = path.join(tempDir, 'extract_abilities_wrapper.ts');
        fs.writeFileSync(wrapperPath, wrapperScript);
        
        // Use esbuild to transpile and bundle
        await esbuild.build({
            entryPoints: [wrapperPath],
            bundle: true,
            format: 'cjs',
            platform: 'node',
            outfile: path.join(tempDir, 'extract_abilities.js'),
            external: [],
            write: true,
            logLevel: 'warning'
        });
        
        // Run the built script
        const { execSync } = require('child_process');
        const builtScript = path.join(tempDir, 'extract_abilities.js');
        
        const result = execSync(`node ${builtScript}`, {
            encoding: 'utf8',
            maxBuffer: 50 * 1024 * 1024,
            cwd: tempDir
        });
        
        // Parse the result
        const abilitiesData = JSON.parse(result);
        
        // Clean up temp files
        fs.rmSync(tempDir, { recursive: true, force: true });
        
        return abilitiesData;
        
    } catch (error) {
        console.error('Abilities extraction failed:', error.message);
        throw error;
    }
}

async function main() {
    try {
        const abilitiesData = await extractAbilities();
        
        // Validate
        if (typeof abilitiesData !== 'object' || abilitiesData === null) {
            throw new Error('Extracted data is not an object');
        }
        
        const abilityCount = Object.keys(abilitiesData).length;
        if (abilityCount === 0) {
            throw new Error('No abilities found in extracted data');
        }
        
        // Create output directory
        const outputDir = path.join(__dirname, '../data/ps-extracted');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }
        
        // Write output
        const outputPath = path.join(outputDir, 'abilities.json');
        fs.writeFileSync(outputPath, JSON.stringify(abilitiesData, null, 2));
        
        console.log(`✅ Successfully extracted ${abilityCount} abilities to ${outputPath}`);
        
        // Show sample
        const sampleAbilities = Object.keys(abilitiesData).slice(0, 5);
        console.log('\\nSample abilities:');
        sampleAbilities.forEach(abilityId => {
            const ability = abilitiesData[abilityId];
            console.log(`  - ${abilityId}: ${ability.name || 'No name'}`);
        });
        
    } catch (error) {
        console.error('❌ Abilities extraction failed:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}