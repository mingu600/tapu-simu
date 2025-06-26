# Constants Module Documentation

The constants module centralizes all game constants and magic numbers used throughout Tapu Simu, eliminating hardcoded values and improving maintainability. It provides organized constants for damage calculation, move effects, item mechanics, and type interactions.

## Architecture Overview

The constants module consists of two main categories:
- **Move Constants**: Damage calculation, move power, status effects, and type interactions
- **Item Constants**: Item multipliers, thresholds, and effect parameters

## Move Constants (`moves.rs`)

Comprehensive constants for move effects, damage calculation, and battle mechanics.

### Damage Calculation Constants

**Damage Variance System:**
```rust
/// Standard damage variance range (85% to 100% of calculated damage)
pub const DAMAGE_VARIANCE_MIN: f32 = 0.85;
pub const DAMAGE_VARIANCE_MAX: f32 = 1.0;

/// Number of damage rolls for variance calculation
pub const DAMAGE_ROLL_COUNT: usize = 16;

/// Damage roll increment (1% per roll)
pub const DAMAGE_ROLL_INCREMENT: f32 = 0.01;
```

**Critical Hit Mechanics:**
```rust
/// Critical hit multiplier for most generations
pub const CRITICAL_HIT_MULTIPLIER: f32 = 1.5;
```

**Damage Bounds:**
```rust
/// Minimum damage percentage (85%)
pub const MIN_DAMAGE_PERCENT: u8 = 85;

/// Minimum damage (1 HP)
pub const MIN_DAMAGE: i16 = 1;
```

### Variable Power Move Constants

**HP-Based Power Calculation:**
```rust
/// HP thresholds for Reversal and Flail power calculation
pub const REVERSAL_HP_THRESHOLDS: &[(f32, u16)] = &[
    (0.0208, 200),   // <= 1/48 HP = 200 power
    (0.0417, 150),   // <= 1/24 HP = 150 power  
    (0.1042, 100),   // <= 1/9.6 HP = 100 power
    (0.2083, 80),    // <= 1/4.8 HP = 80 power
    (0.3542, 40),    // <= 17/48 HP = 40 power
    (1.0, 20),       // > 17/48 HP = 20 power
];
```

**Weight-Based Power Calculation:**
```rust
/// Weight thresholds for Grass Knot and Low Kick power calculation
pub const WEIGHT_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (200.0, 120),    // >= 200.0 kg = 120 power
    (100.0, 100),    // >= 100.0 kg = 100 power
    (50.0, 80),      // >= 50.0 kg = 80 power
    (25.0, 60),      // >= 25.0 kg = 60 power
    (10.0, 40),      // >= 10.0 kg = 40 power
    (0.0, 20),       // < 10.0 kg = 20 power
];

/// Weight ratio thresholds for Heat Crash and Heavy Slam power calculation
pub const WEIGHT_RATIO_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (5.0, 120),      // >= 5x weight ratio = 120 power
    (4.0, 100),      // >= 4x weight ratio = 100 power  
    (3.0, 80),       // >= 3x weight ratio = 80 power
    (2.0, 60),       // >= 2x weight ratio = 60 power
    (0.0, 40),       // < 2x weight ratio = 40 power
];
```

**Speed-Based Power Calculation:**
```rust
/// Speed ratio thresholds for Electro Ball power calculation
pub const SPEED_RATIO_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (4.0, 150),      // >= 4x speed ratio = 150 power
    (3.0, 120),      // >= 3x speed ratio = 120 power
    (2.0, 80),       // >= 2x speed ratio = 80 power
    (1.0, 60),       // >= 1x speed ratio = 60 power
    (0.0, 40),       // < 1x speed ratio = 40 power
];
```

### Status Effect Constants

**Standard Status Probabilities:**
```rust
/// Standard burn chance for moves like Flamethrower
pub const BURN_CHANCE_STANDARD: u8 = 10;

/// Standard paralysis chance for moves like Thunderbolt
pub const PARALYSIS_CHANCE_STANDARD: u8 = 10;

/// Standard freeze chance for moves like Ice Beam
pub const FREEZE_CHANCE_STANDARD: u8 = 10;

/// Standard poison chance for moves like Sludge Bomb
pub const POISON_CHANCE_STANDARD: u8 = 30;

/// Standard flinch chance for moves like Air Slash
pub const FLINCH_CHANCE_STANDARD: u8 = 30;
```

**Dual Effect Probabilities:**
```rust
/// Dual status effect probabilities for moves like Fire Fang
pub const DUAL_EFFECT_NEITHER: f32 = 81.0;    // 81% chance of neither effect
pub const DUAL_EFFECT_FIRST_ONLY: f32 = 9.0;  // 9% chance of first effect only
pub const DUAL_EFFECT_SECOND_ONLY: f32 = 9.0; // 9% chance of second effect only
pub const DUAL_EFFECT_BOTH: f32 = 1.0;        // 1% chance of both effects
```

### Power Multiplier Constants

**Status-Based Power Multipliers:**
```rust
/// Power multiplier for Facade when user has status condition
pub const FACADE_STATUS_MULTIPLIER: u16 = 2;

/// Power multiplier for Hex against statused targets
pub const HEX_STATUS_MULTIPLIER: u16 = 2;

/// Base power for Weather Ball in different weather conditions
pub const WEATHER_BALL_BOOSTED_POWER: u16 = 2;
```

### Type Interaction Constants

**Type Immunity Lists:**
```rust
/// Types that are immune to Electric-type moves
pub const ELECTRIC_IMMUNE_TYPES: &[PokemonType] = &[PokemonType::Ground];

/// Types that resist Poison-type moves
pub const POISON_RESISTANT_TYPES: &[PokemonType] = &[PokemonType::Poison, PokemonType::Steel];

/// Types that can be affected by Freeze-Dry's special effectiveness
pub const FREEZE_DRY_TARGETS: &[PokemonType] = &[PokemonType::Water];
```

### Environmental Move Constants

**Terrain Pulse Type Mappings:**
```rust
/// Type changes for Terrain Pulse based on active terrain
pub const TERRAIN_PULSE_TYPES: &[(Terrain, PokemonType)] = &[
    (Terrain::Electric, PokemonType::Electric),
    (Terrain::Grassy, PokemonType::Grass),
    (Terrain::Misty, PokemonType::Fairy),
    (Terrain::Psychic, PokemonType::Psychic),
];
```

**Weather Ball Type Mappings:**
```rust
/// Type changes for Weather Ball based on active weather
pub const WEATHER_BALL_TYPES: &[(Weather, PokemonType)] = &[
    (Weather::Sun, PokemonType::Fire),
    (Weather::HarshSun, PokemonType::Fire),
    (Weather::Rain, PokemonType::Water),
    (Weather::HeavyRain, PokemonType::Water),
    (Weather::Sand, PokemonType::Rock),
    (Weather::Sandstorm, PokemonType::Rock),
    (Weather::Hail, PokemonType::Ice),
    (Weather::Snow, PokemonType::Ice),
    (Weather::StrongWinds, PokemonType::Flying),
];
```

## Item Constants (`items.rs`)

Constants for item effects, multipliers, and activation thresholds.

### Stat Modification Constants

**Defensive Item Multipliers:**
```rust
/// Assault Vest special defense multiplier
pub const ASSAULT_VEST_SPDEF_MULTIPLIER: f32 = 1.5;

/// Eviolite multiplier for defensive stats
pub const EVIOLITE_DEF_MULTIPLIER: f32 = 1.5;
pub const EVIOLITE_SPDEF_MULTIPLIER: f32 = 1.5;
```

**Offensive Item Multipliers:**
```rust
/// Choice item multipliers
pub const CHOICE_ITEM_ATTACK_MULTIPLIER: f32 = 1.5;

/// Life Orb multipliers
pub const LIFE_ORB_DAMAGE_MULTIPLIER: f32 = 1.3;
pub const LIFE_ORB_RECOIL_FRACTION: f32 = 1.0 / 10.0;

/// Expert Belt multiplier for super effective moves
pub const EXPERT_BELT_MULTIPLIER: f32 = 1.2;

/// Type-enhancing item multipliers
pub const TYPE_ENHANCING_ITEM_MULTIPLIER: f32 = 1.2;
```

### Recovery Item Constants

**Passive Recovery:**
```rust
/// Leftovers heal fraction
pub const LEFTOVERS_HEAL_FRACTION: f32 = 1.0 / 16.0;

/// Black Sludge heal fraction (for Poison types)
pub const BLACK_SLUDGE_HEAL_FRACTION: f32 = 1.0 / 16.0;

/// Black Sludge damage fraction (for non-Poison types)
pub const BLACK_SLUDGE_DAMAGE_FRACTION: f32 = 1.0 / 8.0;
```

**Berry Activation:**
```rust
/// Berry activation threshold (25% HP or less)
pub const BERRY_ACTIVATION_HP_THRESHOLD: f32 = 0.25;

/// Sitrus Berry heal amount (fixed 25 HP in newer generations)
pub const SITRUS_BERRY_HEAL_AMOUNT: u16 = 25;
```

### Recoil and Contact Constants

**Contact Damage:**
```rust
/// Rocky Helmet recoil fraction
pub const ROCKY_HELMET_RECOIL_FRACTION: f32 = 1.0 / 6.0;
```

### Stat Boost Constants

**Item-Triggered Boosts:**
```rust
/// Weakness Policy stat boost stages
pub const WEAKNESS_POLICY_BOOST_STAGES: i8 = 2;
```

## Usage Patterns

### Damage Calculation Integration

```rust
// Using damage variance constants
let damage_rolls = (0..DAMAGE_ROLL_COUNT)
    .map(|i| {
        let multiplier = DAMAGE_ROLL_START + (i as f32 * DAMAGE_ROLL_INCREMENT);
        (base_damage as f32 * multiplier) as u16
    })
    .collect::<Vec<_>>();

// Using critical hit multiplier
if is_critical_hit {
    damage = (damage as f32 * CRITICAL_HIT_MULTIPLIER) as u16;
}
```

### Variable Power Move Implementation

```rust
// Using HP threshold constants for Reversal/Flail
fn calculate_reversal_power(hp_percentage: f32) -> u16 {
    for &(threshold, power) in REVERSAL_HP_THRESHOLDS {
        if hp_percentage <= threshold {
            return power;
        }
    }
    20 // Default power
}

// Using weight thresholds for Grass Knot
fn calculate_grass_knot_power(weight: f32) -> u16 {
    for &(threshold, power) in WEIGHT_POWER_THRESHOLDS {
        if weight >= threshold {
            return power;
        }
    }
    20 // Default minimum power
}
```

### Item Effect Implementation

```rust
// Using item multiplier constants
fn apply_life_orb_effect(damage: u16, user_hp: u16) -> (u16, u16) {
    let boosted_damage = (damage as f32 * LIFE_ORB_DAMAGE_MULTIPLIER) as u16;
    let recoil_damage = (user_hp as f32 * LIFE_ORB_RECOIL_FRACTION) as u16;
    (boosted_damage, recoil_damage)
}

// Using berry activation threshold
fn should_activate_berry(current_hp: u16, max_hp: u16) -> bool {
    (current_hp as f32 / max_hp as f32) <= BERRY_ACTIVATION_HP_THRESHOLD
}
```

### Status Effect Probability

```rust
// Using standard status chances
fn roll_for_burn() -> bool {
    roll_percentage() <= BURN_CHANCE_STANDARD
}

// Using dual effect probabilities
fn roll_dual_effect() -> DualEffectResult {
    let roll = roll_percentage();
    if roll <= DUAL_EFFECT_BOTH {
        DualEffectResult::Both
    } else if roll <= DUAL_EFFECT_BOTH + DUAL_EFFECT_FIRST_ONLY {
        DualEffectResult::FirstOnly
    } else if roll <= DUAL_EFFECT_BOTH + DUAL_EFFECT_FIRST_ONLY + DUAL_EFFECT_SECOND_ONLY {
        DualEffectResult::SecondOnly
    } else {
        DualEffectResult::Neither
    }
}
```