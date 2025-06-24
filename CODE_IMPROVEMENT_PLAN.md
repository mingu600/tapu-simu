# Tapu Simu Code Improvement Plan

## Overview

This document outlines a concrete plan for improving code readability and cleanliness in the tapu-simu codebase. The improvements are organized by priority and include specific steps with file references.

## Phase 1: High Priority Improvements (2-3 days)

### 1.1 Function Decomposition in main.rs

**Target**: `/src/main.rs:55-281` (run_battle function - 226 lines)

**Steps**:
1. Extract `setup_battle_config()` from lines 55-89
   - Parameters: seed, config_file, verbose
   - Returns: `Result<BattleConfig, Box<dyn std::error::Error>>`

2. Extract `create_players()` from lines 90-120
   - Parameters: player_one, player_two
   - Returns: `Result<(Box<dyn Player>, Box<dyn Player>), Box<dyn std::error::Error>>`

3. Extract `execute_battles()` from lines 121-250
   - Parameters: format, players, runs, max_turns, team_index, verbose
   - Returns: `Result<BattleResults, Box<dyn std::error::Error>>`

4. Extract `print_battle_summary()` from lines 251-281
   - Parameters: results, runs, player_one, player_two
   - Returns: `()`

5. Refactor main `run_battle()` to orchestrate these functions

**Files to modify**: `/src/main.rs`

### 1.2 Create Constants Module

**Target**: Extract magic numbers throughout codebase

**Steps**:
1. Create `/src/constants.rs`
2. Add damage calculation constants:
   ```rust
   pub mod damage {
       pub const DAMAGE_ROLL_COUNT: usize = 16;
       pub const MIN_DAMAGE_PERCENT: u8 = 85;
       pub const MAX_DAMAGE_PERCENT: u8 = 100;
   }
   ```

3. Add battle mechanics constants:
   ```rust
   pub mod mechanics {
       pub const MAX_STAT_STAGE: i8 = 6;
       pub const MIN_STAT_STAGE: i8 = -6;
       pub const CRITICAL_HIT_RATIO: f64 = 1.5;
   }
   ```

4. Replace magic numbers in these files:
   - `/src/engine/combat/damage_calc.rs:147` (damage roll loop)
   - `/src/engine/combat/damage_calc.rs:148` (85 + roll calculation)
   - `/tests/basic_damage.rs` (expected damage values)

5. Add constants module to `/src/lib.rs`

**Files to create**: `/src/constants.rs`
**Files to modify**: `/src/lib.rs`, `/src/engine/combat/damage_calc.rs`, `/tests/basic_damage.rs`

### 1.3 Reorganize Move Effects Directory

**Target**: `/src/engine/combat/moves/` (25+ files)

**Steps**:
1. Create subdirectories:
   ```
   /src/engine/combat/moves/
   ├── damage/
   ├── status/
   ├── field/
   ├── special/
   └── mod.rs
   ```

2. Move files to appropriate subdirectories:
   - **damage/**: `drain.rs`, `recoil.rs`, `fixed_damage.rs`, `multi_hit.rs`, `variable_power.rs`, `self_damage.rs`, `self_destruct.rs`
   - **status/**: `status_effects.rs`, `stat_modifying.rs`, `healing.rs`, `item_interaction.rs`
   - **field/**: `weather.rs`, `weather_accuracy.rs`, `terrain_dependent.rs`, `hazards.rs`, `advanced_hazards.rs`, `hazard_removal.rs`, `screens.rs`, `field_manipulation.rs`
   - **special/**: `two_turn.rs`, `form_dependent.rs`, `complex.rs`, `counter.rs`, `priority.rs`, `protection.rs`, `substitute.rs`, `type_changing.rs`, `type_removal.rs`, `utility.rs`

3. Create `mod.rs` files in each subdirectory with proper re-exports

4. Update `/src/engine/combat/moves/mod.rs` to import from subdirectories

5. Update all import statements throughout codebase

**Files to create**: 4 new `mod.rs` files in subdirectories
**Files to move**: 25+ existing move effect files
**Files to modify**: `/src/engine/combat/moves/mod.rs`, all files that import move effects

## Phase 2: Medium Priority Improvements (3-4 days)

### 2.1 Standardize Error Handling

**Target**: Convert `main.rs` and other inconsistent error handling

**Steps**:
1. Create `BattleError` enum in `/src/types/errors.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum BattleError {
       #[error("Configuration error: {0}")]
       Config(#[from] ConfigError),
       #[error("Data error: {0}")]
       Data(#[from] DataError),
       #[error("Invalid format: {message}")]
       InvalidFormat { message: String },
   }
   ```

2. Update function signatures in `/src/main.rs`:
   - `run_battle()` → `Result<(), BattleError>`
   - `setup_battle_config()` → `Result<BattleConfig, BattleError>`
   - `create_players()` → `Result<(Box<dyn Player>, Box<dyn Player>), BattleError>`

3. Replace `Box<dyn std::error::Error>` usage throughout codebase

4. Add error conversion implementations where needed

**Files to create**: `/src/types/errors.rs` (if not exists)
**Files to modify**: `/src/main.rs`, `/src/types/mod.rs`, files using Box error type

### 2.2 Improve Documentation

**Target**: Add comprehensive documentation to key modules

**Steps**:
1. Add module-level documentation to:
   - `/src/engine/combat/mod.rs` - combat system overview
   - `/src/data/mod.rs` - data management architecture
   - `/src/core/mod.rs` - core battle concepts

2. Add detailed function documentation with examples to:
   - `/src/engine/combat/damage_calc.rs:calculate_damage()`
   - `/src/engine/combat/move_effects.rs:apply_move_effect()`
   - `/src/data/repository.rs:Repository` methods

3. Document complex mechanics in move effect files:
   - Add algorithm explanations for multi-hit moves
   - Document stat modification formulas
   - Explain type effectiveness calculations

4. Add usage examples to builder patterns in `/src/builders/`

**Files to modify**: All major module files, key function implementations

### 2.3 Fix Naming Inconsistencies

**Target**: Standardize naming patterns across codebase

**Steps**:
1. Create naming convention document section in this file
2. Rename `BattleInstructions` → `BattleInstructionSet` in:
   - `/src/engine/combat/move_effects.rs`
   - All files using this type

3. Standardize enum variant naming:
   - Review all enums for consistent PascalCase
   - Fix any snake_case enum variants

4. Standardize struct field naming:
   - Ensure all fields use snake_case consistently
   - Fix any camelCase field names

5. Update import statements and references

**Files to modify**: Multiple files using renamed types

## Phase 3: Lower Priority Improvements (4-5 days)

### 3.1 Refactor Repository Pattern

**Target**: `/src/data/repository.rs` (large Repository struct)

**Steps**:
1. Create separate repository files:
   - `/src/data/repositories/move_repository.rs`
   - `/src/data/repositories/pokemon_repository.rs`
   - `/src/data/repositories/item_repository.rs`
   - `/src/data/repositories/ability_repository.rs`

2. Extract functionality from main Repository:
   ```rust
   // MoveRepository
   pub struct MoveRepository {
       data: HashMap<MoveId, MoveData>,
       name_index: HashMap<String, MoveId>,
   }
   impl MoveRepository {
       pub fn find_by_name(&self, name: &str) -> Option<&MoveData>
       pub fn find_by_id(&self, id: &MoveId) -> Option<&MoveData>
   }
   ```

3. Create composite `GameDataRepository`:
   ```rust
   pub struct GameDataRepository {
       moves: MoveRepository,
       pokemon: PokemonRepository,
       items: ItemRepository,
       abilities: AbilityRepository,
   }
   ```

4. Update all callers to use new repository structure
5. Remove old Repository struct

**Files to create**: 4 new repository files, `/src/data/repositories/mod.rs`
**Files to modify**: `/src/data/repository.rs`, all files using Repository

### 3.2 Optimize Performance

**Target**: String processing and allocation optimizations

**Steps**:
1. Optimize `normalize_name()` in `/src/data/repository.rs:43`:
   ```rust
   pub fn normalize_name(name: &str) -> String {
       let mut result = String::with_capacity(name.len());
       for c in name.chars() {
           match c {
               ' ' | '-' | '\'' | '.' => continue,
               _ => result.push(c.to_ascii_lowercase()),
           }
       }
       result
   }
   ```

2. Add `String::with_capacity()` in functions that build strings:
   - Move name formatting functions
   - JSON serialization helpers
   - Error message construction

3. Replace `Vec::new()` with `Vec::with_capacity()` where size is predictable:
   - Battle instruction generation
   - Damage roll calculations

**Files to modify**: `/src/data/repository.rs`, other files with string processing

## Phase 4: Implementation Guidelines

### Naming Conventions

- **Types**: PascalCase (`BattleState`, `MoveData`)
- **Functions/Methods**: snake_case (`calculate_damage`, `apply_effect`)
- **Variables**: snake_case (`user_position`, `target_positions`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_DAMAGE_ROLLS`)
- **Modules**: snake_case (`move_effects`, `damage_calc`)

### Documentation Standards

- All public functions require doc comments with examples
- Module-level documentation explaining purpose and architecture
- Complex algorithms require inline comments explaining logic
- Error cases documented in function documentation