# Engine Module Documentation

The engine module implements the core battle mechanics for Tapu Simu's multi-format Pokemon battle simulation. It provides sophisticated damage calculation, move effects, mechanics integration, and turn resolution with full support for Generations 1-9 and comprehensive multi-format awareness.

## Architecture Overview

The engine module (`src/engine/`) consists of four main components:

- **Combat System** (`combat/`) - Damage calculation, move effects, and battle mechanics
- **Mechanics Integration** (`mechanics/`) - Items, abilities, and switch effects
- **Targeting System** (`targeting/`) - Auto-targeting with Pokemon Showdown compatibility
- **Turn Resolution** (`turn.rs`) - Turn order and instruction generation

## Combat System (`combat/`)

The combat system is the heart of the engine, implementing Pokemon's complex battle mechanics with full generation support and format awareness.

### Module Organization

**Core Architecture:**
```rust
pub mod damage;           // Damage calculation with generation-specific formulas
pub mod damage_context;   // Modern context system for damage calculations
pub mod move_context;     // Move execution context and opponent information
pub mod move_effects;     // Legacy move effects for compatibility
pub mod moves;           // Modern move effects organized by category
pub mod type_effectiveness; // Type chart with generation variations
pub mod composers;       // Reusable move effect patterns
pub mod core;           // Centralized battle systems
```

### Damage Calculation System (`damage/`)

The damage system implements Pokemon's authentic damage calculation with generation-specific mechanics and full multi-format support.

**Core Components:**
```rust
// Main damage calculator entry point
pub fn calculate_damage_with_positions(
    state: &BattleState,
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    is_critical: bool,
    damage_rolls: DamageRolls,
    target_count: u8,
    attacker_position: BattlePosition,
    defender_position: BattlePosition,
) -> i16
```

**Key Features:**
- **16-Roll Damage Variance**: Authentic Pokemon damage variance (85%-100% in discrete steps)
- **Critical Hit Mechanics**: Generation-specific probability calculations and damage multipliers
- **Multi-Format Support**: Position-aware calculations for Singles, Doubles, VGC, Triples
- **Type Effectiveness**: Complete type chart with generation-specific changes

**Generation-Specific Implementations (`damage/generations/`):**
```rust
// Generation dispatch system
pub mod dispatcher;  // Routes calculations to appropriate generation
pub mod gen1;       // Special stat handling, different type effectiveness
pub mod gen2;       // Steel/Dark types, modern formula foundation
pub mod gen3;       // Abilities integration
pub mod gen4;       // Physical/Special split
pub mod gen56;      // Fairy type, Mega Evolution
pub mod modern;     // Gen 7-9 with Z-moves, Dynamax, Terastallization
```

**Damage Modifiers (`damage/modifiers/`):**
- **Abilities** (`abilities.rs`): Thick Fat, Filter, Solid Rock, Adaptability
- **Items** (`items.rs`): Life Orb, Choice items, type-boosting items
- **Weather** (`weather.rs`): Rain/sun damage modifications
- **Terrain** (`terrain.rs`): Electric/Grassy/Psychic/Misty terrain effects
- **Field** (`field.rs`): Trick Room, Gravity, global effects
- **Format** (`format.rs`): Multi-target spread move penalties

### Move Effects System (`moves/`)

Comprehensive move effect system with 200+ implemented moves organized by category and complexity.

**Registry Architecture (`moves/registry.rs`):**
```rust
pub struct MoveRegistry {
    effects: HashMap<Moves, MoveEffectFn>,
}

impl MoveRegistry {
    pub fn new() -> Self; // Pre-allocates capacity for ~200 moves
    fn register_all_moves(&mut self); // Registers all move implementations
    pub fn get_effect(&self, move_name: Moves) -> Option<&MoveEffectFn>;
}
```

**Move Categories:**

#### Damage Moves (`moves/damage/`)
- **Variable Power** (`variable_power.rs`): Context-dependent power (Facade, Hex, Gyro Ball, Avalanche)
- **Multi-Hit** (`multi_hit.rs`): Multi-strike moves (Fury Attack, Scale Shot, Bullet Seed)
- **Fixed Damage** (`fixed_damage.rs`): Level-based damage (Seismic Toss, Night Shade)
- **Self-Targeting** (`self_targeting.rs`): User-affecting damage moves

#### Status Moves (`moves/status/`)
- **Stat Modifying** (`stat_modifying.rs`): Boost/reduction moves (Swords Dance, Growl, Amnesia)
- **Status Effects** (`status_effects.rs`): Status application (Thunder Wave, Will-O-Wisp, Sleep Powder)
- **Healing** (`healing.rs`): HP restoration (Recover, Roost, Heal Pulse)
- **Item Interaction** (`item_interaction.rs`): Item effects (Knock Off, Trick, Switcheroo)

#### Field Moves (`moves/field/`)
- **Weather** (`weather.rs`): Weather setup (Rain Dance, Sunny Day, Sandstorm)
- **Hazards** (`hazards.rs`): Entry hazards (Stealth Rock, Spikes, Toxic Spikes)
- **Advanced Hazards** (`advanced_hazards.rs`): Complex hazard mechanics
- **Hazard Removal** (`hazard_removal.rs`): Field clearing (Rapid Spin, Defog, Tidy Up)
- **Screens** (`screens.rs`): Damage reduction (Light Screen, Reflect, Aurora Veil)
- **Terrain Dependent** (`terrain_dependent.rs`): Terrain-based moves
- **Weather Accuracy** (`weather_accuracy.rs`): Weather-dependent accuracy

#### Special Moves (`moves/special/`)
- **Complex** (`complex.rs`): Advanced move mechanics
- **Counter** (`counter.rs`): Damage reflection (Counter, Mirror Coat, Metal Burst)
- **Two Turn** (`two_turn.rs`): Charge mechanics (Solar Beam, Fly, Skull Bash)
- **Priority** (`priority.rs`): Speed modification (Quick Attack, Bullet Punch)
- **Protection** (`protection.rs`): Damage prevention (Protect, Detect, King's Shield)
- **Type Changing** (`type_changing.rs`): Type modification moves
- **Form Dependent** (`form_dependent.rs`): Forme-specific moves
- **Substitute** (`substitute.rs`): Substitute interaction mechanics

### Core Battle Systems (`combat/core/`)

Centralized systems managing battle mechanics across all move types.

**System Components:**
```rust
pub mod damage_system;        // Unified damage calculation entry point
pub mod status_system;        // Major and volatile status management
pub mod contact_effects;      // Post-contact ability triggers
pub mod move_prevention;      // Move blocking mechanics
pub mod field_system;         // Weather, terrain, and global effects
pub mod substitute_protection; // Substitute interaction handling
pub mod end_of_turn;          // Comprehensive end-of-turn pipeline
pub mod ability_triggers;     // Ability activation management
```

**Damage System (`damage_system.rs`):**
```rust
pub fn apply_damage_with_instructions(
    state: &BattleState,
    attacker_position: BattlePosition,
    target_positions: &[BattlePosition],
    move_data: &MoveData,
    branch_on_damage: bool,
) -> Vec<BattleInstructions>
```

**Status System (`status_system.rs`):**
- Major status application (Burn, Paralysis, Sleep, Freeze, Poison)
- Volatile status management (60+ status types with duration tracking)
- Status prevention and cure mechanics
- Status interaction handling

**Contact Effects (`contact_effects.rs`):**
- Post-contact ability triggers (Static, Flame Body, Rough Skin)
- Contact move detection and effect application
- Substitute interaction with contact effects

**Move Prevention (`move_prevention.rs`):**
- Status condition move blocking (Paralysis, Sleep, Freeze)
- Choice item move locking
- Taunt and Torment effect enforcement
- Disable and Encore move restrictions

**End-of-Turn Processing (`end_of_turn.rs`):**
```rust
pub fn generate_end_of_turn_instructions(state: &BattleState) -> Vec<BattleInstructions> {
    // Comprehensive end-of-turn pipeline:
    // 1. Remove expiring volatile statuses
    // 2. Weather effects and damage
    // 3. Terrain effects and damage
    // 4. Field effect timer decrementation
    // 5. Status condition damage (Burn, Poison)
    // 6. Ability end-of-turn triggers
    // 7. Item end-of-turn effects (Leftovers, Black Sludge)
}
```

### Effect Composition (`combat/composers/`)

Reusable patterns for common move effect types, reducing code duplication.

**Composer Categories:**
```rust
pub mod damage_moves;  // Standard damage move patterns
pub mod status_moves;  // Status effect application patterns  
pub mod field_moves;   // Field effect establishment patterns
```

**Damage Move Composers (`composers/damage_moves.rs`):**
```rust
pub fn simple_damage_move(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>

pub fn condition_dependent_power_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    condition_check: Box<dyn Fn(&BattleState, BattlePosition, BattlePosition) -> bool>,
    power_multiplier: f32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>
```

**Status Move Composers (`composers/status_moves.rs`):**
- Stat modification patterns with boost clamping
- Status condition application with immunity checking
- Healing patterns with HP clamping and percentage calculations

**Field Move Composers (`composers/field_moves.rs`):**
- Weather and terrain establishment patterns
- Hazard placement with position-based effects
- Screen establishment with damage reduction setup

### Context Systems

**Damage Context (`damage_context.rs`):**
```rust
#[derive(Debug, Clone)]
pub struct DamageContext {
    pub attacker: AttackerContext,
    pub defender: DefenderContext,
    pub move_context: MoveContext,
    pub field_context: FieldContext,
    pub format_context: FormatContext,
}
```

**Move Context (`move_context.rs`):**
```rust
#[derive(Debug, Clone)]
pub struct MoveContext {
    pub opponent_moves: Vec<OpponentMoveInfo>,
    pub is_first_turn: bool,
    pub consecutive_uses: u8,
}

pub struct OpponentMoveInfo {
    pub move_name: Moves,
    pub power: u8,
    pub category: MoveCategory,
    pub user_position: BattlePosition,
}
```

## Mechanics Integration (`mechanics/`)

### Abilities System (`abilities.rs`)

Comprehensive ability effect system with context-aware triggers.

**Ability Categories:**
```rust
pub fn apply_ability_effects(
    ability_name: &str,
    context: &AbilityContext,
    state: &BattleState,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>
```

**Ability Types:**
- **Type Immunities**: Levitate, Flash Fire, Water Absorb, Volt Absorb
- **Damage Modification**: Thick Fat, Filter, Solid Rock, Prism Armor
- **Stat Effects**: Intimidate, Download, Contrary, Simple
- **STAB Changes**: Normalize, Aerilate, Pixilate, Refrigerate
- **Weather Abilities**: Drought, Drizzle, Sand Stream, Snow Warning
- **Speed Control**: Quick Feet, Swift Swim, Chlorophyll, Sand Rush

### Items System (`mechanics/items/`)

Comprehensive item effect system organized by functionality.

**Item Categories:**

#### Choice Items (`choice_items.rs`)
```rust
pub fn apply_choice_band_effects(/* ... */) -> Vec<BattleInstruction>;
pub fn apply_choice_specs_effects(/* ... */) -> Vec<BattleInstruction>;
pub fn apply_choice_scarf_effects(/* ... */) -> Vec<BattleInstruction>;
```

#### Type Boosting Items (`type_boosting_items.rs`)
- Type gems, plates, memories with damage modification
- Z-crystals for Z-move activation (Gen 7+)

#### Stat Boosting Items (`stat_boosting_items.rs`)
- Life Orb: 30% damage boost with 10% recoil
- Expert Belt: Super effective move boost
- Muscle Band/Wise Glasses: Category-specific boosts

#### Berry Items (`berry_items.rs`)
- Sitrus Berry, Oran Berry: HP restoration
- Lum Berry: Status cure
- Pinch berries: Conditional stat boosts

#### Status Items (`status_items.rs`)
- Black Sludge, Leftovers: End-of-turn healing
- Flame Orb, Toxic Orb: Status self-infliction

#### Utility Items (`utility_items.rs`)
- Focus Sash: Survival mechanics
- Air Balloon: Ground immunity
- Assault Vest: Special Defense boost with move restrictions

#### Species Items (`species_items.rs`)
- Thick Club: Cubone/Marowak attack boost
- Light Ball: Pikachu stat doubling
- Eviolite: NFE Pokemon defensive boost

### Switch Effects (`switch_effects.rs`)

Entry and exit effect management for Pokemon switching.

**Entry Effects:**
```rust
pub fn apply_entry_effects(
    state: &BattleState,
    position: BattlePosition,
    new_pokemon: &Pokemon,
) -> Vec<BattleInstruction>
```

**Exit Effects:**
```rust
pub fn apply_exit_effects(
    state: &BattleState,
    position: BattlePosition,
    switching_pokemon: &Pokemon,
) -> Vec<BattleInstruction>
```

**Effect Types:**
- Intimidate stat reduction
- Weather and terrain establishment (Drought, Drizzle, etc.)
- Hazard damage application (Stealth Rock, Spikes)
- Healing Wish and Memento activation
- U-turn/Volt Switch momentum mechanics

## Targeting System (`targeting/`)

Auto-targeting system with Pokemon Showdown compatibility for AI and default behaviors.

**Core Implementation (`auto_targeting.rs`):**
```rust
pub struct AutoTargetingEngine {
    format: BattleFormat,
}

impl AutoTargetingEngine {
    pub fn resolve_targets(
        &self,
        target: MoveTarget,
        user_position: BattlePosition,
        state: &BattleState,
    ) -> Vec<BattlePosition>
}
```

**Target Resolution Strategies:**
- **Single Targets**: Normal, Adjacent, Any, Self with format-aware selection
- **Multi-Targets**: AllAdjacentFoes, AllAdjacent with spread move detection
- **Special Targets**: Scripted (Counter), RandomNormal with context awareness
- **Validation**: Ensures targets are valid for move type and battle state

**Format Integration:**
- **Singles**: Direct opponent targeting (always position 0)
- **Doubles**: Adjacent position preference with ally detection
- **VGC**: Tournament-specific targeting rules and restrictions
- **Triples**: Complex adjacency rules with three-position relationships

## Turn Resolution (`turn.rs`)

Simplified turn processing with instruction generation and state mutation tracking.

**Core Turn Function:**
```rust
pub fn generate_instructions(
    state: &BattleState,
    move_choices: (&MoveChoice, &MoveChoice),
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>>
```

**Turn Flow:**
1. **Auto-Target Resolution**: Resolve any unspecified targets using unified targeting system
2. **Move Order Determination**: Priority, speed, special cases (Pursuit + switch)
3. **Context Creation**: Opponent move information for context-aware moves
4. **Instruction Generation**: Convert moves to atomic battle instructions
5. **End-of-Turn Processing**: Comprehensive effect resolution pipeline

**End-of-Turn Sequence:**
```rust
pub mod end_of_turn {
    pub fn process_end_of_turn_effects(state: &BattleState) -> Vec<BattleInstructions> {
        crate::engine::combat::core::end_of_turn::generate_end_of_turn_instructions(state)
    }
}
```

**Special Cases:**
- **Pursuit Interaction**: Switch cancellation with damage calculation
- **Speed Tie Resolution**: Consistent random resolution
- **Multi-Target Priority**: Individual target priority calculation
- **Faint Handling**: Mid-turn replacement and effect continuation

### Generation Abstraction
Generation-specific mechanics are isolated with common interfaces:
```rust
pub trait GenerationMechanics {
    fn calculate_damage(&self, context: &DamageContext) -> u16;
    fn apply_status_effects(&self, status: &str, position: BattlePosition) -> Vec<BattleInstruction>;
    fn resolve_critical_hit(&self, attacker: &Pokemon, move_data: &MoveData) -> bool;
}
```

### Registry Pattern
Move effects are centrally registered rather than using large match statements:
```rust
// Modern approach with registry
let effect_fn = registry.get_effect(move_name)?;
let instructions = effect_fn.execute(context)?;

// vs. Legacy approach with match statements
match move_name {
    Moves::THUNDERBOLT => apply_thunderbolt(/* ... */),
    Moves::FLAMETHROWER => apply_flamethrower(/* ... */),
    // ... 200+ more cases
}
```

### Composer Pattern
Common move patterns are abstracted into reusable functions, combining core systems to reduce duplication across similar move implementations.