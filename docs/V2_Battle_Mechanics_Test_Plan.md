# V2 Battle Mechanics Test Plan

## Overview

This document describes our process for porting and adapting the comprehensive test suite from poke-engine's `test_battle_mechanics.rs` (663 tests) to tapu-simu's position-aware, format-first architecture.

## Objectives

1. **Port all 663 battle mechanics tests** from poke-engine to tapu-simu
2. **Maintain test functionality** while adapting to V2's position-based system
3. **Incrementally implement battle mechanics** needed to make tests pass
4. **Ensure comprehensive coverage** of Pokemon battle mechanics for singles play
5. **Establish foundation** for future doubles/VGC test adaptation

## Approach: Test-Driven Development

We follow a strict test-driven approach where tests are ported as exact functional replicas of poke-engine's tests, then mechanics are implemented to make them pass naturally:

1. **Port Test Exactly**: Create functionally identical replica of poke-engine test
   - Same moves used (e.g., WATERGUN + SPLASH, not different moves)
   - Same expected results (e.g., exact damage values: 32, not approximations)
   - Same instruction count and structure
   - Same battle state setup (HP values, Pokemon stats, etc.)
2. **Implement Missing Mechanics**: Identify and implement the specific battle mechanics needed
3. **Never Force Results**: NEVER modify tests to match incorrect output - always fix the underlying mechanics
4. **Validate Correctness**: Ensure test passes with proper battle logic implementation
5. **Move to Next Test**: Repeat process for next test in poke-engine's sequence

## Test Completion Criteria & Strict Requirements

### ✅ A test is ONLY complete when:
1. **Functionally identical** to the corresponding poke-engine test
2. **Same moves**: Uses exactly the same Choices (WATERGUN, not TACKLE)
3. **Same expected values**: Expects exact damage/HP values from poke-engine (32 damage, not 19)
4. **Same instruction structure**: Same number and type of instructions
5. **Correct mechanics implemented**: All underlying battle logic properly implemented
6. **NO placeholder logic**: All game mechanics actually functional

### ❌ NEVER acceptable:
- **Forcing test results**: Modifying expected values to match incorrect output
- **Different moves**: Using TACKLE when poke-engine uses WATERGUN
- **Approximate values**: Accepting "close enough" damage (19 instead of 32)
- **Placeholder assertions**: Tests that pass with dummy/empty logic
- **Architecture shortcuts**: Bypassing proper turn-based instruction generation

### 🎯 The Goal:
Every tapu-simu test should be a **perfect functional replica** of its poke-engine counterpart, proving that our V2 architecture produces identical battle mechanics with enhanced multi-format support.

## Current Infrastructure ✅

### Core Test Framework
- **File**: `tests/test_battle_mechanics_singles.rs`
- **Helper Functions**: State setup, instruction generation, move creation
- **Integration**: Proper use of existing Choices enum and MOVES HashMap


## Test Categories & Status

### ✅ Completed Tests: 3/663

**Tests with proper battle mechanics implementation:**
- `test_confuseray_into_substitute` - ✅ Substitute blocking logic implemented
- `test_confuseray_without_substitute` - ✅ Status application logic implemented  
- `test_branch_on_crit` - 🔄 **IN PROGRESS** - Turn-based instruction generation implemented, needs damage calculation fix

### 🚧 Major Architectural Achievements
- **Turn-based instruction generation**: Implemented `generate_instructions_from_move_pair` equivalent
- **Multi-format targeting system**: Format-aware move resolution working
- **Status effect system**: Confusion and Substitute mechanics functional
- **Critical hit branching**: Probability-based instruction branching implemented
- **Move order determination**: Priority and speed-based move ordering

### ⏳ Remaining Implementation (660/663)

#### 1. Critical Hits & Damage Calculation (~60 tests)
**Priority**: URGENT - Currently blocking `test_branch_on_crit` completion
- `test_branch_on_crit` - **CURRENT BLOCKER**: Needs exact damage match (32, not 19)
- `test_highcrit_move` - Moves with increased crit rates  
- `test_wickedblow_always_crits_without_a_branch` - Guaranteed crits
- `test_min_damage_killing_does_not_branch` - Damage calculation edge cases

**Required Implementation**:
- **CRITICAL**: Fix damage calculation to match poke-engine exactly (32 damage for WATERGUN)
- Verify Pokemon stats match poke-engine defaults
- Ensure damage formula identical to poke-engine
- Critical hit calculation in `damage_calc.rs`
- Minimum damage (1 HP) rules

**Current Issue**: Our WATERGUN does 19 damage, poke-engine expects 32. Root cause unknown - could be stats, formula, or move data.

#### 2. Status Effects & Core Mechanics (~50 tests)
**Priority**: High - Core battle interactions
- Sleep mechanics (`test_guaranteed_to_stay_asleep_*`)
- Poison/Burn damage over time
- Paralysis speed/move success rates
- Confusion self-damage

**Required Implementation**:
- Status application/removal in instruction generator
- End-of-turn status effects
- Status immunity checking

#### 3. Weather & Environmental Effects (~50 tests)
**Priority**: Medium - Battlefield conditions
- Rain/Sun damage modification
- Weather abilities (Rain Dish, Solar Power, etc.)
- Weather duration mechanics

#### 4. Abilities & Items (~150 tests)
**Priority**: Medium-High - Core Pokemon features
- Basic abilities (Flash Fire, Wonder Guard, etc.)
- Damage-modifying abilities
- Status-immunity abilities
- Berry activation, type-boosting items

#### 5. Advanced Mechanics (~353 tests)
**Priority**: Medium-Low - Complex interactions
- Switching mechanics
- Multi-turn moves
- Protect variations
- Speed calculation edge cases

## Implementation Strategy

### Phase 1: Core Damage System
**Target**: Make `test_branch_on_crit` meaningful and complete
1. Implement basic damage calculation
2. Add critical hit branching to instruction generator
3. Integrate damage calculation with move data
4. Add type effectiveness framework

### Phase 2: Status Effect Foundation  
**Target**: Make `test_confuseray_into_substitute` meaningful and complete
1. Implement Substitute blocking logic for status moves
2. Add status condition application/immunity
3. Create status effect instruction generation
4. Add volatile status duration tracking

### Phase 3: Systematic Test Porting
**Target**: Port tests sequentially from poke-engine
1. Go through tests 1-by-1 in order from `test_battle_mechanics.rs`
2. For each test:
   - Adapt test structure to tapu-simu format
   - Identify missing mechanics
   - Implement only what's needed for that test
   - Ensure test passes before moving to next

### Phase 4: Integration & Optimization
1. Optimize instruction generation performance
2. Add comprehensive error handling
3. Document all implemented mechanics
4. Prepare foundation for doubles/VGC tests

## Key Adaptations for V2

### Instruction Mapping
```rust
// Poke-engine style (V1)
DamageInstruction { 
    side_ref: SideTwo, 
    damage_amount: 50 
}

// Tapu-simu equivalent (V2)
PositionDamage(PositionDamageInstruction {
    target_position: BattlePosition::new(SideReference::SideTwo, 0),
    damage_amount: 50,
})
```

### Test Structure Adaptation
```rust
// Poke-engine style (V1)
fn test_example() {
    let mut state = State::default();
    // Set up state...
    let instructions = generate_instructions_from_move_pair(/*...*/);
    // V1-style assertions
}

// Tapu-simu equivalent (V2)
fn test_example() {
    let mut state = create_basic_singles_state();
    // Set up state with position awareness...
    let instructions = set_moves_on_pkmn_and_call_generate_instructions(
        &mut state, Choices::MOVE1, Choices::MOVE2
    );
    // V2-style position-aware assertions
}
```

### Data Integration
- **V1**: String-based move names, separate move data
- **V2**: Choices enum with comprehensive MOVES HashMap
- **V2**: Proper type conversion between data and state layers
- **V2**: Position-aware targeting from the start

## Success Metrics

1. **663 tests successfully ported** with appropriate V2 adaptations
2. **All tests pass** with meaningful battle mechanics implementation
3. **No regressions** in existing tapu-simu functionality  
4. **Comprehensive coverage** of Pokemon battle mechanics for singles
5. **Clean architecture** maintaining V2's format-first principles
6. **Performance targets** met for instruction generation
7. **Documentation** complete for all implemented mechanics

## Timeline Estimation

- **Phase 1**: Core Damage System (1-2 weeks)
- **Phase 2**: Status Effect Foundation (1-2 weeks)  
- **Phase 3**: Systematic Test Porting (8-12 weeks)
- **Phase 4**: Integration & Optimization (2-3 weeks)

**Total**: 12-19 weeks for complete test suite

## Benefits of This Approach

1. **Incremental Progress**: Each test adds specific functionality
2. **Quality Assurance**: Tests validate correctness at each step
3. **Comprehensive Coverage**: 663 tests ensure thorough validation
4. **Architecture Alignment**: All tests use V2's position-based system
5. **Future Foundation**: Sets up framework for doubles/VGC tests
6. **Performance Validation**: Tests catch performance regressions early

## Current Status

- **Infrastructure**: ✅ Complete
- **Test Framework**: ✅ Working with real Choices enum integration  
- **Turn-based Instruction Generation**: ✅ Complete (major architectural milestone)
- **Completed Tests**: ✅ 3/663 with proper mechanics implementation
- **Current Blocker**: Damage calculation mismatch (19 vs 32 for WATERGUN)
- **Next Target**: Fix damage calculation to complete `test_branch_on_crit` exactly matching poke-engine

## Key Principle Reinforced

**NEVER modify tests to match wrong output.** If our test expects 32 damage but gets 19, we fix the damage calculation system, not the test. This ensures our V2 architecture produces identical battle mechanics to poke-engine while maintaining enhanced multi-format support.

This systematic approach ensures we build a robust, well-tested battle engine that maintains V2's architectural principles while achieving comprehensive Pokemon battle mechanics coverage.