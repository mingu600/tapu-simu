# Move Effects Refactoring Design Document

## Executive Summary

The current move effects system in Tapu Simu suffers from extensive code duplication and lacks proper abstraction layers. This document outlines a comprehensive refactoring plan to centralize battle mechanics, reduce redundancy, and create a more maintainable architecture while preserving the existing categorical organization.

## Current State Analysis

### Major Issues Identified

1. **Massive Code Duplication**: Over 80% of move effect logic is duplicated across different move categories
2. **Lack of Centralized Battle Mechanics**: Core systems like damage calculation, status application, and contact effects are reimplemented in each move
3. **Inconsistent Error Handling**: Different approaches to missing targets, failed moves, and immunity checks
4. **Poor Separation of Concerns**: Move-specific logic mixed with generic battle mechanics
5. **Maintenance Nightmare**: Fixing bugs or adding features requires changes in dozens of files

### Specific Redundancy Patterns

#### Critical Hit Logic (Found in 8+ files)
```rust
// Repeated in special_combat.rs, moves/mod.rs, variable_power.rs, etc.
let crit_chance = critical_hit_probability(user, target, move_data, generation.generation);
let normal_hit_chance = accuracy * (1.0 - crit_chance);
let crit_hit_chance = accuracy * crit_chance;
```

#### Status Application (Found in 15+ files)
```rust
// Nearly identical in status_effects.rs, secondary_effects.rs, etc.
BattleInstruction::Status(StatusInstruction::Apply {
    target: target_position,
    status: PokemonStatus::Burn, // Only this changes
    duration: None,
    previous_status: Some(target.status),
    previous_duration: target.status_duration,
})
```

#### Multi-Hit State Management (Found in 6+ files)
```rust
// Duplicated across all multi-hit move implementations
let mut current_state = state.clone();
for hit_number in 1..=hit_count {
    // Calculate damage using current_state
    // Apply instructions to current_state for next hit
    current_state.apply_instructions(&damage_instructions);
}
```

## Proposed Architecture

### Core Principles

1. **Single Responsibility**: Each move implementation should only handle what's unique to that move
2. **Centralized Battle Mechanics**: Core systems should be handled by the battle engine automatically
3. **Composable Effects**: Build complex moves from simple, reusable components
4. **Consistent Interfaces**: Standardized function signatures and return types
5. **Testable Components**: Battle mechanics should be testable independently of move implementations

### Architecture Layers

```
┌─────────────────────────────────────────┐
│             Move Implementations        │ ← Only move-specific logic
├─────────────────────────────────────────┤
│           Effect Composers              │ ← Combine basic effects
├─────────────────────────────────────────┤
│         Core Battle Systems             │ ← Centralized mechanics
├─────────────────────────────────────────┤
│        Instruction Generation           │ ← Low-level instruction building
└─────────────────────────────────────────┘
```

## Refactoring Plan

### Phase 1: Create Core Battle Systems (2-3 weeks)

#### 1.1 Damage System Refactor
**Files to create:**
- `src/engine/combat/core/damage_system.rs`
- `src/engine/combat/core/critical_hits.rs`

**Key Components:**
```rust
pub struct DamageContext {
    move_data: MoveData,
    user_position: BattlePosition,
    target_position: BattlePosition,
    power_modifier: Option<f32>,
    force_critical: bool,
    generation: GenerationMechanics,
}

pub fn calculate_damage_with_effects(
    state: &BattleState,
    context: DamageContext,
) -> DamageResult {
    // Centralized damage calculation that handles:
    // - Type effectiveness and immunities
    // - Critical hit probability and branching
    // - Substitute awareness
    // - Post-damage contact effects
    // - Ability triggers
}

pub fn execute_multi_hit_sequence(
    state: &BattleState,
    context: DamageContext,
    hit_count: u8,
    hit_modifier: Option<HitModifier>,
) -> Vec<BattleInstruction> {
    // Centralized multi-hit logic that handles:
    // - State mutation between hits
    // - Substitute tracking
    // - Contact effects per hit
    // - Ability triggers per hit
}
```

#### 1.2 Status Effect System
**Files to create:**
- `src/engine/combat/core/status_system.rs`

**Key Components:**
```rust
pub struct StatusApplication {
    status: PokemonStatus,
    target: BattlePosition,
    chance: f32,
    duration: Option<u8>,
}

pub fn apply_status_effect(
    state: &BattleState,
    application: StatusApplication,
) -> Vec<BattleInstruction> {
    // Centralized status application that handles:
    // - Immunity checks (type, ability, item)
    // - Existing status interactions
    // - Duration management
    // - Cure conditions
}

pub fn apply_multiple_status_effects(
    state: &BattleState,
    applications: Vec<StatusApplication>,
) -> Vec<BattleInstructions> {
    // For moves with multiple possible status effects
}
```

#### 1.3 Contact Effects System
**Files to create:**
- `src/engine/combat/core/contact_effects.rs`

**Key Components:**
```rust
pub fn apply_contact_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_position: BattlePosition,
    damage_dealt: i16,
) -> Vec<BattleInstruction> {
    // Centralized contact effect handling:
    // - Rocky Helmet, Static, Flame Body, etc.
    // - Ability triggers (Mummy, Cursed Body)
    // - Item effects (Red Card, Eject Button)
}
```

#### 1.4 Field Effect System
**Files to create:**
- `src/engine/combat/core/field_system.rs`

**Key Components:**
```rust
pub fn set_weather(
    weather: Weather,
    duration: Option<u8>,
    source: Option<BattlePosition>,
) -> BattleInstruction;

pub fn set_terrain(
    terrain: Terrain,
    duration: Option<u8>,
    source: Option<BattlePosition>,
) -> BattleInstruction;

pub fn apply_side_condition(
    side: SideReference,
    condition: SideCondition,
    duration: Option<u8>,
) -> BattleInstruction;
```

### Phase 2: Create Effect Composers (1-2 weeks)

#### 2.1 Common Move Patterns
**Files to create:**
- `src/engine/combat/composers/damage_moves.rs`
- `src/engine/combat/composers/status_moves.rs`
- `src/engine/combat/composers/field_moves.rs`

**Key Components:**
```rust
// Standard damage move with optional secondary effects
pub fn simple_damage_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    modifiers: DamageModifiers,
) -> Vec<BattleInstructions>;

// Status move with optional stat changes
pub fn status_move_with_stats(
    state: &BattleState,
    status_effects: Vec<StatusApplication>,
    stat_changes: Option<HashMap<Stat, i8>>,
    target_positions: &[BattlePosition],
) -> Vec<BattleInstructions>;

// Multi-hit move wrapper
pub fn multi_hit_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    hit_count_calculator: HitCountCalculator,
) -> Vec<BattleInstructions>;
```

#### 2.2 Specialized Composers
```rust
// For moves like Bolt Beak, Fishious Rend
pub fn priority_dependent_power(
    base_context: DamageContext,
    power_multiplier: f32,
) -> DamageContext;

// For moves like Facade, Hex
pub fn condition_dependent_power(
    base_context: DamageContext,
    condition_check: Box<dyn Fn(&BattleState) -> bool>,
    power_multiplier: f32,
) -> DamageContext;

// For moves like Body Press, Foul Play
pub fn stat_substitution_move(
    base_context: DamageContext,
    attack_stat: Stat,
    defense_stat: Option<Stat>,
) -> DamageContext;
```

### Phase 3: Refactor Move Implementations (3-4 weeks)

#### 3.1 Convert Multi-Hit Moves
**Target Files:**
- `src/engine/combat/moves/damage/multi_hit.rs`

**Before (Population Bomb - 80+ lines):**
```rust
pub fn apply_population_bomb(...) -> Vec<BattleInstructions> {
    // Complex hit count calculation
    // Manual state cloning and mutation
    // Hand-rolled substitute checking
    // Manual contact effect application
    // 80+ lines of duplicated logic
}
```

**After (Population Bomb - 10 lines):**
```rust
pub fn apply_population_bomb(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let hit_count = if has_wide_lens(state, user_position) { 9 } else { 6 };
    
    multi_hit_move(
        state,
        move_data,
        user_position,
        target_positions,
        HitCountCalculator::Fixed(hit_count),
    )
}
```

#### 3.2 Convert Status Moves
**Target Files:**
- `src/engine/combat/moves/status/status_effects.rs`
- `src/engine/combat/moves/secondary_effects.rs`

**Before (Thunder Wave - 40+ lines):**
```rust
pub fn apply_thunder_wave(...) -> Vec<BattleInstructions> {
    // Manual immunity checking
    // Manual instruction generation
    // Duplicated error handling
}
```

**After (Thunder Wave - 5 lines):**
```rust
pub fn apply_thunder_wave(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    status_move_with_stats(
        state,
        vec![StatusApplication {
            status: PokemonStatus::Paralysis,
            target: target_positions[0],
            chance: 100.0,
            duration: None,
        }],
        None,
        target_positions,
    )
}
```

#### 3.3 Convert Damage Moves with Special Mechanics
**Target Files:**
- `src/engine/combat/moves/special_combat.rs`
- `src/engine/combat/moves/damage/variable_power.rs`

**Before (Body Press - 60+ lines):**
```rust
pub fn apply_body_press(...) -> Vec<BattleInstructions> {
    // Manual critical hit branching
    // Custom damage calculation
    // Duplicated accuracy handling
}
```

**After (Body Press - 8 lines):**
```rust
pub fn apply_body_press(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let context = DamageContext::new(move_data, user_position, target_positions[0])
        .with_stat_substitution(Stat::Defense, Some(Stat::Defense));
    
    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        DamageModifiers::from_context(context),
    )
}
```

### Phase 4: Cleanup and Optimization (1 week)

#### 4.1 Remove Deprecated Files
- Delete individual move functions that have been replaced
- Consolidate remaining move-specific logic
- Update imports and references

#### 4.2 Performance Optimization
- Optimize hot paths in core systems
- Add caching where appropriate
- Profile and benchmark critical functions

#### 4.3 Testing
- Create comprehensive tests for core systems
- Verify all existing move tests still pass
- Add integration tests for complex interactions

## Implementation Strategy

### Development Approach

1. **Incremental Migration**: Implement new systems alongside existing ones
2. **Backwards Compatibility**: Maintain existing move function signatures during transition
3. **Feature Flags**: Use conditional compilation for gradual rollout
4. **Comprehensive Testing**: Ensure no regressions in battle mechanics

### File Organization

```
src/engine/combat/
├── core/                    # New centralized systems
│   ├── damage_system.rs
│   ├── status_system.rs
│   ├── contact_effects.rs
│   ├── field_system.rs
│   └── mod.rs
├── composers/               # New effect composition layer
│   ├── damage_moves.rs
│   ├── status_moves.rs
│   ├── field_moves.rs
│   └── mod.rs
├── moves/                   # Existing move implementations (simplified)
│   ├── damage/
│   ├── status/
│   ├── field/
│   └── special/
└── mod.rs
```

### Migration Strategy

1. **Phase 1**: Create core systems without breaking existing code
2. **Phase 2**: Build composers on top of core systems
3. **Phase 3**: Migrate move implementations category by category
4. **Phase 4**: Remove old implementations and clean up

## Expected Benefits

### Code Quality Improvements

- **90% Reduction** in code duplication
- **Consistent Error Handling** across all move implementations
- **Unified Testing** of battle mechanics
- **Easier Debugging** with centralized logic

### Maintenance Benefits

- **Single Point of Change** for battle mechanic fixes
- **Easier Feature Addition** using existing composers
- **Reduced Bug Surface** due to less duplicated code
- **Better Code Documentation** through clear separation of concerns

### Performance Benefits

- **Reduced Memory Usage** from eliminating duplicate code paths
- **Faster Compilation** due to smaller individual files
- **Optimized Hot Paths** in centralized systems
- **Better Caching** opportunities in core systems