# Damage Calculation Modularization Migration Plan

## Overview

This document outlines the migration plan to modularize the monolithic `damage_calc.rs` file (~2,172 lines) into focused, maintainable modules while preserving all existing functionality and API compatibility.

## Current State Analysis

The `damage_calc.rs` file contains:

1. **Main Entry Point**: `calculate_damage_with_positions()` - used by 8+ files across the codebase
2. **Generation-Specific Calculators**: `calculate_damage_gen1()` through `calculate_damage_modern_gen789()`
3. **Modifier Functions**: Weather, terrain, screen, spread, item, ability modifiers
4. **Utility Functions**: Critical hit probability, damage rolls, type effectiveness integration
5. **Generation-Specific Helpers**: Gen 1/2 specific critical hit logic, item modifiers

### Usage Analysis

The damage calculation system is used by:
- `src/engine/turn.rs` - Main battle turn processing
- `src/engine/combat/core/damage_system.rs` - Core damage application
- `src/engine/combat/moves/mod.rs` - Move execution
- `src/engine/combat/moves/special/two_turn.rs` - Two-turn moves
- `src/engine/combat/moves/special_combat.rs` - Special combat moves
- `src/engine/mechanics/switch_effects.rs` - Switch-related damage
- `src/engine/combat/core/status_system.rs` - Status effect damage

## Migration Plan

### Phase 1: Core Infrastructure (Week 1)

**Goals:**
- Create new module structure
- Extract core types while maintaining compatibility
- Establish foundation for future phases

**Tasks:**
1. Create `src/engine/combat/damage/` subdirectory
2. Extract `DamageRolls` enum and related types to `damage/types.rs`
3. Create `damage/mod.rs` with re-exports to maintain API compatibility
4. Add integration tests to ensure no regressions

**File Structure:**
```
src/engine/combat/damage/
├── mod.rs              # Re-exports for API compatibility
├── types.rs            # DamageRolls, DamageResult, etc.
└── calculator.rs       # Main calculator entry point
```

### Phase 2: Modifier System (Week 2)

**Goals:**
- Extract all modifier functions into focused modules
- Improve testability and maintainability
- Clear separation of concerns

**Tasks:**
1. Create modifier modules with focused responsibilities
2. Migrate modifier functions maintaining signatures
3. Add unit tests for each modifier category
4. Update internal imports

**File Structure:**
```
src/engine/combat/damage/
├── modifiers/
│   ├── mod.rs          # Modifier system coordination
│   ├── weather.rs      # get_weather_damage_modifier, is_weather_negated
│   ├── terrain.rs      # get_terrain_damage_modifier, is_grounded
│   ├── field.rs        # get_screen_damage_modifier
│   ├── format.rs       # get_spread_move_modifier
│   ├── items.rs        # get_gen2_item_modifier, item effects
│   └── abilities.rs    # ability-based modifiers
```

### Phase 3: Generation System (Week 3)

**Goals:**
- Separate generation-specific calculation logic
- Enable easier addition of new generations
- Maintain accuracy across all generations

**Tasks:**
1. Extract generation-specific calculators
2. Create common calculator trait/interface
3. Implement generation dispatch system
4. Add generation-specific tests

**File Structure:**
```
src/engine/combat/damage/
├── generations/
│   ├── mod.rs          # Generation dispatch and common interface
│   ├── gen1.rs         # calculate_damage_gen1, critical_hit_probability_gen1
│   ├── gen2.rs         # calculate_damage_gen2, critical_hit_probability_gen2
│   ├── gen3.rs         # calculate_damage_gen3
│   ├── gen4.rs         # calculate_damage_gen4
│   ├── gen56.rs        # calculate_damage_gen56
│   └── modern.rs       # calculate_damage_modern_gen789
```

### Phase 4: Calculator Factory (Week 4)

**Goals:**
- Create clean, testable calculator interface
- Centralize damage roll and critical hit logic
- Maintain performance while improving structure

**Tasks:**
1. Implement calculator factory pattern
2. Extract damage roll calculations
3. Centralize critical hit probability system
4. Performance validation

**File Structure:**
```
src/engine/combat/damage/
├── calculator.rs       # Main calculator with generation dispatch
├── rolls.rs           # calculate_all_damage_rolls, get_damage_for_roll
├── critical.rs        # critical_hit_probability system
└── utils.rs           # Utility functions (poke_round, etc.)
```

### Phase 5: Final Integration (Week 5)

**Goals:**
- Complete migration with full API compatibility
- Comprehensive testing and validation
- Performance verification

**Tasks:**
1. Update all imports across 8+ dependent files
2. Maintain `calculate_damage_with_positions()` as main entry point
3. Add comprehensive integration tests
4. Performance benchmarking and validation
5. Documentation updates

## Detailed Module Breakdown

### Core Calculator (`damage/calculator.rs`)
```rust
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: usize,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
) -> i16 {
    let context = build_damage_context(/* ... */);
    let calculator = GenerationCalculator::new(context.generation);
    calculator.calculate(context, damage_rolls)
}
```

### Modifier System (`damage/modifiers/`)
Each modifier module contains:
- Pure functions taking specific contexts
- Clear separation of concerns
- Easy testing and maintenance
- Generation-aware when needed

### Generation Calculators (`damage/generations/`)
Each generation module:
- Implements a common `Calculator` trait
- Handles generation-specific damage formulas
- Contains generation-specific critical hit logic
- Maintains current accuracy while being testable

## Migration Strategy

### Safety Measures
1. **Incremental Migration**: Move one category at a time
2. **API Compatibility**: Maintain all existing function signatures during transition
3. **Comprehensive Testing**: Integration tests for each phase
4. **Performance Monitoring**: Ensure no performance regressions

### Risk Mitigation
1. **Feature Flags**: Use conditional compilation during migration if needed
2. **Rollback Plan**: Keep original file until migration is complete and validated
3. **Cross-Generation Testing**: Validate all generations work correctly
4. **Usage Pattern Analysis**: Ensure all calling patterns continue to work

### Testing Strategy
1. **Unit Tests**: Each new module gets comprehensive unit tests
2. **Integration Tests**: End-to-end damage calculation tests
3. **Performance Tests**: Benchmark against current implementation
4. **Regression Tests**: Ensure all existing functionality preserved

## Expected Benefits

This modular approach will:
- **Improve maintainability** by separating concerns
- **Enhance testability** with focused, pure functions
- **Support future additions** like new generations or mechanics
- **Maintain performance** through careful architecture
- **Preserve correctness** through comprehensive testing
- **Reduce cognitive load** when working with specific damage aspects

## Success Criteria

1. All existing tests continue to pass
2. No performance regressions (< 5% slowdown acceptable)
3. All 8+ dependent files continue to work without changes
4. New modular structure enables easier maintenance and testing
5. Documentation is updated and comprehensive

## Timeline

- **Week 1**: Phase 1 (Infrastructure)
- **Week 2**: Phase 2 (Modifiers)
- **Week 3**: Phase 3 (Generations)
- **Week 4**: Phase 4 (Calculator Factory)
- **Week 5**: Phase 5 (Integration & Validation)

Total estimated time: 5 weeks with proper testing and validation.