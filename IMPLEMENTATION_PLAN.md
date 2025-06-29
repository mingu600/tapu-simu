# Tapu-Simu Performance and Code Quality Implementation Plan

## Overview

This document provides a detailed implementation plan for performance optimizations and code quality improvements in the tapu-simu codebase. Each section includes specific code changes, file locations, and implementation steps.

## Phase 1: Critical Performance Fixes (Week 1)

### 1.1 Fix Repository Loading Overhead

**Problem**: New `GenerationRepository` and `GameDataRepository` created on every move execution in `src/engine/turn.rs:1048-1162`.

**Implementation Steps**:

1. **Add repository fields to BattleState** (`src/core/battle_state/mod.rs`):
```rust
pub struct BattleState {
    // ... existing fields ...
    pub generation_repo: Arc<GenerationRepository>,
    pub game_data_repo: Arc<GameDataRepository>,
}
```

2. **Modify BattleState::new()** to accept repositories:
```rust
impl BattleState {
    pub fn new(
        // ... existing params ...
        generation_repo: Arc<GenerationRepository>,
        game_data_repo: Arc<GameDataRepository>,
    ) -> Self {
        // Initialize with passed repositories
    }
}
```

3. **Update all BattleState instantiations**:
   - `src/simulator.rs`: Create repositories once in `Simulator::new()`
   - `src/builders/battle.rs`: Accept repositories in `BattleBuilder`
   - `tests/utils/builders.rs`: Create test repositories once per test

4. **Replace repository creation in turn.rs**:
```rust
// Remove lines 1048-1053
// Replace with:
let generation_mechanics = &state.generation_repo.mechanics;
let game_data = &state.game_data_repo;
```

### 1.2 Eliminate Pokemon Cloning in Damage Context

**Problem**: Full Pokemon structures cloned in `src/engine/combat/damage_context.rs:216,224`.

**Implementation Steps**:

1. **Add lifetime parameter to context structs**:
```rust
// src/engine/combat/damage_context.rs
pub struct AttackerContext<'a> {
    pub pokemon: &'a Pokemon,
    pub position: BattlePosition,
    pub stat_modifiers: StatModifiers,
    pub damage_modifiers: DamageModifiers,
}

pub struct DefenderContext<'a> {
    pub pokemon: &'a Pokemon,
    pub position: BattlePosition,
    pub stat_modifiers: StatModifiers,
    pub damage_modifiers: DamageModifiers,
}

pub struct DamageContext<'a> {
    pub attacker: AttackerContext<'a>,
    pub defender: DefenderContext<'a>,
    pub move_data: &'a MoveData,
    pub field_context: FieldContext,
    pub generation: GenerationMechanics,
}
```

2. **Update context creation** (lines 195-227):
```rust
let attacker_context = AttackerContext {
    pokemon: attacker_pokemon, // No clone!
    position: attacker_position,
    stat_modifiers,
    damage_modifiers,
};
```

3. **Update all damage calculation functions** to accept references:
   - `src/engine/combat/damage/calculator.rs`: Update `calculate_damage` signature
   - `src/engine/combat/damage/generations/*.rs`: Update all generation-specific calculators
   - `src/engine/combat/damage/modifiers/*.rs`: Update modifier functions

### 1.3 Use Global Move Registry Singleton

**Problem**: New registry created on every move in `src/engine/combat/moves/mod.rs:173`.

**Implementation Steps**:

1. **Make registry initialization lazy_static** (`src/engine/combat/moves/registry.rs`):
```rust
lazy_static! {
    static ref MOVE_REGISTRY: MoveRegistry = {
        let mut registry = MoveRegistry::new();
        registry.register_all_moves();
        registry
    };
}

pub fn get_move_registry() -> &'static MoveRegistry {
    &MOVE_REGISTRY
}
```

2. **Remove local registry creation** (`src/engine/combat/moves/mod.rs:173`):
```rust
// Replace:
// let registry = registry::MoveRegistry::new();
// With:
let registry = registry::get_move_registry();
```

3. **Update registry usage pattern**:
   - Make `MoveRegistry::register_all_moves()` private
   - Ensure thread-safe access (already using HashMap, which is fine for reads)

## Phase 2: Memory Optimization (Week 2)

### 2.1 Optimize Stat Boost Storage

**Problem**: HashMap for stat boosts uses ~80 bytes per Pokemon.

**Implementation Steps**:

1. **Create stat array type** (`src/types/stat.rs`):
```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct StatBoostArray([i8; 8]);

impl StatBoostArray {
    pub fn get(&self, stat: Stat) -> i8 {
        self.0[stat as usize]
    }
    
    pub fn set(&mut self, stat: Stat, value: i8) {
        self.0[stat as usize] = value.clamp(-6, 6);
    }
    
    pub fn modify(&mut self, stat: Stat, delta: i8) {
        let current = self.get(stat);
        self.set(stat, current + delta);
    }
    
    pub fn reset(&mut self) {
        self.0 = [0; 8];
    }
}
```

2. **Update Pokemon struct** (`src/core/battle_state/pokemon.rs`):
```rust
pub struct Pokemon {
    // Replace: pub stat_boosts: HashMap<Stat, i8>,
    pub stat_boosts: StatBoostArray,
    // ... other fields ...
}
```

3. **Update all stat boost accesses**:
   - Search for `stat_boosts.get(&stat)` → `stat_boosts.get(stat)`
   - Search for `stat_boosts.insert(stat, value)` → `stat_boosts.set(stat, value)`
   - Search for `stat_boosts.entry(stat)` → Use `modify()` method

### 2.2 Implement Volatile Status Bitflags

**Problem**: HashSet + HashMap for volatile statuses uses ~128 bytes.

**Implementation Steps**:

1. **Create bitflags type** (`src/types/status.rs`):
```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct VolatileStatusFlags: u64 {
        const CONFUSION = 1 << 0;
        const SUBSTITUTE = 1 << 1;
        const TAUNT = 1 << 2;
        const ENCORE = 1 << 3;
        const DISABLE = 1 << 4;
        const TRAPPED = 1 << 5;
        const INGRAIN = 1 << 6;
        const EMBARGO = 1 << 7;
        // ... up to 64 statuses
    }
}

impl VolatileStatus {
    pub fn to_flag(self) -> VolatileStatusFlags {
        match self {
            VolatileStatus::Confusion => VolatileStatusFlags::CONFUSION,
            VolatileStatus::Substitute => VolatileStatusFlags::SUBSTITUTE,
            // ... etc
        }
    }
}
```

2. **Create duration tracking array**:
```rust
#[derive(Clone, Debug, Default)]
pub struct VolatileStatusDurations {
    // Only track durations for statuses that need them
    confusion_turns: u8,
    encore_turns: u8,
    disable_turns: u8,
    taunt_turns: u8,
    // ... other timed statuses
}
```

3. **Update Pokemon struct**:
```rust
pub struct Pokemon {
    // Replace: pub volatile_statuses: HashSet<VolatileStatus>,
    // Replace: pub volatile_status_durations: HashMap<VolatileStatus, u8>,
    pub volatile_statuses: VolatileStatusFlags,
    pub volatile_durations: VolatileStatusDurations,
}
```

### 2.3 Optimize Move Storage

**Problem**: HashMap for moves uses ~200 bytes when Pokemon typically have 4 moves.

**Implementation Steps**:

1. **Add SmallVec dependency** (`Cargo.toml`):
```toml
smallvec = "1.11"
```

2. **Update Pokemon struct** (`src/core/battle_state/pokemon.rs`):
```rust
use smallvec::SmallVec;

pub struct Pokemon {
    // Replace: pub moves: HashMap<MoveIndex, Move>,
    pub moves: SmallVec<[(MoveIndex, Move); 4]>,
}
```

3. **Create helper methods**:
```rust
impl Pokemon {
    pub fn get_move(&self, index: MoveIndex) -> Option<&Move> {
        self.moves.iter()
            .find(|(idx, _)| *idx == index)
            .map(|(_, m)| m)
    }
    
    pub fn get_move_mut(&mut self, index: MoveIndex) -> Option<&mut Move> {
        self.moves.iter_mut()
            .find(|(idx, _)| *idx == index)
            .map(|(_, m)| m)
    }
}
```

## Phase 3: Type Safety Improvements (Week 3)

### 3.1 Replace String-Based Move Targeting

**Problem**: String parsing for move targets in `src/engine/turn.rs:17-34`.

**Implementation Steps**:

1. **Create target parsing enum** (`src/core/targeting.rs`):
```rust
#[derive(Debug, Clone, Copy)]
pub enum ParsedMoveTarget {
    User,
    Ally(u8),           // ally-1, ally-2
    Opponent(u8),       // opponent-1, opponent-2
    AllAllies,
    AllOpponents,
    All,
}

impl ParsedMoveTarget {
    pub fn from_str(s: &str) -> Result<Self, BattleError> {
        match s {
            "user" => Ok(ParsedMoveTarget::User),
            "ally-1" => Ok(ParsedMoveTarget::Ally(1)),
            "ally-2" => Ok(ParsedMoveTarget::Ally(2)),
            "opponent-1" => Ok(ParsedMoveTarget::Opponent(1)),
            "opponent-2" => Ok(ParsedMoveTarget::Opponent(2)),
            "all-allies" => Ok(ParsedMoveTarget::AllAllies),
            "all-opponents" => Ok(ParsedMoveTarget::AllOpponents),
            "all" => Ok(ParsedMoveTarget::All),
            _ => Err(BattleError::InvalidTarget(s.to_string())),
        }
    }
}
```

2. **Replace parse_move_target function**:
```rust
fn parse_move_target(
    move_target: &str,
    move_data: &MoveData,
    format: &BattleFormat,
    user_position: BattlePosition,
) -> Result<MoveTarget, BattleError> {
    let parsed = ParsedMoveTarget::from_str(move_target)?;
    
    match (parsed, move_data.target) {
        (ParsedMoveTarget::User, _) => Ok(MoveTarget::User),
        (ParsedMoveTarget::Ally(n), target) if target.affects_allies() => {
            Ok(MoveTarget::Ally(n))
        }
        // ... other cases with proper validation
        _ => Err(BattleError::InvalidTargetForMove {
            move_name: move_data.name.clone(),
            attempted_target: move_target.to_string(),
            valid_targets: move_data.target.valid_selections(),
        }),
    }
}
```

### 3.2 Implement FromNormalizedString for Enums

**Problem**: Manual string matching for Nature and Gender in `src/data/random_team_loader.rs`.

**Implementation Steps**:

1. **Implement for Nature** (`src/types/mod.rs` or appropriate location):
```rust
impl FromNormalizedString for Nature {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s {
            "hardy" => Some(Nature::Hardy),
            "lonely" => Some(Nature::Lonely),
            "brave" => Some(Nature::Brave),
            "adamant" => Some(Nature::Adamant),
            "naughty" => Some(Nature::Naughty),
            "bold" => Some(Nature::Bold),
            "docile" => Some(Nature::Docile),
            "relaxed" => Some(Nature::Relaxed),
            "impish" => Some(Nature::Impish),
            "lax" => Some(Nature::Lax),
            "timid" => Some(Nature::Timid),
            "hasty" => Some(Nature::Hasty),
            "serious" => Some(Nature::Serious),
            "jolly" => Some(Nature::Jolly),
            "naive" => Some(Nature::Naive),
            "modest" => Some(Nature::Modest),
            "mild" => Some(Nature::Mild),
            "quiet" => Some(Nature::Quiet),
            "bashful" => Some(Nature::Bashful),
            "rash" => Some(Nature::Rash),
            "calm" => Some(Nature::Calm),
            "gentle" => Some(Nature::Gentle),
            "sassy" => Some(Nature::Sassy),
            "careful" => Some(Nature::Careful),
            "quirky" => Some(Nature::Quirky),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec![
            "hardy", "lonely", "brave", "adamant", "naughty",
            "bold", "docile", "relaxed", "impish", "lax",
            "timid", "hasty", "serious", "jolly", "naive",
            "modest", "mild", "quiet", "bashful", "rash",
            "calm", "gentle", "sassy", "careful", "quirky",
        ]
    }
}
```

2. **Update deserializer** (`src/data/random_team_loader.rs`):
```rust
fn deserialize_optional_nature<'de, D>(deserializer: D) -> Result<Option<Nature>, D::Error>
where D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.and_then(|s| {
        let normalized = normalize_name(&s);
        Nature::from_normalized_str(&normalized)
    }))
}
```

3. **Implement for Gender**:
```rust
impl FromNormalizedString for Gender {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s {
            "m" | "male" => Some(Gender::Male),
            "f" | "female" => Some(Gender::Female),
            "n" | "genderless" => Some(Gender::Genderless),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec!["m", "male", "f", "female", "n", "genderless"]
    }
}
```

### 3.3 Remove Unwrap() Calls

**Problem**: Multiple unwrap() calls that could panic.

**Implementation Steps**:

1. **Fix battle_environment.rs logging**:
```rust
// Line 246-247, replace:
let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(self.log_file.as_ref().ok_or_else(|| BattleError::InvalidState {
        reason: "Log file path not set".to_string()
    })?)
    .map_err(|e| BattleError::InvalidState {
        reason: format!("Failed to open log file: {}", e)
    })?;
```

2. **Fix mutex handling** (`src/data/repositories/mod.rs:75`):
```rust
// Add error variant to DataError:
#[derive(Error, Debug)]
pub enum DataError {
    // ... existing variants ...
    #[error("Repository lock poisoned")]
    LockPoisoned,
}

// Update lock acquisition:
let mut repo = mutex.lock()
    .map_err(|_| DataError::LockPoisoned)?;
```

3. **Fix random team loader** (`src/data/random_team_loader.rs`):
```rust
// Lines 24-25, replace unwrap_or with proper error:
PokemonName::from_normalized_str(&normalize_name(s))
    .ok_or_else(|| de::Error::custom(format!("Unknown Pokemon: {}", s)))?
```

## Phase 4: Code Organization (Week 4)

### 4.1 Unify Move Effect Signatures

**Problem**: 5 different function signatures for move effects.

**Implementation Steps**:

1. **Create unified context** (`src/engine/combat/move_context.rs`):
```rust
pub struct MoveExecutionContext<'a> {
    pub state: &'a BattleState,
    pub move_data: &'a MoveData,
    pub user_position: BattlePosition,
    pub target_positions: &'a [BattlePosition],
    pub generation: &'a GenerationMechanics,
    pub move_context: &'a MoveContext,
    pub repository: &'a GameDataRepository,
    pub branch_on_damage: bool,
}

pub type MoveEffectFn = fn(&mut MoveExecutionContext) -> Vec<BattleInstructions>;
```

2. **Update MoveRegistry** (`src/engine/combat/moves/registry.rs`):
```rust
pub struct MoveRegistry {
    effects: HashMap<MoveName, MoveEffectFn>,
}

impl MoveRegistry {
    pub fn register(&mut self, move_name: MoveName, effect: MoveEffectFn) {
        self.effects.insert(move_name, effect);
    }
    
    pub fn get_effect(&self, move_name: &MoveName) -> Option<MoveEffectFn> {
        self.effects.get(move_name).copied()
    }
}
```

3. **Create adapter functions for existing moves**:
```rust
// For simple moves that don't use all parameters:
fn adapt_simple_move(
    original: fn(&BattleState, BattlePosition, &[BattlePosition], &MoveData) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.user_position, ctx.target_positions, ctx.move_data)
    }
}
```

### 4.2 Remove Duplicate apply_generic_effects Wrappers

**Problem**: Identical wrapper functions in multiple files.

**Implementation Steps**:

1. **Export main function** (`src/engine/combat/moves/mod.rs`):
```rust
pub use self::apply_generic_effects;
```

2. **Remove duplicate wrappers** from:
   - `src/engine/combat/moves/simple.rs:382-392`
   - `src/engine/combat/moves/damage/variable_power.rs:1559-1576`
   - `src/engine/combat/moves/special/two_turn.rs:619-629`
   - `src/engine/combat/moves/field/weather_accuracy.rs:64-76`
   - `src/engine/combat/moves/field/terrain_dependent.rs:231-243`
   - `src/engine/combat/moves/damage/self_targeting.rs:180-192`
   - `src/engine/combat/moves/status/item_interaction.rs:377-389`

3. **Update imports** in each file:
```rust
use crate::engine::combat::moves::apply_generic_effects;
```

### 4.3 Create Power Modifier Composer

**Problem**: Similar power modification patterns repeated across moves.

**Implementation Steps**:

1. **Create composer module** (`src/engine/combat/moves/composers/power_modifier.rs`):
```rust
pub struct PowerModifierBuilder {
    base_calculation: Box<dyn Fn(&BattleState, BattlePosition, &[BattlePosition]) -> f32>,
}

impl PowerModifierBuilder {
    pub fn new<F>(calc: F) -> Self 
    where F: Fn(&BattleState, BattlePosition, &[BattlePosition]) -> f32 + 'static {
        Self {
            base_calculation: Box::new(calc),
        }
    }
    
    pub fn with_weather_boost(self, weather: Weather, boost: f32) -> Self {
        // Chain modifiers
    }
    
    pub fn with_hp_threshold(self, thresholds: &[(f32, f32)]) -> Self {
        // Chain modifiers
    }
    
    pub fn build(self) -> impl Fn(&MoveExecutionContext) -> DamageModifiers {
        move |ctx| {
            let power_multiplier = (self.base_calculation)(
                ctx.state,
                ctx.user_position,
                ctx.target_positions
            );
            DamageModifiers {
                power_multiplier: Some(power_multiplier),
                ..Default::default()
            }
        }
    }
}
```

2. **Refactor HP-based moves** (Eruption, Water Spout, etc.):
```rust
// Instead of separate implementations:
pub fn eruption_effect(ctx: &mut MoveExecutionContext) -> Vec<BattleInstructions> {
    let modifier_fn = PowerModifierBuilder::new(|state, user_pos, _| {
        if let Some(user) = state.get_pokemon_at_position(user_pos) {
            let hp_ratio = user.hp as f32 / user.max_hp as f32;
            150.0 * hp_ratio
        } else {
            1.0
        }
    }).build();
    
    let modifiers = modifier_fn(ctx);
    apply_generic_effects_with_modifiers(ctx, modifiers)
}
```

## Phase 5: Performance Monitoring (Week 5)

### 5.1 Add Performance Benchmarks

**Implementation Steps**:

1. **Create benchmark module** (`benches/battle_performance.rs`):
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_damage_calculation(c: &mut Criterion) {
    c.bench_function("damage_calc_simple", |b| {
        let state = setup_test_state();
        b.iter(|| {
            calculate_damage(black_box(&state), /* ... */)
        });
    });
}

fn benchmark_turn_resolution(c: &mut Criterion) {
    c.bench_function("turn_resolution_singles", |b| {
        let mut state = setup_singles_battle();
        let choices = setup_test_choices();
        b.iter(|| {
            resolve_turn(black_box(&mut state), black_box(&choices))
        });
    });
}

criterion_group!(benches, benchmark_damage_calculation, benchmark_turn_resolution);
criterion_main!(benches);
```

2. **Add Criterion dependency** (`Cargo.toml`):
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "battle_performance"
harness = false
```

### 5.2 Pre-allocate Vec Capacities

**Implementation Steps**:

1. **Identify allocation hotspots**:
   - `src/engine/turn.rs`: Instruction vectors
   - `src/core/targeting.rs`: Target position vectors
   - `src/engine/combat/moves/mod.rs`: Effect instruction vectors

2. **Add capacity hints**:
```rust
// src/engine/turn.rs
let mut all_instructions = Vec::with_capacity(
    expected_moves * average_instructions_per_move
);

// src/core/targeting.rs
let mut targets = Vec::with_capacity(match move_target {
    MoveTarget::AllOpponents => 2,
    MoveTarget::All => 4,
    _ => 1,
});
```

## Implementation Schedule

**Week 1**: Phase 1 - Critical Performance Fixes
- Day 1-2: Repository caching
- Day 3-4: Damage context references
- Day 5: Move registry singleton

**Week 2**: Phase 2 - Memory Optimization
- Day 1-2: Stat boost arrays
- Day 3-4: Volatile status bitflags
- Day 5: Move storage optimization

**Week 3**: Phase 3 - Type Safety
- Day 1-2: Move targeting enums
- Day 3-4: FromNormalizedString implementations
- Day 5: Remove unwrap() calls

**Week 4**: Phase 4 - Code Organization
- Day 1-2: Unify move signatures
- Day 3: Remove duplicate wrappers
- Day 4-5: Power modifier composer

**Week 5**: Phase 5 - Performance Monitoring
- Day 1-2: Add benchmarks
- Day 3-4: Vec pre-allocation
- Day 5: Performance testing and validation

## Success Metrics

1. **Performance**: 
   - 80% reduction in memory allocations per turn
   - 50% reduction in turn resolution time
   - Zero repository loading during battle

2. **Code Quality**:
   - Zero unwrap() calls in production code
   - Consistent move effect signatures
   - Type-safe enum conversions

3. **Memory Usage**:
   - Pokemon struct size reduced from ~500 to ~200 bytes
   - DamageContext size reduced from ~1.5KB to ~100 bytes
   - Battle state mutations without full clones

## Risk Mitigation

1. **Backward Compatibility**: All changes maintain existing public APIs
2. **Testing**: cargo test -test move_categories should pass
3. **Performance Regression**: Benchmark before and after each optimization
4. **Gradual Rollout**: Implement changes in isolated modules first