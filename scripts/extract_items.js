#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const esbuild = require('esbuild');

async function extractItems() {
    console.log('Using esbuild to extract Pokemon Showdown items data...');
    
    try {
        // Path to Pokemon Showdown data
        const psDataDir = path.join(__dirname, '../../pokemon-showdown/data');
        const itemsPath = path.join(psDataDir, 'items.ts');
        
        if (!fs.existsSync(itemsPath)) {
            throw new Error(`Items file not found: ${itemsPath}`);
        }
        
        // Create a temporary directory for building
        const tempDir = path.join(__dirname, '../temp');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }
        
        // Create a wrapper script that will export the items data
        const wrapperScript = `
import { Items } from '${itemsPath}';

// Export as JSON to stdout
console.log(JSON.stringify(Items, null, 2));
`;
        
        const wrapperPath = path.join(tempDir, 'extract_items_wrapper.ts');
        fs.writeFileSync(wrapperPath, wrapperScript);
        
        // Use esbuild to transpile and bundle
        await esbuild.build({
            entryPoints: [wrapperPath],
            bundle: true,
            format: 'cjs',
            platform: 'node',
            outfile: path.join(tempDir, 'extract_items.js'),
            external: [],
            write: true,
            logLevel: 'warning'
        });
        
        // Run the built script
        const { execSync } = require('child_process');
        const builtScript = path.join(tempDir, 'extract_items.js');
        
        const result = execSync(`node ${builtScript}`, {
            encoding: 'utf8',
            maxBuffer: 50 * 1024 * 1024,
            cwd: tempDir
        });
        
        // Parse the result
        const itemsData = JSON.parse(result);
        
        // Clean up temp files
        fs.rmSync(tempDir, { recursive: true, force: true });
        
        return itemsData;
        
    } catch (error) {
        console.error('Items extraction failed:', error.message);
        throw error;
    }
}

async function main() {
    try {
        const itemsData = await extractItems();
        
        // Validate
        if (typeof itemsData !== 'object' || itemsData === null) {
            throw new Error('Extracted data is not an object');
        }
        
        const itemCount = Object.keys(itemsData).length;
        if (itemCount === 0) {
            throw new Error('No items found in extracted data');
        }
        
        // Create output directory
        const outputDir = path.join(__dirname, '../data/ps-extracted');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }
        
        // Write output
        const outputPath = path.join(outputDir, 'items.json');
        fs.writeFileSync(outputPath, JSON.stringify(itemsData, null, 2));
        
        console.log(`✅ Successfully extracted ${itemCount} items to ${outputPath}`);
        
        // Show sample
        const sampleItems = Object.keys(itemsData).slice(0, 5);
        console.log('\\nSample items:');
        sampleItems.forEach(itemId => {
            const item = itemsData[itemId];
            console.log(`  - ${itemId}: ${item.name || 'No name'}`);
        });
        
    } catch (error) {
        console.error('❌ Items extraction failed:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}