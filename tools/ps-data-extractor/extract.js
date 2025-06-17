#!/usr/bin/env node

import { Dex } from '@pkmn/sim';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// No mapping needed - we'll use Pokemon Showdown's naming directly

async function extractMoves(dex) {
    const extractedMoves = {};

    // Get all move names and iterate through them
    for (const moveId in dex.data.Moves) {
        const move = dex.moves.get(moveId);
        // Include all moves for comprehensive testing
        // (Previously filtered out nonstandard moves, but we want complete data)

        const moveData = {
            id: move.id,
            num: move.num,
            name: move.name,
            basePower: move.basePower || 0,
            accuracy: move.accuracy === true ? 100 : (move.accuracy || 0),
            pp: move.pp || 0,
            maxPP: move.pp ? Math.floor(move.pp * 1.6) : 0, // Max PP with PP Ups
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
            
            // Effect description for complex moves
            desc: move.desc || "",
            shortDesc: move.shortDesc || "",
        };
        
        extractedMoves[move.id] = moveData;
    }
    
    return extractedMoves;
}

async function extractPokemon(dex) {
    const extractedPokemon = {};
    
    // Get all Pokemon names and iterate through them
    for (const pokemonId in dex.data.Pokedex) {
        const pokemon = dex.species.get(pokemonId);
        // Include all Pokemon for comprehensive testing
        // (Previously filtered out nonstandard Pokemon, but we want complete data)
        
        const pokemonData = {
            id: pokemon.id,
            num: pokemon.num,
            name: pokemon.name,
            
            // Basic info
            types: pokemon.types,
            baseStats: pokemon.baseStats,
            abilities: pokemon.abilities,
            
            // Physical properties
            heightm: pokemon.heightm,
            weightkg: pokemon.weightkg,
            color: pokemon.color,
            
            // Evolution and form data
            prevo: pokemon.prevo || null,
            evos: pokemon.evos || [],
            evoType: pokemon.evoType || null,
            evoCondition: pokemon.evoCondition || null,
            evoItem: pokemon.evoItem || null,
            evoLevel: pokemon.evoLevel || null,
            
            // Forme data
            baseForme: pokemon.baseForme || null,
            forme: pokemon.forme || null,
            baseSpecies: pokemon.baseSpecies || pokemon.name,
            otherFormes: pokemon.otherFormes || [],
            formeOrder: pokemon.formeOrder || [],
            
            // Battle properties
            gender: pokemon.gender || null,
            genderRatio: pokemon.genderRatio || null,
            maxHP: pokemon.maxHP || null,
            
            // Learnset reference
            learnset: pokemon.learnset || null,
            
            // Special properties
            tags: pokemon.tags || [],
            tier: pokemon.tier || null,
            doublesTier: pokemon.doublesTier || null,
            
            // Mega/Forme properties
            isMega: pokemon.isMega || false,
            isPrimal: pokemon.isPrimal || false,
            cannotDynamax: pokemon.cannotDynamax || false,
            canGigantamax: pokemon.canGigantamax || false,
            gigantamax: pokemon.gigantamax || null,
            
            // Regional variants
            cosmeticFormes: pokemon.cosmeticFormes || [],
            
            // Required items
            requiredItem: pokemon.requiredItem || null,
            requiredItems: pokemon.requiredItems || [],
            
            // Battle-only formes
            battleOnly: pokemon.battleOnly || null,
            
            // Special mechanics
            unreleasedHidden: pokemon.unreleasedHidden || false,
            maleOnlyHidden: pokemon.maleOnlyHidden || false,
            changesFrom: pokemon.changesFrom || null,
        };
        
        extractedPokemon[pokemon.id] = pokemonData;
    }
    
    return extractedPokemon;
}

async function extractItems(dex) {
    const extractedItems = {};
    
    // Get all item names and iterate through them
    for (const itemId in dex.data.Items) {
        const item = dex.items.get(itemId);
        // Include all items for comprehensive testing
        // (Previously filtered out nonstandard items, but we want complete data)
        
        const itemData = {
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
        
        extractedItems[item.id] = itemData;
    }
    
    return extractedItems;
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0] || 'all';
    
    // Use Gen 9 data by default - Dex is already the latest generation
    
    const outputDir = path.join(__dirname, '../../data/ps-extracted');
    await fs.mkdir(outputDir, { recursive: true });
    
    if (command === 'moves' || command === 'all') {
        console.log('Extracting moves...');
        const moves = await extractMoves(Dex);
        const movesPath = path.join(outputDir, 'moves.json');
        await fs.writeFile(movesPath, JSON.stringify(moves, null, 2));
        console.log(`Extracted ${Object.keys(moves).length} moves to ${movesPath}`);
    }
    
    if (command === 'pokemon' || command === 'all') {
        console.log('Extracting Pokemon...');
        const pokemon = await extractPokemon(Dex);
        const pokemonPath = path.join(outputDir, 'pokemon.json');
        await fs.writeFile(pokemonPath, JSON.stringify(pokemon, null, 2));
        console.log(`Extracted ${Object.keys(pokemon).length} Pokemon to ${pokemonPath}`);
    }
    
    if (command === 'items' || command === 'all') {
        console.log('Extracting items...');
        const items = await extractItems(Dex);
        const itemsPath = path.join(outputDir, 'items.json');
        await fs.writeFile(itemsPath, JSON.stringify(items, null, 2));
        console.log(`Extracted ${Object.keys(items).length} items to ${itemsPath}`);
    }
}

main().catch(console.error);