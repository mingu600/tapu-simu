#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Use @pkmn/sim instead of raw pokemon-showdown
let Dex, Teams;
try {
  const sim = require('@pkmn/sim');
  Dex = sim.Dex;
  Teams = sim.Teams;
} catch (error) {
  console.error('Failed to import @pkmn/sim:', error);
  console.error('Make sure to run: npm install');
  process.exit(1);
}

const SEED = [1, 2, 3, 4];
const DEFAULT_TEAM_COUNT = 1000;

// Get supported random battle formats
function getSupportedFormats() {
  const formats = [];
  for (const format of Dex.formats.all()) {
    if (!format.team || !['singles', 'doubles'].includes(format.gameType)) continue;
    if (!format.mod.startsWith('gen') || format.id.includes('random') === false) continue;
    // Focus on main random battle formats
    if (format.id.includes('factory') || format.id.includes('unrated') || 
        format.id.includes('cap') || format.id.includes('monotype')) continue;
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
function generateTeams(format, teamCount) {
  console.log(`Generating ${teamCount} teams for ${format.name}...`);
  
  const generator = Teams.getGenerator(format, SEED);
  const teams = [];
  
  for (let i = 0; i < teamCount; i++) {
    const team = generator.getTeam();
    teams.push(team.map(cleanPokemonSet));
    
    if ((i + 1) % 500 === 0) {
      console.log(`  Generated ${i + 1}/${teamCount} teams`);
    }
  }
  
  return teams;
}

// Save teams to file
function saveTeams(teams, formatId, outputDir) {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  
  const filename = `${outputDir}/${formatId}-teams.json`;
  const jsonString = JSON.stringify(teams, null, 2);
  
  fs.writeFileSync(filename, jsonString);
  console.log(`Saved ${teams.length} teams to ${filename}`);
  console.log(`File size: ${(jsonString.length / 1024 / 1024).toFixed(2)} MB`);
}

// Main function
function main() {
  const args = process.argv.slice(2);
  const formatFilter = args[0]; // Optional format filter (e.g., "gen9" or "gen9randombattle")
  const teamCount = parseInt(args[1]) || DEFAULT_TEAM_COUNT;
  
  console.log('Getting supported formats...');
  const allFormats = getSupportedFormats();
  
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
  
  const outputDir = path.resolve(__dirname, '../../data/random-teams');
  
  for (const format of formats) {
    try {
      const teams = generateTeams(format, teamCount);
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
if (require.main === module) {
  main();
}

module.exports = { generateTeams, cleanPokemonSet, getSupportedFormats };