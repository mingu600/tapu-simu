//! # Move Registry System
//!
//! This module provides the centralized move dispatcher that replaces the large
//! match statement in mod.rs. It organizes move effects by category and provides
//! a clean interface for move effect application.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::{BattleState, Pokemon, Move, Gender, MoveCategory};
use crate::core::instructions::BattleInstructions;
use crate::data::{GameDataRepository, showdown_types::MoveData};
use crate::generation::GenerationMechanics;
use crate::types::BattleResult;
use crate::utils::normalize_name;
use super::MoveContext;

// Import specific functions from each category module
use super::status::status_effects::{
    apply_thunder_wave, apply_sleep_powder, apply_toxic, apply_will_o_wisp,
    apply_stun_spore, apply_poison_powder, apply_glare, apply_spore
};

use super::status::stat_modifying::{
    apply_swords_dance, apply_dragon_dance, apply_nasty_plot, apply_agility,
    apply_growl, apply_leer, apply_tail_whip, apply_string_shot, apply_acid,
    apply_charm, apply_growth, apply_fillet_away, apply_clangorous_soul
};

use super::status::healing::{
    apply_recover, apply_roost, apply_moonlight, apply_synthesis,
    apply_morning_sun, apply_soft_boiled, apply_wish, apply_heal_bell,
    apply_aromatherapy, apply_rest
};

use super::field::weather::{
    apply_sunny_day, apply_rain_dance, apply_sandstorm, apply_hail
};

use super::field::screens::{
    apply_reflect, apply_light_screen, apply_aurora_veil
};

use super::field::hazards::{
    apply_spikes, apply_toxic_spikes, apply_stealth_rock, apply_sticky_web
};

use super::field::hazard_removal::{
    apply_rapid_spin, apply_defog
};

use super::damage::multi_hit::{
    apply_double_slap, apply_fury_attack, apply_pin_missile, apply_spike_cannon,
    apply_barrage, apply_comet_punch, apply_bullet_seed,
    apply_rock_blast, apply_tail_slap, apply_scale_shot
};



use super::special::protection::{
    apply_protect, apply_detect, apply_endure
};

// Additional imports for complex moves from the original match statement
use super::damage::variable_power;
use super::damage::{fixed_damage, self_targeting, multi_hit};
use super::special::{complex, counter};
use super::special_combat::{
    apply_body_press, apply_foul_play, apply_photon_geyser, apply_sky_drop
};
use super::secondary_effects::{
    apply_flamethrower, apply_fire_blast, apply_thunderbolt, apply_ice_beam,
    apply_sludge_bomb, apply_air_slash, apply_iron_head, apply_rock_slide
};
use super::status::healing;

/// Move effect dispatcher function type
type MoveEffectFn = fn(
    &BattleState,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
) -> Vec<BattleInstructions>;

/// Extended move effect dispatcher function type (with move data)
type ExtendedMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
) -> Vec<BattleInstructions>;

/// Variable power move effect dispatcher function type (with branching)
type VariablePowerMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
    bool, // branch_on_damage
) -> Vec<BattleInstructions>;

/// Context-aware move effect dispatcher function type
type ContextAwareMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
    &MoveContext,
    bool, // branch_on_damage
) -> Vec<BattleInstructions>;

/// Repository-aware move effect dispatcher function type
type RepositoryAwareMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
    &MoveContext,
    &GameDataRepository,
    bool, // branch_on_damage
) -> Vec<BattleInstructions>;

/// Move registry that maps move names to their effect functions
pub struct MoveRegistry {
    /// Standard move effects
    standard_moves: std::collections::HashMap<String, MoveEffectFn>,
    /// Extended move effects that need move data
    extended_moves: std::collections::HashMap<String, ExtendedMoveEffectFn>,
    /// Variable power move effects with branching
    variable_power_moves: std::collections::HashMap<String, VariablePowerMoveEffectFn>,
    /// Context-aware move effects
    context_aware_moves: std::collections::HashMap<String, ContextAwareMoveEffectFn>,
    /// Repository-aware move effects
    repository_aware_moves: std::collections::HashMap<String, RepositoryAwareMoveEffectFn>,
}

impl MoveRegistry {
    /// Create a new move registry with all move effects registered
    pub fn new() -> Self {
        let mut registry = Self {
            // Pre-allocate capacity based on expected move counts
            standard_moves: std::collections::HashMap::with_capacity(64),
            extended_moves: std::collections::HashMap::with_capacity(32),
            variable_power_moves: std::collections::HashMap::with_capacity(48),
            context_aware_moves: std::collections::HashMap::with_capacity(16),
            repository_aware_moves: std::collections::HashMap::with_capacity(8),
        };
        
        registry.register_all_moves();
        registry
    }

    /// Register all move effects in the registry
    fn register_all_moves(&mut self) {
        // Status effect moves
        self.register_standard("thunderwave", apply_thunder_wave);
        self.register_standard("sleeppowder", apply_sleep_powder);
        self.register_standard("toxic", apply_toxic);
        self.register_standard("willowisp", apply_will_o_wisp);
        self.register_standard("stunspore", apply_stun_spore);
        self.register_standard("poisonpowder", apply_poison_powder);
        self.register_standard("glare", apply_glare);
        self.register_standard("spore", apply_spore);

        // Stat modifying moves
        self.register_standard("swordsdance", apply_swords_dance);
        self.register_standard("dragondance", apply_dragon_dance);
        self.register_standard("nastyplot", apply_nasty_plot);
        self.register_standard("agility", apply_agility);
        self.register_standard("growl", apply_growl);
        self.register_standard("leer", apply_leer);
        self.register_standard("tailwhip", apply_tail_whip);
        self.register_standard("stringshot", apply_string_shot);
        self.register_extended("acid", apply_acid);
        self.register_standard("charm", apply_charm);
        self.register_standard("growth", apply_growth);
        self.register_standard("filletaway", apply_fillet_away);
        self.register_standard("clangoroussoul", apply_clangorous_soul);

        // Healing moves
        self.register_standard("recover", apply_recover);
        self.register_standard("roost", apply_roost);
        self.register_standard("moonlight", apply_moonlight);
        self.register_standard("synthesis", apply_synthesis);
        self.register_standard("morningsun", apply_morning_sun);
        self.register_standard("softboiled", apply_soft_boiled);
        self.register_standard("wish", apply_wish);
        self.register_standard("healbell", apply_heal_bell);
        self.register_standard("aromatherapy", apply_aromatherapy);
        self.register_standard("rest", apply_rest);

        // Weather moves
        self.register_standard("sunnyday", apply_sunny_day);
        self.register_standard("raindance", apply_rain_dance);
        self.register_standard("sandstorm", apply_sandstorm);
        self.register_standard("hail", apply_hail);

        // Screen moves
        self.register_standard("reflect", apply_reflect);
        self.register_standard("lightscreen", apply_light_screen);
        self.register_standard("auroraveil", apply_aurora_veil);

        // Hazard moves
        self.register_standard("spikes", apply_spikes);
        self.register_standard("toxicspikes", apply_toxic_spikes);
        self.register_standard("stealthrock", apply_stealth_rock);
        self.register_standard("stickyweb", apply_sticky_web);

        // Hazard removal moves
        self.register_standard("rapidspin", apply_rapid_spin);
        self.register_standard("defog", apply_defog);

        // Multi-hit moves (use variable power registration for extended signature)
        self.register_variable_power("surgingstrikes", multi_hit::apply_surging_strikes);
        self.register_variable_power("dragondarts", multi_hit::apply_dragon_darts);
        self.register_variable_power("populationbomb", multi_hit::apply_population_bomb);
        self.register_variable_power("scaleshot", multi_hit::apply_scale_shot);
        
        // Generic multi-hit moves that use apply_multi_hit_move
        self.register_variable_power("doubleslap", multi_hit::apply_multi_hit_move);
        self.register_variable_power("furyattack", multi_hit::apply_multi_hit_move);
        self.register_variable_power("pinmissile", multi_hit::apply_multi_hit_move);
        self.register_variable_power("spikecannon", multi_hit::apply_multi_hit_move);
        self.register_variable_power("barrage", multi_hit::apply_multi_hit_move);
        self.register_variable_power("cometpunch", multi_hit::apply_multi_hit_move);
        self.register_variable_power("bulletseed", multi_hit::apply_multi_hit_move);
        self.register_variable_power("rockblast", multi_hit::apply_multi_hit_move);
        self.register_variable_power("tailslap", multi_hit::apply_multi_hit_move);

        // Recoil moves - now handled automatically via PS data

        // Drain moves - now handled automatically via PS data

        // Protection moves
        self.register_standard("protect", apply_protect);
        self.register_standard("detect", apply_detect);
        self.register_standard("endure", apply_endure);

        // Variable power moves (with branching support)
        self.register_variable_power("facade", variable_power::apply_facade);
        self.register_variable_power("hex", variable_power::apply_hex);
        self.register_variable_power("gyroball", variable_power::apply_gyro_ball);
        self.register_variable_power("reversal", variable_power::apply_reversal);
        self.register_variable_power("acrobatics", variable_power::apply_acrobatics);
        self.register_variable_power("weatherball", variable_power::apply_weather_ball);
        self.register_variable_power("avalanche", variable_power::apply_avalanche);
        self.register_variable_power("electroball", variable_power::apply_electroball);
        self.register_variable_power("eruption", variable_power::apply_eruption);
        self.register_variable_power("waterspout", variable_power::apply_waterspout);
        self.register_variable_power("punishment", variable_power::apply_punishment);
        self.register_variable_power("wakeupslap", variable_power::apply_wakeup_slap);
        self.register_variable_power("dragonenergy", variable_power::apply_dragon_energy);
        self.register_variable_power("grassknot", variable_power::apply_grass_knot);
        self.register_variable_power("lowkick", variable_power::apply_low_kick);
        self.register_variable_power("heatcrash", variable_power::apply_heat_crash);
        self.register_variable_power("heavyslam", variable_power::apply_heavy_slam);
        self.register_variable_power("barbbarrage", variable_power::apply_barb_barrage);
        self.register_variable_power("collisioncourse", variable_power::apply_collision_course);
        self.register_variable_power("electrodrift", variable_power::apply_electro_drift);
        self.register_variable_power("freezedry", variable_power::apply_freeze_dry);
        self.register_variable_power("hardpress", variable_power::apply_hard_press);
        self.register_variable_power("hydrosteam", variable_power::apply_hydro_steam);
        self.register_variable_power("lastrespects", variable_power::apply_last_respects);
        self.register_variable_power("poltergeist", variable_power::apply_poltergeist);
        self.register_variable_power("storedpower", variable_power::apply_stored_power);
        self.register_variable_power("powertrip", variable_power::apply_power_trip);
        self.register_variable_power("terrainpulse", variable_power::apply_terrain_pulse);

        // Context-aware moves (need context parameter)
        self.register_context_aware("boltbeak", variable_power::apply_boltbeak);
        self.register_context_aware("fishiousrend", variable_power::apply_fishious_rend);
        self.register_context_aware("pursuit", variable_power::apply_pursuit);
        self.register_context_aware("suckerpunch", variable_power::apply_sucker_punch);
        self.register_context_aware("thunderclap", variable_power::apply_thunder_clap);
        self.register_context_aware("upperhand", variable_power::apply_upper_hand);

        // Repository-aware moves (need repository parameter)
        self.register_repository_aware("mefirst", variable_power::apply_me_first);

        // Fixed damage moves (use extended registration for move data)
        self.register_standard("seismictoss", fixed_damage::apply_seismic_toss);
        self.register_standard("nightshade", fixed_damage::apply_night_shade);
        self.register_standard("endeavor", fixed_damage::apply_endeavor);
        self.register_standard("finalgambit", fixed_damage::apply_final_gambit);
        self.register_standard("naturesmadness", fixed_damage::apply_natures_madness);
        self.register_standard("ruination", fixed_damage::apply_ruination);
        self.register_standard("superfang", fixed_damage::apply_super_fang);

        // Self-targeting moves (both self-destruct and self-damage)
        self.register_variable_power("explosion", self_targeting::apply_explosion);
        self.register_variable_power("selfdestruct", self_targeting::apply_self_destruct);
        self.register_variable_power("mindblown", self_targeting::apply_mind_blown);

        // Special combat moves
        self.register_variable_power("bodypress", apply_body_press);
        self.register_variable_power("foulplay", apply_foul_play);
        self.register_variable_power("photongeyser", apply_photon_geyser);
        self.register_variable_power("skydrop", apply_sky_drop);


        // Advanced hazards (mortalspin not implemented yet)

        // Secondary effects moves  
        self.register_extended("flamethrower", apply_flamethrower);
        self.register_extended("fireblast", apply_fire_blast);
        self.register_extended("thunderbolt", apply_thunderbolt);
        self.register_extended("icebeam", apply_ice_beam);
        self.register_extended("sludgebomb", apply_sludge_bomb);
        self.register_extended("airslash", apply_air_slash);
        self.register_extended("ironhead", apply_iron_head);
        self.register_extended("rockslide", apply_rock_slide);

        // Counter moves
        self.register_standard("counter", counter::apply_counter);
        self.register_standard("mirrorcoat", counter::apply_mirror_coat);
        self.register_standard("comeuppance", counter::apply_comeuppance);
        self.register_standard("metalburst", counter::apply_metal_burst);

        // Additional healing moves  
        self.register_standard("painsplit", healing::apply_pain_split);

        // Strength Sap (special variable power move without branching)
        self.register_extended("strengthsap", variable_power::apply_strength_sap);
    }

    /// Register a standard move effect
    ///
    /// Use this for simple moves that don't need move data, context, or repository access.
    /// These moves work with only battle state and basic parameters.
    ///
    /// ## When to use:
    /// - Status effect moves (Thunder Wave, Sleep Powder, Toxic)
    /// - Self-targeting stat moves (Swords Dance, Agility, Growl)
    /// - Simple healing moves (Recover, Roost)
    /// - Basic utility moves (Protect, Substitute)
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(
    ///     state: &BattleState,
    ///     user: BattlePosition,
    ///     targets: &[BattlePosition],
    ///     generation: &GenerationMechanics,
    /// ) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register_standard("thunderwave", apply_thunder_wave);
    /// registry.register_standard("swordsdance", apply_swords_dance);
    /// ```
    fn register_standard(&mut self, name: &str, effect_fn: MoveEffectFn) {
        self.standard_moves.insert(name.to_string(), effect_fn);
    }

    /// Register an extended move effect that needs move data
    ///
    /// Use this for moves that need access to MoveData for power, accuracy, or type calculations.
    /// Most damage-dealing moves fall into this category.
    ///
    /// ## When to use:
    /// - Damage moves that use standard power calculations (Flamethrower, Thunderbolt)
    /// - Moves that check accuracy or other MoveData fields
    /// - Moves that need move type information
    /// - Status moves that depend on move data (Acid with stat reduction chance)
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(
    ///     state: &BattleState,
    ///     move_data: &MoveData,
    ///     user: BattlePosition,
    ///     targets: &[BattlePosition],
    ///     generation: &GenerationMechanics,
    /// ) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register_extended("flamethrower", apply_flamethrower);
    /// registry.register_extended("acid", apply_acid); // Needs move data for stat reduction chance
    /// ```
    fn register_extended(&mut self, name: &str, effect_fn: ExtendedMoveEffectFn) {
        self.extended_moves.insert(name.to_string(), effect_fn);
    }

    /// Register a variable power move effect
    ///
    /// Use this for moves with dynamic power calculations that may need damage branching.
    /// These moves modify their power based on battle conditions, Pokemon stats, or other factors.
    ///
    /// ## When to use:
    /// - Moves with conditional power (Facade, Hex, Wake-Up Slap)
    /// - Stat-based power moves (Gyro Ball, Electro Ball)
    /// - Multi-hit moves with variable hit counts (Dragon Darts, Population Bomb)
    /// - Moves that calculate power based on user/target conditions
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(
    ///     state: &BattleState,
    ///     move_data: &MoveData,
    ///     user: BattlePosition,
    ///     targets: &[BattlePosition],
    ///     generation: &GenerationMechanics,
    ///     branch_on_damage: bool,
    /// ) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register_variable_power("facade", apply_facade); // Double power if user has status
    /// registry.register_variable_power("dragondarts", apply_dragon_darts); // Multi-hit targeting
    /// ```
    fn register_variable_power(&mut self, name: &str, effect_fn: VariablePowerMoveEffectFn) {
        self.variable_power_moves.insert(name.to_string(), effect_fn);
    }

    /// Register a context-aware move effect
    ///
    /// Use this for moves that need information about opponent moves, turn order, or battle context.
    /// These moves make decisions based on what the opponent is doing this turn.
    ///
    /// ## When to use:
    /// - Priority-dependent moves (Bolt Beak, Fishious Rend)
    /// - Opponent move-dependent moves (Sucker Punch, Me First)
    /// - Switch-punishing moves (Pursuit, U-turn effects)
    /// - Moves that need turn order information
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(
    ///     state: &BattleState,
    ///     move_data: &MoveData,
    ///     user: BattlePosition,
    ///     targets: &[BattlePosition],
    ///     generation: &GenerationMechanics,
    ///     context: &MoveContext,
    ///     branch_on_damage: bool,
    /// ) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register_context_aware("boltbeak", apply_boltbeak); // Double power if user moves first
    /// registry.register_context_aware("suckerpunch", apply_sucker_punch); // Only works if opponent attacks
    /// ```
    fn register_context_aware(&mut self, name: &str, effect_fn: ContextAwareMoveEffectFn) {
        self.context_aware_moves.insert(name.to_string(), effect_fn);
    }

    /// Register a repository-aware move effect
    ///
    /// Use this for moves that need access to the game data repository to look up other moves,
    /// abilities, items, or Pokemon data. This is the most complex registration type.
    ///
    /// ## When to use:
    /// - Moves that copy other moves (Me First, Copycat, Mirror Move)
    /// - Moves that need to look up move data (Metronome, Sleep Talk)
    /// - Moves that reference external game data
    /// - Moves that need comprehensive database access
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(
    ///     state: &BattleState,
    ///     move_data: &MoveData,
    ///     user: BattlePosition,
    ///     targets: &[BattlePosition],
    ///     generation: &GenerationMechanics,
    ///     context: &MoveContext,
    ///     repository: &GameDataRepository,
    ///     branch_on_damage: bool,
    /// ) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register_repository_aware("mefirst", apply_me_first); // Needs to copy opponent's move
    /// registry.register_repository_aware("metronome", apply_metronome); // Needs random move lookup
    /// ```
    fn register_repository_aware(&mut self, name: &str, effect_fn: RepositoryAwareMoveEffectFn) {
        self.repository_aware_moves.insert(name.to_string(), effect_fn);
    }

    /// Apply move effects using the registry
    pub fn apply_move_effects(
        &self,
        state: &BattleState,
        move_data: &MoveData,
        user_position: BattlePosition,
        target_positions: &[BattlePosition],
        generation: &GenerationMechanics,
        context: &MoveContext,
        repository: &GameDataRepository,
        branch_on_damage: bool,
    ) -> BattleResult<Vec<BattleInstructions>> {
        let move_name = normalize_name(&move_data.name);
        
        // Try repository-aware moves first (most complex)
        if let Some(effect_fn) = self.repository_aware_moves.get(&move_name) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, context, repository, branch_on_damage));
        }
        
        // Try context-aware moves
        if let Some(effect_fn) = self.context_aware_moves.get(&move_name) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, context, branch_on_damage));
        }
        
        // Try variable power moves
        if let Some(effect_fn) = self.variable_power_moves.get(&move_name) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, branch_on_damage));
        }
        
        // Try extended moves
        if let Some(effect_fn) = self.extended_moves.get(&move_name) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation));
        }
        
        // Try standard moves
        if let Some(effect_fn) = self.standard_moves.get(&move_name) {
            return Ok(effect_fn(state, user_position, target_positions, generation));
        }
        
        // Move not found in any registry
        Err(crate::types::BattleError::InvalidMoveChoice { 
            reason: format!("Move '{}' is not registered in the move registry", move_data.name) 
        })
    }

    /// Check if a move is registered
    pub fn is_move_registered(&self, move_name: &str) -> bool {
        let normalized = normalize_name(move_name);
        self.standard_moves.contains_key(&normalized) 
            || self.extended_moves.contains_key(&normalized)
            || self.variable_power_moves.contains_key(&normalized)
            || self.context_aware_moves.contains_key(&normalized)
            || self.repository_aware_moves.contains_key(&normalized)
    }

    /// Get the number of registered moves
    pub fn registered_move_count(&self) -> usize {
        self.standard_moves.len() 
            + self.extended_moves.len()
            + self.variable_power_moves.len()
            + self.context_aware_moves.len()
            + self.repository_aware_moves.len()
    }
}

impl Default for MoveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global registry instance (lazily initialized)
use std::sync::OnceLock;
static GLOBAL_REGISTRY: OnceLock<MoveRegistry> = OnceLock::new();

/// Get the global move registry
pub fn get_move_registry() -> &'static MoveRegistry {
    GLOBAL_REGISTRY.get_or_init(|| MoveRegistry::new())
}

/// Main move effect dispatcher - replacement for the large match statement
pub fn apply_move_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
    repository: &GameDataRepository,
    branch_on_damage: bool,
) -> BattleResult<Vec<BattleInstructions>> {
    get_move_registry().apply_move_effects(
        state, move_data, user_position, target_positions,
        generation, context, repository, branch_on_damage
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = MoveRegistry::new();
        assert!(registry.registered_move_count() > 0);
    }

    #[test]
    fn test_move_registration() {
        let registry = MoveRegistry::new();
        
        // Test some common moves
        assert!(registry.is_move_registered("thunderwave"));
        assert!(registry.is_move_registered("Thunder Wave")); // Should work with spaces
        assert!(registry.is_move_registered("swordsdance"));
        assert!(registry.is_move_registered("protect"));
        
        // Test non-existent move
        assert!(!registry.is_move_registered("nonexistentmove"));
    }

    #[test]
    fn test_global_registry() {
        let registry1 = get_move_registry();
        let registry2 = get_move_registry();
        
        // Should be the same instance
        assert!(std::ptr::eq(registry1, registry2));
        assert!(registry1.registered_move_count() > 0);
    }
}