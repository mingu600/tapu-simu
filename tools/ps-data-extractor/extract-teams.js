#!/usr/bin/env node

import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Import Pokemon Showdown from parent directory
const PS_PATH = resolve(__dirname, '../../../pokemon-showdown');

// Import PS modules - Pokemon Showdown needs to be built first
async function importPSModules() {
  try {
    // Try to use the compiled JS version from build/ directory
    const { Dex } = await import(`file://${PS_PATH}/build/sim/index.js`);
    const { Teams } = await import(`file://${PS_PATH}/build/sim/teams.js`);
    return { Dex, Teams };
  } catch (error) {
    console.error('Failed to import Pokemon Showdown modules:', error);
    console.error('Pokemon Showdown needs to be built first. Try running:');
    console.error(`  cd ${PS_PATH}`);
    console.error('  npm install');
    console.error('  npm run build');
    process.exit(1);
  }
}

const SEED = [1, 2, 3, 4];
const DEFAULT_TEAM_COUNT = 10000;
const SKIP_FORMATS = [
  'factory', 'unrated', 'cup', 'staff', 'cap', 'monotype', 'chimera',
  'blitz', 'generated', 'joltemons', 'pokebilities', 'firstblood', 'apex',
  'aprilfoolsday', 'fusionevolution',
];

// Get supported random battle formats
function getSupportedFormats(Dex) {
  const formats = [];
  for (const format of Dex.formats.all()) {
    if (!format.team || !['singles', 'doubles'].includes(format.gameType)) continue;
    if (SKIP_FORMATS.some(s => format.id.includes(s)) || !format.mod.startsWith('gen')) continue;
    formats.push(format);
  }
  return formats;
}

// Clean up a Pokemon set for export
function cleanPokemonSet(set) {
  return {
    name: set.name,
    species: set.species,
    gender: set.gender,
    item: set.item,
    ability: set.ability,
    moves: set.moves,
    nature: set.nature,
    evs: set.evs,
    ivs: set.ivs,
    level: set.level,
    shiny: set.shiny,
    teraType: set.teraType,
    role: set.role,
    gigantamax: set.gigantamax
  };
}

// Generate teams for a format
function generateTeams(Dex, Teams, format, teamCount) {
  console.log(`Generating ${teamCount} teams for ${format.name}...`);
  
  const generator = Teams.getGenerator(format, SEED);
  const teams = [];
  
  for (let i = 0; i < teamCount; i++) {
    const team = generator.getTeam();
    teams.push(team.map(cleanPokemonSet));
    
    if ((i + 1) % 1000 === 0) {
      console.log(`  Generated ${i + 1}/${teamCount} teams`);
    }
  }
  
  return teams;
}

// Save teams to file
function saveTeams(teams, formatId, outputDir) {
  mkdirSync(outputDir, { recursive: true });
  
  const filename = `${outputDir}/${formatId}-teams.json`;
  const jsonString = JSON.stringify(teams, null, 2);
  
  writeFileSync(filename, jsonString);
  console.log(`Saved ${teams.length} teams to ${filename}`);
  console.log(`File size: ${(jsonString.length / 1024 / 1024).toFixed(2)} MB`);
}

// Main function
async function main() {
  const args = process.argv.slice(2);
  const formatFilter = args[0]; // Optional format filter (e.g., "gen9" or "gen9randombattle")
  const teamCount = parseInt(args[1]) || DEFAULT_TEAM_COUNT;
  
  console.log('Importing Pokemon Showdown modules...');
  const { Dex, Teams } = await importPSModules();
  
  // Include mod data for all generations
  Dex.includeModData();
  
  console.log('Getting supported formats...');
  const allFormats = getSupportedFormats(Dex);
  
  // Filter formats if specified
  const formats = formatFilter 
    ? allFormats.filter(f => f.id.includes(formatFilter.toLowerCase()))
    : allFormats;
  
  if (formats.length === 0) {
    console.error(`No formats found matching filter: ${formatFilter}`);
    console.log('Available formats:');
    allFormats.forEach(f => console.log(`  ${f.id} - ${f.name}`));
    process.exit(1);
  }
  
  console.log(`Will generate teams for ${formats.length} format(s):`);
  formats.forEach(f => console.log(`  ${f.id} - ${f.name}`));
  console.log();
  
  const outputDir = resolve(__dirname, '../../data/random-teams');
  
  for (const format of formats) {
    try {
      const teams = generateTeams(Dex, Teams, format, teamCount);
      saveTeams(teams, format.id, outputDir);
      console.log();
    } catch (error) {
      console.error(`Failed to generate teams for ${format.name}:`, error);
      continue;
    }
  }
  
  console.log('Team generation complete!');
}

// Handle CLI execution
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(error => {
    console.error('Error:', error);
    process.exit(1);
  });
}

export { generateTeams, cleanPokemonSet, getSupportedFormats };