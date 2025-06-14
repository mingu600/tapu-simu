#!/usr/bin/env node

import { Dex } from '@pkmn/dex';
import { Generations } from '@pkmn/data';
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// No mapping needed - we'll use Pokemon Showdown's naming directly

async function extractMoves(gen) {
    const moves = gen.moves;
    const extractedMoves = {};

    for (const move of moves) {
        // Skip nonstandard moves unless they're Z-moves or Max moves
        if (move.isNonstandard && !move.isZ && !move.isMax) {
            continue;
        }

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

async function extractItems(gen) {
    const items = gen.items;
    const extractedItems = {};
    
    for (const item of items) {
        // Skip nonstandard items
        if (item.isNonstandard) {
            continue;
        }
        
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
        };
        
        extractedItems[item.id] = itemData;
    }
    
    return extractedItems;
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0] || 'all';
    
    // Use Gen 9 data by default
    const gens = new Generations(Dex);
    const gen = gens.get(9);
    
    const outputDir = path.join(__dirname, '../../data/ps-extracted');
    await fs.mkdir(outputDir, { recursive: true });
    
    if (command === 'moves' || command === 'all') {
        console.log('Extracting moves...');
        const moves = await extractMoves(gen);
        const movesPath = path.join(outputDir, 'moves.json');
        await fs.writeFile(movesPath, JSON.stringify(moves, null, 2));
        console.log(`Extracted ${Object.keys(moves).length} moves to ${movesPath}`);
    }
    
    if (command === 'items' || command === 'all') {
        console.log('Extracting items...');
        const items = await extractItems(gen);
        const itemsPath = path.join(outputDir, 'items.json');
        await fs.writeFile(itemsPath, JSON.stringify(items, null, 2));
        console.log(`Extracted ${Object.keys(items).length} items to ${itemsPath}`);
    }
}

main().catch(console.error);