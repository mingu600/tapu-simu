# Pokemon Showdown Integration - Working Demo

## Overview

This document demonstrates how the Pokemon Showdown integration provides a cleaner, more comprehensive approach to move data compared to rustemon/PokeAPI.

## Current Issues with Rustemon/PokeAPI

### 1. Limited Move Effect Data
```rust
// Current rustemon move data - very basic
EngineMoveData {
    id: 85,
    name: "Thunderbolt",
    base_power: Some(90),
    accuracy: Some(100),
    pp: 15,
    move_type: "Electric",
    category: MoveCategory::Special,
    priority: 0,
    target: MoveTarget::SelectedPokemon,
    effect_chance: Some(10),
    effect_description: "May paralyze opponent.", // Very basic
    flags: vec![], // Empty - no detailed flags
}
```

### 2. Missing Critical Information
- No detailed move flags (contact, sound, etc.)
- No secondary effect details
- No Z-move/Max move data
- No recoil/drain information
- No multi-hit data

## Pokemon Showdown Data - Complete Information

### 1. Comprehensive Move Data
```json
// PS extracted move data
{
  "thunderbolt": {
    "id": "thunderbolt",
    "num": 85,
    "name": "Thunderbolt",
    "basePower": 90,
    "accuracy": 100,
    "pp": 15,
    "maxPP": 24,
    "type": "Electric", 
    "category": "Special",
    "priority": 0,
    "target": "normal",
    "flags": {
      "protect": true,
      "mirror": true,
      "metronome": true
    },
    "secondary": {
      "chance": 10,
      "status": "par"
    },
    "desc": "Has a 10% chance to paralyze the target.",
    "shortDesc": "10% chance to paralyze the target."
  }
}
```

### 2. Rich Move Effects
```json
// PS Earthquake - shows spread move data
{
  "earthquake": {
    "id": "earthquake",
    "num": 89,
    "name": "Earthquake", 
    "basePower": 100,
    "accuracy": 100,
    "pp": 10,
    "target": "allAdjacent",
    "flags": {
      "protect": true,
      "mirror": true,
      "nonsky": true,
      "metronome": true
    },
    "desc": "Damages all adjacent Pokemon. Double damage to Pokemon using Dig."
  }
}
```

### 3. Complex Move Mechanics
```json
// PS U-turn - shows self-switch
{
  "uturn": {
    "id": "uturn",
    "name": "U-turn",
    "basePower": 70,
    "target": "normal",
    "flags": {
      "contact": true,
      "protect": true,
      "mirror": true
    },
    "selfSwitch": true,
    "desc": "User switches out after damaging the target."
  }
}
```

## Integration Benefits Demonstrated

### 1. Direct Target Usage
```rust
// Before: Complex mapping needed
rustemon_target = "selected-pokemon" 
engine_target = MoveTarget::SelectedPokemon
ps_target = PSMoveTarget::Normal

// After: Direct PS usage
ps_target = "normal"
let targeting_engine = PSAutoTargetingEngine::new(format);
let targets = targeting_engine.resolve_targets(PSMoveTarget::Normal, user_pos, state);
```

### 2. Complete Move Information
```rust
// Before: Missing data
let move_has_contact = false; // Unknown from PokeAPI

// After: Complete flag data
let ps_move = ps_repo.get_move("thunderpunch").unwrap();
let move_has_contact = ps_move.flags.get("contact").unwrap_or(&false);
let move_triggers_iron_fist = ps_move.flags.get("punch").unwrap_or(&false);
```

### 3. Better Battle Mechanics
```rust
// Before: Hardcoded spread damage
if move_targets_multiple {
    damage = (damage as f32 * 0.75) as u32; // Where did 0.75 come from?
}

// After: Based on PS target system  
if ps_target.is_spread_move() {
    damage = apply_spread_damage_reduction(damage, ps_target);
}
```

## Migration Path

### Phase 1: Parallel Systems ✅ (Current)
- PS data structures defined
- PS targeting engine implemented
- Conversion between old/new systems

### Phase 2: PS Data Loading
```bash
# Extract PS data
cd tools/ps-data-extractor
npm install
npm run extract

# Use in Rust
let ps_repo = PSDataRepository::load_from_directory("data/ps-extracted")?;
let thunderbolt = ps_repo.get_move_by_name("Thunderbolt").unwrap();
```

### Phase 3: Replace Rustemon
```rust
// Remove rustemon dependency from Cargo.toml
// Replace MoveDataService with PSDataRepository
// Update instruction generation to use PS data
```

## Real-World Example: Earthquake in Doubles

### Current Implementation
```rust
// Very basic - doesn't capture the nuance
if move_name == "earthquake" {
    // Hardcoded logic
    targets = get_all_adjacent_positions(user_pos, state);
    for target in targets {
        if target.side == user_pos.side {
            // Earthquake hits ally - but what about Flying types?
        }
    }
}
```

### PS-Powered Implementation  
```rust
let earthquake = ps_repo.get_move("earthquake").unwrap();
let targeting_engine = PSAutoTargetingEngine::new(format);

// PS knows it's "allAdjacent" target
let targets = targeting_engine.resolve_targets(
    PSMoveTarget::AllAdjacent, 
    user_pos, 
    state
);

// PS flags tell us about interactions
if earthquake.flags.get("nonsky").unwrap_or(&false) {
    // Doesn't hit Pokemon with Air Balloon, Flying types using Roost, etc.
}
```

## Conclusion

Pokemon Showdown integration provides:

1. **Comprehensive Data** - All move effects, flags, and interactions
2. **Battle-Tested Accuracy** - Used by millions of players
3. **Cleaner Code** - Direct usage vs complex mapping
4. **Future-Proof** - Easy updates as new moves are added
5. **Community Standard** - Same terminology as competitive scene

The integration work establishes a foundation for replacing rustemon entirely, giving tapu-simu access to the most complete and accurate Pokemon move database available.