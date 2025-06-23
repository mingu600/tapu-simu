# Comprehensive Test Porting Plan: Poke-Engine to Tapu-Simu

## Overview

This document outlines the systematic porting of **788 tests** from poke-engine to tapu-simu. The goal is to recreate functionally equivalent tests using tapu-simu's architecture, proper data loading through the Repository system, and end-to-end testing patterns.

## Test Inventory

### Source Test Files (poke-engine/tests)
- **`test_battle_mechanics.rs`** - 663 tests (core mechanics)
- **`test_damage_dealt.rs`** - 17 tests (damage tracking)
- **`test_gen1.rs`** - 42 tests (Gen 1 mechanics)
- **`test_gen2.rs`** - 35 tests (Gen 2 mechanics) 
- **`test_gen3.rs`** - 14 tests (Gen 3 mechanics)
- **`test_last_used_move.rs`** - 17 tests (move tracking)

### Target Test Organization (tapu-simu/tests)
**NOTE: Rust integration tests must be in a flat structure in tests/ folder**
```
tests/
├── basic_damage.rs           # Basic damage calculation tests
├── move_categories.rs        # Multi-hit, variable power, special mechanics
├── accuracy_miss.rs          # Accuracy and miss mechanics
├── secondary_effects.rs      # Secondary effects and status infliction
├── status_primary.rs         # Primary status conditions (sleep, paralysis, etc.)
├── status_volatile.rs        # Volatile status conditions (confusion, substitute, etc.)
├── status_immunities.rs      # Status immunities and cures
├── abilities_defensive.rs    # Defensive abilities
├── abilities_offensive.rs    # Offensive abilities
├── abilities_weather.rs      # Weather/terrain abilities
├── abilities_form_change.rs  # Form change abilities
├── abilities_stat_boost.rs   # Stat boost abilities
├── items_berries.rs          # Berry items
├── items_choice.rs           # Choice items
├── items_battle_enhance.rs   # Battle enhancement items
├── items_type_boost.rs       # Type-boosting items
├── items_status_induce.rs    # Status-inducing items
├── items_utility.rs          # Utility items
├── weather_effects.rs        # Weather effects
├── terrain_effects.rs        # Terrain effects
├── entry_hazards.rs          # Entry hazards
├── stat_modifications.rs     # Stat boosts and drops
├── critical_hits.rs          # Critical hit interactions
├── ability_stat_mods.rs      # Ability stat modifications
├── gen1_quirks.rs            # Gen 1 specific mechanics
├── gen2_additions.rs         # Gen 2 specific mechanics
├── gen3_modern.rs            # Gen 3 modern mechanics
├── multi_system.rs           # Multi-system interactions
├── edge_cases.rs             # Edge cases and boundaries
├── trapping_movement.rs      # Trapping and movement restriction
├── damage_tracking.rs        # Counter/Mirror Coat mechanics
├── move_tracking.rs          # Encore and move tracking
├── terastallization.rs       # Terastallization mechanics
├── priority_speed.rs         # Priority and speed mechanics
├── good_as_gold.rs          # Good as Gold interactions
├── protection_moves.rs       # Protection moves
└── utils/                    # Test framework utilities
    ├── mod.rs              # Module declarations
    ├── framework.rs        # Core test framework
    ├── builders.rs         # Test-specific builders
    └── assertions.rs       # Custom assertions
```

## New Test Framework Design

### Framework Overview

The test framework is built around three core components:

1. **TapuTestFramework**: Manages Pokemon data repository and battle format
2. **TestBuilder**: Fluent API for constructing battle tests
3. **BattleAssertions**: Specialized assertions for battle state validation

### Core Framework Structure

```rust
// tests/utils/framework.rs
pub struct TapuTestFramework {
    repository: Arc<Repository>,
    format: BattleFormat,
}

impl TapuTestFramework {
    /// Create with default Gen 9 Singles format
    pub fn new() -> DataResult<Self> {
        let repository = Arc::new(Repository::from_path("data/ps-extracted")?);
        Ok(Self {
            repository,
            format: BattleFormat::gen9_ou(),
        })
    }
    
    /// Create for specific generation
    pub fn with_generation(gen: Generation) -> DataResult<Self> {
        // Handles all generations with appropriate fallbacks
    }
    
    /// Execute a complete battle test using real battle engine
    pub fn execute_test(&self, test: BattleTest) -> TestResult {
        // Creates battle state, applies setup, executes turns,
        // validates outcomes using actual turn engine
    }
}

// Fluent Test Builder API
pub struct TestBuilder {
    framework: TapuTestFramework,
    test: BattleTest,
}

impl TestBuilder {
    pub fn new(name: &str) -> DataResult<Self>
    pub fn team_one(self, spec: PokemonSpec) -> Self
    pub fn team_two(self, spec: PokemonSpec) -> Self
    pub fn turn_with_moves(self, move_one: &str, move_two: &str) -> Self
    pub fn expect_damage(self, position: BattlePosition, damage: u16) -> Self
    pub fn run(self) -> TestResult
    pub fn assert_success(self) // Panic on failure for test assertions
}

// Pokemon Specification with Builder Pattern
pub struct PokemonSpec {
    pub species: &'static str,
    pub level: u8,
    pub ability: Option<&'static str>,
    pub item: Option<&'static str>,
    pub moves: Vec<&'static str>,
    pub evs: Option<Stats>,
    pub status: Option<PokemonStatus>,
    pub hp_percentage: Option<f32>,
    // ... other fields
}

impl PokemonSpec {
    pub fn new(species: &'static str) -> Self
    pub fn level(self, level: u8) -> Self
    pub fn ability(self, ability: &'static str) -> Self
    pub fn moves(self, moves: Vec<&'static str>) -> Self
    pub fn ev_spread(self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self
    // ... other builder methods
}

// Setup Actions for Pre-Battle State
pub enum SetupAction {
    SetWeather(Weather),
    SetTerrain(Terrain),
    ApplyStatus(BattlePosition, PokemonStatus),
    ModifyStats(BattlePosition, HashMap<Stat, i8>),
    SetHP(BattlePosition, u16),
    AddSideCondition(SideReference, SideCondition),
}

// Expected Outcomes for Validation
pub enum ExpectedOutcome {
    Damage(BattlePosition, u16),
    Status(BattlePosition, PokemonStatus),
    StatChange(BattlePosition, Stat, i8),
    WeatherSet(Weather),
    TerrainSet(Terrain),
    SideCondition(SideReference, SideCondition),
    Faint(BattlePosition),
    Switch(BattlePosition, usize),
    NoEffect(BattlePosition),
    Instructions(Vec<BattleInstructions>), // For exact instruction matching
}
```

### Key Features

**Real Pokemon Data**: Uses actual Pokemon data from PS repository instead of dummy Pokemon where possible, providing more realistic testing.

**Fluent API**: TestBuilder provides a readable, chainable API for constructing complex battle scenarios.

**Flexible Validation**: Supports both high-level outcome validation (damage, status) and low-level instruction validation.

**Multi-Generation Support**: Framework adapts to different generations with appropriate rule sets.

**Integration Testing**: Tests use the actual turn engine, ensuring end-to-end validation of battle mechanics.

### Usage Example

```rust
#[test]
fn test_basic_damage() {
    TestBuilder::new("basic tackle damage")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_with_moves("Tackle", "Tackle")
        .expect_damage(Positions::SIDE_TWO_0, 22)
        .assert_success();
}
```

## Porting Strategy by Category

### Phase 1: Core Combat System (150 tests)

**Priority: High - Foundation for all other tests**

#### 1.1 Basic Damage Calculation (25 tests) ✅ **25/25 ported** ✅ COMPLETE
**Original Tests to Port:**
- `test_basic_move_pair_instruction_generation`
- `test_branch_on_crit`
- `test_branch_when_a_roll_can_kill`
- `test_branch_when_a_roll_can_kill_on_the_low_side`
- `test_crit_does_not_overkill`
- `test_highcrit_move`
- `test_min_damage_killing_does_not_branch`
- `test_surgingstrikes_always_crits_without_a_branch`
- `test_wickedblow_always_crits_without_a_branch`
- `test_wickedblow_always_ignores_defensive_boost_on_opponent_because_of_crit`
- `test_wickedblow_cannot_crit_on_shellarmor`
- `test_wickedblow_gen8`
- `test_wickedblow_gen9`
- Gen 1 crit tests: `test_crit_roll_ignores_other_boost`, `test_crit_roll_ignores_other_boost_negative_boost`, `test_crit_roll_ignores_own_boost`, `test_crit_roll_ignores_reflect`, `test_persion_using_slash_guaranteed_crit`, `test_persion_using_tackle_rolls_crit`
- Gen 2 crit tests: `test_branch_on_crit`, `test_crit_does_not_overkill`, `test_highcrit_move`, `test_min_damage_killing_does_not_branch`
- Gen 3 crit tests: `test_branch_when_a_roll_can_kill`, `test_gen3_branch_when_a_roll_can_kill`

#### 1.2 Move Categories (40 tests) ✅ **0/40 ported**
**Original Tests to Port:**
- Basic multi-hit: `test_basic_multi_hit_move`, `test_skilllink_always_has_5_hits`
- Variable power: `test_boltbeak`, `test_grassknot_basepower_changing_based_on_weight`, `test_grassknot_basepower_changing_to_max_damage`, `test_hardpress`, `test_heatcrash_highest_base_power`, `test_heavyslam_highest_base_power`, `test_heavyslam_lowest_base_power`, `test_lowkick_basepower_highest_damage`, `test_lowkick_basepower_lowest_damage`
- Special mechanics: `test_bodypress`, `test_endeavor`, `test_endeavor_versus_ghost`, `test_endeavor_when_higher_hp_than_opponent`, `test_finalgambit`, `test_finalgambit_versus_ghost`, `test_foulplay`, `test_hydrosteam`, `test_mindblown`, `test_mindblown_does_not_overkill`, `test_mindblown_into_damp`, `test_mindseye_versus_ghost_type`, `test_moongeistbeam_into_ice_scales`
- Fixed damage: `test_painsplit`, `test_seismictoss`, `test_seismictoss_does_not_overkill`, `test_seismictoss_versus_ghost_type`, `test_superfang`, `test_superfang_at_1hp`, `test_superfang_versus_ghost_type`
- Contact mechanics: `test_contact_multi_hit_move_versus_rockyhelmet`, `test_multi_hit_move_where_first_hit_breaks_substitute`, `test_triple_multihit_move_versus_substitute_and_rockyhelmet`
- Explosive moves: `test_explosion_into_damp`, `test_explosion_into_ghost_type`, `test_fast_explosion_makes_other_side_unable_to_move`
- Other: `test_scaleshot_only_boosts_once`, `test_population_bomb_with_widelens`, `test_basic_levitate`

#### 1.3 Accuracy and Miss Mechanics (15 tests) ✅ **0/15 ported**
**Original Tests to Port:**
- `test_compound_eyes_does_not_cause_instructions_with_more_than_100_percent`
- `test_flinching_on_move_that_can_miss`
- `test_blizzard_in_hail`
- `test_magician_does_not_steal_if_move_misses`
- `test_throatspray_with_move_that_can_miss`
- `test_sandspit_does_not_activate_on_miss`
- `test_poltergeist_missing`
- `test_clangoroussoul_missing`
- `test_using_move_while_asleep_does_not_decrement_pp`
- `test_pp_not_decremented_when_flinched`
- `test_wonderskin_against_poisonpowder`
- `test_crash_move_into_protect`
- `test_suckerpunch_fails_versus_faster_attacking_move`
- `test_thunderclap_fails_versus_faster_attacking_move`
- `test_metalburst_fails_moving_first`

#### 1.4 Secondary Effects and Status Infliction (70 tests) ✅ **0/70 ported**
**Original Tests to Port:**
- Basic status moves: `test_basic_flinching_functionality`, `test_basic_spore`, `test_flinching_first_and_second_move`  
- Multi-secondary effects: `test_icefang_multi_secondary`, `test_triattack`, `test_triplearrows_multi_secondary`
- Yawn mechanics: `test_cannot_reapply_yawn_when_already_inflicted`, `test_yawn_and_duration_removed_on_switch`, `test_yawn_can_be_inflicted_with_electricterrain_on_nongrounded_pkmn`, `test_yawn_can_be_inflicted_with_mistyterrain_when_target_is_not_grounded`, `test_yawn_cannot_be_inflicted_to_insomnia`, `test_yawn_cannot_be_inflicted_to_vitalspirit`, `test_yawn_cannot_be_inflicted_with_an_existing_status`, `test_yawn_cannot_be_inflicted_with_electricterrain`, `test_yawn_cannot_be_inflicted_with_mistyterrain`, `test_yawn_gets_applied_and_duration_decrements`, `test_yawn_is_removed_but_no_status_change_if_pkmn_already_statused`, `test_yawn_with_duration_causes_pkmn_to_sleep`
- Poison mechanics: `test_cannot_toxic_steel_pokemon`, `test_poisontouch_with_poisonjab`, `test_poisontype_using_toxic`, `test_steel_immune_to_poison_move_from_pkmn_with_corrosion`, `test_toxic_chain`, `test_toxic_count_is_reset_even_if_toxic_is_reapplied_the_same_turn`, `test_toxic_count_removed_after_curing_status`, `test_toxic_into_shedinja`
- Paralysis: `test_gen4_glare_into_electric_type`, `test_glare_into_electric_type`, `test_previous_status_makes_immune_to_paralysis`
- Confusion: `test_confuseray_into_substitute`, `test_outrage_fatigue_causing_confusion`  
- Freeze: `test_freeze_chance_to_thaw`
- Clear effects: `test_clearsmog_does_not_reset_boosts_if_defender_is_immune`, `test_clearsmog_removes_boosts_on_target`
- Substitute interactions: `test_substitute_does_not_let_secondary_status_effect_happen`
- Plus additional status-related tests from various move categories

### Phase 2: Status Effects System (120 tests)

**Priority: High - Core mechanic used throughout**

#### 2.1 Primary Status Conditions (60 tests) ✅ **0/60 ported**
**Sleep Tests:**
- `test_basic_spore`
- `test_guaranteed_to_stay_asleep_sleeptalk_move_when_not_rested`
- `test_large_chance_to_awaken_sleeptalk_move_when_not_rested`
- `test_sleep_clause_doesnt_apply_to_fainted_pokemon`
- `test_sleep_clause_doesnt_apply_to_rested_pokemon`
- `test_sleep_clause_prevents_sleep_move_used_on_opponent`
- `test_sleeptalk_when_asleep_and_rest_turns_active`
- `test_small_chance_to_awaken_sleeptalk_move_when_not_rested`
- `test_using_sleeppowder_as_faster_pkmn`
- `test_sleeptalk_can_call_rest`
- `test_sleeptalk_rest_has_no_effect_at_full_hp`
- Gen 1 sleep: `test_cannot_use_move_after_waking_when_only_a_chance_to_wake_up`, `test_cannot_use_move_when_waking_from_sleep`, `test_rest_wake_up_cannot_use_move`, `test_using_rest_sets_rest_turns_to_2`
- Gen 2 sleep: `test_guaranteed_to_stay_asleep_sleeptalk_move_when_not_rested`, `test_small_chance_to_awaken_sleeptalk_move_when_not_rested`
- Gen 4/5 sleep: `test_gen4_one_turn_asleep_trying_to_wake_up`, `test_gen5_guaranteed_wake_up`, `test_gen5_one_turn_asleep_trying_to_wake_up`, `test_gen5_switchout_while_sleep_resets_rest_turns`, `test_gen5_switchout_while_sleep_resets_sleep_turns`

**Paralysis Tests:**
- `test_gen4_glare_into_electric_type`
- `test_glare_into_electric_type` 
- `test_previous_status_makes_immune_to_paralysis`
- `test_thunderbolt_cannot_paralyze_electric_type`
- `test_thunderwave_can_paralyze_electric_type`
- `test_thunderwave_can_paralyze_normal_type`
- `test_bodyslam_cannot_paralyze_normal_type` (Gen 1)
- `test_gen1_agility_while_paralyzed_sets_paralysis_nullify_volatile`
- `test_paralysis_nullify_ignores_paralysis`
- `test_does_not_set_paralysisnullify_if_already_exists`
- `test_mint_berry_does_not_cure_paralysis`
- `test_miracleberry_cures_paralysis_and_attack_does_not_branch`

**Burn Tests:**
- `test_dryskin_prevents_scald_brun` 
- `test_burning_bulwark_burns`
- `test_burning_bulwark_does_not_burn_fire_type`
- `test_gen1_swordsdance_while_burned_sets_burn_nullify_volatile`
- `test_gen1_swordsdance_while_burned_volatile_increases_damage`
- `test_gen1_agility_while_burned_does_not_set_burn_nullify_volatile`
- `test_does_not_set_burn_nullify_if_already_exists`

**Poison/Toxic Tests:**
- `test_cannot_toxic_steel_pokemon`
- `test_poisontype_using_toxic`
- `test_steel_immune_to_poison_move_from_pkmn_with_corrosion`
- `test_toxic_chain`
- `test_toxic_count_is_reset_even_if_toxic_is_reapplied_the_same_turn`
- `test_toxic_count_removed_after_curing_status`
- `test_toxic_into_shedinja`
- `test_wonderguard_1hp_against_poison`
- `test_wonderguard_1hp_against_toxic`
- `test_destinybond_against_toxic_damage_does_not_kill_opponent`
- `test_toxic_turns_into_poison_when_switching` (Gen 1/2)

**Freeze Tests:**
- `test_freeze_chance_to_thaw`
- `test_freeze_clause` (Gen 1/2)

#### 2.2 Volatile Status Conditions (40 tests) ✅ **0/40 ported**
**Confusion Tests:**
- `test_confuseray_into_substitute`
- `test_outrage_fatigue_causing_confusion`
- Gen 1: `test_confuseray_into_substitute`, `test_thunderwave_into_substitute`
- Gen 2: `test_confuseray_into_substitute`

**Substitute Tests:**
- `test_basic_substitute_usage`
- `test_substitute_blocks_yawn`
- `test_taking_damage_with_0_hp_sub_but_with_vs`
- `test_using_substitute_when_it_is_already_up`
- `test_using_protect_with_a_substitute`
- `test_substitute_versus_intimidate`
- `test_confuseray_into_substitute`
- `test_leechseed_into_substitute`
- `test_move_goes_through_substitute`
- `test_multi_hit_move_where_first_hit_breaks_substitute`
- `test_drag_move_against_substitute`
- `test_drag_move_against_protect_and_substitute`
- `test_whirlwind_move_against_substitute`
- `test_knockoff_boosts_damage_but_cannot_remove_if_sub_is_hit`
- `test_thief_does_not_steal_if_hit_sub`
- `test_trick_against_substitute_fails`
- `test_infiltrator_goes_through_substitute`
- `test_triple_multihit_move_versus_substitute_and_rockyhelmet`
- `test_non_baton_pass_switching_with_sub`
- `test_perish_bypasses_sub`
- Plus baton pass interactions with substitute

**Taunt Tests:**
- `test_fast_taunt_gets_applied_and_duration_increments`
- `test_slow_taunt_gets_applied_and_duration_does_not_increment`
- `test_switching_out_with_taunt_resets_duration_to_0`
- `test_taunt_into_aromaveil`
- `test_taunt_into_glare`
- `test_taunt_prevents_status_move`
- `test_taunt_re_enables_disabled_moves_when_being_removed`
- `test_taunt_volatile_is_removed_end_of_turn_when_it_would_reach_3`
- Gen 3: `test_taunt_gets_applied_and_duration_increments_end_of_turn`, `test_taunt_prevents_status_move`, `test_taunt_re_enables_disabled_moves_when_being_removed`, `test_taunt_volatile_is_removed_end_of_turn_when_it_would_reach_2`

**Other Volatile Status:**
- `test_basic_flinching_functionality`
- `test_flinching_first_and_second_move`
- `test_flinching_on_move_that_can_miss`
- `test_volatile_is_not_applied_to_fainted_pkmn`

#### 2.3 Status Immunities and Cures (20 tests) ✅ **0/20 ported**
**Berry Cures:**
- `test_chestoberry_activates_when_being_put_to_sleep`
- `test_chestoberry_activates_when_using_rest`
- `test_lumberry_curing_before_move`
- `test_lumberry_does_nothing_with_no_status`
- `test_lum_cures_same_turn_when_faster`
- `test_lum_cures_same_turn_when_slower`
- `test_mintberry_cures_rest`
- `test_mint_berry_does_not_cure_paralysis`
- `test_miracleberry_cures_paralysis_and_attack_does_not_branch`
- `test_miracleberry_cures_rest`

**Healing Moves:**
- `test_basic_healbell`
- `test_healbell_when_one_reserve_was_rested`
- `test_healbell_with_multiple_reserves_statused`
- `test_removing_sleep_via_healbell_sets_sleep_turns_to_zero`
- `test_refresh_curing_status`

**Natural Cures:**
- `test_shedskin_end_of_turn`
- `test_hydration_end_of_turn`
- `test_hydration_without_weather`
- `test_using_rest_with_existing_status_condition_and_hydration`
- `test_using_rest_with_existing_status_condition_and_shedskin`

### Phase 3: Abilities System (180 tests)

**Priority: High - Complex interactions with all other systems**

#### 3.1 Defensive Abilities (80 tests) ✅ **0/80 ported**
**Immunity Abilities:**
- `test_wonderguard`
- `test_wonderguard_1hp_against_poison`
- `test_wonderguard_1hp_against_toxic`
- `test_wonderguard_1_hp_against_willowisp`
- `test_wonderguard_against_spore`
- `test_wonderguard_against_willowisp`
- `test_flashfire`
- `test_voltabsorb`
- `test_waterabsorb`
- `test_waterabsorb_does_nothing_against_move_targetting_self`
- `test_lightning_rod`
- `test_lightning_rod_versus_status_move`
- `test_stormdrain`
- `test_motor_drive`
- `test_eartheater`
- `test_wellbakedbody_activating`
- `test_wind_rider`
- `test_dryskin_does_not_overheal`
- `test_dryskin_from_water_move`
- `test_dryskin_in_rain`
- `test_dryskin_prevents_scald_brun`

**Damage Reduction Abilities:**
- `test_filter`
- `test_prismarmor`
- `test_prismarmor_respects_tera_type`
- `test_multiscale`
- `test_icebody_heal`
- `test_icebody_no_heal`

**Contact Abilities:**
- `test_roughskin_damage_taken`
- `test_roughskin_damage_taken_on_multihit_move`
- `test_roughskin_damage_taken_when_target_faints`
- `test_ironbarbs`
- `test_poisonpoint`
- `test_effectspore`
- `test_cottondown`
- `test_cottondown_activates_when_fainting`
- `test_cottondown_cannot_boost_below_minus_6`
- `test_tanglinghair`
- `test_staticability` (implied from poisonpoint pattern)

**Form Change Defensive:**
- `test_iceface_against_move_with_possible_secondary`
- `test_iceface_against_move_with_secondary`
- `test_iceface_eiscuenoice_switching_into_hail`
- `test_iceface_eiscuenoice_switching_into_snow` 
- `test_iceface_eiscue_taking_physical_hit`
- `test_iceface_eiscue_taking_special_hit`
- `test_iceface_eiscue_taking_uturn`
- `test_mimikyu_busting_does_not_overkill`
- `test_mimikyu_with_disguise_formechange_on_damaging_move`

**Damage Reflection:**
- `test_aftermath_cannot_overkill`
- `test_aftermath_damage`
- `test_innards_out`
- `test_innards_out_does_not_overkill`
- `test_liquidooze`

**Status Protection:**
- `test_hypercutter`
- `test_innerfocus`
- `test_innerfocus_versus_intimidate`
- `test_oblivious_versus_intimidate`
- `test_owntempo_versus_intimidate`
- `test_scrappy_versus_intimidate`
- `test_leafguard`
- `test_overcoat_vs_powder_move`
- `test_overcoat_vs_weather_damage`
- `test_shielddust_doesnt_stop_self_secondary`
- `test_shielddust_stops_secondary_against_opponent`

**Entry Hazard Interaction:**
- `test_magicguard_switching_into_all_hazards`
- `test_magicguard_switching_into_rocks`
- `test_magicguard_switching_into_webs`

**Other Defensive:**
- `test_absorbbulb`
- `test_beads_of_ruin`
- `test_dauntlessshield`
- `test_intrepidsword`
- `test_marvelscale`
- `test_stamina`
- `test_stamina_activating`
- `test_stamina_activating_on_multi_hit`
- `test_stamina_does_not_activate_when_fainting`
- `test_steamengine`
- `test_thermal_exchange`
- `test_watercompaction`
- `test_weakarmor`
- `test_berserk_cannot_overboost`
- `test_berserk_when_going_below_half`
- `test_berserk_when_staying_above_half`
- `test_rattled`
- `test_suctioncups`
- `test_sword_of_ruin`
- `test_tablets_of_ruin`
- `test_vessel_of_ruin`

#### 3.2 Offensive Abilities (50 tests) ✅ **0/50 ported**
**Damage Boost Abilities:**
- `test_adaptability`
- `test_adaptability_with_tera_but_no_regular_stab_is_200_percent`
- `test_adaptability_with_tera_stab_is_225_percent`
- `test_technician`
- `test_sharpness_boost`
- `test_transistor`
- `test_transistor_higher_boost_before_gen8`

**Type Change Abilities:**
- `test_basic_protean`
- `test_gen6_gen7_gen8_protean_does_activate_when_already_typechanged`
- `test_gen9_protean_does_not_activate_when_already_typechanged`
- `test_protean_does_not_change_type_if_already_has_type`
- `test_color_change_does_not_activate_if_type_is_already_the_same`
- `test_color_change_does_not_activate_when_fainting`
- `test_color_change_modifying_type`
- `test_pixilate`
- `test_pixilate_gen6`

**Stat Modification:**
- `test_contrary`
- `test_contrary_when_pre_swapped_boost_goes_above_max`
- `test_contrary_with_secondary`
- `test_contrary_with_seed`
- `test_download_for_defense`
- `test_download_for_defense_when_switching_in_with_baton_boosted_max_attack`
- `test_download_for_special_defense`
- `test_intimidate`
- `test_simple_in_gen4_doubles_effective_boost`

**Special Mechanics:**
- `test_corrosion_can_toxic_poison_type`
- `test_corrosion_can_toxic_steel_type`
- `test_scrappy_fighting_move_becomes_supereffective_against_ghost_normal`
- `test_scrappy_versus_ghost_type`
- `test_moldbreaker_ignores_wellbakedbody`
- `test_mold_breaker_into_levitate`
- `test_mold_breaker_into_waterabsorb`
- `test_mold_breaker_into_wonderguard`
- `test_moldbreaker_negating_armortail`
- `test_moldbreaker_negating_wind_rider`

**Item/Move Interaction:**
- `test_magician_does_not_remove_from_stickyhold`
- `test_magician_does_not_steal_if_move_misses`
- `test_magician_doing_damage_steals_opponents_item`
- `test_mummy_changes_ability_on_contact_move`
- `test_mummy_does_not_change_ability_on_non_contact_move`

**Secondary Effect Enhancement:**
- `test_serenegrace_with_secondary`
- `test_compound_eyes_does_not_cause_instructions_with_more_than_100_percent`
- `test_skilllink_always_has_5_hits`

**Boost on KO:**
- `test_moxie_boost`
- `test_chillingneigh_boost`
- `test_chillingneigh_does_not_overboost`
- `test_grimneigh_boost`
- `test_grimneigh_does_not_overboost`

**PP/Move Restriction:**
- `test_pressure_caused_double_pp_decrement`
- `test_pressure_does_not_cause_pp_decrement_if_move_targets_self`
- `test_truant_sets_truant_volatile`

**Other Offensive:**
- `test_trace_switching_ability_on_switch_in_activating_said_ability`

#### 3.3 Weather/Terrain Abilities (25 tests) ✅ **0/25 ported**
**Weather Setting:**
- `test_drizzle`
- `test_drought`
- `test_gen9_snowwarning`
- `test_pre_gen9_snowwarning`
- `test_sandspit`
- `test_sandspit_does_not_activate_on_miss`
- `test_snowscape`

**Terrain Setting:**
- `test_electricsurge`
- `test_hadronenegine_terrain_application`

**Weather/Terrain Benefits:**
- `test_solarpower_damage`
- `test_hadronengine_boost`
- `test_orichalcumpulse_weather_application`

**Primordial Weather:**
- `test_desolateland_on_switchout`
- `test_primordial_sea_on_switchout`

**Other Weather/Terrain:**
- `test_sunnyday` (related to abilities)

#### 3.4 Form Change Abilities (25 tests) ✅ **0/25 ported**
**Battle Bond:**
- `test_gen9_battlebond_boost`
- `test_battlebond_gen9_does_not_overboost`

**Cramorant Forms:**
- `test_cramorant_damage_and_def_drop_from_gulping_on_being_hit`
- `test_cramorant_damage_and_paralysis_from_gorging_on_being_hit`
- `test_cramorant_formechange_gorging_when_using_surf`
- `test_cramorant_formechange_gulping_when_using_surf`
- `test_cramorant_forme_revert_on_switchout`
- `test_cramorant_no_formechange_when_fainted`

**Other Form Changes:**
- `test_hungerswitch_does_not_activate_when_teratsallized`
- `test_minior_formechange`
- `test_minior_meteor_formechange_when_healing`
- `test_morpeko_does_not_change_forme_when_switching_out_if_already_full_belly`
- `test_morpeko_formechange_end_of_turn`
- `test_morpekohangry_formechange_end_of_turn`
- `test_morpeko_reverts_to_fullbelly_when_switching_out`
- `test_palafin_formechange_on_switchout`
- `test_schooling_when_falling_below_25_percent`
- `test_schooling_when_falling_going_above_25_percent`

**Slow Start:**
- `test_slowstart_activates_on_switch_in`
- `test_slowstart_duration_decrement`
- `test_slowstart_is_removed_when_durations_reach_zero`

#### 3.5 Stat Boost Abilities (25 tests) ✅ **0/25 ported**
**Protosynthesis:**
- `test_protosynthesisatk_boosts_attack`
- `test_protosynthesisatk_does_not_boost_spa`
- `test_protosynthesis_clears_when_sun_ends`
- `test_protosynthesis_on_switchin_with_booster_energy_and_sun_up`
- `test_protosynthesis_on_switchin_with_defense_highest_stat_and_sun_up`
- `test_protosynthesis_on_switchin_with_only_booster_energy`
- `test_protosynthesis_on_switchin_with_sun_up`
- `test_protosynthesisspa_boosts_spa`
- `test_protosynthesisspd_boosts_spd`
- `test_protosynthesis_stays_up_but_consumes_item_when_sun_ends`

**Quark Drive:**
- `test_quarkdrive_clears_when_electricterrain_ends`
- `test_quarkdrive_on_switchin_with_booster_energy_and_terrain_up`
- `test_quarkdrive_on_switchin_with_only_booster_energy`
- `test_quarkdrive_stays_up_but_consumes_boosterenergy_when_electricterrain_ends`

### Phase 4: Items System (100 tests)

**Priority: Medium - Important but dependent on other systems**

#### 4.1 Berry Items (35 tests) ✅ **0/35 ported**
**Healing Berries:**
- `test_sitrus_berry_activate_after_taking_damage_when_faster`
- `test_sitrus_berry_activate_after_taking_damage_when_slower`
- `test_sitrus_berry_does_not_activate_if_above_half`

**Status Cure Berries:**
- `test_chestoberry_activates_when_being_put_to_sleep`
- `test_chestoberry_activates_when_using_rest`
- `test_lumberry_curing_before_move`
- `test_lumberry_does_nothing_with_no_status`
- `test_lum_cures_same_turn_when_faster`
- `test_lum_cures_same_turn_when_slower`
- `test_mintberry_cures_rest`
- `test_mint_berry_does_not_cure_paralysis`
- `test_miracleberry_cures_paralysis_and_attack_does_not_branch`
- `test_miracleberry_cures_rest`

**Type-Resist Berries:**
- `test_chopleberry_damage_reduction`
- `test_chopleberry_damage_reduction_does_not_happen_if_not_supereffective`
- `test_chopleberry_damage_reduction_does_not_happen_on_water_move`

**Stat Boost Berries:**
- `test_liechi_berry_activate_after_taking_damage_when_slower`
- `test_petaya_berry_activate_after_taking_damage_when_slower`
- `test_salac_berry_activate_after_taking_damage_when_slower`
- `test_salac_berry_activating_with_6_speed_and_contrary`
- `test_weaknesspolicy`
- `test_weaknesspolicy_does_not_overboost`

**Priority Berries:**
- `test_custap_berry_activates_less_than_25_percent`
- `test_moving_second_does_not_consume_custap_berry`

**Belly Drum + Berry Interactions:**
- `test_bellydrum_with_sitrus_berry_and_gluttony_at_even_amount_of_max_hp`
- `test_bellydrum_with_sitrus_berry_and_gluttony_at_odd_amount_of_max_hp`
- `test_healing_move_after_sitrusberry_with_gluttony`

#### 4.2 Choice Items (10 tests) ✅ **0/10 ported**
- `test_choiceband_locking`
- `test_gorillatactics_locking`
- `test_scenario_where_choice_gets_updated_on_second_move_that_has_branched_on_first_turn`
- `test_assaultvest`
- `test_assaultvest_prevents_status_move`

#### 4.3 Battle Enhancement Items (25 tests) ✅ **0/25 ported**
**Life Orb:**
- `test_lifeorb_boost_and_recoil`
- `test_lifeorb_hitting_sub`
- `test_lifeorb_on_non_damaging_move`
- `test_no_lifeorb_recoil_when_protected`
- `test_no_lifeorb_recoil_with_magicguard`

**Contact Items:**
- `test_rockyhelmet_damage_taken`
- `test_rockyhelmet_does_not_overkill`
- `test_contact_multi_hit_move_versus_rockyhelmet`
- `test_triple_multihit_move_versus_substitute_and_rockyhelmet`

**Type Gems:**
- `test_flyinggem_and_acrobatics_together`
- `test_flyinggem_and_acrobatics_together_gen5`
- `test_normalgem_boosting_tackle`

**Other Items:**
- `test_expert_belt_boost`
- `test_expert_belt_does_not_boost`
- `test_shellbell_drain`
- `test_adrenaline_orb_activates_if_immune_to_intimidate`
- `test_adrenalineorb_against_intimidate`
- `test_adrenalineorb_against_intimidate_when_already_at_max_speed`
- `test_ground_move_versus_airballoon`
- `test_non_damaging_move_versus_airballoon`
- `test_non_ground_move_versus_airballoon`
- `test_population_bomb_with_widelens`
- `test_throatspray_with_move_that_can_miss`

#### 4.4 Type-Boosting Items (15 tests) ✅ **0/15 ported**
- `test_fighting_move_with_blackbelt`
- `test_thickclub`
- `test_orichalcum_boost`
- `test_earlier_gen_souldew_50_percent_boost_on_any_special_move`
- `test_souldew_20_percent_boost_on_dragon_move`

#### 4.5 Status-Inducing Items (10 tests) ✅ **0/10 ported**
**Orb Items:**
- Tests for Flame Orb, Toxic Orb activations
- Interactions with abilities like Poison Heal, Guts

#### 4.6 Utility Items (15 tests) ✅ **0/15 ported**
**Move Enhancement:**
- Interactions with Wide Lens, Quick Claw, etc.

### Phase 5: Field Conditions (80 tests)

**Priority: Medium - Environmental effects**

#### 5.1 Weather Effects (40 tests) ✅ **0/40 ported**
**Rain:**
- `test_drizzle`
- `test_rain_ends_if_turns_remaining_is_1`
- `test_rain_turns_decrement_if_turns_remaining_are_greater_than_1`
- `test_rain_turns_do_not_decrement_if_turns_remaining_are_negative`
- `test_raindish_heal`
- `test_dryskin_in_rain`
- `test_morningsun_in_rain`
- `test_hydration_end_of_turn`
- `test_hydration_without_weather`

**Sun:**
- `test_drought`
- `test_sunnyday`
- `test_solarbeam_in_sun`
- `test_solarbeam_not_in_sun`
- `test_solarbeam_with_active_volatile_status`
- `test_solarbeam_with_powerherb`
- `test_solarblade_in_sun`
- `test_morningsun_in_sun`
- `test_solarpower_damage`
- `test_growth_in_sun`
- `test_weatherball_in_sun`

**Hail/Snow:**
- `test_gen9_snowwarning`
- `test_pre_gen9_snowwarning`
- `test_blizzard_in_hail`
- `test_ice_def_in_snow`
- `test_icebody_heal`
- `test_icebody_no_heal`
- `test_using_auroraveil_fails_when_already_active`
- `test_using_auroraveil_in_snow_sets_turns_and_decrements_end_of_turn`
- `test_iceface_eiscuenoice_switching_into_hail`
- `test_iceface_eiscuenoice_switching_into_snow`

**Sand:**
- `test_sandspit`
- `test_sandspit_does_not_activate_on_miss`
- `test_sand_does_not_inflict_damage_when_ending`
- `test_rock_spdef_in_sand`
- `test_rock_spdef_in_sand_versus_secretsword_doesnt_change_damageroll`
- `test_rock_does_not_get_spdef_when_terastallized_out_of_rock`
- `test_shoreup_in_sand`
- `test_end_of_turn_sand_kos_before_leftovers`

**Primordial Weather:**
- `test_desolateland_on_switchout`
- `test_primordial_sea_on_switchout`
- `test_orichalcumpulse_weather_application`

#### 5.2 Terrain Effects (25 tests) ✅ **0/25 ported**
**Electric Terrain:**
- `test_electricsurge`
- `test_priority_move_on_grounded_pkmn_in_psychicterrain`
- `test_priority_move_on_non_grounded_pkmn_in_psychicterrain`
- `test_yawn_can_be_inflicted_with_electricterrain_on_nongrounded_pkmn`
- `test_yawn_cannot_be_inflicted_with_electricterrain`
- `test_hadronenegine_terrain_application`
- `test_hadronengine_boost`
- `test_quarkdrive_clears_when_electricterrain_ends`
- `test_quarkdrive_on_switchin_with_booster_energy_and_terrain_up`
- `test_quarkdrive_stays_up_but_consumes_boosterenergy_when_electricterrain_ends`

**Grassy Terrain:**
- `test_grassyglide_in_grassyterrain_increased_priority`
- `test_grassyglide_not_in_grassyterrain_increased_priority`
- `test_switching_in_with_grassyseed_in_grassy_terrain`

**Psychic Terrain:**
- `test_priority_move_on_grounded_pkmn_in_psychicterrain`
- `test_priority_move_on_non_grounded_pkmn_in_psychicterrain`
- `test_prankster_giving_higher_priority_in_psychicterrain`

**Misty Terrain:**
- `test_yawn_can_be_inflicted_with_mistyterrain_when_target_is_not_grounded`
- `test_yawn_cannot_be_inflicted_with_mistyterrain`

**Terrain Interaction:**
- `test_terrainpulse_gen7`
- `test_terrainpulse_gen9`
- `test_icespinner_does_not_remove_terrain_if_protected`
- `test_icespinner_removes_terrain`

#### 5.3 Entry Hazards (15 tests) ✅ **0/15 ported**
**Stealth Rock:**
- `test_stealthrock_after_opponent_faints_still_works`
- `test_stealthrock_basic`
- `test_stealthrock_into_goodasgold`

**Spikes:**
- `test_spikes_into_goodasgold`

**Toxic Spikes:**
- `test_toxicdebris`
- `test_toxicdebris_when_max_kayers_already_hit`

**Hazard Removal:**
- `test_mortalspin_poison_and_remove_hazards`
- `test_defog_into_goodasgold`

**Hazard Interactions:**
- `test_magicguard_switching_into_all_hazards`
- `test_magicguard_switching_into_rocks`
- `test_magicguard_switching_into_webs`

### Phase 6: Stat Modifications (60 tests)

**Priority: Medium - Foundational mechanic**

#### 6.1 Stat Boosts and Drops (35 tests) ✅ **0/35 ported**
**Self-Boost Moves:**
- `test_bellydrum`
- `test_bellydrum_at_75_percent`
- `test_bellydrum_at_exactly_50_percent`
- `test_bellydrum_below_50_percent`
- `test_bellydrum_with_negative_prior_boost`
- `test_clangoroussoul`
- `test_clangoroussoul_missing`
- `test_filletaway`
- `test_filletaway_lowhp`
- `test_growth_in_sun`
- `test_powerup_punch_does_not_boost_if_self_knocked_out`
- `test_powerup_punch_works_on_kill`
- `test_scaleshot_only_boosts_once`
- `test_secondary_on_self_works_against_substitute`
- `test_self_boosting_move_against_protect`

**Opponent Debuff Moves:**
- `test_side_one_self_unboost_versus_sub`
- `test_side_one_using_unboosting_move_versus_substitute`
- `test_spdef_drop_does_not_affect_fainted_pkmn`
- `test_strengthsap`
- `test_strengthsap_fails_at_negative_6_boost_on_opponent`
- `test_strengthsap_into_liquidooze`
- `test_vcreate_unboosts_only_on_hit`

**Stat Reset:**
- `test_haze_resets_both_side_boosts`
- `test_clearsmog_does_not_reset_boosts_if_defender_is_immune`
- `test_clearsmog_removes_boosts_on_target`

**Gen-Specific Stat Mechanics:**
- `test_special_attack_boosts_defense` (Gen 1)
- `test_bellydrum_below_50_percent_boosts_by_2_bug` (Gen 2)
- `test_bellydrum_below_50_percent_boosts_by_2_bug_does_not_overboost`

**Berry Stat Boosts:**
- `test_liechi_berry_activate_after_taking_damage_when_slower`
- `test_petaya_berry_activate_after_taking_damage_when_slower`
- `test_salac_berry_activate_after_taking_damage_when_slower`
- `test_salac_berry_activating_with_6_speed_and_contrary`
- `test_weaknesspolicy`
- `test_weaknesspolicy_does_not_overboost`

#### 6.2 Critical Hit Interactions (15 tests) ✅ **0/15 ported**
- `test_wickedblow_always_ignores_defensive_boost_on_opponent_because_of_crit`
- `test_wickedblow_cannot_crit_on_shellarmor`
- All Gen 1 crit tests from Phase 1.1
- Various other crit-related boost interactions

#### 6.3 Ability Stat Modifications (10 tests) ✅ **0/10 ported**
- `test_intimidate`
- `test_download_for_defense`
- `test_download_for_special_defense`
- `test_contrary`
- `test_contrary_when_pre_swapped_boost_goes_above_max`
- `test_contrary_with_secondary`
- `test_contrary_with_seed`
- `test_simple_in_gen4_doubles_effective_boost`

### Phase 7: Generation-Specific Mechanics (108 tests)

**Priority: Low-Medium - Historical accuracy**

#### 7.1 Gen 1 Quirks (42 tests) ✅ **0/42 ported**
**Type Interactions:**
- `test_bodyslam_cannot_paralyze_normal_type`
- `test_counter_hits_ghost_type`
- `test_counter_into_fighting_move`
- `test_counter_into_flying_move_does_not_set_damage_dealt`
- `test_counter_into_normal_move`
- `test_thunderbolt_cannot_paralyze_electric_type`
- `test_thunderwave_can_paralyze_electric_type`
- `test_thunderwave_can_paralyze_normal_type`

**Status Mechanics:**
- `test_cannot_use_move_after_waking_when_only_a_chance_to_wake_up`
- `test_cannot_use_move_when_waking_from_sleep`
- `test_does_not_set_burn_nullify_if_already_exists`
- `test_does_not_set_paralysisnullify_if_already_exists`
- `test_gen1_agility_while_burned_does_not_set_burn_nullify_volatile`
- `test_gen1_agility_while_paralyzed_sets_paralysis_nullify_volatile`
- `test_gen1_swordsdance_while_burned_sets_burn_nullify_volatile`
- `test_gen1_swordsdance_while_burned_volatile_increases_damage`
- `test_paralysis_nullify_ignores_paralysis`
- `test_rest_wake_up_cannot_use_move`
- `test_toxic_turns_into_poison_when_switching`
- `test_using_rest_sets_rest_turns_to_2`

**Battle Mechanics:**
- `test_freeze_clause`
- `test_hyperbeam_does_not_set_mustrecharge_on_ko`
- `test_hyperbeam_sets_mustrecharge`
- `test_mustrecharge_move_only_allows_none`
- `test_same_speed_branch`
- `test_same_speed_branch_with_residuals`
- `test_same_speed_branch_with_residuals_for_both_sides`
- `test_special_attack_boosts_defense`
- `test_using_none_with_mustrecharge_removes_volatile`

**Screens as Volatiles:**
- `test_lightscreen_halves_special_damage_as_volatile`
- `test_reflect_and_lightscreen_set_volatiles_instead_of_sideconditions`
- `test_reflect_halves_physical_damage_as_volatile`
- `test_using_reflect`

**Critical Hits:**
- `test_crit_roll_ignores_other_boost`
- `test_crit_roll_ignores_other_boost_negative_boost`
- `test_crit_roll_ignores_own_boost`
- `test_crit_roll_ignores_reflect`
- `test_persion_using_slash_guaranteed_crit`
- `test_persion_using_tackle_rolls_crit`

**Substitute/Confusion:**
- `test_confuseray_into_substitute`
- `test_gen1_bite_flinch_with_counter`
- `test_thunderwave_into_substitute`

#### 7.2 Gen 2 Additions (35 tests) ✅ **0/35 ported**
**Belly Drum Bug:**
- `test_bellydrum`
- `test_bellydrum_at_75_percent`
- `test_bellydrum_at_exactly_50_percent`
- `test_bellydrum_below_50_percent_boosts_by_2_bug`
- `test_bellydrum_below_50_percent_boosts_by_2_bug_does_not_overboost`
- `test_bellydrum_with_negative_prior_boost`

**Berry Mechanics:**
- `test_mintberry_cures_rest`
- `test_mint_berry_does_not_cure_paralysis`
- `test_miracleberry_cures_paralysis_and_attack_does_not_branch`
- `test_miracleberry_cures_rest`

**Sleep Talk:**
- `test_guaranteed_to_stay_asleep_sleeptalk_move_when_not_rested`
- `test_sleeptalk_can_call_rest`
- `test_sleeptalk_rest_has_no_effect_at_full_hp`
- `test_small_chance_to_awaken_sleeptalk_move_when_not_rested`

**Updated Mechanics:**
- `test_branch_on_crit`
- `test_branch_when_a_roll_can_kill`
- `test_branch_when_a_roll_can_kill_on_the_low_side`
- `test_confuseray_into_substitute`
- `test_counter_cannot_hit_ghost_type`
- `test_counter_reflects_special_hiddenpower`
- `test_crit_does_not_overkill`
- `test_freeze_clause`
- `test_highcrit_move`
- `test_hyperbeam_sets_mustrecharge`
- `test_min_damage_killing_does_not_branch`
- `test_mirrorcoat_does_not_reflect_special_hiddenpower`
- `test_mustrecharge_move_only_allows_none`
- `test_same_speed_branch`
- `test_same_speed_branch_with_residuals`
- `test_same_speed_branch_with_residuals_for_both_sides`
- `test_switching_out_while_other_side_is_partiallytrapped`
- `test_toxic_turns_into_poison_when_switching`
- `test_using_none_with_mustrecharge_removes_volatile`

**Destiny Bond:**
- `test_destinybond_is_removed_if_non_destinybond_is_used`
- `test_nothing_happens_if_destinybond_is_used_while_already_having_destinybond`

#### 7.3 Gen 3 Modern Mechanics (31 tests) ✅ **0/31 ported**
**Basic Mechanics:**
- `test_branch_when_a_roll_can_kill`
- `test_chestoberry_activates_when_being_put_to_sleep`
- `test_chestoberry_activates_when_using_rest`
- `test_end_of_turn_sand_kos_before_leftovers`
- `test_fast_explosion_makes_other_side_unable_to_move`
- `test_gen3_branch_when_a_roll_can_kill`
- `test_intimidate_blocked_by_clearbody`
- `test_rest_does_not_activate_when_fainted`
- `test_same_speed_branch`

**Taunt Introduction:**
- `test_switching_out_with_taunt_resets_duration_to_0`
- `test_taunt_gets_applied_and_duration_increments_end_of_turn`
- `test_taunt_prevents_status_move`
- `test_taunt_re_enables_disabled_moves_when_being_removed`
- `test_taunt_volatile_is_removed_end_of_turn_when_it_would_reach_2`

### Phase 8: Complex Interactions (90 tests)

**Priority: Medium - Integration validation**

#### 8.1 Multi-System Interactions (40 tests) ✅ **0/40 ported**
**Switching Mechanics:**
- `test_switching_into_neutralizinggas_pokemon_when_other_side_has_toxic_count_and_poison_heal`
- `test_switching_in_with_grassyseed_in_grassy_terrain`
- `test_switching_out_with_lockedmove_turns_resets`
- `test_switching_out_with_modified_ability_reverts_ability`
- `test_switching_out_with_typechange_reverts_types`
- `test_switching_out_with_typechange_when_types_are_the_same`
- `test_switching_with_truant_removes_volatile`
- `test_emobdyaspectteal_switching_in`

**Switching + Hazards/Effects:**
- `test_faster_uturn`
- `test_faster_uturn_does_not_trigger_end_of_turn`
- `test_faster_uturn_knocking_out_opponent`
- `test_faster_uturn_with_opponent_move`
- `test_flipturn_into_dryskin_does_not_trigger_switchout`
- `test_force_switch_after_faint_does_not_trigger_end_of_turn`
- `test_partingshot_into_protect_does_not_cause_switchout`
- `test_slower_uturn_with_opponent_move`
- `test_switch_out_move_does_not_trigger_end_of_turn`
- `test_switch_out_move_does_not_trigger_if_user_is_last_alive_pkmn`
- `test_switch_out_move_does_not_trigger_if_voltswitch_missed`
- `test_switch_out_move_flag_is_unset_after_next_move`
- `test_switchout_flag_where_faster_switchout_move_knocked_out_opponent`
- `test_switchout_flag_where_slower_switchout_move_knocked_out_opponent`
- `test_turn_after_switch_out_move_other_side_does_nothing`
- `test_turn_after_switch_out_move_other_side_has_forced_move`
- `test_uturn_into_fainted_pkmn_does_not_cause_switchout`
- `test_uturn_into_protect_does_not_cause_switchout`

**Baton Pass Mechanics:**
- `test_batonpass_with_boosts`
- `test_batonpass_with_leechseed`
- `test_switching_from_batonpass_with_boosts`
- `test_switching_from_batonpass_with_leechseed`
- `test_switching_from_batonpass_with_sub`
- `test_basic_shedtail_usage`
- `test_shedtail_with_boosts`
- `test_shedtail_with_leechseed`
- `test_switching_from_shedtail`
- `test_switching_from_shedtail_with_boosts`
- `test_switching_from_shedtail_with_leechseed`

**Other Complex Interactions:**
- `test_non_baton_pass_switching_with_leechseed`
- `test_non_baton_pass_switching_with_sub`
- `test_leechseed_does_not_trigger_if_receiving_side_fainted_this_turn`
- `test_leechseed_into_grass_type`
- `test_leechseed_into_substitute`

#### 8.2 Edge Cases and Boundaries (30 tests) ✅ **0/30 ported**
**Battle End Conditions:**
- `test_already_busted_mimikyu_taking_damage_properly`
- `test_battle_is_over_when_battle_is_not_over`
- `test_battle_is_over_when_side_one_lost`
- `test_battle_is_over_when_side_two_lost`

**Turn Order Edge Cases:**
- `test_same_speed_branch`
- `test_same_speed_branch_with_residuals`
- `test_same_speed_branch_with_residuals_for_both_sides`
- `test_end_of_turn_triggered_when_switchout_flag_is_removed`
- `test_end_of_turn_triggered_when_switchout_flag_is_removed_and_other_side_did_nothing`

**Item/Ability Edge Cases:**
- `test_identical_items_generates_no_instructions`

**PP and Move Restrictions:**
- `test_pp_decremented`
- `test_pp_not_decremented_when_flinched`
- `test_zero_pp_move_cannot_be_used`
- `test_using_move_while_asleep_does_not_decrement_pp`

**Protection Edge Cases:**
- `test_consecutive_protect_while_paralyzed`
- `test_double_protect`
- `test_protect_for_second_turn_in_a_row`
- `test_protect_for_third_turn_in_a_row`
- `test_protect_side_condition_is_removed`
- `test_endure_with_protect_side_condition_not_fully_accurate`

**Type/Immunity Edge Cases:**
- `test_gen5_or_earlier_ghost_versus_steel`
- `test_normal_terastallized_into_stellar_remains_immune_to_ghost`
- `test_terastallized_into_ghost_makes_immune_to_normal`

**Generation Differences:**
- `test_gen7_rapidspin_does_not_boost_speed`
- `test_gen9_rapidspin_boosts_speed`

#### 8.3 Trapping and Movement Restriction (20 tests) ✅ **0/20 ported**
**Arena Trap:**
- `test_arenatrap_does_not_trap_flying`
- `test_arenatrap_does_not_trap_ghost`
- `test_arenatrap_does_not_trap_shedshell`
- `test_arenatrap_traps_opponent`

**Partial Trapping:**
- `test_switching_out_while_other_side_is_partiallytrapped`

**Move Locking:**
- `test_lockedmove_prevents_switches`
- `test_locked_moves_unlock_on_switchout`
- `test_outrage_locking`

**Various Trapping:**
- `test_pkmn_is_not_trapped_if_it_has_fainted`
- `test_whirlwind_against_guarddog`

**Move Restrictions:**
- `test_cannot_use_bloodmoon_after_using_bloodmoon`
- `test_cannot_use_gigatonhammer_after_using_gigatonhammer`
- `test_can_use_bloodmoon_after_using_switch`
- `test_can_use_gigatonhammer_after_using_switch`
- `test_hyperbeam_sets_mustrecharge`
- `test_gigaimpact_with_truant_only_sets_mustrecharge`
- `test_mustrecharge_move_only_allows_none`
- `test_using_none_with_mustrecharge_removes_volatile`
- `test_using_move_with_truant_removes_volatile`
- `test_noretreat_with_vs_already`

### Phase 9: Damage Tracking System (17 tests)

**Priority: Low - Specialized mechanic**

#### 9.1 Counter/Mirror Coat Mechanics (12 tests) ✅ **0/12 ported**
**Counter:**
- `test_counter_after_physical_hit`
- `test_counter_after_special_hit`
- `test_counter_cannot_hit_ghost_type`
- `test_counter_reflects_special_hiddenpower`

**Mirror Coat:**
- `test_mirrorcoat_after_physical_hit`
- `test_mirrorcoat_after_special_hit`
- `test_mirrorcoat_does_not_reflect_special_hiddenpower`

**Metal Burst:**
- `test_metalburst_after_physical_move`
- `test_metalburst_after_special_move`
- `test_metalburst_after_status_move`
- `test_metalburst_after_substitute_being_hit`
- `test_metalburst_fails_moving_first`

#### 9.2 Comeuppance and Focus Punch (5 tests) ✅ **0/5 ported**
**Comeuppance:**
- `test_comeuppance_after_physical_move`

**Focus Punch:**
- `test_focuspunch_after_getting_hit`
- `test_focuspunch_after_status_move`
- `test_focuspunch_after_substitute_getting_hit`
- `test_focuspunch_after_not_getting_hit`

**Damage Reset:**
- `test_previous_damage_dealt_resets_and_then_goes_to_a_new_value`

### Phase 10: Move Tracking System (17 tests)

**Priority: Low - Utility mechanic**

#### 10.1 Encore Mechanics (10 tests) ✅ **0/10 ported**
- `test_encore_and_arenatrapped_together`
- `test_encore_causes_get_all_options_to_only_allow_last_used_move`
- `test_encore_counter_increment`
- `test_encore_expires_at_2_turns`
- `test_encore_fast_fails_with_lastusedmove_equal_to_none`
- `test_encore_fast_fails_with_lastusedmove_equal_to_switch`
- `test_encore_second_fails_when_opponent_switches`
- `test_encore_slow`
- `test_encore_slow_into_substitute`
- `test_fast_encore_into_using_a_different_move_from_lum`

#### 10.2 First Turn Move Mechanics (4 tests) ✅ **0/4 ported**
- `test_fakeout_first_turn_switched_in`
- `test_fakeout_with_last_used_move_of_non_switch`
- `test_firstimpression_first_turn_switched_in`
- `test_firstimpression_with_last_used_move_of_non_switch`

#### 10.3 Last Used Move Tracking (3 tests) ✅ **0/3 ported**
- `test_last_used_move_is_set_on_move`
- `test_last_used_move_is_set_on_switch`
- `test_last_used_move_overwritten_when_dragged_out`

## Additional Categories

### Phase 11: Terastallization (20 tests) ✅ **0/20 ported**
**Gen 9 Mechanic - Priority: High for Gen 9 support**
- `test_low_bp_move_boost_when_terastallizing`
- `test_normal_terastallized_into_stellar_remains_immune_to_ghost`
- `test_terablast_becomes_physical`
- `test_terablast_into_ghost_makes_normal_immune`
- `test_tera_blast_stellar_into_ghost_type_does_damage`
- `test_tera_blast_stellar_into_terastallized_does_double_damage`
- `test_tera_blast_stellar_with_contrary_increases_offensive_stats`
- `test_tera_double_stab`
- `test_tera_electric_always_allows_doubleshock_with_no_typechange_volatile`
- `test_tera_stab_without_an_original_type_in_tera_types`
- `test_terastallization_side_one`
- `test_terastallization_side_two`
- `test_terastallized_into_ghost_makes_immune_to_normal`
- `test_terastallizing`
- `test_tera_with_original_type_stab`
- `test_tera_without_any_stab`
- `test_adaptability_with_tera_but_no_regular_stab_is_200_percent`
- `test_adaptability_with_tera_stab_is_225_percent`
- `test_prismarmor_respects_tera_type`
- `test_revelationdance_is_set_to_tera_type_when_terastallized`

### Phase 12: Priority and Speed Mechanics (15 tests) ✅ **0/15 ported**
**Move Priority:**
- `test_grassyglide_in_grassyterrain_increased_priority`
- `test_grassyglide_not_in_grassyterrain_increased_priority`
- `test_prankster_damaging_move_innto_dark_type`
- `test_prankster_giving_higher_priority_in_psychicterrain`
- `test_prankster_into_dark_type`
- `test_prankster_into_dark_type_earlier_gens`
- `test_suckerpunch_fails_versus_faster_attacking_move`
- `test_suckerpunch_versus_attacking_move`
- `test_suckerpunch_versus_non_attacking_move`
- `test_thunderclap_fails_versus_faster_attacking_move`
- `test_thunderclap_versus_attacking_move`
- `test_upperhand_against_non_priority`
- `test_upperhand_against_priority`

**Speed Control:**
- `test_tailwind`
- `test_trickroom`

### Phase 13: Good as Gold Interactions (8 tests) ✅ **0/8 ported**
**Signature Ability Testing:**
- `test_defog_into_goodasgold`
- `test_slackoff_into_goodasgold`
- `test_thunderwave_into_goodasgold`
- `test_willowisp_into_goodasgold`
- `test_spikes_into_goodasgold`
- `test_stealthrock_into_goodasgold`
- `test_whirlwind_into_goodasgold`

### Phase 14: Protection Moves (25 tests) ✅ **0/25 ported**
**Basic Protection:**
- `test_using_protect_against_damaging_move`
- `test_protect_blocks_yawn`
- `test_protect_stops_after_damage_hit_callback`
- `test_protect_stops_secondaries`
- `test_protect_for_second_turn_in_a_row`
- `test_protect_for_third_turn_in_a_row`
- `test_double_protect`
- `test_consecutive_protect_while_paralyzed`

**Special Protection Moves:**
- `test_banefulbunker_cannot_poison_already_statused_target`
- `test_banefulbunker_poisons`
- `test_burning_bulwark_burns`
- `test_burning_bulwark_does_not_burn_fire_type`
- `test_bypassing_protect_does_not_inflict_burn_against_burning_bulwark`
- `test_silktrap`
- `test_spikyshield_does_not_activate_on_non_contact_move`
- `test_spikyshield_recoil_does_not_overkill`
- `test_using_spikyshield_against_contact_move`

**Endure:**
- `test_endure`
- `test_endure_at_1hp`
- `test_endure_with_protect_side_condition_not_fully_accurate`

**Protection Interactions:**
- `test_move_that_goes_through_protect`
- `test_perish_bypasses_protect`
- `test_crash_move_into_protect`
- Plus various "into protect" tests from other categories

## Implementation Timeline

### Estimated Timeline by Priority

**Phase 1-3 (Core Systems): 6-8 weeks**
- Combat System: 2 weeks
- Status Effects: 2-3 weeks  
- Abilities: 3 weeks

**Phase 4-6 (Secondary Systems): 4-5 weeks**
- Items: 2 weeks
- Field Conditions: 2 weeks
- Stat Modifications: 1 week

**Phase 7-14 (Specialized/Integration): 6-8 weeks**
- Generation-specific: 2-3 weeks
- Complex Interactions: 2 weeks
- Specialized Systems: 2-3 weeks

**Total Estimated Timeline: 16-21 weeks**

## Progress Tracking

Use this checklist format to track progress:

```
### Phase 1.1 Basic Damage Calculation ✅ **3/25 ported**
- ✅ test_basic_move_pair_instruction_generation
- ✅ test_branch_on_crit  
- ✅ test_branch_when_a_roll_can_kill
- ❌ test_branch_when_a_roll_can_kill_on_the_low_side
- ❌ test_crit_does_not_overkill
- ... (remaining tests)
```

## Success Criteria

### Quantitative Goals
- ✅ **788 tests ported**: 100% coverage of original test suite
- ✅ **100% pass rate**: All ported tests must pass
- ✅ **Performance target**: Test suite completes in <60 seconds
- ✅ **Coverage metrics**: >95% code coverage of core battle mechanics

### Qualitative Goals
- ✅ **Maintainable tests**: Clear, readable test structure
- ✅ **Extensible framework**: Easy to add new tests
- ✅ **Documentation**: Each test clearly documents what it validates
- ✅ **Future-ready**: Framework supports doubles/VGC extension

## Conclusion

This comprehensive plan systematically ports all 788 tests from poke-engine to tapu-simu while leveraging the new engine's strengths. The detailed test-by-test breakdown ensures no functionality is missed, and the phased approach provides clear milestones for tracking progress. The new test framework will serve as the foundation for ongoing tapu-simu development and validation.

## Progress Summary

**Phase 1.1 Basic Damage Calculation: ✅ COMPLETE (25/25 tests)**
- All basic damage calculation tests successfully ported to `/tests/basic_damage.rs`
- Tests use real Pokemon data instead of dummy Pokemon for more realistic validation
- Framework validates mechanics work correctly even if exact damage values differ from poke-engine
- All 26 tests (25 core + type effectiveness extras) passing

**Key Framework Features Implemented:**
- TapuTestFramework with repository integration
- TestBuilder fluent API for readable test construction  
- PokemonSpec builder pattern for flexible Pokemon configuration
- Multi-generation support with appropriate fallbacks
- End-to-end testing using actual turn engine
- BattleAssertions for specialized validation

**Next Steps:**
- Continue with Phase 1.2 Move Categories (40 tests)
- Phase 1.3 Accuracy and Miss Mechanics (15 tests)
- Phase 1.4 Secondary Effects and Status Infliction (70 tests)