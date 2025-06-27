# Tapu Simu Identifier Cleanup Analysis

## Overview

This document provides a comprehensive analysis of improper identifier handling throughout the Tapu Simu codebase. The codebase has well-defined typed identifiers (`SpeciesId`, `MoveId`, `ItemId`, `AbilityId`, `TypeId`) in `src/types/identifiers.rs`, but much of the code still uses raw string manipulation and ad-hoc normalization instead of leveraging these types.

## Current Typed Identifier System

The codebase defines the following typed identifiers in `src/types/identifiers.rs`:
- `SpeciesId` - Pokemon species names
- `MoveId` - Move names
- `ItemId` - Item names  
- `AbilityId` - Ability names
- `TypeId` - Type names

Each type:
- Automatically normalizes input via `normalize_name()` in constructor
- Validates normalization with debug assertions
- Provides `as_str()` for string access
- Implements necessary traits for serialization and display

## Critical Issues Found

### 1. Move Registry System (`src/engine/combat/moves/registry.rs`)

**Problem**: The entire move registration system uses raw strings instead of `MoveId`.

**Lines 128-136**: Registry HashMaps use `String` keys:
```rust
standard_moves: std::collections::HashMap<String, MoveEffectFn>,
extended_moves: std::collections::HashMap<String, ExtendedMoveEffectFn>,
variable_power_moves: std::collections::HashMap<String, VariablePowerMoveEffectFn>,
context_aware_moves: std::collections::HashMap<String, ContextAwareMoveEffectFn>,
repository_aware_moves: std::collections::HashMap<String, RepositoryAwareMoveEffectFn>,
```

**Lines 157-325**: Hardcoded move names as string literals:
```rust
self.register_standard("thunderwave", apply_thunder_wave);
self.register_standard("sleeppowder", apply_sleep_powder);
self.register_standard("toxic", apply_toxic);
```

**Recommendation**: Change HashMap keys to `MoveId` and use `MoveId::new()` for registration.

### 2. Item System (`src/engine/mechanics/items/`)

**Problem**: All item effect functions use raw strings and manual normalization.

**Files affected**:
- `utility_items.rs` lines 21, 30, 47, 53, 63, 68, 140, 143, 152
- `type_boosting_items.rs` lines 23, 85, 101, 110, 119, 137, 142
- `stat_boosting_items.rs`, `berry_items.rs`, `species_items.rs`, `status_items.rs`

**Example from `utility_items.rs:30`**:
```rust
let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
match normalized_name.as_str() {
    "leftovers" => Some(ItemModifier::default()),
    "protectivepads" => Some(protective_pads_effect()),
}
```

**Example from `type_boosting_items.rs:23`**:
```rust
let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
```

**Recommendation**: Functions should accept `&ItemId` instead of `&str`, eliminating need for manual normalization.

### 3. Ability System (`src/engine/mechanics/abilities.rs`)

**Problem**: Extensive pattern matching on raw ability strings.

**Lines 112-176**: Raw string matching:
```rust
match ability.as_str() {
    "levitate" => apply_levitate(context),
    "flashfire" => apply_flash_fire(context),
    "waterabsorb" => apply_water_absorb(context),
}
```

**Lines 184, 194, 204**: Type checking using string comparisons:
```rust
if normalize_name(move_type.as_str()) == normalize_name("Ground") {
    return AbilityEffectResult::immunity();
}
```

**Recommendation**: Use `AbilityId` and `TypeId` for matching and comparisons.

### 4. Damage Context (`src/engine/combat/damage_context.rs`)

**Problem**: Context structs store raw strings instead of typed identifiers.

**Lines 66, 80, 139**:
```rust
pub name: String,           // Should be MoveId
pub move_type: String,      // Should be TypeId
pub item_name: Option<String>, // Should be Option<ItemId>
```

**Lines 475, 482**: Hardcoded defaults:
```rust
name: "tackle".to_string(),     // Should be MoveId::new("tackle")
move_type: "normal".to_string(), // Should be TypeId::new("normal")
```

**Recommendation**: Update all context structs to use typed identifiers.

### 5. Type Effectiveness System (`src/engine/combat/type_effectiveness.rs`)

**Problem**: String-based type matching instead of using `TypeId`.

**Lines 38-59**: Manual string conversion:
```rust
pub fn from_str(type_str: &str) -> Option<Self> {
    match type_str.to_lowercase().as_str() {
        "normal" => Some(Self::Normal),
        "fire" => Some(Self::Fire),
    }
}
```

**Line 107**: Raw string keys in special cases:
```rust
special_cases: HashMap<(String, PokemonType), f32>,
```

**Recommendation**: Use `TypeId` consistently and update HashMap keys.

### 6. Battle Format System (`src/core/battle_format.rs`)

**Problem**: Manual normalization in ban list checking.

**Lines 115-118**: Manual lowercasing:
```rust
species: species.into_iter().map(|s| s.to_lowercase()).collect(),
moves: moves.into_iter().map(|s| s.to_lowercase()).collect(),
items: items.into_iter().map(|s| s.to_lowercase()).collect(),
abilities: abilities.into_iter().map(|s| s.to_lowercase()).collect(),
```

**Lines 341, 346, 351**: String comparison with manual normalization:
```rust
self.ban_list.species.contains(&species.to_lowercase())
self.ban_list.moves.contains(&move_name.to_lowercase())
self.ban_list.items.contains(&item.to_lowercase())
```

**Recommendation**: Use typed identifiers for ban lists and comparisons.

### 7. Data Generation System (`src/data/generation_loader.rs`)

**Problem**: All HashMap keys use raw strings instead of typed identifiers.

**Lines 26-31**: Raw string keys:
```rust
generation_move_data: HashMap<String, GenerationMoveData>,
generation_item_data: HashMap<String, GenerationItemData>,
generation_pokemon_data: HashMap<String, GenerationPokemonData>,
move_changes: HashMap<String, MoveChangeHistory>,
item_changes: HashMap<String, ItemChangeHistory>,
pokemon_changes: HashMap<String, PokemonChangeHistory>,
```

**Recommendation**: Update HashMap keys to use typed identifiers.

### 8. Random Team Loader (`src/data/random_team_loader.rs`)

**Problem**: Ad-hoc string normalization and raw string usage.

**Line 229**: Manual normalization:
```rust
format.name.to_lowercase().replace(" ", "").replace("-", "")
```

**Line 315**: Direct string usage:
```rust
let name_lower = move_name.to_lowercase();
```

**Recommendation**: Use typed identifiers and eliminate manual normalization.

### 9. Showdown Types (`src/data/showdown_types.rs`)

**Problem**: Stat boost HashMaps use raw strings instead of stat identifiers.

**Lines 408, 413, 527**: Raw string keys for stats:
```rust
pub boosts: Option<std::collections::HashMap<String, i8>>,
```

**Recommendation**: Consider creating a `StatId` type for consistent stat handling.

## Normalization Issues

### Current Normalization Patterns

1. **Proper**: Using `normalize_name()` function (21 files)
2. **Improper**: Manual `to_lowercase()` (35+ instances)
3. **Mixed**: Manual string replacement patterns

### Files with Excessive `to_lowercase()` Usage

1. `src/core/battle_format.rs` - 8 instances
2. `src/engine/mechanics/items/type_boosting_items.rs` - 7 instances
3. `src/generation.rs` - Type effectiveness overrides
4. `src/main.rs` - Player type parsing
5. `src/io.rs` - Format string parsing

## Recommended Cleanup Strategy

### Phase 1: Core Systems (High Impact)
1. **Move Registry**: Convert to `MoveId` keys
2. **Item System**: Convert all item functions to use `ItemId`
3. **Ability System**: Convert to `AbilityId` matching
4. **Damage Context**: Update all context structs

### Phase 2: Data Layer (Medium Impact)
1. **Generation Loader**: Convert HashMap keys to typed identifiers
2. **Battle Format**: Use typed identifiers for ban lists
3. **Type Effectiveness**: Full `TypeId` integration

### Phase 3: Utilities and Edges (Low Impact)
1. **Random Team Loader**: Eliminate manual normalization
2. **Showdown Types**: Consider `StatId` for consistency
3. **Test Files**: Update to use typed identifiers

### Phase 4: Add Missing Types
1. **StatId**: For stat boost handling
2. **NatureId**: If nature handling becomes complex
3. **StatusId**: For status condition consistency

## Implementation Guidelines

### For HashMap Keys
```rust
// Before
HashMap<String, T>

// After  
HashMap<MoveId, T>  // or ItemId, AbilityId, etc.
```

### For Function Parameters
```rust
// Before
fn get_effect(name: &str) -> Effect

// After
fn get_effect(id: &MoveId) -> Effect
```

### For Pattern Matching
```rust
// Before
match name.to_lowercase().as_str() {
    "thunderbolt" => ...,
}

// After
match id {
    id if id == &MoveId::new("thunderbolt") => ...,
}
```

### For Comparisons
```rust
// Before
if name.to_lowercase() == "fire"

// After
if type_id == &TypeId::new("fire")
```

## Benefits of Cleanup

1. **Type Safety**: Compile-time catching of identifier typos
2. **Performance**: Eliminate redundant normalization calls
3. **Consistency**: Single source of truth for normalization
4. **Maintainability**: Clear distinction between raw strings and identifiers
5. **API Clarity**: Function signatures clearly indicate expected identifier types

## Estimated Impact

- **Files to modify**: ~45 files
- **Functions to update**: ~120 functions
- **HashMap declarations**: ~15 declarations
- **Pattern matches**: ~50 match statements

This cleanup would significantly improve type safety and eliminate most ad-hoc string manipulation throughout the codebase.