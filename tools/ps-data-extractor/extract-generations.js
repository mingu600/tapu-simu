#!/usr/bin/env node

import { Dex } from '@pkmn/sim';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Generation configurations
const GENERATIONS = [
    { num: 1, id: 'gen1', name: 'RBY' },
    { num: 2, id: 'gen2', name: 'GSC' },
    { num: 3, id: 'gen3', name: 'RSE' },
    { num: 4, id: 'gen4', name: 'DPPt' },
    { num: 5, id: 'gen5', name: 'BW' },
    { num: 6, id: 'gen6', name: 'XY' },
    { num: 7, id: 'gen7', name: 'SM' },
    { num: 8, id: 'gen8', name: 'SWSH' },
    { num: 9, id: 'gen9', name: 'SV' },
];

async function extractMoveForGeneration(genDex, moveId) {
    const move = genDex.moves.get(moveId);
    
    // Skip if move doesn't exist in this generation
    if (!move.exists || (move.isNonstandard && !move.isZ && !move.isMax)) {
        return null;
    }

    return {
        id: move.id,
        num: move.num,
        name: move.name,
        basePower: move.basePower || 0,
        accuracy: move.accuracy === true ? 100 : (move.accuracy || 0),
        pp: move.pp || 0,
        maxPP: move.pp ? Math.floor(move.pp * 1.6) : 0,
        type: move.type,
        category: move.category,
        priority: move.priority || 0,
        target: move.target,
        flags: move.flags || {},
        
        // Move effects
        drain: move.drain || null,
        recoil: move.recoil || null,
        heal: move.heal || null,
        
        // Status effects
        status: move.status || null,
        volatileStatus: move.volatileStatus || null,
        
        // Secondary effects
        secondary: move.secondary ? {
            chance: move.secondary.chance || 100,
            status: move.secondary.status || null,
            volatileStatus: move.secondary.volatileStatus || null,
            boosts: move.secondary.boosts || null,
        } : null,
        
        // Self effects
        self: move.self ? {
            boosts: move.self.boosts || null,
            volatileStatus: move.self.volatileStatus || null,
        } : null,
        
        // Special properties
        isZ: move.isZ || false,
        isMax: move.isMax || false,
        ohko: move.ohko || false,
        thawsTarget: move.thawsTarget || false,
        forceSwitch: move.forceSwitch || false,
        selfSwitch: move.selfSwitch || false,
        breaksProtect: move.breaksProtect || false,
        ignoreDefensive: move.ignoreDefensive || false,
        ignoreEvasion: move.ignoreEvasion || false,
        ignoreImmunity: move.ignoreImmunity || false,
        multiaccuracy: move.multiaccuracy || false,
        multihit: move.multihit || null,
        noDamageVariance: move.noDamageVariance || false,
        
        // Critical hit properties
        critRatio: move.critRatio || 1,
        willCrit: move.willCrit || false,
        
        // Weather/terrain interactions
        terrain: move.terrain || null,
        weather: move.weather || null,
        
        // Effect description
        desc: move.desc || "",
        shortDesc: move.shortDesc || "",
        
        // Generation metadata
        isNonstandard: move.isNonstandard || null,
    };
}

async function extractGenerationMoves() {
    const generationData = {};
    const moveChanges = {};
    
    // Get all unique move IDs from Gen 9 (most comprehensive)
    const allMoveIds = Object.keys(Dex.data.Moves);
    
    console.log(`Found ${allMoveIds.length} moves to check across generations`);
    
    // Extract moves for each generation
    for (const generation of GENERATIONS) {
        console.log(`Extracting Gen ${generation.num} (${generation.name})...`);
        
        const genDex = Dex.forFormat(`gen${generation.num}ou`);
        const genMoves = {};
        let moveCount = 0;
        
        for (const moveId of allMoveIds) {
            const moveData = await extractMoveForGeneration(genDex, moveId);
            if (moveData) {
                genMoves[moveId] = moveData;
                moveCount++;
            }
        }
        
        generationData[generation.id] = {
            generation: generation.num,
            name: generation.name,
            moveCount,
            moves: genMoves,
        };
        
        console.log(`  ${moveCount} moves available in Gen ${generation.num}`);
    }
    
    // Analyze move changes across generations
    console.log('\nAnalyzing move changes across generations...');
    
    for (const moveId of allMoveIds) {
        const changes = [];
        let previousData = null;
        
        for (const generation of GENERATIONS) {
            const currentData = generationData[generation.id].moves[moveId];
            
            if (currentData && previousData) {
                const changedFields = [];
                
                // Check for changes in key properties
                const fieldsToCheck = [
                    'basePower', 'accuracy', 'pp', 'type', 'category', 
                    'priority', 'target', 'isNonstandard'
                ];
                
                for (const field of fieldsToCheck) {
                    if (JSON.stringify(currentData[field]) !== JSON.stringify(previousData[field])) {
                        changedFields.push({
                            field,
                            from: previousData[field],
                            to: currentData[field],
                        });
                    }
                }
                
                // Check secondary effects
                if (JSON.stringify(currentData.secondary) !== JSON.stringify(previousData.secondary)) {
                    changedFields.push({
                        field: 'secondary',
                        from: previousData.secondary,
                        to: currentData.secondary,
                    });
                }
                
                if (changedFields.length > 0) {
                    changes.push({
                        generation: generation.num,
                        changes: changedFields,
                    });
                }
            }
            
            if (currentData) {
                previousData = currentData;
            }
        }
        
        if (changes.length > 0) {
            moveChanges[moveId] = {
                name: previousData?.name || moveId,
                changes,
            };
        }
    }
    
    return { generationData, moveChanges };
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0] || 'all';
    
    const outputDir = path.join(__dirname, '../../data/ps-extracted');
    await fs.mkdir(outputDir, { recursive: true });
    
    if (command === 'generations' || command === 'all') {
        console.log('🚀 Extracting generation-specific move data...\n');
        const { generationData, moveChanges } = await extractGenerationMoves();
        
        // Save generation data
        const genDataPath = path.join(outputDir, 'moves-by-generation.json');
        await fs.writeFile(genDataPath, JSON.stringify(generationData, null, 2));
        console.log(`\n✅ Generation data saved to ${genDataPath}`);
        
        // Save move changes analysis
        const changesPath = path.join(outputDir, 'move-changes.json');
        await fs.writeFile(changesPath, JSON.stringify(moveChanges, null, 2));
        
        const changesCount = Object.keys(moveChanges).length;
        console.log(`✅ Move changes analysis saved to ${changesPath}`);
        console.log(`📊 Found ${changesCount} moves with changes across generations`);
        
        // Print some interesting statistics
        console.log('\n📈 Generation Statistics:');
        for (const generation of GENERATIONS) {
            const genData = generationData[generation.id];
            console.log(`  Gen ${generation.num} (${generation.name}): ${genData.moveCount} moves`);
        }
        
        // Print some notable changes
        console.log('\n🔥 Notable Move Changes:');
        let notableCount = 0;
        for (const [moveId, changeData] of Object.entries(moveChanges)) {
            if (notableCount >= 5) break;
            
            const hasTypeChange = changeData.changes.some(change => 
                change.changes.some(c => c.field === 'type')
            );
            const hasPowerChange = changeData.changes.some(change => 
                change.changes.some(c => c.field === 'basePower')
            );
            
            if (hasTypeChange || hasPowerChange) {
                console.log(`  ${changeData.name}:`);
                for (const genChange of changeData.changes.slice(0, 2)) {
                    for (const change of genChange.changes.slice(0, 2)) {
                        console.log(`    Gen ${genChange.generation}: ${change.field} ${change.from} → ${change.to}`);
                    }
                }
                notableCount++;
            }
        }
    }
}

main().catch(console.error);