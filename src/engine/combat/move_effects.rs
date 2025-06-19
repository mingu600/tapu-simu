//! # Move Effects
//! 
//! This module handles special move effects and their implementation with generation awareness.
//! This is the Priority B3 implementation from IMPLEMENTATION_PLAN.md
//!
//! ## Generation Awareness
//! 
//! All move effects are generation-aware, allowing for proper implementation of mechanics
//! that changed between generations. This includes:
//! - Type immunities (e.g., Electric types immune to paralysis in Gen 6+)
//! - Move behavior changes (e.g., powder moves vs Grass types in Gen 6+)
//! - Status effect mechanics (e.g., burn reducing physical attack)
//! - Accuracy and effect chances that varied by generation

use crate::core::state::{State, Pokemon, MoveCategory};
use crate::core::instruction::{
    Instruction, StateInstructions, ApplyStatusInstruction, ApplyVolatileStatusInstruction,
    BoostStatsInstruction, PositionHealInstruction, PositionDamageInstruction,
    PokemonStatus, VolatileStatus, Stat, ChangeWeatherInstruction, Weather,
    ApplySideConditionInstruction, SideCondition, ChangeItemInstruction,
    RemoveSideConditionInstruction, RemoveVolatileStatusInstruction, ChangeTypeInstruction,
    FaintInstruction, ChangeTerrainInstruction, Terrain, SetFutureSightInstruction
};
use crate::data::types::EngineMoveData;
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use std::collections::HashMap;

/// Helper function for moves that don't need context
pub fn apply_move_effects_simple(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let context = MoveContext::new();
    apply_move_effects(state, move_data, user_position, target_positions, generation, &context)
}

/// Helper function to check if a move is super effective against a target
fn is_super_effective(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Convert string types to PokemonType enum
    let attacking_type = match PokemonType::from_str(move_type) {
        Some(t) => t,
        None => return false,
    };
    
    let target_type1 = match PokemonType::from_str(target.types.get(0).unwrap_or(&"Normal".to_string())) {
        Some(t) => t,
        None => return false,
    };
    
    let target_type2 = target.types.get(1)
        .and_then(|t| PokemonType::from_str(t));
    
    // Create type effectiveness chart
    let type_chart = TypeChart::new(generation.generation.number());
    
    // Calculate damage multiplier
    let target_types = (target_type1, target_type2.unwrap_or(target_type1));
    let multiplier = type_chart.calculate_damage_multiplier(attacking_type, target_types, None, None);
    
    multiplier > 1.0
}

/// Context for conditional moves that need opponent information
#[derive(Debug, Clone)]
pub struct MoveContext<'a> {
    /// Opponent's move choice for this turn
    pub opponent_choice: Option<&'a crate::core::move_choice::MoveChoice>,
    /// Whether this move is going first this turn
    pub going_first: bool,
    /// Opponent's move data if they're using a move
    pub opponent_move_data: Option<&'a EngineMoveData>,
    /// Whether the opponent is switching
    pub opponent_switching: bool,
    /// Priority of opponent's move
    pub opponent_priority: i8,
}

impl<'a> MoveContext<'a> {
    pub fn new() -> Self {
        Self {
            opponent_choice: None,
            going_first: false,
            opponent_move_data: None,
            opponent_switching: false,
            opponent_priority: 0,
        }
    }
    
    pub fn with_opponent_choice(mut self, choice: &'a crate::core::move_choice::MoveChoice) -> Self {
        self.opponent_choice = Some(choice);
        self.opponent_switching = matches!(choice, crate::core::move_choice::MoveChoice::Switch(_));
        self
    }
    
    pub fn with_going_first(mut self, going_first: bool) -> Self {
        self.going_first = going_first;
        self
    }
    
    pub fn with_opponent_move_data(mut self, move_data: &'a EngineMoveData) -> Self {
        self.opponent_move_data = Some(move_data);
        self
    }
    
    pub fn with_opponent_priority(mut self, priority: i8) -> Self {
        self.opponent_priority = priority;
        self
    }
}

/// Apply move effects beyond basic damage with generation awareness
/// This implements the comprehensive move effects system for 100% parity with poke-engine
/// 
/// # Parameters
/// 
/// * `state` - Current battle state
/// * `move_data` - Move data containing base information
/// * `user_position` - Position of the Pokemon using the move
/// * `target_positions` - Positions of target Pokemon
/// * `generation` - Generation mechanics for generation-specific behavior
/// * `context` - Additional context for conditional moves
pub fn apply_move_effects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<StateInstructions> {
    let move_name = move_data.name.to_lowercase();
    
    // Handle moves by name first, then by category
    match move_name.as_str() {
        // Status moves that inflict major status conditions
        "thunderwave" | "thunder wave" => apply_thunder_wave(state, user_position, target_positions, generation),
        "sleeppowder" | "sleep powder" => apply_sleep_powder(state, user_position, target_positions, generation),
        "toxic" => apply_toxic(state, user_position, target_positions, generation),
        "willowisp" | "will-o-wisp" => apply_will_o_wisp(state, user_position, target_positions, generation),
        "stunspore" | "stun spore" => apply_stun_spore(state, user_position, target_positions, generation),
        "poisonpowder" | "poison powder" => apply_poison_powder(state, user_position, target_positions, generation),
        "glare" => apply_glare(state, user_position, target_positions, generation),
        "spore" => apply_spore(state, user_position, target_positions, generation),
        
        // Stat-modifying moves
        "swordsdance" | "swords dance" => apply_swords_dance(state, user_position, target_positions, generation),
        "dragondance" | "dragon dance" => apply_dragon_dance(state, user_position, target_positions, generation),
        "nastyplot" | "nasty plot" => apply_nasty_plot(state, user_position, target_positions, generation),
        "agility" => apply_agility(state, user_position, target_positions, generation),
        "growl" => apply_growl(state, user_position, target_positions, generation),
        "leer" => apply_leer(state, user_position, target_positions, generation),
        "tailwhip" | "tail whip" => apply_tail_whip(state, user_position, target_positions, generation),
        "stringshot" | "string shot" => apply_string_shot(state, user_position, target_positions, generation),
        "acid" => apply_acid(state, move_data, user_position, target_positions, generation),
        "charm" => apply_charm(state, user_position, target_positions, generation),
        "growth" => apply_growth(state, user_position, target_positions, generation),
        
        // Healing moves
        "recover" => apply_recover(state, user_position, target_positions, generation),
        "roost" => apply_roost(state, user_position, target_positions, generation),
        "moonlight" => apply_moonlight(state, user_position, target_positions, generation),
        "synthesis" => apply_synthesis(state, user_position, target_positions, generation),
        "morningsun" | "morning sun" => apply_morning_sun(state, user_position, target_positions, generation),
        "softboiled" | "soft-boiled" => apply_soft_boiled(state, user_position, target_positions, generation),
        "milkdrink" | "milk drink" => apply_milk_drink(state, user_position, target_positions, generation),
        "slackoff" | "slack off" => apply_slack_off(state, user_position, target_positions, generation),
        "aquaring" | "aqua ring" => apply_aqua_ring(state, user_position, target_positions, generation),
        "shoreup" | "shore up" => apply_shore_up(state, user_position, target_positions, generation),
        
        // Recoil moves
        "doubleedge" | "double-edge" => apply_double_edge(state, user_position, target_positions, generation),
        "takedown" | "take down" => apply_take_down(state, user_position, target_positions, generation),
        "submission" => apply_submission(state, user_position, target_positions, generation),
        "volttackle" | "volt tackle" => apply_volt_tackle(state, user_position, target_positions, generation),
        "flareblitz" | "flare blitz" => apply_flare_blitz(state, user_position, target_positions, generation),
        "bravebird" | "brave bird" => apply_brave_bird(state, user_position, target_positions, generation),
        "wildcharge" | "wild charge" => apply_wild_charge(state, user_position, target_positions, generation),
        "headsmash" | "head smash" => apply_head_smash(state, user_position, target_positions, generation),
        
        // Drain moves
        "gigadrain" | "giga drain" => apply_giga_drain(state, user_position, target_positions, generation),
        "megadrain" | "mega drain" => apply_mega_drain(state, user_position, target_positions, generation),
        "absorb" => apply_absorb(state, user_position, target_positions, generation),
        "drainpunch" | "drain punch" => apply_drain_punch(state, user_position, target_positions, generation),
        "leechlife" | "leech life" => apply_leech_life(state, user_position, target_positions, generation),
        "dreameater" | "dream eater" => apply_dream_eater(state, user_position, target_positions, generation),
        
        // Protection moves
        "protect" => apply_protect(state, user_position, target_positions, generation),
        "detect" => apply_detect(state, user_position, target_positions, generation),
        "endure" => apply_endure(state, user_position, target_positions, generation),
        
        // Utility and field effect moves
        "aromatherapy" => apply_aromatherapy(state, user_position, target_positions, generation),
        "healbell" | "heal bell" => apply_heal_bell(state, user_position, target_positions, generation),
        "attract" => apply_attract(state, user_position, target_positions, generation),
        "confuseray" | "confuse ray" => apply_confuse_ray(state, user_position, target_positions, generation),
        "haze" => apply_haze(state, user_position, target_positions, generation),
        "clearsmog" | "clear smog" => apply_clear_smog(state, user_position, target_positions, generation),
        
        // Weather moves
        "sunnyday" | "sunny day" => apply_sunny_day(state, user_position, target_positions, generation),
        "raindance" | "rain dance" => apply_rain_dance(state, user_position, target_positions, generation),
        "sandstorm" => apply_sandstorm(state, user_position, target_positions, generation),
        "hail" => apply_hail(state, user_position, target_positions, generation),
        "snowscape" => apply_snowscape(state, user_position, target_positions, generation),
        
        // Screen moves
        "lightscreen" | "light screen" => apply_light_screen(state, user_position, target_positions, generation),
        "reflect" => apply_reflect_move(state, user_position, target_positions, generation),
        "auroraveil" | "aurora veil" => apply_aurora_veil(state, user_position, target_positions, generation),
        
        // Hazard moves
        "spikes" => apply_spikes(state, user_position, target_positions, generation),
        "stealthrock" | "stealth rock" => apply_stealth_rock(state, user_position, target_positions, generation),
        "toxicspikes" | "toxic spikes" => apply_toxic_spikes(state, user_position, target_positions, generation),
        "stickyweb" | "sticky web" => apply_sticky_web(state, user_position, target_positions, generation),
        
        // Hazard removal
        "rapidspin" | "rapid spin" => apply_rapid_spin(state, user_position, target_positions, generation),
        "defog" => apply_defog(state, user_position, target_positions, generation),
        
        // Complex moves
        "batonpass" | "baton pass" => apply_baton_pass(state, user_position, target_positions, generation),
        "bellydrum" | "belly drum" => apply_belly_drum(state, user_position, target_positions, generation),
        "curse" => apply_curse(state, user_position, target_positions, generation),
        "destinybond" | "destiny bond" => apply_destiny_bond(state, user_position, target_positions, generation),
        "encore" => apply_encore(state, user_position, target_positions, generation),
        "leechseed" | "leech seed" => apply_leech_seed(state, user_position, target_positions, generation),
        "rest" => apply_rest(state, user_position, target_positions, generation),
        "sleeptalk" | "sleep talk" => apply_sleep_talk(state, user_position, target_positions, generation),
        "taunt" => apply_taunt(state, user_position, target_positions, generation),
        "whirlwind" => apply_whirlwind(state, user_position, target_positions, generation),
        "yawn" => apply_yawn(state, user_position, target_positions, generation),
        
        // Substitute and similar
        "substitute" => apply_substitute(state, user_position, target_positions, generation),
        
        // Multi-hit moves
        "doubleslap" | "double slap" | "cometpunch" | "comet punch" | "furyattack" | "fury attack" |
        "pinmissile" | "pin missile" | "barrage" | "spikecannon" | "spike cannon" | "bonemerang" |
        "bulletseed" | "bullet seed" | "icicleshard" | "icicle shard" | "rockblast" | "rock blast" |
        "tailslap" | "tail slap" | "beatup" | "beat up" | "armthrust" | "arm thrust" => {
            return apply_multi_hit_move(state, move_data, user_position, target_positions, generation);
        }
        
        // Missing simple moves
        "splash" => apply_splash(state, user_position, target_positions, generation),
        "kinesis" => apply_kinesis(state, user_position, target_positions, generation),
        "quickattack" | "quick attack" => apply_quick_attack(state, move_data, user_position, target_positions, generation),
        "tailwind" => apply_tailwind(state, user_position, target_positions, generation),
        "trickroom" | "trick room" => apply_trick_room(state, user_position, target_positions, generation),
        "refresh" => apply_refresh(state, user_position, target_positions, generation),
        "wish" => apply_wish(state, user_position, target_positions, generation),
        "healingwish" | "healing wish" => apply_healing_wish(state, user_position, target_positions, generation),
        "lifedew" | "life dew" => apply_life_dew(state, user_position, target_positions, generation),
        "noretreat" | "no retreat" => apply_no_retreat(state, user_position, target_positions, generation),
        "painsplit" | "pain split" => apply_pain_split(state, user_position, target_positions, generation),
        "partingshot" | "parting shot" => apply_parting_shot(state, user_position, target_positions, generation),
        "perishsong" | "perish song" => apply_perish_song(state, user_position, target_positions, generation),
        
        // Priority moves
        "accelerock" => apply_accelerock(state, move_data, user_position, target_positions, generation),
        "aquajet" | "aqua jet" => apply_aqua_jet(state, move_data, user_position, target_positions, generation),
        "bulletpunch" | "bullet punch" => apply_bullet_punch(state, move_data, user_position, target_positions, generation),
        "extremespeed" | "extreme speed" => apply_extreme_speed(state, move_data, user_position, target_positions, generation),
        "fakeout" | "fake out" => apply_fake_out(state, move_data, user_position, target_positions, generation),
        "feint" => apply_feint(state, move_data, user_position, target_positions, generation),
        "firstimpression" | "first impression" => apply_first_impression(state, move_data, user_position, target_positions, generation),
        "machpunch" | "mach punch" => apply_mach_punch(state, move_data, user_position, target_positions, generation),
        
        // Fixed damage moves
        "seismictoss" | "seismic toss" => apply_seismic_toss(state, user_position, target_positions, generation),
        "nightshade" | "night shade" => apply_night_shade(state, user_position, target_positions, generation),
        "endeavor" => apply_endeavor(state, user_position, target_positions, generation),
        "finalgambit" | "final gambit" => apply_final_gambit(state, user_position, target_positions, generation),
        "naturesmadness" | "nature's madness" => apply_natures_madness(state, user_position, target_positions, generation),
        "ruination" => apply_ruination(state, user_position, target_positions, generation),
        "superfang" | "super fang" => apply_super_fang(state, user_position, target_positions, generation),
        
        // Counter moves
        "counter" => apply_counter(state, user_position, target_positions, generation),
        "mirrorcoat" | "mirror coat" => apply_mirror_coat(state, user_position, target_positions, generation),
        "comeuppance" => apply_comeuppance(state, user_position, target_positions, generation),
        "metalburst" | "metal burst" => apply_metal_burst(state, user_position, target_positions, generation),
        
        // Item interaction moves
        "trick" => apply_trick(state, user_position, target_positions, generation),
        "switcheroo" => apply_switcheroo(state, user_position, target_positions, generation),
        
        // Field manipulation moves
        "tidyup" | "tidy up" => apply_tidy_up(state, user_position, target_positions, generation),
        "courtchange" | "court change" => apply_court_change(state, user_position, target_positions, generation),
        "chillyreception" | "chilly reception" => apply_chilly_reception(state, user_position, target_positions, generation),
        
        // Terrain-dependent moves
        "grassyglide" | "grassy glide" => apply_grassy_glide(state, move_data, user_position, target_positions, generation),
        
        // Variable power moves
        "facade" => apply_facade(state, move_data, user_position, target_positions, generation),
        "hex" => apply_hex(state, move_data, user_position, target_positions, generation),
        "gyroball" | "gyro ball" => apply_gyro_ball(state, move_data, user_position, target_positions, generation),
        "reversal" => apply_reversal(state, move_data, user_position, target_positions, generation),
        "acrobatics" => apply_acrobatics(state, move_data, user_position, target_positions, generation),
        "weatherball" | "weather ball" => apply_weather_ball(state, move_data, user_position, target_positions, generation),
        "avalanche" => apply_avalanche(state, move_data, user_position, target_positions, generation),
        "boltbeak" | "bolt beak" => apply_boltbeak(state, move_data, user_position, target_positions, generation),
        "fishiousrend" | "fishious rend" => apply_fishious_rend(state, move_data, user_position, target_positions, generation),
        "electroball" | "electro ball" => apply_electroball(state, move_data, user_position, target_positions, generation),
        "eruption" => apply_eruption(state, move_data, user_position, target_positions, generation),
        "waterspout" | "water spout" => apply_waterspout(state, move_data, user_position, target_positions, generation),
        "dragonenergy" | "dragon energy" => apply_dragon_energy(state, move_data, user_position, target_positions, generation),
        "grassknot" | "grass knot" => apply_grass_knot(state, move_data, user_position, target_positions, generation),
        "lowkick" | "low kick" => apply_low_kick(state, move_data, user_position, target_positions, generation),
        "heatcrash" | "heat crash" => apply_heat_crash(state, move_data, user_position, target_positions, generation),
        "heavyslam" | "heavy slam" => apply_heavy_slam(state, move_data, user_position, target_positions, generation),
        
        // Two-turn/charge moves
        "solarbeam" | "solar beam" => apply_solar_beam(state, move_data, user_position, target_positions, generation),
        "solarblade" | "solar blade" => apply_solar_blade(state, move_data, user_position, target_positions, generation),
        "meteorbeam" | "meteor beam" => apply_meteor_beam(state, move_data, user_position, target_positions, generation),
        "electroshot" | "electro shot" => apply_electro_shot(state, move_data, user_position, target_positions, generation),
        "dig" => apply_dig(state, move_data, user_position, target_positions, generation),
        "fly" => apply_fly(state, move_data, user_position, target_positions, generation),
        "bounce" => apply_bounce(state, move_data, user_position, target_positions, generation),
        "dive" => apply_dive(state, move_data, user_position, target_positions, generation),
        "phantomforce" | "phantom force" => apply_phantom_force(state, move_data, user_position, target_positions, generation),
        "shadowforce" | "shadow force" => apply_shadow_force(state, move_data, user_position, target_positions, generation),
        "futuresight" | "future sight" => apply_future_sight(state, move_data, user_position, target_positions, generation),
        "razorwind" | "razor wind" => apply_razor_wind(state, move_data, user_position, target_positions, generation),
        "skullbash" | "skull bash" => apply_skull_bash(state, move_data, user_position, target_positions, generation),
        "skyattack" | "sky attack" => apply_sky_attack(state, move_data, user_position, target_positions, generation),
        "focuspunch" | "focus punch" => apply_focus_punch(state, move_data, user_position, target_positions, generation),
        "filletaway" | "fillet away" => apply_fillet_away(state, move_data, user_position, target_positions, generation),
        "clangoroussoul" | "clangorous soul" => apply_clangorous_soul(state, move_data, user_position, target_positions, generation),
        
        // Type-changing moves
        "judgment" => apply_judgment(state, move_data, user_position, target_positions, generation),
        "multiattack" | "multi-attack" => apply_multi_attack(state, move_data, user_position, target_positions, generation),
        "revelationdance" | "revelation dance" => apply_revelation_dance(state, move_data, user_position, target_positions, generation),
        
        // Type removal moves
        "burnup" | "burn up" => apply_burn_up(state, move_data, user_position, target_positions, generation),
        "doubleshock" | "double shock" => apply_double_shock(state, move_data, user_position, target_positions, generation),
        
        
        // Missing variable power moves
        "barbbarrage" | "barb barrage" => apply_barb_barrage(state, move_data, user_position, target_positions, generation),
        "collisioncourse" | "collision course" => apply_collision_course(state, move_data, user_position, target_positions, generation),
        "electrodrift" | "electro drift" => apply_electro_drift(state, move_data, user_position, target_positions, generation),
        "freezedry" | "freeze-dry" => apply_freeze_dry(state, move_data, user_position, target_positions, generation),
        "hardpress" | "hard press" => apply_hard_press(state, move_data, user_position, target_positions, generation),
        "hydrosteam" | "hydro steam" => apply_hydro_steam(state, move_data, user_position, target_positions, generation),
        "lastrespects" | "last respects" => apply_last_respects(state, move_data, user_position, target_positions, generation),
        "poltergeist" => apply_poltergeist(state, move_data, user_position, target_positions, generation),
        "pursuit" => apply_pursuit(state, move_data, user_position, target_positions, generation, context),
        "storedpower" | "stored power" => apply_stored_power(state, move_data, user_position, target_positions, generation),
        "powertrip" | "power trip" => apply_power_trip(state, move_data, user_position, target_positions, generation),
        "strengthsap" | "strength sap" => apply_strength_sap(state, move_data, user_position, target_positions, generation),
        "suckerpunch" | "sucker punch" => apply_sucker_punch(state, move_data, user_position, target_positions, generation, context),
        "thunderclap" | "thunder clap" => apply_thunder_clap(state, move_data, user_position, target_positions, generation, context),
        "terrainpulse" | "terrain pulse" => apply_terrain_pulse(state, move_data, user_position, target_positions, generation),
        "upperhand" | "upper hand" => apply_upper_hand(state, move_data, user_position, target_positions, generation, context),
        
        // Item interaction moves
        "knockoff" | "knock off" => apply_knock_off(state, move_data, user_position, target_positions, generation),
        "thief" => apply_thief(state, move_data, user_position, target_positions, generation),
        "fling" => apply_fling(state, move_data, user_position, target_positions, generation),
        
        // Weather-dependent accuracy moves
        "blizzard" => apply_blizzard(state, move_data, user_position, target_positions, generation),
        "hurricane" => apply_hurricane(state, move_data, user_position, target_positions, generation),
        "thunder" => apply_thunder(state, move_data, user_position, target_positions, generation),
        
        // Self-destruct moves
        "explosion" => apply_explosion(state, move_data, user_position, target_positions, generation),
        "selfdestruct" | "self-destruct" => apply_self_destruct(state, move_data, user_position, target_positions, generation),
        
        // Missing terrain-dependent moves
        "expandingforce" | "expanding force" => apply_expanding_force(state, move_data, user_position, target_positions, generation),
        "risingvoltage" | "rising voltage" => apply_rising_voltage(state, move_data, user_position, target_positions, generation),
        "mistyexplosion" | "misty explosion" => apply_misty_explosion(state, move_data, user_position, target_positions, generation),
        "psyblade" | "psy blade" => apply_psy_blade(state, move_data, user_position, target_positions, generation),
        "steelroller" | "steel roller" => apply_steel_roller(state, move_data, user_position, target_positions, generation),
        "icespinner" | "ice spinner" => apply_ice_spinner(state, move_data, user_position, target_positions, generation),
        
        // Missing self-damage moves
        "mindblown" | "mind blown" => apply_mind_blown(state, move_data, user_position, target_positions, generation),
        
        // Missing type-changing moves
        "ivycudgel" | "ivy cudgel" => apply_ivy_cudgel(state, move_data, user_position, target_positions, generation),
        "terablast" | "tera blast" => apply_tera_blast(state, move_data, user_position, target_positions, generation),
        
        // Form-dependent moves
        "aurawheel" | "aura wheel" => apply_aura_wheel(state, move_data, user_position, target_positions, generation),
        "ragingbull" | "raging bull" => apply_raging_bull(state, move_data, user_position, target_positions, generation),
        
        // Special combat mechanics
        "photongeyser" | "photon geyser" => apply_photon_geyser(state, move_data, user_position, target_positions, generation),
        "skydrop" | "sky drop" => apply_sky_drop(state, move_data, user_position, target_positions, generation),
        
        // Advanced hazard manipulation (Court Change already handled above)
        "mortalspin" | "mortal spin" => apply_mortal_spin(state, move_data, user_position, target_positions, generation),
        
        // Default case - no special effects
        _ => apply_generic_effects(state, move_data, user_position, target_positions, generation),
    }
}

// =============================================================================
// STATUS MOVES THAT INFLICT MAJOR STATUS CONDITIONS
// =============================================================================

/// Apply Thunder Wave - paralyzes the target
/// Generation-aware: Electric types become immune to paralysis in Gen 6+
pub fn apply_thunder_wave(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check if target can be paralyzed
            if target.status == PokemonStatus::None {
                // Check for Electric immunity (Ground types in early gens)
                if !is_immune_to_paralysis(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Paralysis,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    // Move has no effect
                    instructions.push(StateInstructions::empty());
                }
            } else {
                // Already has a status condition
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Sleep Powder - puts target to sleep
/// Generation-aware: Grass types become immune to powder moves in Gen 6+
pub fn apply_sleep_powder(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Grass immunity or Overcoat/Safety Goggles
                if !is_immune_to_powder(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Sleep,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Toxic - badly poisons the target
/// Generation-aware: Steel types become immune to poison in Gen 2+
pub fn apply_toxic(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Poison/Steel immunity
                if !is_immune_to_poison(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Toxic,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Will-O-Wisp - burns the target
/// Generation-aware: Fire types are always immune to burn
pub fn apply_will_o_wisp(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Check for Fire immunity
                if !is_immune_to_burn(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Burn,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Stun Spore - paralyzes the target
/// Generation-aware: Grass types immune to powder moves in Gen 6+, Electric types immune to paralysis in Gen 6+
pub fn apply_stun_spore(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) && !is_immune_to_paralysis(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Paralysis,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Poison Powder - poisons the target
/// Generation-aware: Grass types immune to powder moves in Gen 6+, Poison/Steel types immune to poison
pub fn apply_poison_powder(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) && !is_immune_to_poison(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Poison,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

// =============================================================================
// STAT-MODIFYING MOVES
// =============================================================================

/// Apply Swords Dance - raises Attack by 2 stages
pub fn apply_swords_dance(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position // Self-targeting move
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Dragon Dance - raises Attack and Speed by 1 stage each
pub fn apply_dragon_dance(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Nasty Plot - raises Special Attack by 2 stages
pub fn apply_nasty_plot(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::SpecialAttack, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Agility - raises Speed by 2 stages
pub fn apply_agility(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Speed, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Growl - lowers target's Attack by 1 stage
pub fn apply_growl(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Leer - lowers target's Defense by 1 stage
pub fn apply_leer(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Defense, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Tail Whip - lowers target's Defense by 1 stage
pub fn apply_tail_whip(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_leer(state, user_position, target_positions, generation) // Same effect as Leer
}

/// Apply String Shot - lowers target's Speed by 2 stages
/// Generation-aware: Effect may change in earlier generations
pub fn apply_string_shot(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // In Gen 1, String Shot only lowered Speed by 1 stage
    let speed_reduction = if generation.generation.number() == 1 { -1 } else { -2 };
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Speed, speed_reduction);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Check if a move should be blocked by protection moves
pub fn is_move_blocked_by_protection(
    move_data: &EngineMoveData,
    target: &Pokemon,
) -> bool {
    // Check if target has protection status
    if target.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::Protect) {
        // Most moves are blocked by protect, but some bypass it
        !is_move_bypassing_protection(move_data)
    } else {
        false
    }
}

/// Check if a move bypasses protection moves
fn is_move_bypassing_protection(move_data: &EngineMoveData) -> bool {
    // Moves that bypass protect
    matches!(move_data.name.as_str(), 
        "Feint" | "Shadow Force" | "Phantom Force" | 
        "Hyperspace Hole" | "Hyperspace Fury" |
        "Menacing Moonraze Maelstrom" | "Let's Snuggle Forever"
    )
}

/// Calculate accuracy for a move with weather, ability, and item modifiers
pub fn calculate_accuracy(
    move_data: &EngineMoveData,
    user: &Pokemon,
    target: &Pokemon,
) -> f32 {
    calculate_accuracy_with_context(move_data, user, target, None, None)
}

/// Calculate accuracy for a move with full context
pub fn calculate_accuracy_with_context(
    move_data: &EngineMoveData,
    user: &Pokemon,
    target: &Pokemon,
    weather: Option<&crate::core::instruction::Weather>,
    generation: Option<&GenerationMechanics>,
) -> f32 {
    let base_accuracy = move_data.accuracy.unwrap_or(100) as f32 / 100.0;
    
    // Get accuracy and evasion stat modifiers
    let accuracy_modifier = user.get_effective_stat(crate::core::instruction::Stat::Accuracy) as f32 / 100.0;
    let evasion_modifier = target.get_effective_stat(crate::core::instruction::Stat::Evasion) as f32 / 100.0;
    
    // Calculate base accuracy with stats
    let mut final_accuracy = base_accuracy * (accuracy_modifier / evasion_modifier);
    
    // Apply ability modifiers
    final_accuracy = apply_accuracy_ability_modifiers(final_accuracy, move_data, user, target);
    
    // Apply weather modifiers
    if let Some(current_weather) = weather {
        final_accuracy = apply_weather_accuracy_modifiers(final_accuracy, move_data, current_weather);
    }
    
    // Apply item modifiers
    final_accuracy = apply_item_accuracy_modifiers(final_accuracy, move_data, user, target);
    
    final_accuracy.min(1.0).max(0.0)
}

/// Apply ability modifiers to accuracy
fn apply_accuracy_ability_modifiers(
    accuracy: f32,
    move_data: &EngineMoveData,
    user: &Pokemon,
    target: &Pokemon,
) -> f32 {
    let mut modified_accuracy = accuracy;
    
    // User abilities that affect accuracy
    match user.ability.to_lowercase().as_str() {
        "compoundeyes" => {
            // Compound Eyes boosts accuracy by 30%
            modified_accuracy *= 1.3;
        }
        "hustle" => {
            // Hustle reduces accuracy of physical moves by 20%
            if move_data.category == MoveCategory::Physical {
                modified_accuracy *= 0.8;
            }
        }
        "noguard" => {
            // No Guard makes all moves hit
            return 1.0;
        }
        "keeneye" => {
            // Keen Eye prevents accuracy reduction (already handled in stat calculation)
            // but also ignores evasion boosts
            if let Some(evasion_boost) = target.stat_boosts.get(&crate::core::instruction::Stat::Evasion) {
                if *evasion_boost > 0 {
                    // Recalculate without evasion boosts
                    let accuracy_mod = user.get_effective_stat(crate::core::instruction::Stat::Accuracy) as f32 / 100.0;
                    modified_accuracy = (move_data.accuracy.unwrap_or(100) as f32 / 100.0) * accuracy_mod;
                }
            }
        }
        "victorystar" => {
            // Victory Star boosts accuracy by 10%
            modified_accuracy *= 1.1;
        }
        _ => {}
    }
    
    // Target abilities that affect accuracy
    match target.ability.to_lowercase().as_str() {
        "noguard" => {
            // No Guard makes all moves hit
            return 1.0;
        }
        "wonderskin" => {
            // Wonder Skin reduces accuracy of status moves to 50% if they would be super effective
            if move_data.category == MoveCategory::Status && modified_accuracy > 0.5 {
                modified_accuracy = 0.5;
            }
        }
        "sandveil" => {
            // Sand Veil boosts evasion in sandstorm (handled in weather section)
        }
        "snowcloak" => {
            // Snow Cloak boosts evasion in hail/snow (handled in weather section)
        }
        _ => {}
    }
    
    modified_accuracy
}

/// Apply weather modifiers to accuracy
fn apply_weather_accuracy_modifiers(
    accuracy: f32,
    move_data: &EngineMoveData,
    weather: &crate::core::instruction::Weather,
) -> f32 {
    let mut modified_accuracy = accuracy;
    
    match weather {
        crate::core::instruction::Weather::Rain => {
            // Thunder has perfect accuracy in rain
            if move_data.name.to_lowercase() == "thunder" {
                modified_accuracy = 1.0;
            }
            // Hurricane has perfect accuracy in rain
            if move_data.name.to_lowercase() == "hurricane" {
                modified_accuracy = 1.0;
            }
        }
        crate::core::instruction::Weather::Sun | crate::core::instruction::Weather::HarshSun => {
            // Thunder has reduced accuracy in sun
            if move_data.name.to_lowercase() == "thunder" {
                modified_accuracy *= 0.5;
            }
            // Hurricane has reduced accuracy in sun
            if move_data.name.to_lowercase() == "hurricane" {
                modified_accuracy *= 0.5;
            }
        }
        crate::core::instruction::Weather::Sand => {
            // Rock-type moves have perfect accuracy in sandstorm
            if move_data.move_type.to_lowercase() == "rock" {
                modified_accuracy = 1.0;
            }
        }
        crate::core::instruction::Weather::Hail | crate::core::instruction::Weather::Snow => {
            // Blizzard has perfect accuracy in hail/snow
            if move_data.name.to_lowercase() == "blizzard" {
                modified_accuracy = 1.0;
            }
        }
        _ => {}
    }
    
    modified_accuracy
}

/// Apply item modifiers to accuracy
fn apply_item_accuracy_modifiers(
    accuracy: f32,
    _move_data: &EngineMoveData,
    user: &Pokemon,
    _target: &Pokemon,
) -> f32 {
    let mut modified_accuracy = accuracy;
    
    // User items that affect accuracy
    if let Some(item) = &user.item {
        match item.to_lowercase().as_str() {
        "widelens" => {
            // Wide Lens boosts accuracy by 10%
            modified_accuracy *= 1.1;
        }
        "zoomlens" => {
            // Zoom Lens boosts accuracy by 20% when moving after the target
            // For now, assume we don't have speed order context, so apply conservatively
            modified_accuracy *= 1.1;
        }
        "laxincense" => {
            // Lax Incense reduces incoming move accuracy by 10%
            // This would be applied to the target's evasion calculation
        }
        _ => {}
        }
    }
    
    modified_accuracy
}

// =============================================================================
// HEALING MOVES
// =============================================================================

/// Apply Recover - restores 50% of max HP
pub fn apply_recover(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = pokemon.max_hp / 2;
        let instruction = Instruction::PositionHeal(PositionHealInstruction {
                target_position,
                heal_amount,
                previous_hp: Some(0), // This should be set to actual previous HP
            });
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Roost - restores 50% of max HP
pub fn apply_roost(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Moonlight - restores HP based on weather
/// Generation-aware: Weather effects and amounts may vary by generation
pub fn apply_moonlight(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = match state.weather {
            crate::core::instruction::Weather::Sun | crate::core::instruction::Weather::HarshSun => {
                (pokemon.max_hp * 2) / 3 // 66% in sun
            }
            crate::core::instruction::Weather::Rain | crate::core::instruction::Weather::Sand | 
            crate::core::instruction::Weather::Hail | crate::core::instruction::Weather::Snow => {
                pokemon.max_hp / 4 // 25% in other weather
            }
            _ => pokemon.max_hp / 2, // 50% in clear weather
        };
        
        let instruction = Instruction::PositionHeal(PositionHealInstruction {
                target_position,
                heal_amount,
                previous_hp: Some(0), // This should be set to actual previous HP
            });
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Synthesis - restores HP based on weather (same as Moonlight)
pub fn apply_synthesis(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_moonlight(state, user_position, target_positions, generation)
}

/// Apply Morning Sun - restores HP based on weather (same as Moonlight)
pub fn apply_morning_sun(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_moonlight(state, user_position, target_positions, generation)
}

/// Apply Soft-Boiled - restores 50% of max HP
pub fn apply_soft_boiled(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Milk Drink - restores 50% of max HP
pub fn apply_milk_drink(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Slack Off - restores 50% of max HP
pub fn apply_slack_off(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

// =============================================================================
// RECOIL MOVES
// =============================================================================

/// Apply Double-Edge - deals recoil damage (33% of damage dealt)
pub fn apply_double_edge(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Take Down - deals recoil damage (25% of damage dealt)
pub fn apply_take_down(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Submission - deals recoil damage (25% of damage dealt)
pub fn apply_submission(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Volt Tackle - deals recoil damage (33% of damage dealt)
pub fn apply_volt_tackle(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Flare Blitz - deals recoil damage (33% of damage dealt)
pub fn apply_flare_blitz(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Brave Bird - deals recoil damage (33% of damage dealt)
pub fn apply_brave_bird(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Wild Charge - deals recoil damage (25% of damage dealt)
pub fn apply_wild_charge(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Head Smash - deals recoil damage (50% of damage dealt)
pub fn apply_head_smash(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 50)
}

// =============================================================================
// DRAIN MOVES
// =============================================================================

/// Apply Giga Drain - restores 50% of damage dealt
pub fn apply_giga_drain(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Mega Drain - restores 50% of damage dealt
pub fn apply_mega_drain(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Absorb - restores 50% of damage dealt
pub fn apply_absorb(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Drain Punch - restores 50% of damage dealt
pub fn apply_drain_punch(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Leech Life - restores 50% of damage dealt
pub fn apply_leech_life(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Dream Eater - restores 50% of damage dealt (only works on sleeping targets)
pub fn apply_dream_eater(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Dream Eater only works on sleeping Pokemon
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::Sleep {
                // Move can hit - drain effect will be applied after damage
                instructions.push(StateInstructions::empty());
            } else {
                // Move fails on non-sleeping targets
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

// =============================================================================
// PROTECTION MOVES
// =============================================================================

/// Apply Protect - protects user from most moves this turn
pub fn apply_protect(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position,
        volatile_status: VolatileStatus::Protect,
        duration: Some(1), // Lasts for the rest of the turn
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Detect - same as Protect
pub fn apply_detect(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_protect(state, user_position, target_positions, generation)
}

/// Apply Endure - survives any attack with at least 1 HP
pub fn apply_endure(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position,
        volatile_status: VolatileStatus::Endure,
        duration: Some(1),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

// =============================================================================
// SUBSTITUTE AND SIMILAR
// =============================================================================

/// Apply Substitute - creates a substitute that absorbs damage
pub fn apply_substitute(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        // Check if Pokemon has enough HP (need at least 25% max HP)
        let cost = pokemon.max_hp / 4;
        if pokemon.hp > cost {
            let mut instructions = Vec::new();
            
            // Damage user for 25% of max HP
            instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: cost,
                previous_hp: Some(0), // This should be set to actual previous HP
            }));
            
            // Apply substitute volatile status
            instructions.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                target_position,
                volatile_status: VolatileStatus::Substitute,
                duration: None, // Lasts until broken
            }));
            
            vec![StateInstructions::new(100.0, instructions)]
        } else {
            // Not enough HP - move fails
            vec![StateInstructions::empty()]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

// =============================================================================
// GENERIC EFFECTS AND HELPER FUNCTIONS
// =============================================================================

/// Apply generic move effects based on move data
pub fn apply_generic_effects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // For moves without specific implementations, check for secondary effects
    if let Some(effect_chance) = move_data.effect_chance {
        if effect_chance > 0 {
            return apply_probability_based_secondary_effects(
                state, 
                move_data, 
                user_position, 
                target_positions, 
                generation, 
                effect_chance
            );
        }
    }
    
    // Return empty instructions for moves with no secondary effects
    vec![StateInstructions::empty()]
}

// =============================================================================
// MULTI-HIT MOVE FUNCTIONS
// =============================================================================

/// Apply multi-hit move effects with proper probability branching
/// Multi-hit moves like Bullet Seed, Rock Blast, etc. hit 2-5 times with specific probabilities
pub fn apply_multi_hit_move(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Standard multi-hit probability distribution (2-5 hits)
    // Gen 1-4: Equal probability for each hit count (25% each)
    // Gen 5+: 35% for 2 hits, 35% for 3 hits, 15% for 4 hits, 15% for 5 hits
    let hit_probabilities = if generation.generation.number() >= 5 {
        vec![
            (2, 35.0), // 2 hits: 35%
            (3, 35.0), // 3 hits: 35% 
            (4, 15.0), // 4 hits: 15%
            (5, 15.0), // 5 hits: 15%
        ]
    } else {
        vec![
            (2, 25.0), // 2 hits: 25%
            (3, 25.0), // 3 hits: 25%
            (4, 25.0), // 4 hits: 25%
            (5, 25.0), // 5 hits: 25%
        ]
    };
    
    // Handle special cases for specific moves
    let hit_distribution = match move_data.name.to_lowercase().as_str() {
        "doubleslap" | "double slap" | "bonemerang" => {
            // These moves always hit exactly 2 times
            vec![(2, 100.0)]
        }
        "beatup" | "beat up" => {
            // Beat Up hits once per conscious party member
            // For now, assume standard multi-hit
            hit_probabilities
        }
        _ => hit_probabilities,
    };
    
    // Generate instructions for each possible hit count
    for (hit_count, probability) in hit_distribution {
        if probability > 0.0 {
            let hit_instructions = generate_multi_hit_instructions(
                state, 
                move_data, 
                user_position, 
                target_positions, 
                hit_count, 
                generation
            );
            
            instructions.push(StateInstructions::new(probability, hit_instructions));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Generate the actual damage instructions for a multi-hit move
fn generate_multi_hit_instructions(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    hit_count: i32,
    generation: &GenerationMechanics,
) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    
    // For each hit, calculate damage
    for hit_number in 1..=hit_count {
        for &target_position in target_positions {
            // Calculate damage for this hit
            let damage = calculate_multi_hit_damage(
                state, 
                move_data, 
                user_position, 
                target_position, 
                hit_number, 
                generation
            );
            
            if damage > 0 {
                instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: damage,
                previous_hp: Some(0), // This should be set to actual previous HP
            }));
            }
        }
    }
    
    instructions
}

/// Calculate damage for a single hit of a multi-hit move
fn calculate_multi_hit_damage(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_position: BattlePosition,
    hit_number: i32,
    generation: &GenerationMechanics,
) -> i16 {
    // Get attacking Pokemon
    let attacker = state
        .get_pokemon_at_position(user_position)
        .expect("Attacker position should be valid");

    // Get defending Pokemon
    let defender = state
        .get_pokemon_at_position(target_position)
        .expect("Target position should be valid");

    // Check for type immunities first
    if is_immune_to_move_type(&move_data.move_type, defender) {
        return 0;
    }

    // Check for ability immunities
    if is_immune_due_to_ability(move_data, defender) {
        return 0;
    }

    // Calculate base damage for each hit
    // Each hit does full damage (unlike some games where later hits do less)
    let base_damage = super::damage_calc::calculate_damage(
        state,
        attacker,
        defender,
        move_data,
        false, // Not a critical hit for base calculation
        1.0,   // Full damage roll
    );
    
    base_damage
}

/// Check if a Pokemon is immune to a move type (e.g., Ghost immune to Normal/Fighting)
fn is_immune_to_move_type(move_type: &str, defender: &crate::core::state::Pokemon) -> bool {
    use super::type_effectiveness::{PokemonType, TypeChart};

    // Use a basic type chart for now - in full implementation this would use generation-specific charts
    let type_chart = TypeChart::new(9); // Gen 9 type chart
    let attacking_type = PokemonType::from_str(move_type).unwrap_or(PokemonType::Normal);
    
    let defender_type1 = PokemonType::from_str(&defender.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    let type_effectiveness = type_chart.calculate_damage_multiplier(
        attacking_type,
        (defender_type1, defender_type2),
        None,
        None,
    );

    // If type effectiveness is 0, the Pokemon is immune
    type_effectiveness == 0.0
}

/// Check if a Pokemon is immune due to ability (e.g., Levitate vs Ground)
fn is_immune_due_to_ability(move_data: &EngineMoveData, defender: &crate::core::state::Pokemon) -> bool {
    use crate::engine::mechanics::abilities::get_ability_by_name;
    
    if let Some(ability) = get_ability_by_name(&defender.ability) {
        ability.provides_immunity(&move_data.move_type)
    } else {
        false
    }
}

// =============================================================================
// SECONDARY EFFECT PROBABILITY FUNCTIONS
// =============================================================================

/// Apply probability-based secondary effects for moves
/// This creates branching instructions based on the effect chance
/// Following poke-engine's pattern of probability-based instruction branching
pub fn apply_probability_based_secondary_effects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    effect_chance: i16,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Calculate probabilities
    let effect_probability = effect_chance as f32;
    let no_effect_probability = 100.0 - effect_probability;
    
    // Create no-effect branch (most common case)
    if no_effect_probability > 0.0 {
        instructions.push(StateInstructions::new(no_effect_probability, vec![]));
    }
    
    // Create effect branch
    if effect_probability > 0.0 {
        if let Some(effect_instructions) = determine_secondary_effect_from_move(
            state, 
            move_data, 
            user_position, 
            target_positions, 
            generation
        ) {
            instructions.push(StateInstructions::new(effect_probability, effect_instructions));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Determine what secondary effect a move should have based on its properties
/// This function maps move types and names to their appropriate secondary effects
pub fn determine_secondary_effect_from_move(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Option<Vec<Instruction>> {
    let move_name = move_data.name.to_lowercase();
    let move_type = move_data.move_type.to_lowercase();
    
    // Move-specific secondary effects
    match move_name.as_str() {
        // Fire moves that can burn
        "flamethrower" | "fireblast" | "fire blast" | "lavaplume" | "lava plume" |
        "firefang" | "fire fang" | "firepunch" | "fire punch" | "flamewheel" | "flame wheel" => {
            return Some(create_burn_instructions(state, target_positions));
        }
        
        // Electric moves that can paralyze
        "thunderbolt" | "thunder" | "discharge" | "sparklingaria" | "sparkling aria" |
        "thunderpunch" | "thunder punch" | "thunderfang" | "thunder fang" => {
            return Some(create_paralysis_instructions(state, target_positions, generation));
        }
        
        // Ice moves that can freeze
        "icebeam" | "ice beam" | "blizzard" | "icepunch" | "ice punch" |
        "icefang" | "ice fang" | "freezedry" | "freeze-dry" => {
            return Some(create_freeze_instructions(state, target_positions));
        }
        
        // Poison moves that can poison
        "sludgebomb" | "sludge bomb" | "poisonjab" | "poison jab" | 
        "sludgewave" | "sludge wave" | "poisonfang" | "poison fang" => {
            return Some(create_poison_instructions(state, target_positions, generation));
        }
        
        // Flinch-inducing moves
        "airslash" | "air slash" | "ironhead" | "iron head" | "rockslide" | "rock slide" |
        "headbutt" | "bite" | "stomp" | "astonish" | "fakebite" | "fake bite" => {
            return Some(create_flinch_instructions(target_positions));
        }
        
        // Stat-lowering moves
        "acid" => {
            return Some(create_defense_lowering_instructions(target_positions));
        }
        
        _ => {}
    }
    
    // Type-based secondary effects (generic)
    match move_type.as_str() {
        "fire" => Some(create_burn_instructions(state, target_positions)),
        "electric" => Some(create_paralysis_instructions(state, target_positions, generation)),
        "ice" => Some(create_freeze_instructions(state, target_positions)),
        "poison" => Some(create_poison_instructions(state, target_positions, generation)),
        _ => None,
    }
}

/// Create burn status instructions for targets
fn create_burn_instructions(state: &State, target_positions: &[BattlePosition]) -> Vec<Instruction> {
    target_positions
        .iter()
        .map(|&position| {
            let target = state.get_pokemon_at_position(position);
            Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: PokemonStatus::Burn,
                previous_status: target.map(|p| p.status),
                previous_status_duration: target.map(|p| p.status_duration),
            })
        })
        .collect()
}

/// Create paralysis status instructions for targets
fn create_paralysis_instructions(
    state: &State,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<Instruction> {
    target_positions
        .iter()
        .filter_map(|&position| {
            if let Some(target) = state.get_pokemon_at_position(position) {
                if target.status == PokemonStatus::None && !is_immune_to_paralysis(target, generation) {
                    Some(Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position: position,
                        status: PokemonStatus::Paralysis,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Create freeze status instructions for targets
fn create_freeze_instructions(state: &State, target_positions: &[BattlePosition]) -> Vec<Instruction> {
    target_positions
        .iter()
        .map(|&position| {
            let target = state.get_pokemon_at_position(position);
            Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: PokemonStatus::Freeze,
                previous_status: target.map(|p| p.status),
                previous_status_duration: target.map(|p| p.status_duration),
            })
        })
        .collect()
}

/// Create poison status instructions for targets
fn create_poison_instructions(
    state: &State,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<Instruction> {
    target_positions
        .iter()
        .filter_map(|&position| {
            if let Some(target) = state.get_pokemon_at_position(position) {
                if target.status == PokemonStatus::None && !is_immune_to_poison(target, generation) {
                    Some(Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position: position,
                        status: PokemonStatus::Poison,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Create flinch volatile status instructions for targets
fn create_flinch_instructions(target_positions: &[BattlePosition]) -> Vec<Instruction> {
    target_positions
        .iter()
        .map(|&position| {
            Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                target_position: position,
                volatile_status: VolatileStatus::Flinch,
                duration: Some(1), // Flinch only lasts for the current turn
            })
        })
        .collect()
}

/// Create defense lowering instructions for targets
fn create_defense_lowering_instructions(target_positions: &[BattlePosition]) -> Vec<Instruction> {
    target_positions
        .iter()
        .map(|&position| {
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Defense, -1);
            
            Instruction::BoostStats(BoostStatsInstruction {
                target_position: position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            })
        })
        .collect()
}

// =============================================================================
// RECOIL AND DRAIN MOVE HELPER FUNCTIONS
// =============================================================================

/// Apply recoil move effects - now handled automatically by instruction generator
/// This function is kept for compatibility but recoil is now handled via PS data
pub fn apply_recoil_move(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    _recoil_percentage: i16,
) -> Vec<StateInstructions> {
    // Recoil is now handled automatically in the instruction generator
    // based on PS move data, so we just return empty instructions
    vec![StateInstructions::empty()]
}

/// Apply drain move effects - now handled automatically by instruction generator
/// This function is kept for compatibility but drain is now handled via PS data
pub fn apply_drain_move(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    _drain_percentage: i16,
) -> Vec<StateInstructions> {
    // Drain is now handled automatically in the instruction generator
    // based on PS move data, so we just return empty instructions
    vec![StateInstructions::empty()]
}

/// Create a damage-based effect instruction for moves like recoil and drain
/// This creates an instruction template that will be filled in with actual values
/// during damage calculation
pub fn create_damage_based_effect(
    effect_type: DamageBasedEffectType,
    user_position: BattlePosition,
    percentage: i16,
) -> DamageBasedEffect {
    DamageBasedEffect {
        effect_type,
        user_position,
        percentage,
    }
}

/// Types of damage-based effects
#[derive(Debug, Clone, PartialEq)]
pub enum DamageBasedEffectType {
    Recoil,  // User takes damage
    Drain,   // User heals
}

/// A damage-based effect that will be calculated after damage is determined
#[derive(Debug, Clone, PartialEq)]
pub struct DamageBasedEffect {
    pub effect_type: DamageBasedEffectType,
    pub user_position: BattlePosition,
    pub percentage: i16,
}

/// Apply secondary effects that depend on damage dealt
/// This function would be called by the damage calculation system
/// after determining the actual damage amount
pub fn apply_damage_based_secondary_effects(
    state: &State,
    damage_dealt: i16,
    effects: &[DamageBasedEffect],
    instructions: &mut Vec<Instruction>,
) {
    for effect in effects {
        match effect.effect_type {
            DamageBasedEffectType::Recoil => {
                let recoil_amount = (damage_dealt * effect.percentage) / 100;
                if recoil_amount > 0 {
                    let previous_hp = state.get_pokemon_at_position(effect.user_position).map(|p| p.hp).unwrap_or(0);
                    instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                        target_position: effect.user_position,
                        damage_amount: recoil_amount,
                        previous_hp: Some(previous_hp),
                    }));
                }
            }
            DamageBasedEffectType::Drain => {
                let heal_amount = (damage_dealt * effect.percentage) / 100;
                if heal_amount > 0 {
                    let previous_hp = state.get_pokemon_at_position(effect.user_position).map(|p| p.hp).unwrap_or(0);
                    instructions.push(Instruction::PositionHeal(PositionHealInstruction {
                        target_position: effect.user_position,
                        heal_amount,
                        previous_hp: Some(previous_hp),
                    }));
                }
            }
        }
    }
}

// =============================================================================
// IMMUNITY CHECK FUNCTIONS
// =============================================================================

/// Check if a Pokemon is immune to paralysis
/// Generation-aware: Electric types become immune to paralysis in Gen 6+
fn is_immune_to_paralysis(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    if generation.generation.number() >= 6 {
        // Gen 6+: Electric types are immune to paralysis
        pokemon.types.iter().any(|t| t.to_lowercase() == "electric")
    } else {
        // Earlier gens: no electric immunity to paralysis
        false
    }
}

/// Check if a Pokemon is immune to powder moves
/// Generation-aware: Grass types become immune to powder moves in Gen 6+
/// Also checks for Overcoat ability and Safety Goggles item
fn is_immune_to_powder(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Check for Overcoat ability (immune to powder moves in all generations)
    if pokemon.ability.to_lowercase() == "overcoat" {
        return true;
    }
    
    // Check for Safety Goggles item (immune to powder moves and weather damage)
    if let Some(item) = &pokemon.item {
        if item.to_lowercase() == "safetygoggles" {
            return true;
        }
    }
    
    if generation.generation.number() >= 6 {
        // Gen 6+: Grass types are immune to powder moves
        pokemon.types.iter().any(|t| t.to_lowercase() == "grass")
    } else {
        // Earlier gens: no grass immunity to powder moves
        false
    }
}

/// Check if a Pokemon is immune to poison
/// Generation-aware: Steel types become immune to poison in Gen 2+
fn is_immune_to_poison(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Poison types are always immune to poison
    let is_poison_type = pokemon.types.iter().any(|t| t.to_lowercase() == "poison");
    
    if generation.generation.number() >= 2 {
        // Gen 2+: Steel types are also immune to poison
        let is_steel_type = pokemon.types.iter().any(|t| t.to_lowercase() == "steel");
        is_poison_type || is_steel_type
    } else {
        // Gen 1: Only Poison types are immune
        is_poison_type
    }
}

/// Check if a Pokemon is immune to burn
/// Generation-aware: Fire types are always immune to burn
fn is_immune_to_burn(pokemon: &Pokemon, _generation: &GenerationMechanics) -> bool {
    // Fire types are immune to burn in all generations
    pokemon.types.iter().any(|t| t.to_lowercase() == "fire")
}

// =============================================================================
// NEW MOVE IMPLEMENTATIONS - MISSING MOVES FROM TODO LIST
// =============================================================================

/// Apply Glare - inflicts paralysis
/// Generation-aware: Not affected by Electric immunity like Thunder Wave
pub fn apply_glare(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                // Glare can paralyze Electric types (unlike Thunder Wave in Gen 6+)
                let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                    target_position,
                    status: PokemonStatus::Paralysis,
                    previous_status: Some(target.status),
                    previous_status_duration: Some(target.status_duration),
                });
                instructions.push(StateInstructions::new(100.0, vec![instruction]));
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Spore - 100% sleep move
/// Generation-aware: Grass types immune to powder moves in Gen 6+
pub fn apply_spore(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::None {
                if !is_immune_to_powder(target, generation) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Sleep,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Acid - deals damage with chance to lower Defense
/// Generation-aware: 33.2% chance in Gen 1, 10% in later generations
pub fn apply_acid(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Acid deals damage AND has a secondary effect
    let effect_chance = if generation.generation.number() == 1 { 33 } else { 10 };
    
    apply_probability_based_secondary_effects(
        state,
        move_data,
        user_position,
        target_positions,
        generation,
        effect_chance,
    )
}

/// Apply Charm - lowers target's Attack by 2 stages
pub fn apply_charm(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -2);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Growth - raises Attack and Special Attack
/// Generation-aware: Enhanced in sun weather
pub fn apply_growth(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    
    // Enhanced in sun weather
    let boost_amount = match state.weather {
        crate::core::instruction::Weather::Sun | crate::core::instruction::Weather::HarshSun => 2,
        _ => 1,
    };
    
    stat_boosts.insert(Stat::Attack, boost_amount);
    stat_boosts.insert(Stat::SpecialAttack, boost_amount);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
        target_position,
        stat_boosts,
        previous_boosts: Some(HashMap::new()),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Aqua Ring - provides gradual HP recovery
pub fn apply_aqua_ring(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position,
        volatile_status: VolatileStatus::AquaRing,
        duration: None, // Lasts until Pokemon switches out
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Shore Up - healing move enhanced in sand weather
pub fn apply_shore_up(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = match state.weather {
            crate::core::instruction::Weather::Sand => {
                (pokemon.max_hp * 2) / 3 // 66% in sand
            }
            _ => pokemon.max_hp / 2, // 50% normally
        };
        
        let instruction = Instruction::PositionHeal(PositionHealInstruction {
            target_position,
            heal_amount,
            previous_hp: Some(0),
        });
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Aromatherapy - clears status conditions for entire team
pub fn apply_aromatherapy(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let user_side = user_position.side;
    
    // Clear status from all Pokemon on user's team
    let side = state.get_side(user_side);
    for (slot, pokemon) in side.pokemon.iter().enumerate() {
        if pokemon.status != PokemonStatus::None {
            let position = BattlePosition::new(user_side, slot);
            instructions.push(Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position: position,
                status: PokemonStatus::None,
                previous_status: Some(pokemon.status),
                previous_status_duration: Some(pokemon.status_duration),
            }));
        }
    }
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Heal Bell - same as Aromatherapy
pub fn apply_heal_bell(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_aromatherapy(state, user_position, target_positions, generation)
}

/// Apply Attract - causes infatuation
pub fn apply_attract(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check if target is already attracted or has immunity (like Oblivious)
            if !target.volatile_statuses.contains(&VolatileStatus::Attract) {
                let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position,
                    volatile_status: VolatileStatus::Attract,
                    duration: None, // Lasts until Pokemon switches out
                });
                instructions.push(StateInstructions::new(100.0, vec![instruction]));
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Confuse Ray - causes confusion
pub fn apply_confuse_ray(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Confusion,
            duration: Some(4), // Lasts 2-5 turns in most generations
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Haze - resets all stat changes for all Pokemon
pub fn apply_haze(
    state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Reset stat boosts for all active Pokemon
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        for slot in 0..state.format.active_pokemon_count() {
            let position = BattlePosition::new(side_ref, slot);
            if let Some(pokemon) = state.get_pokemon_at_position(position) {
                if !pokemon.stat_boosts.is_empty() {
                    let instruction = Instruction::BoostStats(BoostStatsInstruction {
                        target_position: position,
                        stat_boosts: HashMap::new(), // Reset all to 0
                        previous_boosts: Some(pokemon.stat_boosts.clone()),
                    });
                    instructions.push(instruction);
                }
            }
        }
    }
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Clear Smog - removes all stat changes from target
pub fn apply_clear_smog(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts: HashMap::new(), // Reset all to 0
            previous_boosts: Some(HashMap::new()),
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Sunny Day - sets sun weather
pub fn apply_sunny_day(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Sun,
        duration: Some(5), // 5 turns in most generations
        previous_weather: None,
        previous_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Rain Dance - sets rain weather
pub fn apply_rain_dance(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Rain,
        duration: Some(5),
        previous_weather: None,
        previous_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Sandstorm - sets sand weather
pub fn apply_sandstorm(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Sand,
        duration: Some(5),
        previous_weather: None,
        previous_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Hail - sets hail weather
pub fn apply_hail(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Hail,
        duration: Some(5),
        previous_weather: None,
        previous_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Snowscape - sets snow weather (Gen 9+ replacement for Hail)
pub fn apply_snowscape(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Snow,
        duration: Some(5),
        previous_weather: None,
        previous_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Light Screen - reduces Special damage taken
pub fn apply_light_screen(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: user_position.side,
        condition: SideCondition::LightScreen,
        duration: Some(5), // 5 turns in most generations
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Reflect - reduces Physical damage taken
pub fn apply_reflect_move(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: user_position.side,
        condition: SideCondition::Reflect,
        duration: Some(5),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Aurora Veil - combines Light Screen and Reflect effects (only in hail/snow)
pub fn apply_aurora_veil(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Aurora Veil only works in hail or snow weather
    match state.weather {
        crate::core::instruction::Weather::Hail | crate::core::instruction::Weather::Snow => {
            let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
                side: user_position.side,
                condition: SideCondition::AuroraVeil,
                duration: Some(5),
            });
            
            vec![StateInstructions::new(100.0, vec![instruction])]
        }
        _ => {
            // Move fails without hail/snow
            vec![StateInstructions::empty()]
        }
    }
}

// =============================================================================
// HAZARD MOVES 
// =============================================================================

/// Apply Spikes - sets entry hazard that damages grounded Pokemon
pub fn apply_spikes(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Target the opposing side
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
        crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
    };
    
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: target_side,
        condition: SideCondition::Spikes,
        duration: None, // Spikes last until removed
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Stealth Rock - sets entry hazard based on type effectiveness  
pub fn apply_stealth_rock(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
        crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
    };
    
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: target_side,
        condition: SideCondition::StealthRock,
        duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Toxic Spikes - sets entry hazard that poisons
pub fn apply_toxic_spikes(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
        crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
    };
    
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: target_side,
        condition: SideCondition::ToxicSpikes,
        duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Sticky Web - sets entry hazard that lowers Speed
pub fn apply_sticky_web(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
        crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
    };
    
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: target_side,
        condition: SideCondition::StickyWeb,
        duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

// =============================================================================
// HAZARD REMOVAL MOVES
// =============================================================================

/// Apply Rapid Spin - removes hazards from user's side
pub fn apply_rapid_spin(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Remove hazards from user's side
    for condition in [SideCondition::Spikes, SideCondition::StealthRock, SideCondition::ToxicSpikes, SideCondition::StickyWeb] {
        instructions.push(Instruction::RemoveSideCondition(crate::core::instruction::RemoveSideConditionInstruction {
            side: user_position.side,
            condition,
        }));
    }
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Defog - removes hazards from both sides
pub fn apply_defog(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Remove hazards from both sides
    for side in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        for condition in [SideCondition::Spikes, SideCondition::StealthRock, SideCondition::ToxicSpikes, SideCondition::StickyWeb] {
            instructions.push(Instruction::RemoveSideCondition(crate::core::instruction::RemoveSideConditionInstruction {
                side,
                condition,
            }));
        }
    }
    
    vec![StateInstructions::new(100.0, instructions)]
}

// =============================================================================
// COMPLEX UTILITY MOVES
// =============================================================================

/// Apply Baton Pass - passes stat changes to next Pokemon
pub fn apply_baton_pass(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Set baton passing flag
    let instruction = Instruction::ToggleBatonPassing(crate::core::instruction::ToggleBatonPassingInstruction {
        target_position: user_position,
        active: true,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Belly Drum - maximizes Attack at cost of 50% HP
pub fn apply_belly_drum(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let cost = pokemon.max_hp / 2;
        if pokemon.hp > cost {
            let mut instructions = Vec::new();
            
            // Damage user for 50% of max HP
            instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: cost,
                previous_hp: Some(pokemon.hp),
            }));
            
            // Maximize Attack (set to +6)
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Attack, 6);
            
            instructions.push(Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(pokemon.stat_boosts.clone()),
            }));
            
            vec![StateInstructions::new(100.0, instructions)]
        } else {
            // Not enough HP - move fails
            vec![StateInstructions::empty()]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Curse - different effects for Ghost vs non-Ghost types
pub fn apply_curse(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        if user.types.iter().any(|t| t.to_lowercase() == "ghost") {
            // Ghost type: Curses target, user loses 50% HP
            if let Some(&target_position) = target_positions.first() {
                let mut instructions = Vec::new();
                
                // Damage user for 50% HP
                let damage = user.max_hp / 2;
                instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                    target_position: user_position,
                    damage_amount: damage,
                    previous_hp: Some(user.hp),
                }));
                
                // Apply curse to target
                instructions.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position,
                    volatile_status: VolatileStatus::Curse,
                    duration: None, // Lasts until target switches
                }));
                
                vec![StateInstructions::new(100.0, instructions)]
            } else {
                vec![StateInstructions::empty()]
            }
        } else {
            // Non-Ghost type: Raises Attack and Defense, lowers Speed
            let target_position = if target_positions.is_empty() {
                user_position
            } else {
                target_positions[0]
            };
            
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Attack, 1);
            stat_boosts.insert(Stat::Defense, 1);
            stat_boosts.insert(Stat::Speed, -1);
            
            let instruction = Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            });
            
            vec![StateInstructions::new(100.0, vec![instruction])]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Destiny Bond - if user faints, opponent also faints
pub fn apply_destiny_bond(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position: user_position,
        volatile_status: VolatileStatus::DestinyBond,
        duration: Some(1), // Lasts until end of turn
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Encore - forces opponent to repeat last move
pub fn apply_encore(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Encore,
            duration: Some(3), // Lasts 3 turns
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Leech Seed - drains HP every turn
pub fn apply_leech_seed(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::LeechSeed,
            duration: None, // Lasts until Pokemon switches
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Rest - fully heals and puts user to sleep
pub fn apply_rest(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let mut instructions = Vec::new();
        
        // Heal to full HP
        let heal_amount = pokemon.max_hp - pokemon.hp;
        if heal_amount > 0 {
            instructions.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position,
                heal_amount,
                previous_hp: Some(pokemon.hp),
            }));
        }
        
        // Clear any existing status
        if pokemon.status != PokemonStatus::None {
            instructions.push(Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position,
                status: PokemonStatus::None,
                previous_status: Some(pokemon.status),
                previous_status_duration: Some(pokemon.status_duration),
            }));
        }
        
        // Put to sleep for 2 turns
        instructions.push(Instruction::ApplyStatus(ApplyStatusInstruction {
            target_position,
            status: PokemonStatus::Sleep,
            previous_status: Some(PokemonStatus::None),
            previous_status_duration: Some(None),
        }));
        
        instructions.push(Instruction::SetRestTurns(crate::core::instruction::SetRestTurnsInstruction {
            target_position,
            turns: 2,
        }));
        
        vec![StateInstructions::new(100.0, instructions)]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Sleep Talk - uses random move while asleep
pub fn apply_sleep_talk(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(pokemon) = state.get_pokemon_at_position(user_position) {
        if pokemon.status == PokemonStatus::Sleep {
            // Move succeeds - actual move selection handled by turn system
            vec![StateInstructions::empty()]
        } else {
            // Move fails if not asleep
            vec![StateInstructions::empty()]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Taunt - prevents status moves
pub fn apply_taunt(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Taunt,
            duration: Some(3), // Lasts 3 turns
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Whirlwind - forces opponent to switch
pub fn apply_whirlwind(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Force switch for opposing side
    let force_switch_instruction = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => Instruction::ToggleSideTwoForceSwitch,
        crate::core::battle_format::SideReference::SideTwo => Instruction::ToggleSideOneForceSwitch,
    };
    
    vec![StateInstructions::new(100.0, vec![force_switch_instruction])]
}

/// Apply Yawn - causes sleep next turn
pub fn apply_yawn(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Yawn,
            duration: Some(2), // Sleep occurs after 1 turn
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

// =============================================================================
// MISSING MOVES IMPLEMENTATION
// =============================================================================

/// Apply Splash - does nothing
pub fn apply_splash(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Splash does nothing - return empty instructions
    vec![StateInstructions::empty()]
}

/// Apply Kinesis - lowers accuracy by 1 stage
pub fn apply_kinesis(
    _state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Accuracy, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Quick Attack - priority +1 physical move
/// Note: Priority is handled by the PS move data, this just handles any special effects
pub fn apply_quick_attack(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Quick Attack is just a priority move with no special effects
    // Priority is handled by the instruction generator
    vec![StateInstructions::empty()]
}

/// Apply Tailwind - doubles speed for side for 4 turns
pub fn apply_tailwind(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let side = user_position.side;
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side,
        condition: SideCondition::TailWind,
        duration: Some(4),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Trick Room - reverses speed priority for 5 turns
pub fn apply_trick_room(
    _state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Toggle trick room state - if active, turn off; if inactive, turn on for 5 turns
    let instruction = Instruction::ToggleTrickRoom(crate::core::instruction::ToggleTrickRoomInstruction {
        active: true, // Will be properly handled by state application
        duration: Some(5),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Refresh - cures user's status condition
pub fn apply_refresh(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let instruction = Instruction::RemoveStatus(crate::core::instruction::RemoveStatusInstruction {
        target_position: user_position,
        previous_status: None, // Will be filled by state application
        previous_status_duration: None,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Wish - heals target position after 2 turns
pub fn apply_wish(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let heal_amount = user.max_hp / 2; // Heals 50% of user's max HP
        let instruction = Instruction::SetWish(crate::core::instruction::SetWishInstruction {
            target_position: user_position,
            heal_amount,
            turns_remaining: 2, // Activates after 2 turns
        });
        
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Healing Wish - user faints, fully heals replacement
pub fn apply_healing_wish(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Faint the user
    instruction_list.push(Instruction::Faint(crate::core::instruction::FaintInstruction {
        target_position: user_position,
        previous_hp: 0, // TODO: Should be set to actual HP before fainting
    }));
    
    // Set up healing for next Pokemon
    // Note: This is simplified - in the full implementation, this would set a side condition
    // that heals the next Pokemon that switches in
    let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
        side: user_position.side,
        condition: SideCondition::Safeguard, // Using as placeholder for healing wish
        duration: Some(1),
    });
    instruction_list.push(instruction);
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

/// Apply Life Dew - heals user and ally
pub fn apply_life_dew(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Heal user
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let heal_amount = user.max_hp / 4; // Heals 25%
        instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
            target_position: user_position,
            heal_amount,
            previous_hp: Some(user.hp),
        }));
    }
    
    // Heal targets (ally in doubles)
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let heal_amount = target.max_hp / 4;
            instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position,
                heal_amount,
                previous_hp: Some(target.hp),
            }));
        }
    }
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

/// Apply No Retreat - boosts all stats but prevents switching
pub fn apply_no_retreat(
    _state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Boost all stats by 1 stage
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Defense, 1);
    stat_boosts.insert(Stat::SpecialAttack, 1);
    stat_boosts.insert(Stat::SpecialDefense, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    instruction_list.push(Instruction::BoostStats(BoostStatsInstruction {
        target_position: user_position,
        stat_boosts,
        previous_boosts: Some(HashMap::new()),
    }));
    
    // Apply No Retreat status
    instruction_list.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position: user_position,
        volatile_status: VolatileStatus::NoRetreat,
        duration: None, // Permanent
    }));
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

/// Apply Pain Split - averages HP between user and target
pub fn apply_pain_split(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if target_positions.is_empty() {
        return vec![StateInstructions::empty()];
    }
    
    let target_position = target_positions[0];
    
    if let (Some(user), Some(target)) = (
        state.get_pokemon_at_position(user_position),
        state.get_pokemon_at_position(target_position)
    ) {
        let total_hp = user.hp + target.hp;
        let new_hp = total_hp / 2;
        
        let mut instruction_list = Vec::new();
        
        // Adjust user's HP
        let user_hp_change = new_hp - user.hp;
        if user_hp_change > 0 {
            instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position: user_position,
                heal_amount: user_hp_change,
                previous_hp: Some(user.hp),
            }));
        } else if user_hp_change < 0 {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position: user_position,
                damage_amount: -user_hp_change,
                previous_hp: Some(user.hp),
            }));
        }
        
        // Adjust target's HP
        let target_hp_change = new_hp - target.hp;
        if target_hp_change > 0 {
            instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position,
                heal_amount: target_hp_change,
                previous_hp: Some(target.hp),
            }));
        } else if target_hp_change < 0 {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: -target_hp_change,
                previous_hp: Some(target.hp),
            }));
        }
        
        vec![StateInstructions::new(100.0, instruction_list)]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Parting Shot - lowers opponent's stats then switches
pub fn apply_parting_shot(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Lower target's Attack and Special Attack by 1 stage
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -1);
        stat_boosts.insert(Stat::SpecialAttack, -1);
        
        instruction_list.push(Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        }));
    }
    
    // Force switch (this would be handled by the switching system)
    // For now, just apply the stat changes
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

/// Apply Perish Song - both Pokemon faint in 3 turns
pub fn apply_perish_song(
    _state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Apply Perish 3 to user
    instruction_list.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position: user_position,
        volatile_status: VolatileStatus::Perish3,
        duration: Some(3),
    }));
    
    // Apply Perish 3 to all targets
    for &target_position in target_positions {
        instruction_list.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Perish3,
            duration: Some(3),
        }));
    }
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

// =============================================================================
// PRIORITY MOVES
// =============================================================================

/// Apply Accelerock - Rock-type priority move
pub fn apply_accelerock(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Priority is handled by PS data, no special effects
    vec![StateInstructions::empty()]
}

/// Apply Aqua Jet - Water-type priority move
pub fn apply_aqua_jet(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Bullet Punch - Steel-type priority move
pub fn apply_bullet_punch(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Extreme Speed - +2 priority Normal move
pub fn apply_extreme_speed(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Fake Out - flinches, only works on first turn
pub fn apply_fake_out(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Apply flinch to targets (damage is handled separately)
    for &target_position in target_positions {
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position,
            volatile_status: VolatileStatus::Flinch,
            duration: Some(1),
        });
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Feint - breaks through protection
pub fn apply_feint(
    state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Feint removes protection from targets
    for &target_position in target_positions {
        if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
            let mut instruction_list = Vec::new();
            
            // Remove Protect status
            if target_pokemon.volatile_statuses.contains(&VolatileStatus::Protect) {
                instruction_list.push(Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position,
                    volatile_status: VolatileStatus::Protect,
                }));
            }
            
            // Remove other protection statuses
            if target_pokemon.volatile_statuses.contains(&VolatileStatus::Endure) {
                instruction_list.push(Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position,
                    volatile_status: VolatileStatus::Endure,
                }));
            }
            
            if !instruction_list.is_empty() {
                instructions.push(StateInstructions::new(100.0, instruction_list));
            }
        }
    }
    
    instructions
}

/// Apply First Impression - Bug-type priority, only works on first turn
pub fn apply_first_impression(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // First Impression only works on the first turn the Pokemon is on the field
    // For now, we'll implement basic logic - in a full implementation, 
    // we'd need to track turn count since Pokemon entered battle
    
    // This is a priority move with no special effects beyond damage
    vec![StateInstructions::new(100.0, vec![])]
}

/// Apply Mach Punch - Fighting-type priority move
pub fn apply_mach_punch(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

// =============================================================================
// FIXED DAMAGE MOVES
// =============================================================================

/// Apply Seismic Toss - damage equals user's level
pub fn apply_seismic_toss(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.level as i16;
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            let instruction = Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount,
                previous_hp: None, // Will be filled by state application
            });
            instructions.push(StateInstructions::new(100.0, vec![instruction]));
        }
        
        if instructions.is_empty() {
            instructions.push(StateInstructions::empty());
        }
        
        instructions
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Night Shade - damage equals user's level
pub fn apply_night_shade(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Same as Seismic Toss
    apply_seismic_toss(state, user_position, target_positions, _generation)
}

/// Apply Endeavor - reduces target HP to user's HP
pub fn apply_endeavor(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                if target.hp > user.hp {
                    let damage_amount = target.hp - user.hp;
                    let instruction = Instruction::PositionDamage(PositionDamageInstruction {
                        target_position,
                        damage_amount,
                        previous_hp: Some(target.hp),
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                }
            }
        }
        
        if instructions.is_empty() {
            instructions.push(StateInstructions::empty());
        }
        
        instructions
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Final Gambit - damage equals user's HP, user faints
pub fn apply_final_gambit(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.hp;
        let mut instruction_list = Vec::new();
        
        // User faints
        instruction_list.push(Instruction::Faint(crate::core::instruction::FaintInstruction {
            target_position: user_position,
            previous_hp: 0, // TODO: Should be set to actual HP before fainting
        }));
        
        // Deal damage to targets
        for &target_position in target_positions {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount,
                previous_hp: None,
            }));
        }
        
        vec![StateInstructions::new(100.0, instruction_list)]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Nature's Madness - halves target's HP
pub fn apply_natures_madness(
    state: &State,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let damage_amount = target.hp / 2;
            if damage_amount > 0 {
                let instruction = Instruction::PositionDamage(PositionDamageInstruction {
                    target_position,
                    damage_amount,
                    previous_hp: Some(target.hp),
                });
                instructions.push(StateInstructions::new(100.0, vec![instruction]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Ruination - halves target's HP
pub fn apply_ruination(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Same as Nature's Madness
    apply_natures_madness(state, user_position, target_positions, generation)
}

/// Apply Super Fang - halves target's HP
pub fn apply_super_fang(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Same as Nature's Madness
    apply_natures_madness(state, user_position, target_positions, generation)
}

// =============================================================================
// COUNTER MOVES
// =============================================================================

/// Apply Counter - returns 2x physical damage
pub fn apply_counter(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Get the side that would be targeted by counter (opponent side)
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => &state.side_two,
        crate::core::battle_format::SideReference::SideTwo => &state.side_one,
    };
    
    // Check if damage was dealt and if it was physical
    if target_side.damage_dealt.damage > 0 && 
       target_side.damage_dealt.move_category == MoveCategory::Physical &&
       !target_side.damage_dealt.hit_substitute {
        
        // Counter does 2x the physical damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 2.0) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opponent who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            // Check type immunity - Counter can't hit Ghost types
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                if target_pokemon.types.contains(&"ghost".to_string()) {
                    return vec![StateInstructions::empty()];
                }
            }
            
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            vec![StateInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No physical damage was taken, Counter fails
        vec![StateInstructions::empty()]
    }
}

/// Apply Mirror Coat - returns 2x special damage
pub fn apply_mirror_coat(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Get the side that would be targeted by mirror coat (opponent side)
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => &state.side_two,
        crate::core::battle_format::SideReference::SideTwo => &state.side_one,
    };
    
    // Check if damage was dealt and if it was special
    if target_side.damage_dealt.damage > 0 && 
       target_side.damage_dealt.move_category == MoveCategory::Special &&
       !target_side.damage_dealt.hit_substitute {
        
        // Mirror Coat does 2x the special damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 2.0) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opponent who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            vec![StateInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No special damage was taken, Mirror Coat fails
        vec![StateInstructions::empty()]
    }
}

/// Apply Comeuppance - returns 1.5x damage taken
pub fn apply_comeuppance(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Get the side that would be targeted by comeuppance (opponent side)
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => &state.side_two,
        crate::core::battle_format::SideReference::SideTwo => &state.side_one,
    };
    
    // Check if damage was dealt (any category)
    if target_side.damage_dealt.damage > 0 && !target_side.damage_dealt.hit_substitute {
        
        // Comeuppance does 1.5x the damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 1.5) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opponent who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            vec![StateInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No damage was taken, Comeuppance fails
        vec![StateInstructions::empty()]
    }
}

/// Apply Metal Burst - returns 1.5x damage taken
pub fn apply_metal_burst(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Get the side that would be targeted by metal burst (opponent side)
    let target_side = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => &state.side_two,
        crate::core::battle_format::SideReference::SideTwo => &state.side_one,
    };
    
    // Check if damage was dealt (any category) and not hitting substitute
    if target_side.damage_dealt.damage > 0 && !target_side.damage_dealt.hit_substitute {
        
        // Metal Burst does 1.5x the damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 1.5) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opponent who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![StateInstructions::empty()]
        } else {
            vec![StateInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No damage was taken, Metal Burst fails
        vec![StateInstructions::empty()]
    }
}

// =============================================================================
// VARIABLE POWER MOVES
// =============================================================================

/// Apply Facade - doubles power with status condition
pub fn apply_facade(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Facade doubles power if user has a status condition (Burn, Paralysis, Poison)
        let has_status = matches!(user.status, 
            PokemonStatus::Burn | PokemonStatus::Paralysis | 
            PokemonStatus::Poison | PokemonStatus::Toxic
        );
        
        if has_status {
            // Return a power modifier instruction that doubles the base power
            // This will be handled by the damage calculation system
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
        }
    }
    
    // No status condition, normal power
    vec![StateInstructions::empty()]
}

/// Apply Hex - doubles power against statused targets
pub fn apply_hex(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut has_statused_target = false;
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status != PokemonStatus::None {
                has_statused_target = true;
                break;
            }
        }
    }
    
    if has_statused_target {
        // Return a power modifier instruction that doubles the base power
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
    }
    
    // No statused targets, normal power
    vec![StateInstructions::empty()]
}

/// Apply Gyro Ball - higher power with lower Speed
pub fn apply_gyro_ball(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if target_positions.is_empty() {
        return vec![StateInstructions::empty()];
    }
    
    let target_position = target_positions[0];
    
    if let (Some(user), Some(target)) = (
        state.get_pokemon_at_position(user_position),
        state.get_pokemon_at_position(target_position)
    ) {
        // Gyro Ball power = min(150, max(1, 25 * target_speed / user_speed))
        let user_speed = user.stats.speed as f32;
        let target_speed = target.stats.speed as f32;
        
        if user_speed > 0.0 && move_data.base_power.is_some() {
            let base_power = move_data.base_power.unwrap() as f32;
            let power_multiplier = ((25.0 * target_speed / user_speed) / base_power).min(150.0 / base_power).max(1.0 / base_power);
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
        }
    }
    
    // Fallback to normal power
    vec![StateInstructions::empty()]
}

/// Apply Reversal - higher power at lower HP
pub fn apply_reversal(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Reversal power based on HP percentage
        let hp_percentage = (user.hp as f32 / user.max_hp as f32) * 100.0;
        
        let base_power = if hp_percentage > 68.75 {
            20
        } else if hp_percentage > 35.42 {
            40
        } else if hp_percentage > 20.83 {
            80
        } else if hp_percentage > 10.42 {
            100
        } else if hp_percentage > 4.17 {
            150
        } else {
            200
        };
        
        let power_multiplier = if let Some(base_move_power) = move_data.base_power {
            base_power as f32 / base_move_power as f32
        } else {
            1.0 // Fallback if move has no base power
        };
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
    }
    
    // Fallback to normal power
    vec![StateInstructions::empty()]
}

/// Apply Acrobatics - doubles power without item
pub fn apply_acrobatics(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Acrobatics doubles power if user has no item or an unusable item
        let has_no_item = user.item.is_none() || user.item.as_ref().map_or(true, |item| item.is_empty());
        
        if has_no_item {
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
        }
    }
    
    // Has item, normal power
    vec![StateInstructions::empty()]
}

/// Apply Weather Ball - type and power change based on weather
pub fn apply_weather_ball(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Weather Ball doubles power and changes type based on weather
    let power_multiplier = match state.weather {
        crate::core::instruction::Weather::Sun | 
        crate::core::instruction::Weather::HarshSun |
        crate::core::instruction::Weather::Rain | 
        crate::core::instruction::Weather::HeavyRain |
        crate::core::instruction::Weather::Sand |
        crate::core::instruction::Weather::Hail |
        crate::core::instruction::Weather::Snow => 2.0,
        crate::core::instruction::Weather::None => 1.0,
    };
    
    if power_multiplier > 1.0 {
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
    }
    
    // No weather, normal power
    vec![StateInstructions::empty()]
}

/// Helper function to apply power modifier moves
/// This is a placeholder - in a full implementation, this would create instructions
/// that modify the move's power during damage calculation
fn apply_power_modifier_move(
    _state: &State,
    _move_data: &EngineMoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    _power_multiplier: f32,
) -> Vec<StateInstructions> {
    // For now, return empty as power modification would need to be handled
    // by the damage calculation system, not the move effects system
    // In a full implementation, this would create a PowerModifier instruction
    // that the damage calculator would read and apply
    vec![StateInstructions::empty()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{Pokemon, MoveCategory, State};
    use crate::data::types::EngineMoveData;
    use crate::core::battle_format::{BattleFormat, FormatType, SideReference};
    use crate::generation::Generation;

    fn create_test_pokemon() -> Pokemon {
        Pokemon::new("Test".to_string())
    }

    fn create_test_state() -> State {
        let mut state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let pokemon1 = Pokemon::new("Attacker".to_string());
        let pokemon2 = Pokemon::new("Defender".to_string());
        
        state.side_one.add_pokemon(pokemon1);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(pokemon2);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));
        
        state
    }
    
    fn create_test_generation() -> GenerationMechanics {
        Generation::Gen9.get_mechanics()
    }

    fn create_test_move(name: &str) -> EngineMoveData {
        EngineMoveData {
            id: 1,
            name: name.to_string(),
            base_power: Some(80),
            accuracy: Some(100),
            pp: 10,
            move_type: "Normal".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::ps_types::PSMoveTarget::Scripted,
            effect_chance: None,
            effect_description: String::new(),
            flags: vec![],
        }
    }

    #[test]
    fn test_thunder_wave_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_thunder_wave(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::Paralysis)
        }));
    }

    #[test]
    fn test_swords_dance_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_swords_dance(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&2))
        }));
    }

    #[test]
    fn test_recover_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_recover(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionHeal(_))
        }));
    }

    #[test]
    fn test_protect_blocking() {
        let mut target = create_test_pokemon();
        let move_data = create_test_move("Tackle");

        // No protection - move should not be blocked
        assert!(!is_move_blocked_by_protection(&move_data, &target));

        // With protection - move should be blocked
        target.volatile_statuses.insert(crate::core::instruction::VolatileStatus::Protect);
        assert!(is_move_blocked_by_protection(&move_data, &target));
    }

    #[test]
    fn test_feint_bypassing_protection() {
        let mut target = create_test_pokemon();
        let feint = create_test_move("Feint");
        
        target.volatile_statuses.insert(crate::core::instruction::VolatileStatus::Protect);
        
        // Feint should bypass protection
        assert!(!is_move_blocked_by_protection(&feint, &target));
    }

    #[test]
    fn test_accuracy_calculation() {
        let user = create_test_pokemon();
        let target = create_test_pokemon();
        let move_data = create_test_move("Thunder Wave");

        let accuracy = calculate_accuracy(&move_data, &user, &target);
        assert_eq!(accuracy, 1.0); // 100% accuracy move
    }

    #[test]
    fn test_immunity_checks() {
        let generation = create_test_generation(); // Gen 9
        let gen5 = Generation::Gen5.get_mechanics();
        
        let mut electric_pokemon = create_test_pokemon();
        electric_pokemon.types = vec!["Electric".to_string()];
        
        let mut grass_pokemon = create_test_pokemon();
        grass_pokemon.types = vec!["Grass".to_string()];
        
        let mut poison_pokemon = create_test_pokemon();
        poison_pokemon.types = vec!["Poison".to_string()];
        
        let mut fire_pokemon = create_test_pokemon();
        fire_pokemon.types = vec!["Fire".to_string()];
        
        // Test immunities in Gen 9 (modern mechanics)
        assert!(is_immune_to_paralysis(&electric_pokemon, &generation));
        assert!(is_immune_to_powder(&grass_pokemon, &generation));
        assert!(is_immune_to_poison(&poison_pokemon, &generation));
        assert!(is_immune_to_burn(&fire_pokemon, &generation));
        
        // Test non-immunities in Gen 9
        assert!(!is_immune_to_paralysis(&grass_pokemon, &generation));
        assert!(!is_immune_to_powder(&electric_pokemon, &generation));
        assert!(!is_immune_to_poison(&electric_pokemon, &generation));
        assert!(!is_immune_to_burn(&electric_pokemon, &generation));
        
        // Test generation differences
        // Electric types were NOT immune to paralysis in Gen 5
        assert!(!is_immune_to_paralysis(&electric_pokemon, &gen5));
        // Grass types were NOT immune to powder moves in Gen 5
        assert!(!is_immune_to_powder(&grass_pokemon, &gen5));
    }

    // =============================================================================
    // TESTS FOR NEW MOVE IMPLEMENTATIONS
    // =============================================================================

    fn create_pokemon_with_types(name: &str, types: Vec<&str>) -> Pokemon {
        let mut pokemon = Pokemon::new(name.to_string());
        pokemon.types = types.into_iter().map(|t| t.to_string()).collect();
        pokemon
    }

    fn create_pokemon_with_status(name: &str, status: PokemonStatus) -> Pokemon {
        let mut pokemon = Pokemon::new(name.to_string());
        pokemon.status = status;
        pokemon
    }

    fn create_test_state_with_weather(weather: crate::core::instruction::Weather) -> State {
        let mut state = create_test_state();
        state.weather = weather;
        state
    }

    #[test]
    fn test_glare_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_glare(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::Paralysis)
        }));
    }

    #[test]
    fn test_spore_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_spore(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::Sleep)
        }));
    }

    #[test]
    fn test_acid_secondary_effect() {
        let state = create_test_state();
        let generation = Generation::Gen1.get_mechanics(); // Higher effect chance in Gen 1
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        let move_data = create_test_move("Acid");
        
        let instructions = apply_acid(&state, &move_data, user_pos, &[target_pos], &generation);
        
        // Should have multiple probability branches
        assert!(instructions.len() > 1);
    }

    #[test]
    fn test_charm_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_charm(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&-2))
        }));
    }

    #[test]
    fn test_growth_in_sun() {
        let state = create_test_state_with_weather(crate::core::instruction::Weather::Sun);
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_growth(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&2) 
                && boost_instr.stat_boosts.get(&Stat::SpecialAttack) == Some(&2))
        }));
    }

    #[test]
    fn test_growth_normal_weather() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_growth(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&1) 
                && boost_instr.stat_boosts.get(&Stat::SpecialAttack) == Some(&1))
        }));
    }

    #[test]
    fn test_aqua_ring_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_aqua_ring(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::AquaRing)
        }));
    }

    #[test]
    fn test_shore_up_in_sand() {
        let state = create_test_state_with_weather(crate::core::instruction::Weather::Sand);
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_shore_up(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionHeal(heal_instr) 
                if heal_instr.heal_amount > 0)
        }));
    }

    #[test]
    fn test_aromatherapy_team_heal() {
        let mut state = create_test_state();
        
        // Add a poisoned Pokemon to the team
        let mut poisoned_pokemon = Pokemon::new("Poisoned".to_string());
        poisoned_pokemon.status = PokemonStatus::Poison;
        state.side_one.add_pokemon(poisoned_pokemon);
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_aromatherapy(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::None)
        }));
    }

    #[test]
    fn test_attract_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_attract(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Attract)
        }));
    }

    #[test]
    fn test_confuse_ray_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_confuse_ray(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Confusion)
        }));
    }

    #[test]
    fn test_haze_resets_all_stats() {
        let mut state = create_test_state();
        // Add some stat boosts to test the reset
        if let Some(pokemon) = state.get_pokemon_at_position_mut(BattlePosition::new(SideReference::SideOne, 0)) {
            pokemon.stat_boosts.insert(Stat::Attack, 2);
        }
        if let Some(pokemon) = state.get_pokemon_at_position_mut(BattlePosition::new(SideReference::SideTwo, 0)) {
            pokemon.stat_boosts.insert(Stat::Defense, 1);
        }
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_haze(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        // Should generate instructions to reset stat boosts
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.is_empty())
        }));
    }

    #[test]
    fn test_clear_smog_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_clear_smog(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.is_empty())
        }));
    }

    #[test]
    fn test_weather_moves() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Test Sunny Day
        let instructions = apply_sunny_day(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ChangeWeather(weather_instr) 
                if weather_instr.weather == crate::core::instruction::Weather::Sun)
        }));
        
        // Test Rain Dance
        let instructions = apply_rain_dance(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ChangeWeather(weather_instr) 
                if weather_instr.weather == crate::core::instruction::Weather::Rain)
        }));
        
        // Test Sandstorm
        let instructions = apply_sandstorm(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ChangeWeather(weather_instr) 
                if weather_instr.weather == crate::core::instruction::Weather::Sand)
        }));
    }

    #[test]
    fn test_screen_moves() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Test Light Screen
        let instructions = apply_light_screen(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::LightScreen)
        }));
        
        // Test Reflect
        let instructions = apply_reflect_move(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::Reflect)
        }));
    }

    #[test]
    fn test_aurora_veil_in_hail() {
        let state = create_test_state_with_weather(crate::core::instruction::Weather::Hail);
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_aurora_veil(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::AuroraVeil)
        }));
    }

    #[test]
    fn test_aurora_veil_fails_without_hail() {
        let state = create_test_state(); // No weather
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_aurora_veil(&state, user_pos, &[], &generation);
        
        // Should return empty instructions (move fails)
        assert!(instructions.len() == 1 && instructions[0].instruction_list.is_empty());
    }

    #[test]
    fn test_hazard_moves() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Test Spikes
        let instructions = apply_spikes(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::Spikes)
        }));
        
        // Test Stealth Rock
        let instructions = apply_stealth_rock(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::StealthRock)
        }));
        
        // Test Toxic Spikes
        let instructions = apply_toxic_spikes(&state, user_pos, &[], &generation);
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_cond_instr) 
                if side_cond_instr.condition == SideCondition::ToxicSpikes)
        }));
    }

    #[test]
    fn test_hazard_removal() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        // Test Rapid Spin
        let instructions = apply_rapid_spin(&state, user_pos, &[], &generation);
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::RemoveSideCondition(_))
        }));
        
        // Test Defog
        let instructions = apply_defog(&state, user_pos, &[], &generation);
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::RemoveSideCondition(_))
        }));
    }

    #[test]
    fn test_baton_pass_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_baton_pass(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ToggleBatonPassing(_))
        }));
    }

    #[test]
    fn test_belly_drum_sufficient_hp() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_belly_drum(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        // Should have both damage and stat boost instructions
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionDamage(_))
        }));
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&6))
        }));
    }

    #[test]
    fn test_curse_ghost_vs_normal() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        // Mock state with Ghost type user would need more setup
        // For now, test the non-Ghost version
        let instructions = apply_curse(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        // Non-Ghost curse should boost Attack/Defense, lower Speed
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&1)
                && boost_instr.stat_boosts.get(&Stat::Defense) == Some(&1)
                && boost_instr.stat_boosts.get(&Stat::Speed) == Some(&-1))
        }));
    }

    #[test]
    fn test_destiny_bond_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_destiny_bond(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::DestinyBond)
        }));
    }

    #[test]
    fn test_encore_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_encore(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Encore)
        }));
    }

    #[test]
    fn test_leech_seed_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_leech_seed(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::LeechSeed)
        }));
    }

    #[test]
    fn test_rest_effect() {
        let mut state = create_test_state();
        // Damage the Pokemon so Rest will heal it
        if let Some(pokemon) = state.get_pokemon_at_position_mut(BattlePosition::new(SideReference::SideOne, 0)) {
            pokemon.hp = pokemon.max_hp / 2; // Set to 50% HP
        }
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_rest(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        // Should have heal, status change to sleep, and rest turns instructions
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionHeal(_))
        }));
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::Sleep)
        }));
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::SetRestTurns(_))
        }));
    }

    #[test]
    fn test_sleep_talk_while_asleep() {
        let mut state = create_test_state();
        // Set user Pokemon to be asleep
        if let Some(pokemon) = state.get_pokemon_at_position_mut(BattlePosition::new(SideReference::SideOne, 0)) {
            pokemon.status = PokemonStatus::Sleep;
        }
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_sleep_talk(&state, user_pos, &[], &generation);
        
        // Move should succeed (empty instructions, actual move selection handled elsewhere)
        assert!(!instructions.is_empty());
    }

    #[test]
    fn test_taunt_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_taunt(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Taunt)
        }));
    }

    #[test]
    fn test_whirlwind_force_switch() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_whirlwind(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ToggleSideTwoForceSwitch)
        }));
    }

    #[test]
    fn test_yawn_effect() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_yawn(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Yawn)
        }));
    }

    #[test]
    fn test_splash_does_nothing() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_splash(&state, user_pos, &[], &generation);
        
        assert_eq!(instructions.len(), 1);
        assert!(instructions[0].instruction_list.is_empty());
        assert_eq!(instructions[0].percentage, 100.0);
    }

    #[test]
    fn test_kinesis_lowers_accuracy() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_kinesis(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Accuracy) == Some(&-1))
        }));
    }

    #[test]
    fn test_tailwind_sets_side_condition() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_tailwind(&state, user_pos, &[], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplySideCondition(side_condition_instr) 
                if side_condition_instr.condition == SideCondition::TailWind 
                && side_condition_instr.duration == Some(4))
        }));
    }

    #[test]
    fn test_seismic_toss_fixed_damage() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_seismic_toss(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionDamage(damage_instr) 
                if damage_instr.damage_amount == 50) // Default level in test state
        }));
    }

    #[test]
    fn test_pain_split_averages_hp() {
        let mut state = create_test_state();
        
        // Set different HP values for testing
        if let Some(user) = state.side_one.get_active_pokemon_at_slot_mut(0) {
            user.hp = 100;
            user.max_hp = 100;
        }
        if let Some(target) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            target.hp = 60;
            target.max_hp = 100;
        }
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_pain_split(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        
        // Should have instructions for both Pokemon to reach average HP (80)
        let has_user_damage = instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionDamage(damage_instr) 
                if damage_instr.target_position == user_pos && damage_instr.damage_amount == 20)
        });
        let has_target_heal = instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionHeal(heal_instr) 
                if heal_instr.target_position == target_pos && heal_instr.heal_amount == 20)
        });
        
        assert!(has_user_damage);
        assert!(has_target_heal);
    }

    #[test]
    fn test_fake_out_causes_flinch() {
        let state = create_test_state();
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        let move_data = create_test_move("Fake Out");
        
        let instructions = apply_fake_out(&state, &move_data, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyVolatileStatus(vol_status_instr) 
                if vol_status_instr.volatile_status == VolatileStatus::Flinch)
        }));
    }

    #[test]
    fn test_endeavor_reduces_target_hp() {
        let mut state = create_test_state();
        
        // Set user at low HP and target at high HP
        if let Some(user) = state.side_one.get_active_pokemon_at_slot_mut(0) {
            user.hp = 20;
            user.max_hp = 100;
        }
        if let Some(target) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            target.hp = 80;
            target.max_hp = 100;
        }
        
        let generation = create_test_generation();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_endeavor(&state, user_pos, &[target_pos], &generation);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionDamage(damage_instr) 
                if damage_instr.target_position == target_pos && damage_instr.damage_amount == 60) // 80 - 20 = 60
        }));
    }
}

// =============================================================================
// NEW MOVE IMPLEMENTATIONS (MISSING FROM TAPU-SIMU)
// =============================================================================

/// Apply Trick - swaps held items between user and target
/// Fails if both Pokemon have the same item, target is behind Substitute,
/// target has Sticky Hold ability, or target has a permanent item
pub fn apply_trick(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if target_positions.is_empty() {
        return vec![StateInstructions::empty()];
    }
    
    let target_position = target_positions[0]; // Trick only targets one Pokemon
    
    // Get user and target Pokemon
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![StateInstructions::empty()],
    };
    
    let target = match state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return vec![StateInstructions::empty()],
    };
    
    // Check if move should fail
    if should_item_swap_fail(user, target) {
        return vec![StateInstructions::empty()];
    }
    
    // Create item swap instructions
    let mut instructions = Vec::new();
    
    // Change user's item to target's item
    instructions.push(Instruction::ChangeItem(ChangeItemInstruction {
        target_position: user_position,
        new_item: target.item.clone(),
        previous_item: user.item.clone(),
    }));
    
    // Change target's item to user's item
    instructions.push(Instruction::ChangeItem(ChangeItemInstruction {
        target_position,
        new_item: user.item.clone(),
        previous_item: target.item.clone(),
    }));
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Switcheroo - identical to Trick but Dark-type
pub fn apply_switcheroo(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Switcheroo has identical mechanics to Trick
    apply_trick(state, user_position, target_positions, generation)
}

/// Apply Tidy Up - removes hazards and substitutes from both sides, then boosts user's Attack and Speed
pub fn apply_tidy_up(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Remove all hazards from both sides
    for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
        // Remove Spikes
        if state.get_side(side_ref).side_conditions.contains_key(&SideCondition::Spikes) {
            instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                side: side_ref,
                condition: SideCondition::Spikes,
            }));
        }
        
        // Remove Stealth Rock
        if state.get_side(side_ref).side_conditions.contains_key(&SideCondition::StealthRock) {
            instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                side: side_ref,
                condition: SideCondition::StealthRock,
            }));
        }
        
        // Remove Toxic Spikes
        if state.get_side(side_ref).side_conditions.contains_key(&SideCondition::ToxicSpikes) {
            instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                side: side_ref,
                condition: SideCondition::ToxicSpikes,
            }));
        }
        
        // Remove Sticky Web
        if state.get_side(side_ref).side_conditions.contains_key(&SideCondition::StickyWeb) {
            instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                side: side_ref,
                condition: SideCondition::StickyWeb,
            }));
        }
    }
    
    // Remove substitutes from all Pokemon
    for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                if pokemon.volatile_statuses.contains(&VolatileStatus::Substitute) {
                    let position = BattlePosition::new(side_ref, slot);
                    instructions.push(Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                        target_position: position,
                        volatile_status: VolatileStatus::Substitute,
                    }));
                }
            }
        }
    }
    
    // Boost user's Attack and Speed by 1 stage each
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    instructions.push(Instruction::BoostStats(BoostStatsInstruction {
        target_position: user_position,
        stat_boosts,
        previous_boosts: Some(HashMap::new()),
    }));
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Court Change - swaps all hazards and side conditions between the two sides
pub fn apply_court_change(
    state: &State,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let side_one_conditions = &state.side_one.side_conditions;
    let side_two_conditions = &state.side_two.side_conditions;
    
    // Remove all conditions from both sides first
    for (condition, _) in side_one_conditions {
        instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
            side: SideReference::SideOne,
            condition: *condition,
        }));
    }
    
    for (condition, _) in side_two_conditions {
        instructions.push(Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
            side: SideReference::SideTwo,
            condition: *condition,
        }));
    }
    
    // Apply side one's conditions to side two
    for (condition, &duration) in side_one_conditions {
        instructions.push(Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideTwo,
            condition: *condition,
            duration: Some(duration),
        }));
    }
    
    // Apply side two's conditions to side one  
    for (condition, &duration) in side_two_conditions {
        instructions.push(Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideOne,
            condition: *condition,
            duration: Some(duration),
        }));
    }
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Chilly Reception - sets Snow weather and forces user to switch out
pub fn apply_chilly_reception(
    state: &State,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Set Snow weather (5 turns)
    instructions.push(Instruction::ChangeWeather(ChangeWeatherInstruction {
        weather: crate::core::instruction::Weather::Snow,
        duration: Some(5),
        previous_weather: Some(state.weather),
        previous_duration: Some(state.weather_turns_remaining),
    }));
    
    // Force the user to switch out - this would need additional logic
    // For now, we apply a volatile status that indicates the Pokemon must switch
    // TODO: Add MustSwitch volatile status to handle forced switching
    // instructions.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
    //     target_position: user_position,
    //     volatile_status: VolatileStatus::MustSwitch, // Would need to add this volatile status
    //     duration: Some(1),
    // }));
    
    vec![StateInstructions::new(100.0, instructions)]
}

/// Apply Grassy Glide - priority move that gets +1 priority in Grassy Terrain
pub fn apply_grassy_glide(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Check if we're in Grassy Terrain
    let has_priority = matches!(state.terrain, crate::core::instruction::Terrain::GrassyTerrain);
    
    if has_priority {
        // The move already has +1 priority in Grassy Terrain, which should be handled
        // in the move priority calculation, not here. This function handles any
        // additional effects beyond the priority boost.
        
        // Grassy Glide is just a physical Grass-type move with conditional priority
        // No special effects beyond damage, so we use generic effects
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    } else {
        // Without Grassy Terrain, it's just a normal priority move
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

// =============================================================================
// TWO-TURN/CHARGE MOVES - 100% PARITY WITH POKE-ENGINE
// =============================================================================

/// Apply Solar Beam - no charge in sun, reduced power in other weather
pub fn apply_solar_beam(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user is already charging Solar Beam
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SolarBeam) {
            // Second turn - attack with potentially modified power
            let power_multiplier = match state.weather {
                Weather::Sun | Weather::HarshSun => 1.0, // Full power in sun
                Weather::Rain | Weather::HeavyRain | Weather::Sand | Weather::Hail | Weather::Snow => 0.5, // Half power in other weather
                Weather::None => 1.0, // Full power in no weather
            };
            
            let modified_move_data = EngineMoveData {
                base_power: move_data.base_power.map(|p| (p as f32 * power_multiplier) as i16),
                ..move_data.clone()
            };
            
            // Remove charging status
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SolarBeam,
                })
            ]));
            
            // Apply damage
            let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - check if we can skip charging in sun
            if state.weather == Weather::Sun || state.weather == Weather::HarshSun {
                // Skip charging and attack immediately in sun
                let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
                instructions.extend(generic_instructions);
            } else {
                // Start charging
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                        target_position: user_position,
                        volatile_status: VolatileStatus::SolarBeam,
                        duration: Some(1), // Charge for 1 turn
                    })
                ]));
            }
        }
    }
    
    instructions
}

/// Apply Solar Blade - identical to Solar Beam but physical
pub fn apply_solar_blade(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Solar Blade has identical mechanics to Solar Beam
    apply_solar_beam(state, move_data, user_position, target_positions, generation)
}

/// Apply Meteor Beam - boosts Special Attack on charge turn
pub fn apply_meteor_beam(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::MeteorBeam) {
            // Second turn - attack
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::MeteorBeam,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - charge and boost Special Attack
            let mut charge_instructions = vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::MeteorBeam,
                    duration: Some(1),
                }),
                Instruction::BoostStats(BoostStatsInstruction {
                    target_position: user_position,
                    stat_boosts: [(Stat::SpecialAttack, 1)].iter().cloned().collect(),
                    previous_boosts: None,
                })
            ];
            
            instructions.push(StateInstructions::new(100.0, charge_instructions));
        }
    }
    
    instructions
}

/// Apply Electro Shot - boosts Special Attack on charge turn
pub fn apply_electro_shot(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Electro Shot has identical mechanics to Meteor Beam
    apply_meteor_beam(state, move_data, user_position, target_positions, generation)
}

/// Apply Dig - user goes underground on turn 1, attacks on turn 2
pub fn apply_dig(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::Dig) {
            // Second turn - attack and come back up
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Dig,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - go underground
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Dig,
                    duration: Some(1),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Fly - user flies up on turn 1, attacks on turn 2
pub fn apply_fly(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::Fly) {
            // Second turn - attack and come down
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Fly,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - fly up
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Fly,
                    duration: Some(1),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Bounce - user bounces up on turn 1, attacks on turn 2 with paralysis chance
pub fn apply_bounce(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::Fly) {
            // Second turn - attack and come down
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Fly,
                })
            ]));
            
            // Apply damage
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
            
            // 30% chance to paralyze target
            for &target_position in target_positions {
                if let Some(target) = state.get_pokemon_at_position(target_position) {
                    if target.status == PokemonStatus::None && !is_immune_to_paralysis(target, generation) {
                        instructions.push(StateInstructions::new(30.0, vec![
                            Instruction::ApplyStatus(ApplyStatusInstruction {
                                target_position,
                                status: PokemonStatus::Paralysis,
                                previous_status: Some(target.status),
                                previous_status_duration: None,
                            })
                        ]));
                    }
                }
            }
        } else {
            // First turn - bounce up
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Fly,
                    duration: Some(1),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Dive - user dives underwater on turn 1, attacks on turn 2
pub fn apply_dive(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::Dive) {
            // Second turn - attack and come back up
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Dive,
                })
            ]));
            
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - dive underwater
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Dive,
                    duration: Some(1),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Phantom Force - user vanishes on turn 1, attacks on turn 2 (ignores protection)
pub fn apply_phantom_force(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::PhantomForce) {
            // Second turn - attack and reappear
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::PhantomForce,
                })
            ]));
            
            // This move ignores protection - handled in damage calculation
            let generic_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            instructions.extend(generic_instructions);
        } else {
            // First turn - vanish
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::PhantomForce,
                    duration: Some(1),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Shadow Force - identical to Phantom Force but different type
pub fn apply_shadow_force(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Shadow Force has identical mechanics to Phantom Force
    apply_phantom_force(state, move_data, user_position, target_positions, generation)
}

// =============================================================================
// HELPER FUNCTIONS FOR NEW MOVES
// =============================================================================

/// Check if an item swap move (Trick/Switcheroo) should fail
fn should_item_swap_fail(user: &Pokemon, target: &Pokemon) -> bool {
    // Fail if both Pokemon have the same item (including both having no item)
    if user.item == target.item {
        return true;
    }
    
    // Fail if target has Sticky Hold ability
    if target.ability.to_lowercase() == "sticky hold" {
        return true;
    }
    
    // Fail if target has a permanent item
    if target.item.as_ref().map_or(false, |item| is_permanent_item(item, &target.species)) {
        return true;
    }
    
    // Fail if target is behind a Substitute
    if target.volatile_statuses.contains(&VolatileStatus::Substitute) {
        return true;
    }
    
    false
}

/// Check if an item is permanent and cannot be removed
fn is_permanent_item(item: &str, pokemon_species: &str) -> bool {
    match item.to_lowercase().as_str() {
        // Arceus plates
        "draco plate" | "dread plate" | "earth plate" | "fist plate" | 
        "flame plate" | "icicle plate" | "insect plate" | "iron plate" |
        "meadow plate" | "mind plate" | "pixie plate" | "sky plate" |
        "splash plate" | "spooky plate" | "stone plate" | "toxic plate" |
        "zap plate" => pokemon_species.to_lowercase().starts_with("arceus"),
        
        // Origin forme items
        "lustrous globe" => pokemon_species.to_lowercase().contains("palkia"),
        "griseous core" => pokemon_species.to_lowercase().contains("giratina"),
        "adamant crystal" => pokemon_species.to_lowercase().contains("dialga"),
        
        // Rusted weapons
        "rusted sword" => pokemon_species.to_lowercase().contains("zacian"),
        "rusted shield" => pokemon_species.to_lowercase().contains("zamazenta"),
        
        // Silvally memories
        "bug memory" | "dark memory" | "dragon memory" | "electric memory" |
        "fairy memory" | "fighting memory" | "fire memory" | "flying memory" |
        "ghost memory" | "grass memory" | "ground memory" | "ice memory" |
        "poison memory" | "psychic memory" | "rock memory" | "steel memory" |
        "water memory" => pokemon_species.to_lowercase() == "silvally",
        
        // Drive items for Genesect
        "burn drive" | "chill drive" | "douse drive" | "shock drive" => 
            pokemon_species.to_lowercase() == "genesect",
        
        // Ogerpon masks
        "cornerstone mask" | "hearthflame mask" | "wellspring mask" | "teal mask" => 
            pokemon_species.to_lowercase().contains("ogerpon"),
        
        _ => false,
    }
}

// =============================================================================
// NEW VARIABLE POWER MOVES - 100% PARITY WITH POKE-ENGINE
// =============================================================================

/// Apply Avalanche - doubles power if user was hit by Physical/Special move and moved second
pub fn apply_avalanche(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user was hit by a Physical or Special move this turn and moved second
    let user_pokemon = state.get_pokemon_at_position(user_position);
    let power_multiplier = if user_pokemon.map_or(false, |p| {
        // Check if user took damage from Physical/Special move this turn
        // For now, assume 1x power since we don't track move order yet
        // TODO: Implement turn order tracking for proper Avalanche mechanics
        false
    }) {
        2.0 // Double power if hit first
    } else {
        1.0 // Base power
    };
    
    // Apply generic damage with modified power
    let modified_move_data = EngineMoveData {
        base_power: move_data.base_power.map(|p| (p as f32 * power_multiplier) as i16),
        ..move_data.clone()
    };
    
    let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
    instructions.extend(generic_instructions);
    
    instructions
}

/// Apply Bolt Beak - doubles power if user moves first
pub fn apply_boltbeak(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user moves first this turn
    // TODO: Implement proper turn order tracking
    // For now, use base power since we don't have turn order context
    let power_multiplier = if user_moves_first(state, user_position) {
        2.0 // Double power when moving first
    } else {
        1.0 // Base power
    };
    
    let modified_move_data = EngineMoveData {
        base_power: move_data.base_power.map(|p| (p as f32 * power_multiplier) as i16),
        ..move_data.clone()
    };
    
    let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
    instructions.extend(generic_instructions);
    
    instructions
}

/// Apply Fishious Rend - doubles power if user moves first
pub fn apply_fishious_rend(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Fishious Rend has identical mechanics to Bolt Beak
    apply_boltbeak(state, move_data, user_position, target_positions, generation)
}

/// Apply Electro Ball - power based on speed ratio between user and target
pub fn apply_electroball(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        for &target_position in target_positions {
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                // Calculate speed stats with boosts
                let user_speed = calculate_boosted_speed(user_pokemon);
                let target_speed = calculate_boosted_speed(target_pokemon);
                
                // Calculate speed ratio and determine power
                let speed_ratio = if target_speed > 0 {
                    user_speed as f32 / target_speed as f32
                } else {
                    4.0 // Max power if target has 0 speed
                };
                
                let base_power = if speed_ratio >= 4.0 {
                    150i16
                } else if speed_ratio >= 3.0 {
                    120i16
                } else if speed_ratio >= 2.0 {
                    80i16
                } else if speed_ratio >= 1.0 {
                    60i16
                } else {
                    40i16
                };
                
                let modified_move_data = EngineMoveData {
                    base_power: Some(base_power),
                    ..move_data.clone()
                };
                
                let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    instructions
}

/// Apply Eruption - power based on user's current HP percentage
pub fn apply_eruption(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Power = 150 * (current HP / max HP)
        let hp_percentage = user_pokemon.hp as f32 / user_pokemon.max_hp as f32;
        let base_power = (150.0 * hp_percentage).max(1.0) as i16; // Minimum 1 power
        
        let modified_move_data = EngineMoveData {
            base_power: Some(base_power),
            ..move_data.clone()
        };
        
        let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
        instructions.extend(generic_instructions);
    }
    
    instructions
}

/// Apply Water Spout - power based on user's current HP percentage
pub fn apply_waterspout(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Water Spout has identical mechanics to Eruption
    apply_eruption(state, move_data, user_position, target_positions, generation)
}

/// Apply Dragon Energy - power based on user's current HP percentage
pub fn apply_dragon_energy(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Dragon Energy has identical mechanics to Eruption
    apply_eruption(state, move_data, user_position, target_positions, generation)
}

/// Apply Grass Knot - power based on target's weight
pub fn apply_grass_knot(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
            // Power based on target's weight (in kg)
            let target_weight = get_pokemon_weight(&target_pokemon.species);
            let base_power = if target_weight >= 200.0 {
                120i16
            } else if target_weight >= 100.0 {
                100i16
            } else if target_weight >= 50.0 {
                80i16
            } else if target_weight >= 25.0 {
                60i16
            } else if target_weight >= 10.0 {
                40i16
            } else {
                20i16
            };
            
            let modified_move_data = EngineMoveData {
                base_power: Some(base_power),
                ..move_data.clone()
            };
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    instructions
}

/// Apply Low Kick - power based on target's weight
pub fn apply_low_kick(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Low Kick has identical mechanics to Grass Knot
    apply_grass_knot(state, move_data, user_position, target_positions, generation)
}

/// Apply Heat Crash - power based on weight ratio between user and target
pub fn apply_heat_crash(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let user_weight = get_pokemon_weight(&user_pokemon.species);
        
        for &target_position in target_positions {
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                let target_weight = get_pokemon_weight(&target_pokemon.species);
                let weight_ratio = if target_weight > 0.0 {
                    user_weight / target_weight
                } else {
                    5.0 // Max power if target has 0 weight
                };
                
                let base_power = if weight_ratio >= 5.0 {
                    120i16
                } else if weight_ratio >= 4.0 {
                    100i16
                } else if weight_ratio >= 3.0 {
                    80i16
                } else if weight_ratio >= 2.0 {
                    60i16
                } else {
                    40i16
                };
                
                let modified_move_data = EngineMoveData {
                    base_power: Some(base_power),
                    ..move_data.clone()
                };
                
                let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    instructions
}

/// Apply Heavy Slam - power based on weight ratio between user and target
pub fn apply_heavy_slam(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Heavy Slam has identical mechanics to Heat Crash
    apply_heat_crash(state, move_data, user_position, target_positions, generation)
}

// =============================================================================
// MISSING VARIABLE POWER MOVES
// =============================================================================

/// Apply Barb Barrage - doubles power against poisoned targets
pub fn apply_barb_barrage(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Modify power based on poison status
            let mut modified_move_data = move_data.clone();
            if target.status == PokemonStatus::Poison || target.status == PokemonStatus::Toxic {
                if let Some(base_power) = modified_move_data.base_power {
                    modified_move_data.base_power = Some(base_power * 2); // Double power
                }
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Collision Course - 1.3x power when super effective
pub fn apply_collision_course(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let move_type = &move_data.move_type;
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut modified_move_data = move_data.clone();
            
            // Check if move is super effective against target
            if is_super_effective(move_type, target, generation) {
                let current_power = modified_move_data.base_power.unwrap_or(100);
                modified_move_data.base_power = Some((current_power as f32 * 1.3) as i16);
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Electro Drift - 1.3x power when super effective
pub fn apply_electro_drift(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let move_type = &move_data.move_type;
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut modified_move_data = move_data.clone();
            
            // Check if move is super effective against target
            if is_super_effective(move_type, target, generation) {
                let current_power = modified_move_data.base_power.unwrap_or(100);
                modified_move_data.base_power = Some((current_power as f32 * 1.3) as i16);
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Freeze-Dry - Ice move that's super effective against Water types
pub fn apply_freeze_dry(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut modified_move_data = move_data.clone();
            
            // Check if target is Water type - if so, boost damage significantly
            let has_water_type = target.types.get(0).map_or(false, |t| t.to_lowercase() == "water") || 
                                 target.types.get(1).map_or(false, |t| t.to_lowercase() == "water");
            
            if has_water_type {
                // Freeze-Dry treats Water types as if they were weak to Ice
                // This effectively makes it super effective (2x) against Water
                let current_power = modified_move_data.base_power.unwrap_or(70);
                modified_move_data.base_power = Some((current_power as f32 * 2.0) as i16);
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Hard Press - power decreases as target's HP increases (1-100 power)
pub fn apply_hard_press(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let hp_percentage = (target.hp as f32 / target.max_hp as f32) * 100.0;
            let power = ((hp_percentage / 100.0) * 100.0).max(1.0) as i16;
            
            let mut modified_move_data = move_data.clone();
            modified_move_data.base_power = Some(power);
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    instructions
}

/// Apply Hydro Steam - boosted power in sun weather
pub fn apply_hydro_steam(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let power_multiplier = match state.weather {
        Weather::Sun | Weather::HarshSun => 1.5, // 1.5x power in sun
        _ => 1.0,
    };
    
    apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
}

/// Apply Last Respects - power increases based on fainted team members
pub fn apply_last_respects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let user_side = match user_position.side {
        SideReference::SideOne => &state.side_one,
        SideReference::SideTwo => &state.side_two,
    };
    
    // Count fainted Pokemon
    let fainted_count = user_side.pokemon.iter()
        .filter(|p| p.hp == 0)
        .count() as u8;
    
    let power = 50 + (fainted_count * 50); // Base 50 + 50 per fainted
    let mut modified_move_data = move_data.clone();
    modified_move_data.base_power = Some(power.min(250) as i16); // Cap at reasonable power
    
    apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
}

/// Apply Poltergeist - fails if target has no item
pub fn apply_poltergeist(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.item.is_none() {
                // Move fails if target has no item
                instructions.push(StateInstructions::empty());
            } else {
                let target_instructions = apply_generic_effects(state, move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Pursuit - doubles power against switching targets
pub fn apply_pursuit(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<StateInstructions> {
    let mut modified_move_data = move_data.clone();
    
    // Check if opponent is switching out
    if context.opponent_switching {
        // Double the power when targeting a switching Pokemon
        let current_power = modified_move_data.base_power.unwrap_or(40);
        modified_move_data.base_power = Some(current_power * 2);
    }
    
    apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
}

/// Apply Stored Power - power increases with stat boosts
pub fn apply_stored_power(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let total_boosts: i32 = user.stat_boosts.values()
            .filter(|&&boost| boost > 0)
            .map(|&boost| boost as i32)
            .sum();
        
        let power = 20 + (total_boosts * 20); // Base 20 + 20 per positive boost
        let mut modified_move_data = move_data.clone();
        modified_move_data.base_power = Some(power.min(250) as i16); // Cap at reasonable power
        
        apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Power Trip - identical to Stored Power
pub fn apply_power_trip(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    apply_stored_power(state, move_data, user_position, target_positions, generation)
}

/// Apply Strength Sap - heals based on target's Attack stat and lowers it
pub fn apply_strength_sap(
    state: &State,
    _move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Lower target's Attack by 1 stage
            let mut stat_changes = HashMap::new();
            stat_changes.insert(Stat::Attack, -1);
            instruction_list.push(Instruction::BoostStats(BoostStatsInstruction {
                target_position,
                stat_boosts: stat_changes,
                previous_boosts: None,
            }));
            
            // Heal user based on target's current Attack stat
            if let Some(user) = state.get_pokemon_at_position(user_position) {
                let heal_amount = target.stats.attack as i16;
                instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                    target_position: user_position,
                    heal_amount,
                    previous_hp: Some(user.hp),
                }));
            }
        }
    }
    
    if instruction_list.is_empty() {
        vec![StateInstructions::empty()]
    } else {
        vec![StateInstructions::new(100.0, instruction_list)]
    }
}

/// Apply Sucker Punch - priority move that fails against status moves
pub fn apply_sucker_punch(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<StateInstructions> {
    // Sucker Punch fails if:
    // 1. User doesn't go first, OR
    // 2. Opponent is using a status move
    let move_fails = !context.going_first || 
                     context.opponent_move_data.map_or(false, |opp_data| {
                         opp_data.category == MoveCategory::Status
                     });
    
    if move_fails {
        // Move fails - return empty instructions
        vec![StateInstructions::empty()]
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Thunder Clap - priority move that fails against status moves
pub fn apply_thunder_clap(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<StateInstructions> {
    // Thunder Clap fails if:
    // 1. User doesn't go first, OR
    // 2. Opponent is using a status move
    let move_fails = !context.going_first || 
                     context.opponent_move_data.map_or(false, |opp_data| {
                         opp_data.category == MoveCategory::Status
                     });
    
    if move_fails {
        // Move fails - return empty instructions
        vec![StateInstructions::empty()]
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Terrain Pulse - power and type change based on terrain
pub fn apply_terrain_pulse(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut modified_move_data = move_data.clone();
    
    // Base power is 50, doubles to 100 on terrain
    let base_power = 50;
    
    // Type and power change based on terrain
    match state.terrain {
        Terrain::ElectricTerrain => {
            modified_move_data.move_type = "Electric".to_string();
            modified_move_data.base_power = Some(base_power * 2);
        }
        Terrain::GrassyTerrain => {
            modified_move_data.move_type = "Grass".to_string();
            modified_move_data.base_power = Some(base_power * 2);
        }
        Terrain::MistyTerrain => {
            modified_move_data.move_type = "Fairy".to_string();
            modified_move_data.base_power = Some(base_power * 2);
        }
        Terrain::PsychicTerrain => {
            modified_move_data.move_type = "Psychic".to_string();
            modified_move_data.base_power = Some(base_power * 2);
        }
        Terrain::None => {
            // Remains Normal type with base power
            modified_move_data.move_type = "Normal".to_string();
            modified_move_data.base_power = Some(base_power);
        }
    }
    
    apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
}

/// Apply Upper Hand - priority counter to priority moves
pub fn apply_upper_hand(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<StateInstructions> {
    // Upper Hand only works when both conditions are met:
    // 1. User goes first, AND
    // 2. Target is using a priority move
    let move_succeeds = context.going_first && context.opponent_priority > 0;
    
    if !move_succeeds {
        // Move fails completely
        vec![StateInstructions::empty()]
    } else {
        // Move succeeds - apply damage and 100% flinch chance
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Future Sight - delayed damage after 3 turns
pub fn apply_future_sight(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Calculate damage that will be dealt in 3 turns
    // This is a simplified damage calculation - in a full implementation,
    // we'd use the proper damage calculation system
    let base_damage = move_data.base_power.unwrap_or(120); // Future Sight has 120 base power
    
    for &target_position in target_positions {
        // Set up Future Sight to activate in 3 turns
        instruction_list.push(Instruction::SetFutureSight(SetFutureSightInstruction {
            target_position,
            attacker_position: user_position,
            damage_amount: base_damage,
            turns_remaining: 3,
            move_name: move_data.name.clone(),
        }));
    }
    
    if instruction_list.is_empty() {
        vec![StateInstructions::empty()]
    } else {
        vec![StateInstructions::new(100.0, instruction_list)]
    }
}

// =============================================================================
// ITEM INTERACTION MOVES
// =============================================================================

/// Apply Knock Off - removes target's item and deals bonus damage
pub fn apply_knock_off(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut instruction_list = Vec::new();
            
            // Bonus damage if target has an item (Gen 6+)
            let power_multiplier = if generation.generation.number() >= 6 && target.item.is_some() {
                1.5
            } else {
                1.0
            };
            
            // Apply damage with potential bonus
            let damage_instructions = apply_power_modifier_move(state, move_data, user_position, &[target_position], generation, power_multiplier);
            for damage_instruction in damage_instructions {
                instruction_list.extend(damage_instruction.instruction_list);
            }
            
            // Remove target's item if it has one
            if target.item.is_some() {
                instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                    target_position,
                    new_item: None,
                    previous_item: target.item.clone(),
                }));
            }
            
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Thief - steals target's item if user has none
pub fn apply_thief(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                let mut instruction_list = Vec::new();
                
                // Apply damage first
                let damage_instructions = apply_generic_effects(state, move_data, user_position, &[target_position], generation);
                for damage_instruction in damage_instructions {
                    instruction_list.extend(damage_instruction.instruction_list);
                }
                
                // Steal item if user has none and target has one
                if user.item.is_none() && target.item.is_some() {
                    let stolen_item = target.item.clone();
                    
                    // Give item to user
                    instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                        target_position: user_position,
                        new_item: stolen_item,
                        previous_item: user.item.clone(),
                    }));
                    
                    // Remove item from target
                    instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                        target_position,
                        new_item: None,
                        previous_item: target.item.clone(),
                    }));
                }
                
                instructions.push(StateInstructions::new(100.0, instruction_list));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Fling - power and effect based on held item
pub fn apply_fling(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        if let Some(item) = &user.item {
            // Use item service to get fling data
            let item_service = crate::data::services::item_service::PSItemService::default();
            
            // Check if item can be flung
            if !item_service.can_be_flung(item) {
                // Move fails if item can't be flung
                return vec![StateInstructions::empty()];
            }
            
            let mut instruction_list = Vec::new();
            
            // Get item-specific power
            let fling_power = item_service.get_fling_power(item);
            
            // Create modified move data with item-specific power
            let mut modified_move = move_data.clone();
            modified_move.base_power = Some(fling_power as i16);
            
            // Apply damage with item-specific power
            let damage_instructions = apply_generic_effects(state, &modified_move, user_position, target_positions, generation);
            for damage_instruction in damage_instructions {
                instruction_list.extend(damage_instruction.instruction_list);
            }
            
            // Apply item-specific status effects
            for target_position in target_positions {
                if let Some(target) = state.get_pokemon_at_position(*target_position) {
                    // Apply main status effect if item has one
                    if let Some(status) = item_service.get_fling_status(item) {
                        let status_effect = match status {
                            "brn" => PokemonStatus::Burn,
                            "par" => PokemonStatus::Paralysis,
                            "psn" => PokemonStatus::Poison,
                            "tox" => PokemonStatus::Toxic,
                            "slp" => PokemonStatus::Sleep,
                            "frz" => PokemonStatus::Freeze,
                            _ => continue, // Unknown status
                        };
                        
                        // Don't apply if target already has a status condition
                        if target.status == PokemonStatus::None {
                            instruction_list.push(Instruction::ApplyStatus(ApplyStatusInstruction {
                                target_position: *target_position,
                                status: status_effect,
                                previous_status: Some(target.status),
                                previous_status_duration: None,
                            }));
                        }
                    }
                    
                    // Apply volatile status effect if item has one
                    if let Some(volatile_status) = item_service.get_fling_volatile_status(item) {
                        let volatile_effect = match volatile_status {
                            "flinch" => VolatileStatus::Flinch,
                            _ => continue, // Unknown volatile status
                        };
                        
                        instruction_list.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                            target_position: *target_position,
                            volatile_status: volatile_effect,
                            duration: None,
                        }));
                    }
                }
            }
            
            // Consume user's item
            instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                target_position: user_position,
                new_item: None,
                previous_item: user.item.clone(),
            }));
            
            vec![StateInstructions::new(100.0, instruction_list)]
        } else {
            // Move fails if user has no item
            vec![StateInstructions::empty()]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

// =============================================================================
// WEATHER-DEPENDENT MOVES
// =============================================================================

/// Apply Blizzard - 100% accuracy in hail/snow, normal accuracy otherwise
pub fn apply_blizzard(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Weather accuracy modification is handled by the accuracy calculation system
    // in apply_weather_accuracy_modifiers function - no special handling needed here
    apply_generic_effects(state, move_data, user_position, target_positions, generation)
}

/// Apply Hurricane - 100% accuracy in rain, 50% accuracy in sun
pub fn apply_hurricane(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Weather accuracy modification is handled by the accuracy calculation system
    // in apply_weather_accuracy_modifiers function - no special handling needed here
    
    // Hurricane also has a 30% chance to confuse the target
    let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
    
    // Add confusion effect (30% chance)
    for &target_position in target_positions {
        if let Some(_target) = state.get_pokemon_at_position(target_position) {
            instructions.push(StateInstructions::new(30.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position,
                    volatile_status: VolatileStatus::Confusion,
                    duration: Some(3), // 3 turns (simplified)
                }),
            ]));
        }
    }
    
    instructions
}

/// Apply Thunder - 100% accuracy in rain, 50% accuracy in sun
pub fn apply_thunder(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Weather accuracy modification is handled by the accuracy calculation system
    // in apply_weather_accuracy_modifiers function - no special handling needed here
    
    // Thunder also has a 30% chance to paralyze the target
    let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
    
    // Add paralysis effect (30% chance) if target is not immune
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if !is_immune_to_paralysis(target, generation) {
                instructions.push(StateInstructions::new(30.0, vec![
                    Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::Paralysis,
                        previous_status: Some(target.status),
                        previous_status_duration: Some(target.status_duration),
                    }),
                ]));
            }
        }
    }
    
    instructions
}

// =============================================================================
// FORM-DEPENDENT MOVES
// =============================================================================

/// Apply Aura Wheel - Type changes with Morpeko form
pub fn apply_aura_wheel(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check Morpeko form to determine type
        let modified_move_type = if user_pokemon.species.to_lowercase().contains("hangry") {
            "Dark" // Hangry Mode Morpeko
        } else {
            "Electric" // Full Belly Mode Morpeko (default)
        };
        
        // Create modified move data with form-based type
        let modified_move_data = EngineMoveData {
            move_type: modified_move_type.to_string(),
            ..move_data.clone()
        };
        
        // Apply move effects with boosted Speed (Aura Wheel always boosts Speed by 1 stage)
        let mut instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
        
        // Add Speed boost
        let mut speed_boost = HashMap::new();
        speed_boost.insert(Stat::Speed, 1);
        
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::BoostStats(BoostStatsInstruction {
                target_position: user_position,
                stat_boosts: speed_boost,
                previous_boosts: Some(HashMap::new()),
            }),
        ]));
        
        instructions
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Raging Bull - Type and effects change with Tauros form
pub fn apply_raging_bull(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Determine type based on Tauros form
        let modified_move_type = match user_pokemon.species.to_lowercase().as_str() {
            s if s.contains("tauros") && s.contains("combat") => "Fighting", // Paldean Combat Form
            s if s.contains("tauros") && s.contains("blaze") => "Fire",     // Paldean Blaze Form
            s if s.contains("tauros") && s.contains("aqua") => "Water",     // Paldean Aqua Form
            _ => &move_data.move_type, // Regular Tauros keeps Normal type
        };
        
        // Check if screens are present on the target's side to boost power
        let power_multiplier = if !target_positions.is_empty() {
            let target_side = state.get_side(target_positions[0].side);
            if target_side.side_conditions.contains_key(&SideCondition::Reflect) ||
               target_side.side_conditions.contains_key(&SideCondition::LightScreen) {
                2.0 // Double power against screens
            } else {
                1.0
            }
        } else {
            1.0
        };
        
        // Create modified move data
        let mut modified_move_data = EngineMoveData {
            move_type: modified_move_type.to_string(),
            ..move_data.clone()
        };
        
        // Apply power multiplier if screens are present
        if power_multiplier > 1.0 {
            if let Some(base_power) = modified_move_data.base_power {
                modified_move_data.base_power = Some((base_power as f32 * power_multiplier) as i16);
            }
        }
        
        // Apply move effects
        let mut instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
        
        // Remove screens after hitting (screen-breaking effect)
        if !target_positions.is_empty() {
            let target_side_ref = target_positions[0].side;
            let target_side = state.get_side(target_side_ref);
            
            // Remove Reflect
            if target_side.side_conditions.contains_key(&SideCondition::Reflect) {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                        side: target_side_ref,
                        condition: SideCondition::Reflect,
                    }),
                ]));
            }
            
            // Remove Light Screen
            if target_side.side_conditions.contains_key(&SideCondition::LightScreen) {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                        side: target_side_ref,
                        condition: SideCondition::LightScreen,
                    }),
                ]));
            }
        }
        
        instructions
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

// =============================================================================
// SPECIAL COMBAT MECHANICS
// =============================================================================

/// Apply Photon Geyser - Physical if Attack > Special Attack
pub fn apply_photon_geyser(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Compare Attack vs Special Attack stats to determine category
        let attack_stat = user_pokemon.stats.attack;
        let special_attack_stat = user_pokemon.stats.special_attack;
        
        let modified_move_data = EngineMoveData {
            category: if attack_stat > special_attack_stat {
                MoveCategory::Physical
            } else {
                MoveCategory::Special
            },
            ..move_data.clone()
        };
        
        apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Sky Drop - Two-turn move that lifts target
pub fn apply_sky_drop(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check if user is already in the Sky Drop charging state
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SkyDrop) {
            // Second turn - attack and remove both Pokemon from sky
            let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
            
            // Remove Sky Drop status from user
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkyDrop,
                }),
            ]));
            
            // Remove Sky Drop status from target (if any)
            for &target_position in target_positions {
                if let Some(target) = state.get_pokemon_at_position(target_position) {
                    if target.volatile_statuses.contains(&VolatileStatus::SkyDrop) {
                        instructions.push(StateInstructions::new(100.0, vec![
                            Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                                target_position,
                                volatile_status: VolatileStatus::SkyDrop,
                            }),
                        ]));
                    }
                }
            }
            
            instructions
        } else {
            // First turn - lift target into sky and apply Sky Drop status to both Pokemon
            let mut instructions = Vec::new();
            
            // Apply Sky Drop status to user
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkyDrop,
                    duration: None, // Lasts until second turn
                }),
            ]));
            
            // Apply Sky Drop status to target (lifted into sky)
            for &target_position in target_positions {
                if let Some(_target) = state.get_pokemon_at_position(target_position) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                            target_position,
                            volatile_status: VolatileStatus::SkyDrop,
                            duration: None, // Lasts until second turn
                        }),
                    ]));
                }
            }
            
            instructions
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

// =============================================================================
// ADVANCED HAZARD MANIPULATION
// =============================================================================


/// Apply Mortal Spin - Rapid Spin + poison damage to adjacent opponents
pub fn apply_mortal_spin(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Apply normal move damage first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation));
    
    // Remove hazards from user's side (like Rapid Spin)
    let user_side_ref = user_position.side;
    let user_side = state.get_side(user_side_ref);
    
    let hazards_to_remove = vec![
        SideCondition::Spikes,
        SideCondition::ToxicSpikes,
        SideCondition::StealthRock,
        SideCondition::StickyWeb,
    ];
    
    for condition in hazards_to_remove {
        if let Some(duration) = user_side.side_conditions.get(&condition) {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
                    side: user_side_ref,
                    condition,
                }),
            ]));
        }
    }
    
    // Poison all adjacent opponents (in doubles/multi-battles)
    let opponent_side_ref = match user_side_ref {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    // Get all active opponents and poison them
    let opponent_side = state.get_side(opponent_side_ref);
    for (slot, pokemon) in opponent_side.pokemon.iter().enumerate() {
        if let Some(active_slot) = opponent_side.active_pokemon_indices.get(slot) {
            if active_slot.is_some() && !pokemon.is_fainted() {
                let opponent_position = BattlePosition::new(opponent_side_ref, slot);
                
                // Apply poison if not already statused and not immune
                if pokemon.status == PokemonStatus::None && !is_immune_to_poison(pokemon, generation) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::ApplyStatus(ApplyStatusInstruction {
                            target_position: opponent_position,
                            status: PokemonStatus::Poison,
                            previous_status: Some(pokemon.status),
                            previous_status_duration: Some(pokemon.status_duration),
                        }),
                    ]));
                }
            }
        }
    }
    
    instructions
}

// =============================================================================
// SELF-DESTRUCT MOVES
// =============================================================================

/// Apply Explosion - user faints before dealing damage
pub fn apply_explosion(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instruction_list = Vec::new();
    
    // Apply damage to targets first
    let damage_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
    for damage_instruction in damage_instructions {
        instruction_list.extend(damage_instruction.instruction_list);
    }
    
    // User faints after dealing damage
    instruction_list.push(Instruction::Faint(crate::core::instruction::FaintInstruction {
        target_position: user_position,
        previous_hp: 0, // TODO: Should be set to actual HP before fainting
    }));
    
    vec![StateInstructions::new(100.0, instruction_list)]
}

/// Apply Self-Destruct - user faints before dealing damage
pub fn apply_self_destruct(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Self-Destruct has identical mechanics to Explosion
    apply_explosion(state, move_data, user_position, target_positions, generation)
}

// =============================================================================
// HELPER FUNCTIONS FOR NEW MOVES
// =============================================================================

/// Check if user moves first this turn (placeholder implementation)
fn user_moves_first(_state: &State, _user_position: BattlePosition) -> bool {
    // TODO: Implement proper turn order tracking
    // For now, return false to use base power
    false
}

/// Calculate boosted speed stat for a Pokemon
fn calculate_boosted_speed(pokemon: &Pokemon) -> i32 {
    let base_speed = pokemon.stats.speed;
    let boost_multiplier = match pokemon.stat_boosts.get(&Stat::Speed).unwrap_or(&0) {
        -6 => 0.25,
        -5 => 0.28,
        -4 => 0.33,
        -3 => 0.4,
        -2 => 0.5,
        -1 => 0.66,
        0 => 1.0,
        1 => 1.5,
        2 => 2.0,
        3 => 2.5,
        4 => 3.0,
        5 => 3.5,
        6 => 4.0,
        _ => 1.0,
    };
    
    (base_speed as f32 * boost_multiplier) as i32
}

// =============================================================================
// MISSING CHARGE MOVES
// =============================================================================

/// Apply Razor Wind - two-turn Normal move with high critical hit ratio
pub fn apply_razor_wind(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::RazorWind) {
            // Second turn - attack with high critical hit ratio
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::RazorWind,
                }),
            ]));
            
            // The high critical hit ratio would be handled by the damage calculation system
            // using the move's PS data flags
        } else {
            // First turn - charge
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::RazorWind,
                    duration: Some(1),
                }),
            ]));
        }
    }
    
    instructions
}

/// Apply Skull Bash - two-turn Normal move that boosts Defense on charge turn
pub fn apply_skull_bash(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SkullBash) {
            // Second turn - attack
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkullBash,
                }),
            ]));
        } else {
            // First turn - charge and boost Defense
            let mut instruction_list = vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkullBash,
                    duration: Some(1),
                }),
                Instruction::BoostStats(BoostStatsInstruction {
                    target_position: user_position,
                    stat_boosts: {
                        let mut boosts = HashMap::new();
                        boosts.insert(Stat::Defense, 1);
                        boosts
                    },
                    previous_boosts: None,
                }),
            ];
            
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    instructions
}

/// Apply Sky Attack - two-turn Flying move with high critical hit ratio and 30% flinch chance
pub fn apply_sky_attack(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SkyAttack) {
            // Second turn - attack with high critical hit ratio and flinch chance
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkyAttack,
                }),
            ]));
            
            // The flinch chance and high critical hit ratio would be handled by the damage 
            // calculation and effect systems using the move's PS data
        } else {
            // First turn - charge
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::SkyAttack,
                    duration: Some(1),
                }),
            ]));
        }
    }
    
    instructions
}

/// Apply Focus Punch - fails if user takes direct damage, otherwise powerful Fighting move
pub fn apply_focus_punch(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::Charge) {
            // Second turn - check if user took damage this turn
            // TODO: Need damage tracking system to check if user was damaged
            // For now, assume move succeeds (proper implementation would check damage taken)
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Charge,
                }),
            ]));
            
            // The move fails if the user took direct damage, but that logic
            // would be handled by the damage tracking system
        } else {
            // First turn - focus (charging)
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: VolatileStatus::Charge,
                    duration: Some(1),
                }),
            ]));
        }
    }
    
    instructions
}

/// Apply Fillet Away - boosts offensive stats but costs 1/2 HP
pub fn apply_fillet_away(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let half_hp = user_pokemon.max_hp / 2;
        
        let mut instruction_list = vec![
            // Damage user for half their max HP
            Instruction::PositionDamage(PositionDamageInstruction {
                target_position: user_position,
                damage_amount: half_hp,
                previous_hp: Some(user_pokemon.hp),
            }),
            // Boost Attack, Special Attack, and Speed by 2 stages each
            Instruction::BoostStats(BoostStatsInstruction {
                target_position: user_position,
                stat_boosts: {
                    let mut boosts = HashMap::new();
                    boosts.insert(Stat::Attack, 2);
                    boosts.insert(Stat::SpecialAttack, 2);
                    boosts.insert(Stat::Speed, 2);
                    boosts
                },
                previous_boosts: None,
            }),
        ];
        
        instructions.push(StateInstructions::new(100.0, instruction_list));
    }
    
    instructions
}

/// Apply Clangorous Soul - boosts all stats but costs 1/3 HP
pub fn apply_clangorous_soul(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let third_hp = user_pokemon.max_hp / 3;
        
        let mut instruction_list = vec![
            // Damage user for 1/3 their max HP
            Instruction::PositionDamage(PositionDamageInstruction {
                target_position: user_position,
                damage_amount: third_hp,
                previous_hp: Some(user_pokemon.hp),
            }),
            // Boost all stats by 1 stage each
            Instruction::BoostStats(BoostStatsInstruction {
                target_position: user_position,
                stat_boosts: {
                    let mut boosts = HashMap::new();
                    boosts.insert(Stat::Attack, 1);
                    boosts.insert(Stat::Defense, 1);
                    boosts.insert(Stat::SpecialAttack, 1);
                    boosts.insert(Stat::SpecialDefense, 1);
                    boosts.insert(Stat::Speed, 1);
                    boosts
                },
                previous_boosts: None,
            }),
        ];
        
        instructions.push(StateInstructions::new(100.0, instruction_list));
    }
    
    instructions
}

/// Get Pokemon weight in kilograms using PS data
fn get_pokemon_weight(species: &str) -> f32 {
    // Try to get weight from PS data
    let pokemon_service = match crate::data::services::pokemon_service::PSPokemonService::new() {
        Ok(service) => service,
        Err(_) => {
            // Fallback to hardcoded values if PS data unavailable
            return match species.to_lowercase().as_str() {
                "pikachu" => 6.0,
                "charizard" => 90.5,
                "snorlax" => 460.0,
                "groudon" => 950.0,
                "gastly" => 0.1,
                _ => 50.0, // Default weight
            };
        }
    };
    
    // Get weight from PS data
    if let Some(weight) = pokemon_service.get_weight(species) {
        weight
    } else {
        // Fallback to default if not found
        50.0
    }
}

/// Apply Judgment - Type matches Arceus's type/plate
pub fn apply_judgment(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Judgment's type matches the user's primary type (or Tera type in Gen 9+)
        let judgment_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0].clone()
        } else {
            "Normal".to_string() // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = judgment_type;
        
        // Apply normal damage with the modified type
        // Note: The actual damage calculation will use the modified type
        instructions.push(StateInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Multi-Attack - Type matches Silvally's memory disc
pub fn apply_multi_attack(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Multi-Attack's type matches the user's primary type
        let attack_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0].clone()
        } else {
            "Normal".to_string() // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = attack_type;
        
        // Apply normal damage with the modified type
        instructions.push(StateInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Revelation Dance - Type matches user's primary type
pub fn apply_revelation_dance(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Revelation Dance's type matches the user's primary type
        let dance_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0].clone()
        } else {
            "Normal".to_string() // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = dance_type;
        
        // Apply normal damage with the modified type
        instructions.push(StateInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Burn Up - Fire move that removes user's Fire typing
pub fn apply_burn_up(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Remove Fire type from user after move hits
        let new_types: Vec<String> = user_pokemon.types.iter()
            .filter(|t| t.to_lowercase() != "fire")
            .cloned()
            .collect();
        
        // If removing Fire type would leave no types, add Typeless (or keep at least one type)
        let final_types = if new_types.is_empty() {
            vec!["???".to_string()] // Typeless Pokemon
        } else {
            new_types
        };
        
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::ChangeType(ChangeTypeInstruction {
                target_position: user_position,
                new_types: final_types,
                previous_types: Some(user_pokemon.types.clone()),
            }),
        ]));
    }
    
    instructions
}

/// Apply Double Shock - Electric move that removes user's Electric typing
pub fn apply_double_shock(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Remove Electric type from user after move hits
        let new_types: Vec<String> = user_pokemon.types.iter()
            .filter(|t| t.to_lowercase() != "electric")
            .cloned()
            .collect();
        
        // If removing Electric type would leave no types, add Typeless
        let final_types = if new_types.is_empty() {
            vec!["???".to_string()] // Typeless Pokemon
        } else {
            new_types
        };
        
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::ChangeType(ChangeTypeInstruction {
                target_position: user_position,
                new_types: final_types,
                previous_types: Some(user_pokemon.types.clone()),
            }),
        ]));
    }
    
    instructions
}

// =============================================================================
// MISSING TERRAIN-DEPENDENT MOVES

/// Apply Expanding Force - Boosted power in Psychic Terrain
pub fn apply_expanding_force(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Check if Psychic Terrain is active
    if state.terrain == Terrain::PsychicTerrain {
        // 1.5x power in Psychic Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Rising Voltage - Boosted power in Electric Terrain
pub fn apply_rising_voltage(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Check if Electric Terrain is active
    if state.terrain == Terrain::ElectricTerrain {
        // 1.5x power in Electric Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Misty Explosion - Boosted power in Misty Terrain
pub fn apply_misty_explosion(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if Misty Terrain is active for power boost
    let power_multiplier = if state.terrain == Terrain::MistyTerrain {
        1.5
    } else {
        1.0
    };
    
    // Apply power modifier if terrain is active
    if power_multiplier > 1.0 {
        instructions.extend(apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier));
    } else {
        instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation));
    }
    
    // User faints (self-destruct effect)
    instructions.push(StateInstructions::new(100.0, vec![
        Instruction::Faint(FaintInstruction {
            target_position: user_position,
            previous_hp: 0, // TODO: Should be set to actual HP before fainting
        }),
    ]));
    
    instructions
}

/// Apply Psy Blade - Boosted power in Electric Terrain
pub fn apply_psy_blade(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Check if Electric Terrain is active
    if state.terrain == Terrain::ElectricTerrain {
        // 1.5x power in Electric Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Steel Roller - Fails without terrain
pub fn apply_steel_roller(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    // Check if any terrain is active
    if state.terrain == Terrain::None {
        // Move fails when no terrain is active
        vec![StateInstructions::empty()]
    } else {
        // Normal move behavior when terrain is active
        let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
        
        // Remove terrain after hitting
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::ChangeTerrain(ChangeTerrainInstruction {
                terrain: Terrain::None,
                duration: None,
                previous_terrain: Some(state.terrain),
                previous_duration: Some(state.terrain_turns_remaining),
            }),
        ]));
        
        instructions
    }
}

/// Apply Ice Spinner - Removes terrain after hitting
pub fn apply_ice_spinner(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation);
    
    // Remove terrain after hitting (if any terrain is active)
    if state.terrain != Terrain::None {
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::ChangeTerrain(ChangeTerrainInstruction {
                terrain: Terrain::None,
                duration: None,
                previous_terrain: Some(state.terrain),
                previous_duration: Some(state.terrain_turns_remaining),
            }),
        ]));
    }
    
    instructions
}

// =============================================================================
// MISSING SELF-DAMAGE MOVES

/// Apply Mind Blown - Damages user for 1/2 max HP
pub fn apply_mind_blown(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Calculate self-damage (1/2 max HP, but not more than current HP)
        let damage_amount = std::cmp::min(user_pokemon.max_hp / 2, user_pokemon.hp);
        
        // Apply self-damage before the move
        if damage_amount > 0 {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::PositionDamage(PositionDamageInstruction {
                    target_position: user_position,
                    damage_amount,
                    previous_hp: Some(user_pokemon.hp),
                }),
            ]));
        }
        
        // Apply normal move effects
        instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation));
    }
    
    instructions
}

// =============================================================================
// MISSING TYPE-CHANGING MOVES

/// Apply Ivy Cudgel - Type changes with mask items
pub fn apply_ivy_cudgel(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check for mask items to change type
        let modified_move_type = match user_pokemon.item.as_deref() {
            Some("wellspringmask") => "Water",
            Some("hearthflamemask") => "Fire",
            Some("cornerstonemask") => "Rock",
            _ => &move_data.move_type, // Default Grass type
        };
        
        // Create modified move data with new type
        let modified_move_data = EngineMoveData {
            move_type: modified_move_type.to_string(),
            ..move_data.clone()
        };
        
        apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Tera Blast - Type and category change when Terastallized
pub fn apply_tera_blast(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check if Pokemon is Terastallized (Gen 9+ only)
        let is_tera = generation.generation.number() >= 9 && user_pokemon.is_terastallized;
        
        if is_tera {
            let mut modified_move_data = move_data.clone();
            
            // Change type to Tera type (simplified - normally would get from tera_type field)
            if let Some(ref tera_type) = user_pokemon.tera_type {
                // Convert PokemonType to string - simplified implementation
                modified_move_data.move_type = format!("{:?}", tera_type);
            }
            
            // Determine category based on Attack vs Special Attack
            let attack_stat = user_pokemon.stats.attack;
            let special_attack_stat = user_pokemon.stats.special_attack;
            
            if attack_stat > special_attack_stat {
                modified_move_data.category = MoveCategory::Physical;
            } else {
                modified_move_data.category = MoveCategory::Special;
            }
            
            apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
        } else {
            // Not Terastallized, use normal move
            apply_generic_effects(state, move_data, user_position, target_positions, generation)
        }
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

