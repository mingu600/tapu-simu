
const fs = require('fs');
const path = require('path');

// Change to Pokemon Showdown directory and try to load the module
process.chdir('/home/mingukim/Documents/pokemon/pokemon-showdown');

try {
    // Try to compile and load the TypeScript moves file
    const { execSync } = require('child_process');
    
    // First try to transpile the TypeScript to JavaScript
    const movesPath = path.join(process.cwd(), 'data/moves.ts');
    const content = fs.readFileSync(movesPath, 'utf8');
    
    // Simple TypeScript to JavaScript conversion
    let jsContent = content
        .replace(/import\s+.*?from\s+['"][^'"]*['"];?\s*/g, '')
        .replace(/export\s+const\s+(\w+):\s*[^=]+=\s*/g, 'const $1 = ')
        .replace(/\/\/.*$/gm, '') // Remove comments
        .replace(/\/\*[\s\S]*?\*\//g, ''); // Remove multi-line comments
    
    // Wrap in a function that returns the data
    jsContent = jsContent + '\nif (typeof Moves !== "undefined") console.log(JSON.stringify(Moves, null, 2));';
    
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
