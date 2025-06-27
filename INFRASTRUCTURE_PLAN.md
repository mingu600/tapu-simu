# Tapu-Simu Infrastructure Modernization Plan

## Executive Summary

Tapu-simu has **excellent infrastructure that is being underutilized**. Instead of building new systems, we need to **standardize usage** of existing well-designed components and eliminate ad-hoc workarounds.

## Problem Analysis

### Current Issues
1. **Type System Fragmentation**: 3 different `PokemonType` enums + string-based types
2. **Infrastructure Bypassing**: Manual implementations instead of using composers/core systems  
3. **Redundant Conversions**: Multiple string parsing implementations for same functionality
4. **Inconsistent Patterns**: No clear guidelines on which system to use when

### Root Cause
Good infrastructure exists (`DamageContext`, composers, registry system) but developers are creating ad-hoc solutions instead of using the proper APIs.

---

## Phase 1: Type System Unification (High Priority)

### 1.1 Create Unified PokemonType System

**Goal**: Single source of truth for all type operations

**Files to Modify**:
- `src/types/pokemon_type.rs` (NEW)
- `src/core/battle_state/pokemon.rs`
- `src/engine/combat/type_effectiveness.rs`

**Implementation**:
```rust
// New file: src/types/pokemon_type.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PokemonType {
    Normal = 0, Fire = 1, Water = 2, // ... all 18 types
}

impl PokemonType {
    pub fn from_normalized_str(s: &str) -> Option<Self> { /* case-insensitive */ }
    pub fn to_normalized_str(&self) -> &'static str { /* lowercase */ }
    pub fn display_name(&self) -> &'static str { /* Title Case */ }
}

impl From<&TypeId> for PokemonType { /* automatic conversion */ }
impl From<PokemonType> for TypeId { /* automatic conversion */ }
```

**Migration Steps**:
1. Create unified `PokemonType` enum with all needed traits
2. Replace `Vec<String>` types in Pokemon struct with `Vec<PokemonType>`
3. Replace string move types with `PokemonType`
4. Update type effectiveness system to use unified enum
5. Add conversion traits for seamless interop

### 1.2 Eliminate String-Based Type Handling

**Files to Update**:
- `src/core/battle_state/pokemon.rs:154` - Change `types: Vec<String>` to `types: Vec<PokemonType>`
- `src/core/battle_state/move.rs:58` - Change `move_type: String` to `move_type: PokemonType`
- `src/data/showdown_types.rs` - Update type fields in PS data structures

**Implementation**:
```rust
// Before
pub struct Pokemon {
    pub types: Vec<String>, // REMOVE
    // ...
}

// After  
pub struct Pokemon {
    pub types: Vec<PokemonType>, // TYPE-SAFE
    // ...
}
```

---

## Phase 2: Mandate Infrastructure Usage (High Priority)

### 2.1 Move Implementation Standards

**Rule**: All moves MUST use existing composers or core systems. Manual implementations are prohibited.

**Composer Usage Matrix**:
| Move Type | Use Composer | File |
|-----------|--------------|------|
| Basic damage | `simple_damage_move()` | `composers/damage_moves.rs:86` |
| Damage + status | `damage_move_with_secondary_status()` | `composers/damage_moves.rs:198` |
| Multi-hit | `multi_hit_move()` | `composers/damage_moves.rs:310` |
| Stat changes | `stat_modification_move()` | `composers/status_moves.rs:45` |
| Weather/terrain | `weather_move()`, `terrain_move()` | `composers/field_moves.rs` |
| Pure status | `apply_status_effect()` | `core/status_system.rs:156` |

### 2.2 Registry Signature Simplification

**Current Problem**: 5 different function signatures in registry
**Solution**: Reduce to 3 clear categories

**New Registry Categories**:
```rust
// Basic moves (no context needed)
type BasicMoveEffect = fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics) -> Vec<BattleInstructions>;

// Context-aware moves (need turn order, opponent info)  
type ContextMoveEffect = fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, &MoveContext) -> Vec<BattleInstructions>;

// Repository-aware moves (need to query other moves/data)
type RepositoryMoveEffect = fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, &MoveContext, &GameDataRepository) -> Vec<BattleInstructions>;
```

### 2.3 Migrate Existing Manual Implementations

**Target Files** (in priority order):
1. `src/engine/combat/moves/damage/fixed_damage.rs` - Replace with `damage_system::calculate_damage_with_effects()`
2. `src/engine/combat/moves/damage/variable_power.rs` - Use power modifier system instead of manual `MoveData` cloning
3. `src/engine/combat/moves/status/status_effects.rs` - Replace with `status_system::apply_status_effect()`

**Migration Pattern**:
```rust
// Before (manual implementation)
pub fn apply_super_fang(/* params */) -> Vec<BattleInstructions> {
    // 50+ lines of manual calculation, type checking, etc.
}

// After (using infrastructure)
pub fn apply_super_fang(/* params */) -> Vec<BattleInstructions> {
    let context = DamageCalculationContext::for_fixed_damage(user_position, target_positions, |target| target.hp / 2);
    calculate_damage_with_effects(state, context, generation)
}
```

---

## Phase 3: Code Generation and DRY (Medium Priority) 

### 3.1 Macro-Generate Identifier Types

**Problem**: All types in `src/types/identifiers.rs` have identical implementations

**Solution**: Single macro generates all ID types
```rust
macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);
        
        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                let normalized = normalize_name(&id.into());
                Self::validate_normalized(&normalized);
                Self(normalized)
            }
            // ... common methods
        }
        // ... common trait impls
    };
}

define_id_type!(SpeciesId);
define_id_type!(MoveId);
define_id_type!(ItemId);
define_id_type!(AbilityId);
// etc.
```

### 3.2 Unified String-to-Enum Conversion

**Create trait**: `src/types/from_string.rs`
```rust
pub trait FromNormalizedString: Sized {
    fn from_normalized_str(s: &str) -> Option<Self>;
}

// Implement for all enums that need string parsing
impl FromNormalizedString for PokemonType { /* ... */ }
impl FromNormalizedString for MoveCategory { /* ... */ }
impl FromNormalizedString for Stat { /* ... */ }
```

---

## Phase 4: Documentation and Guidelines (Medium Priority)

### 4.1 Developer Guidelines Document

**File**: `MOVE_IMPLEMENTATION_GUIDE.md`

**Content**:
- When to use each composer
- How to choose registry signature
- Type system usage rules
- Common patterns and anti-patterns
- Migration examples

### 4.2 Code Comments and Examples

**Add inline documentation** to:
- All composer functions explaining when to use them
- Registry registration methods with examples
- Type conversion functions with usage patterns

---

## Phase 5: Performance and Cleanup (Low Priority)

### 5.1 Eliminate Redundant Operations

**Target Areas**:
- Remove unnecessary `MoveData` cloning in variable power moves
- Cache type effectiveness calculations where appropriate
- Optimize string normalization (already efficient but could be faster)

### 5.2 Repository System Optimization  

**Current State**: Repository system is well-designed and performant
**Action**: Minor optimizations only, no major changes needed

---

## Implementation Timeline

### Week 1: Type System Unification
- [ ] Create unified `PokemonType` system
- [ ] Update Pokemon/Move structs to use typed fields
- [ ] Update type effectiveness system
- [ ] Create conversion traits

### Week 2: Infrastructure Mandating
- [ ] Simplify registry signatures
- [ ] Document composer usage patterns
- [ ] Migrate 5 highest-impact moves to proper infrastructure

### Week 3: Mass Migration
- [ ] Migrate all remaining manual implementations
- [ ] Update tests to use new systems
- [ ] Remove deprecated code paths

### Week 4: Code Generation & Documentation
- [ ] Implement ID type macro system
- [ ] Create developer guidelines
- [ ] Add comprehensive inline documentation

---

## Success Metrics

### Quantitative
- **Type Conversions**: Reduce from ~50 manual conversion sites to 0
- **Move Implementations**: 100% use proper infrastructure (0 manual implementations)
- **Code Duplication**: Eliminate ~300 lines of duplicate ID type code

### Qualitative  
- **Developer Experience**: Clear guidelines on which system to use when
- **Maintainability**: Changes to common patterns only require updating composers
- **Type Safety**: Compile-time prevention of type mismatches

---

## Risk Mitigation

### Breaking Changes
- **Risk**: Type system changes affect many files
- **Mitigation**: Implement conversion traits for gradual migration

### Test Compatibility
- **Risk**: Test framework expects string-based types
- **Mitigation**: Add `Display`/`FromStr` traits for test compatibility

### Performance Regression
- **Risk**: More type-safe code might be slower
- **Mitigation**: Benchmark critical paths, most changes are zero-cost abstractions

---

## Conclusion

This plan **leverages existing excellent infrastructure** rather than building new systems. The codebase already has sophisticated damage calculation, status application, and move composition systems - we just need to **mandate their usage** and **eliminate workarounds**.

The type system unification is the critical first step that enables everything else. Once types are unified, using the existing infrastructure becomes natural and type-safe.

**Key Insight**: This is primarily a **standardization and migration effort**, not a rewrite. The foundation is solid.