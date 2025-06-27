# Move Implementation Guide

This guide provides clear patterns for implementing moves in Tapu Simu using the existing infrastructure. **All moves MUST use the composer system or core infrastructure** - manual implementations are prohibited.

## 🎯 Quick Reference

### When to Use Each Composer

| Move Pattern | Use This Composer | Example Moves |
|--------------|------------------|---------------|
| **Basic Damage** | `simple_damage_move()` | Tackle, Quick Attack |
| **Multi-Hit** | `multi_hit_move()` | Double Slap, Fury Attack |
| **Conditional Power** | `condition_dependent_power_move()` | Facade, Hex, Wake-Up Slap |
| **Priority-Based Power** | `priority_dependent_power_move()` | Bolt Beak, Fishious Rend |
| **Stat Substitution** | `stat_substitution_move()` | Body Press, Foul Play |
| **Recoil Damage** | `recoil_move()` | Take Down, Double-Edge |
| **HP Draining** | `draining_move()` | Absorb, Giga Drain |
| **Always Critical** | `always_crit_move()` | Frost Breath, Storm Throw |
| **Dynamic Category** | `dynamic_category_move()` | Photon Geyser, Shell Side Arm |
| **Status Changes** | `stat_modification_move()` | Swords Dance, Growl |
| **Apply Status** | `single_status_move()` | Thunder Wave, Sleep Powder |
| **Status + Stats** | `status_plus_stat_move()` | Swagger, Charm |
| **Multi-Status** | `multi_status_move()` | Tri Attack, Secret Power |
| **Healing** | `healing_move()` | Recover, Soft-Boiled |
| **Weather** | `weather_setting_move()` | Sunny Day, Rain Dance |
| **Terrain** | `terrain_setting_move()` | Electric Terrain, Grassy Terrain |
| **Screens** | `reflect_move()`, `light_screen_move()`, `aurora_veil_move()` | Reflect, Light Screen |
| **Hazards** | `spikes_move()`, `stealth_rock_move()` | Spikes, Stealth Rock |
| **Hazard Removal** | `rapid_spin_move()`, `defog_move()` | Rapid Spin, Defog |

## 📁 File Locations

```
src/engine/combat/composers/
├── mod.rs              # Re-exports all composers
├── damage_moves.rs     # All damage-dealing move patterns
├── status_moves.rs     # Status effects and stat modifications  
├── field_moves.rs      # Weather, terrain, hazards, screens
```

## 🔧 Implementation Patterns

### 1. Basic Damage Move

```rust
use crate::engine::combat::composers::simple_damage_move;

pub fn tackle(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    simple_damage_move(state, move_data, user, targets, generation)
}
```

### 2. Conditional Power Move

```rust
use crate::engine::combat::composers::condition_dependent_power_move;

pub fn facade(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    condition_dependent_power_move(
        state,
        move_data,
        user,
        targets,
        generation,
        |state, user_pos| {
            // Double power if user has status condition
            if state.get_pokemon(user_pos).has_major_status() {
                2.0
            } else {
                1.0
            }
        }
    )
}
```

### 3. Status Move with Stat Changes

```rust
use crate::engine::combat::composers::status_plus_stat_move;
use crate::types::{StatusCondition, Stat};

pub fn swagger(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    status_plus_stat_move(
        state,
        move_data,
        user,
        targets,
        generation,
        StatusCondition::Confusion,
        vec![(Stat::Attack, 2)], // +2 Attack
    )
}
```

### 4. Weather Setting Move

```rust
use crate::engine::combat::composers::sunny_day_move;

pub fn sunny_day(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    sunny_day_move(state, move_data, user, targets, generation)
}
```

## 🚫 Anti-Patterns (DO NOT DO)

### ❌ Manual Damage Calculation
```rust
// WRONG - Manual implementation
pub fn bad_tackle(/* ... */) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let damage = calculate_base_damage(/* ... */);
    let effectiveness = get_type_effectiveness(/* ... */);
    // 50+ lines of manual calculation...
    instructions
}
```

### ❌ String-Based Type Handling
```rust
// WRONG - String types
let move_type = "fire".to_string();
let pokemon_types = vec!["water".to_string(), "ground".to_string()];

// CORRECT - Type-safe enums
let move_type = PokemonType::Fire;
let pokemon_types = vec![PokemonType::Water, PokemonType::Ground];
```

### ❌ Single-Format Assumptions
```rust
// WRONG - Assumes single target
let target = targets[0]; // Crashes in multi-target scenarios

// CORRECT - Handle all targets
for &target in targets {
    // Process each target
}
```

## 📋 Registry Function Signatures

Choose the appropriate signature based on what your move needs:

### Basic Moves (No Additional Context)
```rust
fn basic_move(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>
```

### Context-Aware Moves (Need Turn Order, Speed, etc.)
```rust
fn context_move(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<BattleInstruction>
```

### Repository-Aware Moves (Need to Query Other Data)
```rust
fn repository_move(
    state: &BattleState,
    move_data: &MoveData,
    user: BattlePosition,
    targets: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
    repository: &GameDataRepository,
) -> Vec<BattleInstruction>
```

## 🎨 Best Practices

### 1. Always Use Composers First
Before implementing custom logic, check if a composer exists for your pattern. Most moves follow established patterns.

### 2. Multi-Format Design
Every move must work in both Singles and Doubles formats:
- Use `targets: &[BattlePosition]` correctly
- Populate `affected_positions` in instructions
- Test in both formats

### 3. Type Safety
- Use `PokemonType` enum, not strings
- Use type-safe identifiers (`MoveId`, `SpeciesId`, etc.)
- Leverage the `FromNormalizedString` trait for parsing

### 4. Position Awareness
```rust
// GOOD - Position-aware
for &target_pos in targets {
    let target_pokemon = state.get_pokemon(target_pos);
    instructions.push(BattleInstruction::DamageTarget {
        target: target_pos,
        amount: damage,
        affected_positions: vec![target_pos],
    });
}
```

### 5. Instruction Generation
- Instructions are atomic and immutable
- Always populate `affected_positions`
- Use appropriate instruction types
- Follow the existing instruction patterns

## 🧪 Testing Requirements

Every move implementation must include:

1. **Format Tests**: Verify behavior in Singles and Doubles
2. **Edge Cases**: Test with immunities, abilities, status conditions
3. **Multi-Target**: Ensure correct behavior with multiple targets
4. **Position Tracking**: Verify `affected_positions` are correct

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_move_in_singles() { /* ... */ }
    
    #[test] 
    fn test_move_in_doubles() { /* ... */ }
    
    #[test]
    fn test_move_with_immunity() { /* ... */ }
    
    #[test]
    fn test_move_affected_positions() { /* ... */ }
}
```

## 🔍 Common Debugging

### Type Mismatches
- Ensure you're using `PokemonType` enum, not strings
- Check that identifiers are properly typed
- Use `.into()` for automatic conversions

### Position Errors
- Verify `affected_positions` includes all modified positions
- Check that targets are handled as a slice, not single value
- Ensure format compatibility

### Missing Infrastructure
- Don't create manual implementations
- File an issue if a needed composer doesn't exist
- Use core systems (`damage_system`, `status_system`) as fallbacks

## 📚 Related Documentation

- **Type System**: See `src/types/` for all type-safe enums and identifiers
- **Core Systems**: See `src/engine/combat/core/` for underlying mechanics
- **Instructions**: See `src/core/instructions/` for available instruction types
- **Battle State**: See `src/core/battle_state/` for state access patterns

---

**Remember**: The goal is consistency and maintainability. Use existing infrastructure, follow established patterns, and implement with multi-format support from day one.