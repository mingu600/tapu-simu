# Pokemon Showdown Integration - Refactoring Plan

## Overview
To integrate Pokemon Showdown data seamlessly, we'll adopt PS's naming conventions throughout tapu-simu. This ensures consistency and eliminates mapping complexity.

## Move Target Refactoring

### Current tapu-simu → Pokemon Showdown naming:

| Current (tapu-simu) | New (PS) | Description |
|---------------------|----------|-------------|
| SingleAdjacentTarget | normal | Standard single-target move |
| User | self | Targets the user |
| SingleAlly | adjacentAlly | Targets an adjacent ally |
| UserOrAlly | adjacentAllyOrSelf | User or adjacent ally |
| SingleAdjacentOpponent | adjacentFoe | Single adjacent opponent |
| AllAdjacentOpponents | allAdjacentFoes | All adjacent opponents |
| AllAdjacent | allAdjacent | All adjacent Pokémon |
| EntireField | all | Affects the entire field |
| UsersTeam | allyTeam | User's entire team |
| UsersSide | allySide | User's side of field |
| OpponentsSide | foeSide | Opponent's side of field |
| SingleTarget | any | Any single target |
| RandomOpponent | randomNormal | Random opponent |
| SpecificMove | scripted | Scripted target (Counter, etc.) |

### Files to refactor:

1. **src/data/types.rs** - Update `MoveTarget` enum
2. **src/genx/format_targeting.rs** - Update `AutoTargetingEngine` 
3. **src/data/conversion.rs** - Update rustemon conversion mappings
4. **src/genx/format_instruction_generator.rs** - Update target handling
5. **tests/test_battle_mechanics_singles.rs** - Update test cases

## Move Category Refactoring

Already aligned - no changes needed:
- Physical
- Special  
- Status

## Benefits of this approach:

1. **Direct data usage** - PS JSON data can be used without transformation
2. **Consistency** - Same terminology as the most popular battle simulator
3. **Documentation** - Can reference PS documentation directly
4. **Future updates** - Easier to incorporate PS updates

## Migration strategy:

1. Create type aliases for gradual migration:
   ```rust
   type MoveTarget = str; // Use PS strings directly
   ```

2. Update the enum to use PS names:
   ```rust
   pub enum MoveTarget {
       Normal,           // was SingleAdjacentTarget
       Self_,            // was User (Self is reserved)
       AdjacentAlly,     // was SingleAlly
       // ... etc
   }
   ```

3. Implement Display/FromStr to use lowercase PS strings

4. Update all references throughout the codebase

This refactoring will make the PS integration much cleaner and maintain better compatibility with the broader Pokemon community's conventions.