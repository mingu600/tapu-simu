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

async function extractItemForGeneration(genDex, itemId) {
    const item = genDex.items.get(itemId);
    
    // Skip if item doesn't exist in this generation
    if (!item.exists || (item.isNonstandard && item.isNonstandard !== 'Past')) {
        return null;
    }

    return {
        id: item.id,
        num: item.num,
        name: item.name,
        
        // Item categories
        isBerry: item.isBerry || false,
        isGem: item.isGem || false,
        isPokeball: item.isPokeball || false,
        isChoice: item.isChoice || false,
        
        // Mega stone properties
        megaStone: item.megaStone || null,
        megaEvolves: item.megaEvolves || null,
        
        // Z crystal properties
        zMove: item.zMove || null,
        zMoveType: item.zMoveType || null,
        zMoveFrom: item.zMoveFrom || null,
        
        // Stat boosts
        boosts: item.boosts || null,
        
        // Natural Gift properties
        naturalGift: item.naturalGift || null,
        
        // Fling properties
        fling: item.fling ? {
            basePower: item.fling.basePower || 0,
            status: item.fling.status || null,
            volatileStatus: item.fling.volatileStatus || null,
        } : null,
        
        // Item effects
        desc: item.desc || "",
        shortDesc: item.shortDesc || "",
        
        // Special flags
        ignoreKlutz: item.ignoreKlutz || false,
        
        // Plate/Memory/Drive types
        onPlate: item.onPlate || null,
        onMemory: item.onMemory || null,
        onDrive: item.onDrive || null,
        
        // Generation metadata
        isNonstandard: item.isNonstandard || null,
        
        // Berry-specific properties
        berryType: item.berryType || null,
        berryPower: item.berryPower || null,
        
        // Healing items
        heal: item.heal || null,
        
        // Status cure items
        cure: item.cure || null,
        
        // Other berry effects
        onEat: item.onEat ? true : false,
        onResidual: item.onResidual ? true : false,
        
        // Unreleased status
        unreleased: item.unreleased || false,
    };
}

async function extractGenerationItems() {
    const generationData = {};
    const itemChanges = {};
    
    // Get all unique item IDs from Gen 9 (most comprehensive)
    const allItemIds = Object.keys(Dex.data.Items);
    
    console.log(`Found ${allItemIds.length} items to check across generations`);
    
    // Extract items for each generation
    for (const generation of GENERATIONS) {
        console.log(`Extracting Gen ${generation.num} (${generation.name})...`);
        
        const genDex = Dex.forFormat(`gen${generation.num}ou`);
        const genItems = {};
        let itemCount = 0;
        
        for (const itemId of allItemIds) {
            const itemData = await extractItemForGeneration(genDex, itemId);
            if (itemData) {
                genItems[itemId] = itemData;
                itemCount++;
            }
        }
        
        generationData[generation.id] = {
            generation: generation.num,
            name: generation.name,
            itemCount,
            items: genItems,
        };
        
        console.log(`  ${itemCount} items available in Gen ${generation.num}`);
    }
    
    // Analyze item changes across generations
    console.log('\nAnalyzing item changes across generations...');
    
    for (const itemId of allItemIds) {
        const changes = [];
        let previousData = null;
        
        for (const generation of GENERATIONS) {
            const currentData = generationData[generation.id].items[itemId];
            
            if (currentData && previousData) {
                const changedFields = [];
                
                // Check for changes in key properties
                const fieldsToCheck = [
                    'isBerry', 'isGem', 'isPokeball', 'isChoice', 'megaStone', 'megaEvolves',
                    'zMove', 'zMoveType', 'zMoveFrom', 'boosts', 'naturalGift', 'fling',
                    'ignoreKlutz', 'onPlate', 'onMemory', 'onDrive', 'isNonstandard',
                    'berryType', 'berryPower', 'heal', 'cure', 'onEat', 'onResidual', 'unreleased'
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
            itemChanges[itemId] = {
                name: previousData?.name || itemId,
                changes,
            };
        }
    }
    
    return { generationData, itemChanges };
}

async function extractPokemonForGeneration(genDex, pokemonId) {
    const pokemon = genDex.species.get(pokemonId);
    
    // Skip if Pokemon doesn't exist in this generation
    if (!pokemon.exists || (pokemon.isNonstandard && pokemon.isNonstandard !== 'Past')) {
        return null;
    }

    return {
        id: pokemon.id,
        num: pokemon.num,
        name: pokemon.name,
        types: pokemon.types,
        baseStats: {
            hp: pokemon.baseStats.hp,
            attack: pokemon.baseStats.atk,
            defense: pokemon.baseStats.def,
            special_attack: pokemon.baseStats.spa,
            special_defense: pokemon.baseStats.spd,
            speed: pokemon.baseStats.spe,
        },
        abilities: pokemon.abilities,
        heightm: pokemon.heightm,
        weightkg: pokemon.weightkg,
        color: pokemon.color,
        evos: pokemon.evos || null,
        prevo: pokemon.prevo || null,
        evoType: pokemon.evoType || null,
        evoCondition: pokemon.evoCondition || null,
        eggGroups: pokemon.eggGroups,
        tier: pokemon.tier || null,
        doublesTier: pokemon.doublesTier || null,
        natDexTier: pokemon.natDexTier || null,
        
        // Form data
        baseSpecies: pokemon.baseSpecies,
        forme: pokemon.forme || null,
        baseForme: pokemon.baseForme || null,
        cosmeticFormes: pokemon.cosmeticFormes || null,
        otherFormes: pokemon.otherFormes || null,
        formeOrder: pokemon.formeOrder || null,
        
        // Special properties
        gender: pokemon.gender || null,
        genderRatio: pokemon.genderRatio || null,
        maleOnlyHidden: pokemon.maleOnlyHidden || false,
        gmaxUnreleased: pokemon.gmaxUnreleased || false,
        cannotDynamax: pokemon.cannotDynamax || false,
        
        // Mega Evolution
        isMega: pokemon.isMega || false,
        requiredItem: pokemon.requiredItem || null,
        requiredItems: pokemon.requiredItems || null,
        
        // Battle properties
        unreleasedHidden: pokemon.unreleasedHidden || false,
        battleOnly: pokemon.battleOnly || null,
        isNonstandard: pokemon.isNonstandard || null,
        
        // Regional variants
        canHatch: pokemon.canHatch !== false,
        
        // Tags for filtering
        tags: pokemon.tags || [],
    };
}

async function extractGenerationPokemon() {
    const generationData = {};
    const pokemonChanges = {};
    
    // Get all unique Pokemon IDs from Gen 9 (most comprehensive)
    const allPokemonIds = Object.keys(Dex.data.Species);
    
    console.log(`Found ${allPokemonIds.length} Pokemon to check across generations`);
    
    // Extract Pokemon for each generation
    for (const generation of GENERATIONS) {
        console.log(`Extracting Gen ${generation.num} (${generation.name})...`);
        
        const genDex = Dex.forFormat(`gen${generation.num}ou`);
        const genPokemon = {};
        let pokemonCount = 0;
        
        for (const pokemonId of allPokemonIds) {
            const pokemonData = await extractPokemonForGeneration(genDex, pokemonId);
            if (pokemonData) {
                genPokemon[pokemonId] = pokemonData;
                pokemonCount++;
            }
        }
        
        generationData[generation.id] = {
            generation: generation.num,
            name: generation.name,
            pokemonCount,
            pokemon: genPokemon,
        };
        
        console.log(`  ${pokemonCount} Pokemon available in Gen ${generation.num}`);
    }
    
    // Analyze Pokemon changes across generations
    console.log('\nAnalyzing Pokemon changes across generations...');
    
    for (const pokemonId of allPokemonIds) {
        const changes = [];
        let previousData = null;
        
        for (const generation of GENERATIONS) {
            const currentData = generationData[generation.id].pokemon[pokemonId];
            
            if (currentData && previousData) {
                const changedFields = [];
                
                // Check for changes in base stats
                const statsToCheck = ['hp', 'attack', 'defense', 'special_attack', 'special_defense', 'speed'];
                for (const stat of statsToCheck) {
                    if (currentData.baseStats[stat] !== previousData.baseStats[stat]) {
                        changedFields.push({
                            field: `baseStats.${stat}`,
                            from: previousData.baseStats[stat],
                            to: currentData.baseStats[stat],
                        });
                    }
                }
                
                // Check for other important changes
                const fieldsToCheck = [
                    'types', 'abilities', 'tier', 'doublesTier', 'natDexTier',
                    'isNonstandard', 'unreleasedHidden'
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
            pokemonChanges[pokemonId] = {
                name: previousData?.name || pokemonId,
                changes,
            };
        }
    }
    
    return { generationData, pokemonChanges };
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0] || 'all';
    
    const outputDir = path.join(__dirname, '../../data/ps-extracted');
    await fs.mkdir(outputDir, { recursive: true });
    
    if (command === 'moves' || command === 'all') {
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
    
    if (command === 'items' || command === 'all') {
        console.log('🚀 Extracting generation-specific item data...\n');
        const { generationData, itemChanges } = await extractGenerationItems();
        
        // Save generation data
        const genDataPath = path.join(outputDir, 'items-by-generation.json');
        await fs.writeFile(genDataPath, JSON.stringify(generationData, null, 2));
        console.log(`\n✅ Generation data saved to ${genDataPath}`);
        
        // Save item changes analysis
        const changesPath = path.join(outputDir, 'item-changes.json');
        await fs.writeFile(changesPath, JSON.stringify(itemChanges, null, 2));
        
        const changesCount = Object.keys(itemChanges).length;
        console.log(`✅ Item changes analysis saved to ${changesPath}`);
        console.log(`📊 Found ${changesCount} items with changes across generations`);
        
        // Print some interesting statistics
        console.log('\n📈 Generation Statistics:');
        for (const generation of GENERATIONS) {
            const genData = generationData[generation.id];
            console.log(`  Gen ${generation.num} (${generation.name}): ${genData.itemCount} items`);
        }
        
        // Print some notable changes
        console.log('\n🔥 Notable Item Changes:');
        let notableCount = 0;
        for (const [itemId, changeData] of Object.entries(itemChanges)) {
            if (notableCount >= 5) break;
            
            const hasImportantChange = changeData.changes.some(change => 
                change.changes.some(c => ['fling', 'naturalGift', 'berryPower', 'heal'].includes(c.field))
            );
            
            if (hasImportantChange) {
                console.log(`  ${changeData.name}:`);
                for (const genChange of changeData.changes.slice(0, 2)) {
                    for (const change of genChange.changes.slice(0, 2)) {
                        console.log(`    Gen ${genChange.generation}: ${change.field} ${JSON.stringify(change.from)} → ${JSON.stringify(change.to)}`);
                    }
                }
                notableCount++;
            }
        }
    }
    
    if (command === 'pokemon' || command === 'all') {
        console.log('🚀 Extracting generation-specific Pokemon data...\n');
        const { generationData, pokemonChanges } = await extractGenerationPokemon();
        
        // Save generation data
        const genDataPath = path.join(outputDir, 'pokemon-by-generation.json');
        await fs.writeFile(genDataPath, JSON.stringify(generationData, null, 2));
        console.log(`\n✅ Generation data saved to ${genDataPath}`);
        
        // Save Pokemon changes analysis
        const changesPath = path.join(outputDir, 'pokemon-changes.json');
        await fs.writeFile(changesPath, JSON.stringify(pokemonChanges, null, 2));
        
        const changesCount = Object.keys(pokemonChanges).length;
        console.log(`✅ Pokemon changes analysis saved to ${changesPath}`);
        console.log(`📊 Found ${changesCount} Pokemon with changes across generations`);
        
        // Print some interesting statistics
        console.log('\n📈 Generation Statistics:');
        for (const generation of GENERATIONS) {
            const genData = generationData[generation.id];
            console.log(`  Gen ${generation.num} (${generation.name}): ${genData.pokemonCount} Pokemon`);
        }
        
        // Print some notable stat changes
        console.log('\n🔥 Notable Pokemon Stat Changes:');
        let notableCount = 0;
        for (const [pokemonId, changeData] of Object.entries(pokemonChanges)) {
            if (notableCount >= 10) break;
            
            const hasStatChange = changeData.changes.some(change => 
                change.changes.some(c => c.field.startsWith('baseStats.'))
            );
            
            if (hasStatChange) {
                console.log(`  ${changeData.name}:`);
                for (const genChange of changeData.changes.slice(0, 3)) {
                    const statChanges = genChange.changes.filter(c => c.field.startsWith('baseStats.'));
                    for (const change of statChanges.slice(0, 3)) {
                        const statName = change.field.replace('baseStats.', '');
                        console.log(`    Gen ${genChange.generation}: ${statName} ${change.from} → ${change.to}`);
                    }
                }
                notableCount++;
            }
        }
    }
}

main().catch(console.error);