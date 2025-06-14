# Pokemon Showdown Rust Port - Architecture Plan

## Overview

This document outlines the architecture plan for porting Pokemon Showdown's battle simulator to Rust, with a focus on state serialization and efficient turn undoing for AI/RL agent development.

## Core Design Principles

1. **State Immutability**: Battle state should be immutable during turn execution
2. **Efficient Serialization**: Fast binary serialization for state snapshots
3. **Turn-Level Undo**: Ability to efficiently revert to previous turn states
4. **Deterministic Randomness**: Reproducible battles using PRNG seeds
5. **Format Agnostic**: Support all Pokemon battle formats from the ground up

## Architecture Components

### 1. Core Data Structures

#### Battle State (`src/battle_state.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub turn: u32,
    pub sides: [Side; 4], // Support up to 4 players (FFA)
    pub field: Field,
    pub queue: ActionQueue,
    pub random: PRNGState,
    pub format: BattleFormat,
    pub ended: bool,
    pub winner: Option<SideId>,
}

impl BattleState {
    pub fn to_bytes(&self) -> Vec<u8>;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
    pub fn to_json(&self) -> String; // Human-readable
    pub fn from_json(json: &str) -> Result<Self, Error>;
}
```

#### Pokemon State (`src/pokemon.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct Pokemon {
    pub species: SpeciesData,
    pub hp: u16,
    pub max_hp: u16,
    pub stats: StatsTable,
    pub boosts: BoostsTable,
    pub status: Option<StatusCondition>,
    pub volatiles: HashMap<String, VolatileStatus>,
    pub moves: [MoveSlot; 4],
    pub ability: AbilityData,
    pub item: Option<ItemData>,
    pub types: [Type; 2],
    pub level: u8,
    pub gender: Gender,
    pub nature: Nature,
    pub ivs: StatsTable,
    pub evs: StatsTable,
}
```

#### Side Management (`src/side.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct Side {
    pub id: SideId,
    pub name: String,
    pub pokemon: Vec<Pokemon>,
    pub active: Vec<Option<PokemonRef>>,
    pub choice: Choice,
    pub conditions: HashMap<String, SideCondition>,
    pub fainted_last_turn: Option<PokemonRef>,
    pub fainted_this_turn: Option<PokemonRef>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Choice {
    pub actions: Vec<ChosenAction>,
    pub forced_switches_left: u8,
    pub cant_undo: bool,
    pub error: Option<String>,
}
```

### 2. Action System

#### Action Queue (`src/action_queue.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct ActionQueue {
    actions: Vec<Action>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Move(MoveAction),
    Switch(SwitchAction),
    MegaEvo(MegaAction),
    Terastallize(TeraAction),
    // ... other action types
}

impl ActionQueue {
    pub fn add_choice(&mut self, choice: ActionChoice);
    pub fn resolve_action(&mut self, choice: ActionChoice) -> Vec<Action>;
    pub fn sort_by_priority(&mut self, battle: &BattleState);
    pub fn execute_next(&mut self, battle: &mut BattleState) -> BattleResult;
}
```

### 3. Data Management System

#### Dex Interface (`src/dex/mod.rs`)
```rust
pub trait Dex {
    fn get_move(&self, id: &str) -> Option<&MoveData>;
    fn get_species(&self, id: &str) -> Option<&SpeciesData>;
    fn get_ability(&self, id: &str) -> Option<&AbilityData>;
    fn get_item(&self, id: &str) -> Option<&ItemData>;
    fn get_type_chart(&self) -> &TypeChart;
}

pub struct ShowdownDex {
    moves: HashMap<String, MoveData>,
    species: HashMap<String, SpeciesData>,
    abilities: HashMap<String, AbilityData>,
    items: HashMap<String, ItemData>,
    type_chart: TypeChart,
    generation: u8,
}
```

#### Pokemon Showdown Data Integration (`src/dex/showdown_data.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub type_: Type,
    pub category: MoveCategory,
    pub base_power: u16,
    pub accuracy: Option<u8>,
    pub pp: u8,
    pub target: MoveTarget,
    pub priority: i8,
    pub flags: MoveFlags,
    pub secondary: Option<SecondaryEffect>,
    pub crit_ratio: u8,
    pub multihit: Option<MultihitData>,
    pub drain: Option<[u8; 2]>, // [numerator, denominator]
    pub recoil: Option<[u8; 2]>,
    // ... other fields from PS data
}
```

### 4. PRNG System

#### Deterministic Random Number Generation (`src/prng.rs`)
```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum PRNGState {
    Sodium(SodiumRNG),
    Gen5(Gen5RNG),
}

impl PRNGState {
    pub fn from_seed(seed: &str) -> Self;
    pub fn next_u32(&mut self) -> u32;
    pub fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool;
    pub fn sample<T>(&mut self, items: &[T]) -> &T;
    pub fn shuffle<T>(&mut self, items: &mut [T]);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SodiumRNG {
    seed: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Gen5RNG {
    seed: [u16; 4],
}
```

### 5. Battle Engine Core

#### Battle Executor (`src/battle.rs`)
```rust
pub struct Battle {
    state: BattleState,
    dex: Box<dyn Dex>,
    turn_history: Vec<BattleSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BattleSnapshot {
    pub turn: u32,
    pub state: BattleState,
    pub actions_taken: Vec<Action>,
}

impl Battle {
    pub fn new(format: BattleFormat, teams: Vec<Team>, seed: Option<String>) -> Self;
    
    // Core battle flow
    pub fn add_choice(&mut self, side_id: SideId, choice: &str) -> Result<(), BattleError>;
    pub fn step(&mut self) -> BattleResult;
    pub fn make_choices(&mut self) -> BattleResult;
    
    // State management
    pub fn save_snapshot(&mut self);
    pub fn undo_to_turn(&mut self, turn: u32) -> Result<(), BattleError>;
    pub fn clone_state(&self) -> BattleState;
    
    // Serialization
    pub fn serialize_state(&self) -> Vec<u8>;
    pub fn deserialize_state(&mut self, data: &[u8]) -> Result<(), BattleError>;
    pub fn to_json(&self) -> String;
    pub fn from_json(json: &str, dex: Box<dyn Dex>) -> Result<Self, BattleError>;
}
```

### 6. Event System

#### Event Processing (`src/events.rs`)
```rust
pub enum BattleEvent {
    TurnStart(u32),
    PokemonDamage { target: PokemonRef, damage: u16, source: Option<PokemonRef> },
    PokemonFaint(PokemonRef),
    StatusApply { target: PokemonRef, status: StatusCondition },
    MoveUse { user: PokemonRef, move_id: String, targets: Vec<PokemonRef> },
    // ... many more event types
}

pub trait EventProcessor {
    fn process_event(&mut self, event: BattleEvent, state: &mut BattleState);
}
```

### 7. Move Execution Engine

#### Move Effects (`src/moves/mod.rs`)
```rust
pub trait MoveEffect {
    fn execute(&self, context: &MoveContext, state: &mut BattleState) -> Vec<BattleEvent>;
}

pub struct MoveContext {
    pub user: PokemonRef,
    pub targets: Vec<PokemonRef>,
    pub move_data: &MoveData,
    pub hit_count: u8,
    pub crit: bool,
}

pub struct MoveExecutor {
    effects: HashMap<String, Box<dyn MoveEffect>>,
}

impl MoveExecutor {
    pub fn execute_move(&self, context: MoveContext, state: &mut BattleState) -> Vec<BattleEvent>;
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-2)
- [ ] Basic data structures (BattleState, Pokemon, Side)
- [ ] Serialization/deserialization with `serde`
- [ ] PRNG system implementation
- [ ] Basic Dex interface

### Phase 2: Action System (Weeks 3-4)
- [ ] ActionQueue implementation
- [ ] Choice parsing and validation
- [ ] Priority ordering system
- [ ] Basic action execution

### Phase 3: Move System (Weeks 5-6)
- [ ] Move effect framework
- [ ] Damage calculation
- [ ] Status effects and conditions
- [ ] Target resolution

### Phase 4: Battle Engine (Weeks 7-8)
- [ ] Turn-based battle loop
- [ ] Event system
- [ ] State snapshots and undo
- [ ] Format support (Singles, Doubles, VGC)

### Phase 5: Data Integration (Weeks 9-10)
- [ ] Pokemon Showdown data loader
- [ ] Type effectiveness
- [ ] Ability system
- [ ] Item effects

### Phase 6: Testing & Validation (Weeks 11-12)
- [ ] Port PS test suite to Rust
- [ ] Battle replay verification
- [ ] Performance benchmarks
- [ ] AI/RL integration examples

## Key Technical Decisions

### Serialization Strategy
- **Binary**: Use `bincode` for fast state snapshots
- **JSON**: Use `serde_json` for human-readable debugging
- **Compression**: Optional `lz4` compression for state storage

### Memory Management
- Use `Rc<RefCell<T>>` or `Arc<RwLock<T>>` for shared references
- Consider arena allocation for Pokemon/Move objects
- Implement Copy-on-Write for undo functionality

### Error Handling
- Custom error types for different failure modes
- `Result<T, BattleError>` for all fallible operations
- Graceful degradation for non-critical failures

### Performance Optimizations
- Pre-allocate common data structures
- Use string interning for identifiers
- Lazy loading of complex move effects
- SIMD for damage calculations where applicable

## Testing Strategy

### Unit Tests
- Individual component testing (moves, abilities, status conditions)
- State serialization round-trip tests
- PRNG determinism verification

### Integration Tests
- Full battle simulations
- Ported Pokemon Showdown test cases
- Cross-validation with PS reference implementation

### Performance Tests
- State serialization/deserialization benchmarks
- Turn execution speed tests
- Memory usage profiling

## AI/RL Integration Points

### State Representation
```rust
pub trait StateEncoder {
    fn encode_state(&self, state: &BattleState) -> Vec<f32>;
    fn decode_state(&self, encoded: &[f32]) -> BattleState;
}
```

### Action Space
```rust
pub trait ActionSpace {
    fn legal_actions(&self, state: &BattleState, side: SideId) -> Vec<ActionChoice>;
    fn action_from_index(&self, index: usize) -> ActionChoice;
    fn index_from_action(&self, action: &ActionChoice) -> usize;
}
```

### Rollout Interface
```rust
pub trait Rollout {
    fn step(&mut self, action: ActionChoice) -> (BattleState, f32, bool); // state, reward, done
    fn reset(&mut self) -> BattleState;
    fn fork(&self) -> Box<dyn Rollout>; // For tree search
}
```

## Success Metrics

1. **Correctness**: 100% compatibility with PS test suite
2. **Performance**: <1ms per turn execution
3. **Memory**: <100MB for typical battle state
4. **Serialization**: <10μs for state snapshot
5. **Undo**: <1μs to revert one turn

This architecture provides a solid foundation for a high-performance Pokemon battle simulator that supports the AI/RL use cases while maintaining full compatibility with Pokemon Showdown's mechanics.