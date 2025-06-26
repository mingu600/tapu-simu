# Engine Module Documentation

The engine module implements the core battle mechanics for Tapu Simu's multi-format Pokemon battle simulation. It provides sophisticated damage calculation, move effects, mechanics integration, and turn resolution with full support for Generations 1-9.

## Architecture Overview

The engine module consists of four main components:
- **Combat System**: Damage calculation, move effects, and battle mechanics
- **Mechanics Integration**: Items, abilities, and switch effects
- **Targeting System**: Auto-targeting with Pokemon Showdown compatibility
- **Turn Resolution**: Turn order and instruction generation

## Combat System (`combat/`)

### Damage Calculation (`damage_calc.rs` & `damage/`)

The damage system implements Pokemon's authentic damage calculation with generation-specific mechanics and full multi-format support.

**Core Components:**
- `DamageCalculationContext`: Encapsulates attacker, defender, move, field, and format state
- `DamageRolls`: Enum controlling damage variance (Average, Min, Max, All 16 rolls)
- Generation-specific modules (`gen1.rs` through `modern.rs`)

**Key Features:**
- **16-Roll System**: Authentic Pokemon damage variance (85%-100% in discrete steps)
- **Critical Hit Mechanics**: Generation-specific probability calculations
- **Position-Based**: All calculations use explicit `BattlePosition` targeting
- **Modifier Pipeline**: STAB, type effectiveness, weather, abilities, items in correct order

**Generation Differences:**
```rust
// Gen 1: Different rounding and special mechanics
fn gen1_damage_formula(context: &DamageContext) -> u16

// Gen 2: Item introduction and modern base formula
fn gen2_damage_formula(context: &DamageContext) -> u16

// Gen 3+: Ability integration and modern modifier system
fn modern_damage_formula(context: &DamageContext) -> u16
```

**Critical Hit Evolution:**
- **Gen 1**: Speed-based formula with base stat dependency
- **Gen 2**: Fixed stage system (17/256 base rate)
- **Gen 3+**: Modern stage-based system with ability interactions

### Move Effects System (`moves/`)

Registry-based move effect system replacing large match statements with organized, composable functions.

**Registry Architecture:**
```rust
// Function type hierarchy for different complexity levels
type MoveEffectFn = fn(&BattleState, BattlePosition, &[BattlePosition], &GenerationMechanics) -> Vec<BattleInstructions>;

type ContextAwareMoveEffectFn = fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, &MoveContext, bool) -> Vec<BattleInstructions>;
```

**Move Categories:**

#### Damage Moves (`damage/`)
- **Simple Damage**: Basic attacking moves with standard calculation
- **Variable Power**: Context-dependent power (Facade, Hex, Gyro Ball, Avalanche)
- **Multi-Hit**: Fury Attack, Scale Shot with hit count determination
- **Drain Moves**: Giga Drain, Leech Life with HP recovery
- **Recoil Moves**: Take Down, Flare Blitz with self-damage
- **Self-Destruct**: Explosion, Self-Destruct with fainting

#### Status Moves (`status/`)
- **Stat Modifying**: Swords Dance, Growl with boost application
- **Status Effects**: Thunder Wave, Will-O-Wisp with condition application
- **Healing**: Recover, Roost with HP restoration
- **Item Interaction**: Knock Off, Trick with item manipulation

#### Field Moves (`field/`)
- **Weather**: Rain Dance, Sunny Day with weather establishment
- **Terrain**: Electric Terrain, Grassy Terrain with terrain setting
- **Hazards**: Stealth Rock, Spikes with entry hazard placement
- **Hazard Removal**: Rapid Spin, Defog with hazard clearing
- **Screens**: Light Screen, Reflect with damage reduction

#### Special Moves (`special/`)
- **Complex Mechanics**: Body Press (uses Defense for Attack), Foul Play (uses target's Attack)
- **Counter Moves**: Counter, Mirror Coat with damage reflection
- **Protection**: Protect, Detect with priority-based protection
- **Two-Turn**: Solar Beam, Fly with charge-then-execute pattern
- **Form Changes**: Revelation Dance with user type adaptation

### Core Battle Systems (`core/`)

Centralized systems managing battle mechanics across all move types.

**Damage System (`damage_system.rs`):**
- Unified damage calculation entry point
- Multi-target damage with individual target calculation
- Critical hit determination and damage variance
- Secondary effect application post-damage

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

**Field System (`field_system.rs`):**
- Weather and terrain effect application
- Global effects (Trick Room, Gravity) management
- Turn-based effect duration tracking
- Field condition interactions

### Effect Composition (`composers/`)

Reusable patterns for common move effect types, reducing code duplication.

**Damage Move Composers:**
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

**Status Move Composers:**
- Stat modification patterns with boost clamping
- Status condition application with immunity checking
- Healing patterns with HP clamping and percentage calculations

**Field Move Composers:**
- Weather and terrain establishment patterns
- Hazard placement with position-based effects
- Screen establishment with damage reduction setup

## Mechanics Integration (`mechanics/`)

### Items System (`items/`)

Comprehensive item effect system organized by functionality.

**Item Categories:**
- **Choice Items**: Choice Band, Choice Specs, Choice Scarf with move locking
- **Type Boosting**: Type gems, plates, memories with damage modification
- **Stat Boosting**: Life Orb, Expert Belt with power increases
- **Berry Items**: Sitrus Berry, Lum Berry with consumption triggers
- **Status Items**: Black Sludge, Leftovers with end-of-turn effects
- **Utility Items**: Focus Sash, Air Balloon with battle mechanic changes
- **Species Items**: Thick Club, Light Ball with species-specific boosts

**Generation Integration:**
```rust
pub fn apply_item_effects(
    item_name: &str,
    state: &BattleState,
    position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>
```

### Abilities System (`abilities.rs`)

Comprehensive ability effect system with context-aware triggers.

**Ability Categories:**
- **Type Immunities**: Levitate, Flash Fire, Water Absorb
- **Damage Modification**: Thick Fat, Filter, Solid Rock
- **Stat Boosts**: Intimidate, Download, Contrary
- **STAB Changes**: Normalize, Aerilate, Pixilate
- **Weather Abilities**: Drought, Drizzle, Sand Stream
- **Speed Control**: Quick Feet, Swift Swim, Chlorophyll

**Context Integration:**
```rust
pub fn apply_ability_effects(
    ability_name: &str,
    context: &AbilityContext,
    state: &BattleState,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction>
```

### Switch Effects (`switch_effects.rs`)

Entry and exit effect management for Pokemon switching.

**Entry Effects:**
- Intimidate stat reduction
- Weather and terrain establishment
- Hazard damage application
- Ability activation triggers

**Exit Effects:**
- Healing Wish activation
- Memento stat reduction
- U-turn/Volt Switch momentum

## Targeting System (`targeting/`)

Auto-targeting system with Pokemon Showdown compatibility for AI and default behaviors.

**Core Components:**
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

**Target Resolution:**
- **Single Targets**: Normal, Adjacent, Any, Self with format-aware selection
- **Multi-Targets**: AllAdjacentFoes, AllAdjacent with spread move detection
- **Special Targets**: Scripted (Counter), RandomNormal with context awareness
- **Validation**: Ensures targets are valid for move type and battle state

**Format Integration:**
- Singles: Direct opponent targeting
- Doubles: Adjacent position preference with ally detection
- VGC: Tournament-specific targeting rules
- Triples: Complex adjacency rules with position relationships

## Turn Resolution (`turn.rs`)

Comprehensive turn processing with instruction generation and state mutation tracking.

**Turn Flow:**
1. **Move Order Determination**: Priority, speed, special cases (Pursuit + switch)
2. **Context Creation**: Opponent move information for context-aware moves  
3. **Instruction Generation**: Convert moves to atomic battle instructions
4. **State Application**: Apply instructions sequentially for accurate progression
5. **End-of-Turn Processing**: Comprehensive effect resolution pipeline

**End-of-Turn Sequence:**
```rust
pub fn process_end_of_turn(
    state: &mut BattleState,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    // 1. Remove expiring volatile statuses
    // 2. Weather effects and damage
    // 3. Terrain effects and damage
    // 4. Field effect timer decrementation
    // 5. Status condition damage (Burn, Poison)
    // 6. Ability end-of-turn triggers
    // 7. Item end-of-turn effects (Leftovers, Black Sludge)
}
```

**Special Cases:**
- **Pursuit Interaction**: Switch cancellation with damage calculation
- **Speed Tie Resolution**: Consistent random resolution
- **Multi-Target Priority**: Individual target priority calculation
- **Faint Handling**: Mid-turn replacement and effect continuation

## Key Design Patterns

### Position-Based Architecture
Every calculation and effect explicitly uses `BattlePosition` rather than implicit targeting, enabling seamless multi-format support.

### Instruction-Based State Mutation
All battle changes are atomic `BattleInstruction` objects, allowing probability branching and deterministic replay.

### Context Encapsulation
Complex information is packaged into context objects to eliminate large parameter lists:
- `DamageCalculationContext`: Attacker, defender, move, field state
- `MoveContext`: Opponent moves, battle history, secondary information
- `AbilityContext`: Trigger conditions, relevant Pokemon, timing information

### Generation Abstraction
Generation-specific mechanics are isolated with common interfaces:
```rust
pub trait GenerationMechanics {
    fn calculate_damage(&self, context: &DamageContext) -> u16;
    fn apply_status_effects(&self, status: &str, position: BattlePosition) -> Vec<BattleInstruction>;
    fn resolve_critical_hit(&self, attacker: &Pokemon, move_data: &MoveData) -> bool;
}
```

### Composer Pattern
Common move patterns are abstracted into reusable functions, combining core systems to reduce duplication across similar move implementations.

## Integration Points

The engine module integrates with:
- **Core Module**: Battle state, instructions, move choices
- **Data Module**: Pokemon, move, ability, and item data
- **Generation Module**: Mechanics variations across generations
- **Testing Module**: Battle simulation and effect verification