#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Use the existing randbats data and generate teams from it
const RANDBATS_DATA_URL = 'https://data.pkmn.cc/randbats/';

async function fetchRandbatsData(format) {
  try {
    const url = `${RANDBATS_DATA_URL}${format}.json`;
    console.log(`Fetching data from ${url}...`);
    
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error(`Failed to fetch randbats data for ${format}:`, error);
    return null;
  }
}

// Generate a random Pokemon set based on randbats data
function generateRandomPokemon(pokemonName, pokemonData) {
  const pokemon = {
    name: pokemonName,
    species: pokemonName,
    level: pokemonData.level,
    gender: Math.random() > 0.5 ? 'M' : 'F',
    shiny: Math.random() < 0.01, // 1% shiny rate
  };

  // Handle abilities
  if (pokemonData.abilities && pokemonData.abilities.length > 0) {
    pokemon.ability = pokemonData.abilities[Math.floor(Math.random() * pokemonData.abilities.length)];
  }

  // Handle items
  if (pokemonData.items && pokemonData.items.length > 0) {
    pokemon.item = pokemonData.items[Math.floor(Math.random() * pokemonData.items.length)];
  }

  // Handle EVs and IVs
  pokemon.evs = pokemonData.evs || { hp: 85, atk: 85, def: 85, spa: 85, spd: 85, spe: 85 };
  pokemon.ivs = pokemonData.ivs || { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 };

  // Handle roles (Gen 9 style)
  if (pokemonData.roles) {
    const roleNames = Object.keys(pokemonData.roles);
    const roleName = roleNames[Math.floor(Math.random() * roleNames.length)];
    const role = pokemonData.roles[roleName];
    
    pokemon.role = roleName;
    
    // Override with role-specific data
    if (role.abilities && role.abilities.length > 0) {
      pokemon.ability = role.abilities[Math.floor(Math.random() * role.abilities.length)];
    }
    if (role.items && role.items.length > 0) {
      pokemon.item = role.items[Math.floor(Math.random() * role.items.length)];
    }
    if (role.teraTypes && role.teraTypes.length > 0) {
      pokemon.teraType = role.teraTypes[Math.floor(Math.random() * role.teraTypes.length)];
    }
    if (role.moves && role.moves.length >= 4) {
      // Pick 4 random moves
      const shuffled = [...role.moves].sort(() => 0.5 - Math.random());
      pokemon.moves = shuffled.slice(0, 4);
    } else if (role.moves) {
      pokemon.moves = [...role.moves];
    }
    if (role.evs) {
      pokemon.evs = { ...pokemon.evs, ...role.evs };
    }
    if (role.ivs) {
      pokemon.ivs = { ...pokemon.ivs, ...role.ivs };
    }
  } else {
    // Non-role based (older generations)
    if (pokemonData.moves && pokemonData.moves.length >= 4) {
      const shuffled = [...pokemonData.moves].sort(() => 0.5 - Math.random());
      pokemon.moves = shuffled.slice(0, 4);
    } else if (pokemonData.moves) {
      pokemon.moves = [...pokemonData.moves];
    }
  }

  // Generate random nature (simplified)
  const natures = [
    'Hardy', 'Lonely', 'Brave', 'Adamant', 'Naughty',
    'Bold', 'Docile', 'Relaxed', 'Impish', 'Lax',
    'Timid', 'Hasty', 'Serious', 'Jolly', 'Naive',
    'Modest', 'Mild', 'Quiet', 'Bashful', 'Rash',
    'Calm', 'Gentle', 'Sassy', 'Careful', 'Quirky'
  ];
  pokemon.nature = natures[Math.floor(Math.random() * natures.length)];

  return pokemon;
}

// Generate a full team
function generateTeam(formatData) {
  const pokemonNames = Object.keys(formatData);
  if (pokemonNames.length < 6) {
    throw new Error(`Not enough Pokemon in format data: ${pokemonNames.length}`);
  }

  const team = [];
  const usedPokemon = new Set();

  while (team.length < 6) {
    const pokemonName = pokemonNames[Math.floor(Math.random() * pokemonNames.length)];
    
    if (!usedPokemon.has(pokemonName)) {
      usedPokemon.add(pokemonName);
      const pokemon = generateRandomPokemon(pokemonName, formatData[pokemonName]);
      team.push(pokemon);
    }
  }

  return team;
}

// Generate multiple teams
async function generateTeams(format, count) {
  console.log(`Generating ${count} teams for ${format}...`);
  
  const formatData = await fetchRandbatsData(format);
  if (!formatData) {
    throw new Error(`Failed to fetch data for format: ${format}`);
  }

  const teams = [];
  for (let i = 0; i < count; i++) {
    try {
      const team = generateTeam(formatData);
      teams.push(team);
      
      if ((i + 1) % 100 === 0) {
        console.log(`  Generated ${i + 1}/${count} teams`);
      }
    } catch (error) {
      console.error(`Failed to generate team ${i + 1}:`, error);
      i--; // Retry this team
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
async function main() {
  const args = process.argv.slice(2);
  const format = args[0] || 'gen9randombattle';
  const teamCount = parseInt(args[1]) || 1000;
  
  const supportedFormats = [
    'gen9randombattle', 'gen9randomdoublesbattle', 'gen9babyrandombattle',
    'gen8randombattle', 'gen8randomdoublesbattle', 'gen8bdsprandombattle',
    'gen7randombattle', 'gen7letsgorandombattle',
    'gen6randombattle', 'gen5randombattle', 'gen4randombattle',
    'gen3randombattle', 'gen2randombattle', 'gen1randombattle'
  ];

  if (!supportedFormats.includes(format)) {
    console.error(`Unsupported format: ${format}`);
    console.log('Supported formats:');
    supportedFormats.forEach(f => console.log(`  ${f}`));
    process.exit(1);
  }

  try {
    const teams = await generateTeams(format, teamCount);
    const outputDir = path.resolve(__dirname, '../../data/random-teams');
    saveTeams(teams, format, outputDir);
    console.log('Team generation complete!');
  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

// Handle CLI execution
if (require.main === module) {
  main();
}

module.exports = { generateTeams, generateTeam, generateRandomPokemon };