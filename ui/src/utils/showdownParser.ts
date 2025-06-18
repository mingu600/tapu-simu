import { Pokemon } from '../types'

interface ShowdownPokemon {
  species: string
  nickname?: string
  gender?: string
  item?: string
  ability: string
  level: number
  shiny?: boolean
  happiness?: number
  pokeball?: string
  nature?: string
  ivs: number[]
  evs: number[]
  moves: string[]
  teraType?: string
}

export function parseShowdownTeam(pasteText: string): ShowdownPokemon[] {
  const team: ShowdownPokemon[] = []
  const lines = pasteText.split('\n').map(line => line.trim()).filter(line => line)
  
  let currentPokemon: Partial<ShowdownPokemon> | null = null
  
  for (const line of lines) {
    // Skip empty lines
    if (!line) continue
    
    // New Pokemon (species line)
    if (!line.startsWith('-') && !line.startsWith('EVs:') && !line.startsWith('IVs:') && 
        !line.startsWith('Ability:') && !line.startsWith('Level:') && 
        !line.startsWith('Shiny:') && !line.startsWith('Happiness:') && 
        !line.startsWith('Pokeball:') && !line.startsWith('Nature:') && 
        !line.startsWith('Tera Type:') && !line.endsWith('Nature')) {
      
      // Save previous Pokemon if exists
      if (currentPokemon && currentPokemon.species) {
        team.push(finalizeShowdownPokemon(currentPokemon))
      }
      
      // Parse species line: "Nickname (Species) (Gender) @ Item"
      currentPokemon = parseSpeciesLine(line)
    }
    // Parse move
    else if (line.startsWith('-') || line.startsWith('•')) {
      if (currentPokemon) {
        const move = line.replace(/^[-•]\s*/, '').trim()
        if (!currentPokemon.moves) currentPokemon.moves = []
        if (currentPokemon.moves.length < 4) {
          currentPokemon.moves.push(move)
        }
      }
    }
    // Parse other attributes
    else {
      if (currentPokemon) {
        parseAttributeLine(line, currentPokemon)
      }
    }
  }
  
  // Don't forget the last Pokemon
  if (currentPokemon && currentPokemon.species) {
    team.push(finalizeShowdownPokemon(currentPokemon))
  }
  
  return team
}

function parseSpeciesLine(line: string): Partial<ShowdownPokemon> {
  const pokemon: Partial<ShowdownPokemon> = {}
  
  // Handle item: "@ Item"
  let workingLine = line
  const itemMatch = workingLine.match(/@\s*(.+)$/)
  if (itemMatch) {
    pokemon.item = itemMatch[1].trim()
    workingLine = workingLine.replace(/@\s*.+$/, '').trim()
  }
  
  // Handle gender: "(M)" or "(F)"
  const genderMatch = workingLine.match(/\s*\((M|F)\)\s*$/)
  if (genderMatch) {
    pokemon.gender = genderMatch[1] === 'M' ? 'Male' : 'Female'
    workingLine = workingLine.replace(/\s*\((M|F)\)\s*$/, '').trim()
  }
  
  // Handle species vs nickname: "Nickname (Species)" or just "Species"
  const nicknameMatch = workingLine.match(/^(.+?)\s*\((.+?)\)$/)
  if (nicknameMatch) {
    pokemon.nickname = nicknameMatch[1].trim()
    pokemon.species = nicknameMatch[2].trim()
  } else {
    pokemon.species = workingLine.trim()
  }
  
  // Set defaults
  pokemon.level = 50
  pokemon.moves = []
  pokemon.ivs = [31, 31, 31, 31, 31, 31] // HP, Att, Def, SpA, SpD, Spe
  pokemon.evs = [0, 0, 0, 0, 0, 0]
  
  return pokemon
}

function parseAttributeLine(line: string, pokemon: Partial<ShowdownPokemon>) {
  if (line.startsWith('Ability:')) {
    pokemon.ability = line.replace('Ability:', '').trim()
  }
  else if (line.startsWith('Level:')) {
    pokemon.level = parseInt(line.replace('Level:', '').trim()) || 50
  }
  else if (line.startsWith('Shiny:')) {
    pokemon.shiny = line.replace('Shiny:', '').trim().toLowerCase() === 'yes'
  }
  else if (line.startsWith('Happiness:')) {
    pokemon.happiness = parseInt(line.replace('Happiness:', '').trim()) || 255
  }
  else if (line.startsWith('Pokeball:')) {
    pokemon.pokeball = line.replace('Pokeball:', '').trim()
  }
  else if (line.includes('Nature')) {
    // Handle "Timid Nature" or "Nature: Timid" formats
    let nature = line.replace(/Nature:?/i, '').trim()
    if (nature) {
      pokemon.nature = nature
    }
  }
  else if (line.startsWith('Tera Type:')) {
    pokemon.teraType = line.replace('Tera Type:', '').trim()
  }
  else if (line.startsWith('EVs:')) {
    const evText = line.replace('EVs:', '').trim()
    pokemon.evs = parseEVsIVs(evText)
  }
  else if (line.startsWith('IVs:')) {
    const ivText = line.replace('IVs:', '').trim()
    pokemon.ivs = parseEVsIVs(ivText, true)
  }
}

function parseEVsIVs(text: string, isIVs = false): number[] {
  const stats = isIVs ? [31, 31, 31, 31, 31, 31] : [0, 0, 0, 0, 0, 0]
  const statMap: Record<string, number> = {
    'HP': 0, 'Atk': 1, 'Def': 2, 'SpA': 3, 'SpD': 4, 'Spe': 5,
    // Alternative spellings
    'Attack': 1, 'Defense': 2, 'Special Attack': 3, 'Special Defense': 4, 'Speed': 5
  }
  
  const parts = text.split('/')
  for (const part of parts) {
    const trimmed = part.trim()
    const match = trimmed.match(/(\d+)\s+(.+)/)
    if (match) {
      const value = parseInt(match[1])
      const statName = match[2].trim()
      if (statMap.hasOwnProperty(statName)) {
        stats[statMap[statName]] = value
      }
    }
  }
  
  return stats
}

function finalizeShowdownPokemon(pokemon: Partial<ShowdownPokemon>): ShowdownPokemon {
  return {
    species: pokemon.species || '',
    nickname: pokemon.nickname,
    gender: pokemon.gender || 'N',
    item: pokemon.item,
    ability: pokemon.ability || '',
    level: pokemon.level || 50,
    shiny: pokemon.shiny || false,
    happiness: pokemon.happiness || 255,
    pokeball: pokemon.pokeball,
    nature: pokemon.nature,
    ivs: pokemon.ivs || [31, 31, 31, 31, 31, 31],
    evs: pokemon.evs || [0, 0, 0, 0, 0, 0],
    moves: pokemon.moves || [],
    teraType: pokemon.teraType
  }
}

export async function convertShowdownToTapuPokemon(showdownPokemon: ShowdownPokemon[]): Promise<Pokemon[]> {
  const convertedTeam: Pokemon[] = []
  
  for (const sp of showdownPokemon) {
    try {
      // Use the new create-custom endpoint which supports full customization
      const requestData = {
        species: sp.species,
        level: sp.level,
        ivs: sp.ivs.map(v => Math.max(0, Math.min(31, v))), // Ensure IVs are 0-31
        evs: sp.evs.map(v => Math.max(0, Math.min(252, v))), // Ensure EVs are 0-252
        nature: sp.nature || 'Hardy',
        ability: sp.ability || null,
        item: sp.item || null,
        moves: sp.moves && sp.moves.length > 0 ? sp.moves : null,
        tera_type: sp.teraType || null
      }
      
      console.log(`Creating Pokemon with full customization:`, requestData)
      
      const response = await fetch('/api/pokemon/create-custom', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData)
      })
      
      if (response.ok) {
        let pokemon = await response.json()
        console.log(`Successfully created Pokemon:`, pokemon)
        console.log(`Original Showdown data:`, sp)
        
        // The recalculate endpoint gives us a Pokemon with proper stats
        // but we may need to customize other fields that aren't supported by the API
        // For now, we'll take what we get since moves, abilities, and items 
        // are auto-populated by the backend
        
        // Log what was preserved vs what was auto-assigned
        if (sp.moves && sp.moves.length > 0) {
          console.log(`Note: Requested moves [${sp.moves.join(', ')}] but got auto-assigned moves [${pokemon.moves.map(m => m.name).join(', ')}]`)
        }
        if (sp.ability && sp.ability !== pokemon.ability) {
          console.log(`Note: Requested ability "${sp.ability}" but got auto-assigned ability "${pokemon.ability}"`)
        }
        if (sp.item && sp.item !== pokemon.item) {
          console.log(`Note: Requested item "${sp.item}" but got auto-assigned item "${pokemon.item || 'none'}"`)
        }
        
        convertedTeam.push(pokemon)
      } else {
        const errorText = await response.text()
        console.error(`Failed to create Pokemon ${sp.species}:`, response.status, errorText)
        console.error(`Request data was:`, requestData)
        
        // Create a basic fallback Pokemon using the basic endpoint
        console.log(`Attempting fallback creation for ${sp.species}`)
        const fallbackResponse = await fetch('/api/pokemon/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ species: sp.species, level: sp.level })
        })
        
        if (fallbackResponse.ok) {
          const pokemon = await fallbackResponse.json()
          console.log(`Fallback creation successful:`, pokemon)
          convertedTeam.push(pokemon)
        } else {
          const fallbackError = await fallbackResponse.text()
          console.error(`Fallback creation also failed:`, fallbackResponse.status, fallbackError)
        }
      }
    } catch (error) {
      console.error(`Error converting Pokemon ${sp.species}:`, error)
      // Try basic creation as last resort
      try {
        const fallbackResponse = await fetch('/api/pokemon/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ species: sp.species, level: sp.level })
        })
        
        if (fallbackResponse.ok) {
          const pokemon = await fallbackResponse.json()
          convertedTeam.push(pokemon)
        }
      } catch (fallbackError) {
        console.error(`Final fallback failed for ${sp.species}:`, fallbackError)
      }
    }
  }
  
  return convertedTeam
}