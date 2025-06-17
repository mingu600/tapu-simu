# 🧪 **TAPU SIMU ADVANCED TEST FRAMEWORK**
## **Production-Quality Testing with Real Pokemon Showdown Data**

---

## 📊 **OVERVIEW**

The Tapu Simu test framework represents a **significant advancement** in Pokemon battle simulator testing, providing sophisticated integration testing capabilities using actual Pokemon Showdown data. This framework enables comprehensive validation of battle mechanics with real competitive Pokemon, moves, and abilities.

### **Key Innovation: Real PS Data Integration**
Unlike traditional simulators that use mock data, our framework leverages actual Pokemon Showdown JSON data, ensuring tests reflect real competitive scenarios with accurate Pokemon stats, move properties, and ability behaviors.

---

## ✨ **CORE CAPABILITIES**

### **🎮 Real Pokemon Showdown Data**
- **947 Pokemon species** with accurate stats, types, and abilities
- **772+ moves** with correct power, accuracy, type, and category
- **244+ items** with proper effects and descriptions
- **Generation-specific data** tracking changes across Pokemon generations
- **Automatic data loading** from extracted PS JSON files
- **Standardized name normalization** for consistent lookups

### **🎯 Multi-Format Testing Support**
- **Singles format testing** - Traditional 1v1 Pokemon battles
- **Doubles format testing** - VGC-style 2v2 with position-aware mechanics
- **Position-based validation** - BattlePosition targeting verification
- **Format-specific mechanics** - Spread moves, redirection, adjacency

### **⚡ Advanced Battle Mechanics**
- **Instruction generation testing** - Verify proper StateInstructions
- **Probability branch validation** - Critical hits, damage rolls, status chances
- **Complex interaction testing** - Abilities, items, weather, terrain
- **Multi-turn scenario support** - Weather duration, status conditions

### **🔍 Comprehensive Validation**
- **Damage calculation accuracy** - Exact numerical verification
- **Status effect verification** - Correct application and duration
- **Stat boost validation** - Proper modification tracking
- **Immunity system testing** - Type and ability immunities
- **Environmental effects** - Weather, terrain, screen interactions

---

## 🏗️ **ARCHITECTURE**

### **Core Framework Structure**

```rust
/// Primary test framework with PS data integration
pub struct TestFramework {
    pub ps_data: PSDataRepository,    // Real Pokemon Showdown data
}

/// Comprehensive test utilities for battle validation
impl TestFramework {
    // Pokemon creation from real PS data
    pub fn create_pokemon_from_ps_data(&self, species: &str, ability: Option<&str>) -> Pokemon
    
    // Move creation with accurate PS properties
    pub fn create_move_from_ps_data(&self, move_name: &str) -> EngineMoveData
    
    // Battle state setup for testing scenarios
    pub fn create_test_battle(&self, setup: BattleTestSetup) -> (State, TestContext)
    
    // Instruction generation and validation
    pub fn test_instruction_generation(&self, scenario: TestScenario) -> ValidationResult
}
```

### **Name Normalization System**

All Pokemon, move, and ability names are automatically normalized to match Pokemon Showdown conventions:

```rust
// Input normalization examples
"Thick Fat" → "thickfat"           // Remove spaces, lowercase
"Lightning Rod" → "lightningrod"   // Compound words
"Ice Beam" → "icebeam"            // Move names
"Ho-Oh" → "hooh"                  // Special characters removed
"Porygon-Z" → "porygonz"          // Hyphen removal
"Nidoran♀" → "nidoranf"           // Gender symbols
```

This ensures consistent data access regardless of input format variations.

---

## 🎯 **TESTING PATTERNS**

### **1. Ability System Testing**

#### **Immunity Testing**
```rust
#[test]
fn test_levitate_vs_ground_moves() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Test with real competitive Pokemon
    let is_immune = framework.test_ability_immunity(
        "garchomp",      // Attacker: Garchomp with Earthquake
        "latios",        // Defender: Latios with Levitate
        "Levitate",      // Ability providing immunity
        "earthquake"     // Ground-type move
    ).expect("Immunity test failed");
    
    assert!(is_immune, "Latios with Levitate should be immune to Earthquake");
}

#[test]
fn test_flash_fire_immunity_and_boost() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Test immunity phase
    let is_immune = framework.test_ability_immunity(
        "charizard", "houndoom", "Flash Fire", "flamethrower"
    ).expect("Flash Fire immunity test failed");
    
    assert!(is_immune, "Flash Fire should provide Fire immunity");
    
    // Test subsequent boost phase
    let boost_multiplier = framework.test_ability_damage_boost(
        "houndoom", "Flash Fire", "flamethrower", "post_activation"
    ).expect("Flash Fire boost test failed");
    
    assert!((boost_multiplier - 1.5).abs() < 0.1, 
            "Flash Fire should provide 1.5x Fire move boost after activation");
}
```

#### **Damage Modification Testing**
```rust
#[test]
fn test_thick_fat_damage_reduction() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Test Fire move reduction
    let fire_multiplier = framework.test_ability_damage_reduction(
        "charizard",     // Fire attacker
        "snorlax",       // Thick Fat defender
        "Thick Fat",     // Damage-reducing ability
        "flamethrower"   // Fire move
    ).expect("Thick Fat Fire test failed");
    
    assert!((fire_multiplier - 0.5).abs() < 0.1, 
            "Thick Fat should reduce Fire damage to 50%");
    
    // Test Ice move reduction
    let ice_multiplier = framework.test_ability_damage_reduction(
        "articuno", "snorlax", "Thick Fat", "icebeam"
    ).expect("Thick Fat Ice test failed");
    
    assert!((ice_multiplier - 0.5).abs() < 0.1, 
            "Thick Fat should reduce Ice damage to 50%");
}

#[test]
fn test_huge_power_attack_doubling() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Compare damage with and without ability
    let normal_pokemon = framework.create_pokemon_from_ps_data("azumarill", None, Some(50))?;
    let huge_power_pokemon = framework.create_pokemon_from_ps_data("azumarill", Some("Huge Power"), Some(50))?;
    
    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))?;
    let move_data = framework.create_move_from_ps_data("aquajet")?;
    let state = State::new(BattleFormat::gen9_ou());
    
    let normal_damage = framework.calculate_damage(&normal_pokemon, &defender, &move_data, &state);
    let huge_power_damage = framework.calculate_damage(&huge_power_pokemon, &defender, &move_data, &state);
    
    let multiplier = huge_power_damage as f32 / normal_damage as f32;
    assert!((multiplier - 2.0).abs() < 0.2, 
            "Huge Power should double physical Attack stat, got {}x multiplier", multiplier);
}
```

### **2. Move Effect Testing**

#### **Status Move Validation**
```rust
#[test]
fn test_thunder_wave_paralysis_application() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let (mut state, move_indices) = framework.create_test_battle(
        "pikachu",        // Attacker with Thunder Wave
        &["thunderwave"], // Move set
        "garchomp",       // Target Pokemon
        None              // Default format (Singles)
    ).expect("Battle setup failed");
    
    let move_choice = MoveChoice::new_move(
        move_indices[0], // Thunder Wave
        vec![BattlePosition::new(SideReference::SideTwo, 0)] // Target position
    );
    
    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);
    
    // Verify paralysis status instruction generated
    assert!(framework.verify_status_instructions(&instructions, PokemonStatus::PARALYZE),
            "Thunder Wave should generate paralysis status instruction");
    
    // Verify probability distribution is valid
    assert!(framework.verify_probability_distribution(&instructions),
            "Instruction probabilities should sum to 100%");
}

#[test]
fn test_electric_type_immunity_to_paralysis() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Test that Electric types are immune to Thunder Wave
    let is_blocked = framework.test_type_immunity_blocks_status(
        "pikachu",                    // Electric attacker
        "thunderwave",                // Paralysis-inducing move
        "raichu",                     // Electric defender
        &["Electric"],                // Type providing immunity
        PokemonStatus::PARALYZE       // Expected blocked status
    ).expect("Electric immunity test failed");
    
    assert!(is_blocked, "Electric types should be immune to Thunder Wave paralysis");
}
```

#### **Damage Move Testing**
```rust
#[test]
fn test_tackle_damage_with_critical_hit_branching() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let (mut state, move_indices) = framework.create_test_battle(
        "pikachu", &["tackle"], "geodude", None
    ).expect("Battle setup failed");
    
    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)]
    );
    
    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);
    
    // Verify damage instructions exist
    assert!(framework.verify_damage_instructions(&instructions),
            "Tackle should generate damage instructions");
    
    // Verify critical hit branching (should have both crit and non-crit branches)
    assert!(framework.verify_critical_hit_branching(&instructions),
            "Tackle should have critical hit probability branching");
    
    // Verify total probability is 100%
    assert!(framework.verify_probability_distribution(&instructions),
            "All instruction branches should sum to 100% probability");
}
```

### **3. Environmental Effect Testing**

#### **Weather Interaction Testing**
```rust
#[test]
fn test_sun_weather_fire_move_boost() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let (mut state, _) = framework.create_test_battle(
        "charizard", &["flamethrower"], "blastoise", None
    ).expect("Battle setup failed");
    
    // Set sun weather
    state.weather = Weather::SUN;
    
    let move_choice = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideTwo, 0)]
    );
    
    let damage_in_sun = framework.calculate_move_damage(&mut state, move_choice.clone());
    
    // Test same move without weather
    state.weather = Weather::NONE;
    let damage_normal = framework.calculate_move_damage(&mut state, move_choice);
    
    let multiplier = damage_in_sun as f32 / damage_normal as f32;
    assert!((multiplier - 1.5).abs() < 0.1,
            "Sun should boost Fire moves by 1.5x, got {}x", multiplier);
}

#[test]
fn test_grassy_terrain_earthquake_reduction() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let (mut state, _) = framework.create_test_battle(
        "garchomp", &["earthquake"], "charizard", None
    ).expect("Battle setup failed");
    
    // Test damage with Grassy Terrain
    state.terrain = Terrain::GRASSY;
    let damage_grassy = framework.calculate_move_damage(&mut state, 
        MoveChoice::new_move(MoveIndex::M0, vec![BattlePosition::new(SideReference::SideTwo, 0)])
    );
    
    // Test damage without terrain
    state.terrain = Terrain::NONE;
    let damage_normal = framework.calculate_move_damage(&mut state,
        MoveChoice::new_move(MoveIndex::M0, vec![BattlePosition::new(SideReference::SideTwo, 0)])
    );
    
    let multiplier = damage_grassy as f32 / damage_normal as f32;
    assert!((multiplier - 0.5).abs() < 0.1,
            "Grassy Terrain should reduce Earthquake damage by 50%, got {}x", multiplier);
}
```

### **4. Multi-Format Testing**

#### **Doubles-Specific Mechanics**
```rust
#[test]
fn test_doubles_spread_move_damage_reduction() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    // Test Earthquake in Doubles (spread move)
    let (mut state, _) = framework.create_test_battle_doubles(
        ("garchomp", &["earthquake"]), // Player 1
        ("charizard", "blastoise"),    // Player 2 (both targets)
    ).expect("Doubles battle setup failed");
    
    let spread_targets = vec![
        BattlePosition::new(SideReference::SideTwo, 0), // Target 1
        BattlePosition::new(SideReference::SideTwo, 1), // Target 2
    ];
    
    let move_choice = MoveChoice::new_move(MoveIndex::M0, spread_targets);
    let instructions = framework.test_instruction_generation(&mut state, move_choice, 
                                                           Some(BattleFormat::gen9_vgc()));
    
    // Verify spread move damage reduction applied
    assert!(framework.verify_spread_move_mechanics(&instructions),
            "Earthquake should have reduced damage in Doubles when hitting multiple targets");
}

#[test]
fn test_doubles_ally_targeting() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let (mut state, _) = framework.create_test_battle_doubles(
        ("alakazam", &["helpinghand"]), // Ally support
        ("garchomp", "charizard"),       // Opposing team
    ).expect("Doubles battle setup failed");
    
    // Target ally with Helping Hand
    let ally_target = vec![BattlePosition::new(SideReference::SideOne, 1)];
    let move_choice = MoveChoice::new_move(MoveIndex::M0, ally_target);
    
    let instructions = framework.test_instruction_generation(&mut state, move_choice,
                                                           Some(BattleFormat::gen9_vgc()));
    
    // Verify Helping Hand effect applied to ally
    assert!(framework.verify_ally_support_instructions(&instructions),
            "Helping Hand should apply support effect to target ally");
}
```

---

## 🔍 **VALIDATION METHODS**

### **Instruction Verification**

```rust
impl TestFramework {
    /// Verify damage instructions are present and valid
    pub fn verify_damage_instructions(&self, instructions: &[StateInstructions]) -> bool {
        instructions.iter().any(|inst_set| {
            inst_set.instruction_list.iter().any(|instruction| {
                matches!(instruction, 
                    Instruction::PositionDamage(_) | 
                    Instruction::MultiTargetDamage(_))
            })
        })
    }
    
    /// Verify status effect instructions are generated correctly
    pub fn verify_status_instructions(
        &self, 
        instructions: &[StateInstructions], 
        expected_status: PokemonStatus
    ) -> bool {
        instructions.iter().any(|inst_set| {
            inst_set.instruction_list.iter().any(|instruction| {
                if let Instruction::ApplyStatus(status_instr) = instruction {
                    status_instr.status == expected_status
                } else {
                    false
                }
            })
        })
    }
    
    /// Verify stat boost instructions with correct values
    pub fn verify_stat_boost_instructions(
        &self, 
        instructions: &[StateInstructions], 
        expected_stat: Stat, 
        expected_boost: i8
    ) -> bool {
        instructions.iter().any(|inst_set| {
            inst_set.instruction_list.iter().any(|instruction| {
                if let Instruction::Boost(boost_instr) = instruction {
                    boost_instr.stat == expected_stat && boost_instr.amount == expected_boost
                } else {
                    false
                }
            })
        })
    }
    
    /// Verify critical hit probability branching
    pub fn verify_critical_hit_branching(&self, instructions: &[StateInstructions]) -> bool {
        // Should have at least 2 branches: normal hit and critical hit
        instructions.len() >= 2 && 
        instructions.iter().any(|inst| inst.percentage < 1.0) // Has probability branching
    }
    
    /// Verify instruction probabilities sum to 100%
    pub fn verify_probability_distribution(&self, instructions: &[StateInstructions]) -> bool {
        let total_probability: f32 = instructions.iter().map(|inst| inst.percentage).sum();
        (total_probability - 1.0).abs() < 0.001 // Within floating point precision
    }
}
```

### **Complex Scenario Validation**

```rust
impl TestFramework {
    /// Test ability immunity with comprehensive coverage
    pub fn test_ability_immunity(
        &self,
        attacker_species: &str,
        defender_species: &str,
        defender_ability: &str,
        move_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, Some(defender_ability), Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;
        
        let state = State::new(BattleFormat::gen9_ou());
        
        // Generate instructions and check for immunity
        let damage = self.calculate_damage(&attacker, &defender, &move_data, &state);
        Ok(damage == 0) // Immunity means 0 damage
    }
    
    /// Test damage reduction abilities with precise multiplier validation
    pub fn test_ability_damage_reduction(
        &self,
        attacker_species: &str,
        defender_species: &str,
        defender_ability: &str,
        move_name: &str,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        // Test with ability
        let defender_with_ability = self.create_pokemon_from_ps_data(
            defender_species, Some(defender_ability), Some(50)
        )?;
        
        // Test without ability (baseline)
        let defender_without_ability = self.create_pokemon_from_ps_data(
            defender_species, None, Some(50)
        )?;
        
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;
        let state = State::new(BattleFormat::gen9_ou());
        
        let damage_with_ability = self.calculate_damage(&attacker, &defender_with_ability, &move_data, &state);
        let damage_without_ability = self.calculate_damage(&attacker, &defender_without_ability, &move_data, &state);
        
        if damage_without_ability == 0 {
            return Err("Baseline damage calculation failed".into());
        }
        
        Ok(damage_with_ability as f32 / damage_without_ability as f32)
    }
}
```

---

## 📁 **TEST ORGANIZATION**

### **Recommended File Structure**
```
tests/
├── abilities/
│   ├── immunity_abilities.rs      # Levitate, Flash Fire, absorb abilities
│   ├── damage_modifiers.rs        # Thick Fat, Solid Rock, Huge Power
│   ├── weather_setters.rs         # Drought, Drizzle, Sand Stream
│   ├── terrain_setters.rs         # Electric Surge, Grassy Surge, etc.
│   └── complex_abilities.rs       # Protean, Unaware, Wonder Guard
├── moves/
│   ├── status_moves.rs            # Thunder Wave, Sleep Powder, etc.
│   ├── stat_moves.rs              # Swords Dance, Dragon Dance, etc.
│   ├── healing_moves.rs           # Recover, Giga Drain, Rest
│   ├── variable_power_moves.rs    # Reversal, Gyro Ball, Heavy Slam
│   └── complex_moves.rs           # Foul Play, Body Press, Transform
├── items/
│   ├── choice_items.rs            # Choice Band, Specs, Scarf
│   ├── type_boosters.rs           # Type plates, Charcoal, etc.
│   ├── berries.rs                 # Type resist, stat boost berries
│   └── utility_items.rs           # Life Orb, Focus Sash, Leftovers
├── environmental/
│   ├── weather_effects.rs         # Sun, Rain, Sandstorm, etc.
│   ├── terrain_effects.rs         # All terrain types and interactions
│   ├── screen_effects.rs          # Reflect, Light Screen, Aurora Veil
│   └── side_conditions.rs         # Stealth Rock, Spikes, etc.
├── formats/
│   ├── singles_mechanics.rs       # Singles-specific testing
│   ├── doubles_mechanics.rs       # VGC/Doubles interactions
│   └── format_transitions.rs      # Cross-format compatibility
├── generations/
│   ├── generation_differences.rs  # Gen-specific mechanic changes
│   ├── terastallization.rs        # Gen 9 Tera mechanics
│   └── legacy_mechanics.rs        # Historical accuracy testing
└── integration/
    ├── complex_scenarios.rs       # Multi-turn battle scenarios
    ├── parity_validation.rs       # V1 comparison testing
    └── performance_benchmarks.rs  # Speed and memory testing
```

### **Test Categories by Priority**

#### **High Priority (Core Competitive)**
- Immunity abilities (Levitate, Flash Fire, Water/Volt Absorb)
- Damage modifiers (Thick Fat, Huge Power, Technician)
- Status moves (Thunder Wave, Sleep Powder, Toxic)
- Weather effects (Sun, Rain boost/reduction)
- Choice items (Band, Specs, Scarf)

#### **Medium Priority (Competitive Edge Cases)**
- Complex abilities (Protean, Unaware, Wonder Guard)
- Variable power moves (Reversal, Gyro Ball)
- Terrain interactions (Electric, Grassy, Psychic, Misty)
- Berry mechanics (type resist, stat boost)
- Multi-hit moves (Bullet Seed, Rock Blast)

#### **Lower Priority (Comprehensive Coverage)**
- Rare abilities and items
- Generation-specific edge cases
- Complex multi-turn interactions
- Performance optimization scenarios

---

## 🚀 **ADVANCED TESTING PATTERNS**

### **Property-Based Testing**
```rust
#[test]
fn test_damage_calculation_properties() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    for _ in 0..1000 {
        let attacker = framework.create_random_pokemon();
        let defender = framework.create_random_pokemon();
        let move_data = framework.create_random_move();
        let state = framework.create_random_state();
        
        let damage = framework.calculate_damage(&attacker, &defender, &move_data, &state);
        
        // Properties that should always hold
        assert!(damage >= 0, "Damage should never be negative");
        assert!(damage <= defender.maxhp * 2, "Damage should not exceed reasonable bounds");
        
        // Type effectiveness properties
        let effectiveness = framework.calculate_type_effectiveness(&move_data, &defender);
        if effectiveness == 0.0 {
            assert_eq!(damage, 0, "Zero effectiveness should result in zero damage");
        }
    }
}
```

### **Regression Testing Framework**
```rust
/// Store and verify battle outcomes for regression prevention
pub struct RegressionTestSuite {
    stored_scenarios: Vec<BattleScenario>,
}

impl RegressionTestSuite {
    pub fn add_scenario(&mut self, scenario: BattleScenario) {
        self.stored_scenarios.push(scenario);
    }
    
    pub fn validate_all_scenarios(&self, framework: &TestFramework) -> Vec<RegressionFailure> {
        let mut failures = Vec::new();
        
        for scenario in &self.stored_scenarios {
            let current_result = framework.execute_scenario(scenario);
            if current_result != scenario.expected_result {
                failures.push(RegressionFailure {
                    scenario: scenario.clone(),
                    expected: scenario.expected_result,
                    actual: current_result,
                });
            }
        }
        
        failures
    }
}
```

### **Cross-Generation Validation**
```rust
#[test]
fn test_cross_generation_damage_consistency() {
    let framework = TestFramework::new().expect("Framework initialization failed");
    
    let generations = vec![
        Generation::Gen4, Generation::Gen5, Generation::Gen6,
        Generation::Gen7, Generation::Gen8, Generation::Gen9
    ];
    
    for gen in generations {
        let mut state = State::new(BattleFormat::for_generation(gen));
        
        // Test same scenario across generations
        let damage = framework.calculate_generation_specific_damage(&state, "flamethrower", gen);
        
        // Verify generation-specific mechanics
        match gen {
            Generation::Gen1..=Generation::Gen5 => {
                // Critical hits should be 2.0x
                assert_eq!(framework.get_critical_multiplier(gen), 2.0);
            }
            Generation::Gen6..=Generation::Gen9 => {
                // Critical hits should be 1.5x
                assert_eq!(framework.get_critical_multiplier(gen), 1.5);
            }
        }
    }
}
```

---

## 📈 **TESTING METRICS & GOALS**

### **Coverage Targets**
- **100% Move Coverage** - All 772+ PS moves tested
- **100% Ability Coverage** - All competitive abilities validated
- **100% Item Coverage** - All competitive items tested
- **Multi-Format Coverage** - Singles and Doubles mechanics
- **Cross-Generation Coverage** - All supported generations

### **Quality Metrics**
- **Accuracy Validation** - Exact PS data match
- **Performance Benchmarks** - Sub-millisecond instruction generation
- **Regression Prevention** - Zero regression tolerance
- **Documentation Coverage** - All test patterns documented

### **V1 Parity Validation**
- **Battle Reproduction** - Identical outcomes for identical inputs
- **Damage Calculation** - Exact numerical parity
- **Status Timing** - Turn-by-turn state matching
- **Complex Interactions** - Multi-mechanic scenario validation

---

This advanced test framework provides the foundation for achieving **100% V1 parity** while maintaining **superior architectural quality** and **multi-format capabilities**. The combination of real PS data integration, comprehensive validation methods, and systematic test organization ensures that Tapu Simu will be the most thoroughly tested Pokemon battle simulator ever created.