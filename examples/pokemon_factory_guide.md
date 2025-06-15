# Pokemon Factory Methods Guide

The Pokemon factory methods dramatically simplify Pokemon creation for testing and battle setup. Instead of manually constructing 50+ lines of data structures, you can create Pokemon in just one line.

## Overview

Three factory methods are available:

1. **`Pokemon::from_dex()`** - Create any Pokemon with real game data
2. **`Pokemon::test_pokemon()`** - Quick Pokemon for testing and development
3. **`Pokemon::competitive_pokemon()`** - Competitive builds with custom EVs

## Basic Usage

### 1. Creating Pokemon with `from_dex()`

```rust
use tapu_simu::pokemon::Pokemon;
use tapu_simu::dex::ShowdownDex;
use tapu_simu::types::{Nature, Gender};

let dex = ShowdownDex::new(Path::new("data/ps-extracted"))?;

// Create a Pikachu with Thunderbolt
let pikachu = Pokemon::from_dex(
    &dex,
    "pikachu",                    // species name
    50,                           // level
    &["thunderbolt", "quick-attack"], // moves (up to 4)
    None,                         // ability (uses first ability if None)
    None,                         // item (no item if None)
    Some(Nature::Modest),         // nature (+SpA, -Atk)
    Some(Gender::Male),           // gender
)?;
```

### 2. Test Pokemon (Ultra Easy)

```rust
// Perfect for unit tests and quick development
let test_pokemon = Pokemon::test_pokemon(&dex, Some(50))?;
// Creates a level 50 Pikachu with Thunderbolt, Quick Attack, Double Team, Substitute
// Modest nature, Male gender, perfect IVs, no EVs
```

### 3. Competitive Pokemon

```rust
use tapu_simu::types::{StatsTable, EVStatType};

// Create a competitive Garchomp
let garchomp = Pokemon::competitive_pokemon(
    &dex,
    "garchomp",                   // species
    50,                           // level
    &["earthquake", "dragon-claw", "stone-edge", "swords-dance"], // moves
    "rough-skin",                 // ability
    Some("choice-band"),          // item
    Nature::Jolly,                // nature (+Speed, -SpA)
    Some(StatsTable::competitive_evs(EVStatType::Attack, EVStatType::Speed)), // EVs
)?;
```

## Team Building

Creating teams is now incredibly easy:

```rust
// Quick test team
let team = vec![
    Pokemon::test_pokemon(&dex, Some(50))?,
    Pokemon::test_pokemon(&dex, Some(50))?,
    Pokemon::test_pokemon(&dex, Some(50))?,
];

// Mixed team with different Pokemon
let team = vec![
    Pokemon::from_dex(&dex, "pikachu", 50, &["thunderbolt"], None, None, None, None)?,
    Pokemon::from_dex(&dex, "charizard", 50, &["flamethrower"], None, None, None, None)?,
    Pokemon::from_dex(&dex, "blastoise", 50, &["surf"], None, None, None, None)?,
];
```

## Advanced Features

### EV Spreads

The `competitive_evs()` helper creates optimized EV spreads:

```rust
// 252 Attack, 252 Speed, 4 HP
let evs = StatsTable::competitive_evs(EVStatType::Attack, EVStatType::Speed);

// Use with competitive_pokemon()
let pokemon = Pokemon::competitive_pokemon(
    &dex, "garchomp", 50, &["earthquake"], "rough-skin", 
    None, Nature::Jolly, Some(evs)
)?;
```

### Perfect IVs

All factory methods use perfect IVs (31 in all stats) by default, which is ideal for competitive analysis and testing.

### Error Handling

Factory methods return `BattleResult<Pokemon>`, so use `?` for error propagation:

```rust
fn create_team(dex: &dyn Dex) -> BattleResult<Vec<Pokemon>> {
    Ok(vec![
        Pokemon::test_pokemon(dex, Some(50))?,
        Pokemon::from_dex(dex, "charizard", 50, &["flamethrower"], None, None, None, None)?,
    ])
}
```

## Benefits

### Code Reduction
- **Before**: ~50 lines per Pokemon (manual struct creation)
- **After**: 1 line per Pokemon (factory method)
- **Improvement**: 98% reduction in boilerplate code

### Testing Impact
- Tests can focus on battle logic, not Pokemon creation
- Easy to create test scenarios with different Pokemon
- No more brittle manual data structure creation

### Maintainability
- Works automatically when data structures change
- Uses real Pokemon Showdown data
- Less error-prone than manual construction

## Migration from Manual Construction

### Old Way (Don't do this!)
```rust
// 50+ lines of manual data creation
let species = SpeciesData { /* 10+ fields */ };
let moves = [
    MoveData { /* 15+ fields */ },
    MoveData { /* 15+ fields */ },
    // ...
];
let ability = AbilityData { /* 5+ fields */ };
let pokemon = Pokemon::new(species, level, moves, ability, /* many more args */);
```

### New Way (Do this!)
```rust
// 1 line with real game data
let pokemon = Pokemon::from_dex(&dex, "pikachu", 50, &["thunderbolt"], None, None, None, None)?;
```

## Data Requirements

The factory methods require Pokemon Showdown data to be extracted. Run:

```bash
npm run extract-data
```

This creates the `data/ps-extracted/` directory with:
- `pokedex.json` - Pokemon species data
- `moves.json` - Move data
- `abilities.json` - Ability data  
- `items.json` - Item data
- `typechart.json` - Type effectiveness data

## Examples

See the `examples/` directory for complete working examples:
- `simple_factory_demo.rs` - Overview of all factory methods
- `easy_pokemon_creation.rs` - Detailed usage examples

The factory methods make Pokemon creation enjoyable instead of tedious!