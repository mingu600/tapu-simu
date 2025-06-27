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
use crate::types::{BattleResult, Moves};
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

/// Move registry that maps move enums to their effect functions
pub struct MoveRegistry {
    /// Standard move effects
    standard_moves: std::collections::HashMap<Moves, MoveEffectFn>,
    /// Extended move effects that need move data
    extended_moves: std::collections::HashMap<Moves, ExtendedMoveEffectFn>,
    /// Variable power move effects with branching
    variable_power_moves: std::collections::HashMap<Moves, VariablePowerMoveEffectFn>,
    /// Context-aware move effects
    context_aware_moves: std::collections::HashMap<Moves, ContextAwareMoveEffectFn>,
    /// Repository-aware move effects
    repository_aware_moves: std::collections::HashMap<Moves, RepositoryAwareMoveEffectFn>,
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
        self.register_standard(Moves::THUNDERWAVE, apply_thunder_wave);
        self.register_standard(Moves::SLEEPPOWDER, apply_sleep_powder);
        self.register_standard(Moves::TOXIC, apply_toxic);
        self.register_standard(Moves::WILLOWISP, apply_will_o_wisp);
        self.register_standard(Moves::STUNSPORE, apply_stun_spore);
        self.register_standard(Moves::POISONPOWDER, apply_poison_powder);
        self.register_standard(Moves::GLARE, apply_glare);
        self.register_standard(Moves::SPORE, apply_spore);

        // Stat modifying moves
        self.register_standard(Moves::SWORDSDANCE, apply_swords_dance);
        self.register_standard(Moves::DRAGONDANCE, apply_dragon_dance);
        self.register_standard(Moves::NASTYPLOT, apply_nasty_plot);
        self.register_standard(Moves::AGILITY, apply_agility);
        self.register_standard(Moves::GROWL, apply_growl);
        self.register_standard(Moves::LEER, apply_leer);
        self.register_standard(Moves::TAILWHIP, apply_tail_whip);
        self.register_standard(Moves::STRINGSHOT, apply_string_shot);
        self.register_extended(Moves::ACID, apply_acid);
        self.register_standard(Moves::CHARM, apply_charm);
        self.register_standard(Moves::GROWTH, apply_growth);
        self.register_standard(Moves::FILLETAWAY, apply_fillet_away);
        self.register_standard(Moves::CLANGOROUSSOUL, apply_clangorous_soul);

        // Healing moves
        self.register_standard(Moves::RECOVER, apply_recover);
        self.register_standard(Moves::ROOST, apply_roost);
        self.register_standard(Moves::MOONLIGHT, apply_moonlight);
        self.register_standard(Moves::SYNTHESIS, apply_synthesis);
        self.register_standard(Moves::MORNINGSUN, apply_morning_sun);
        self.register_standard(Moves::SOFTBOILED, apply_soft_boiled);
        self.register_standard(Moves::WISH, apply_wish);
        self.register_standard(Moves::HEALBELL, apply_heal_bell);
        self.register_standard(Moves::AROMATHERAPY, apply_aromatherapy);
        self.register_standard(Moves::REST, apply_rest);

        // Weather moves
        self.register_standard(Moves::SUNNYDAY, apply_sunny_day);
        self.register_standard(Moves::RAINDANCE, apply_rain_dance);
        self.register_standard(Moves::SANDSTORM, apply_sandstorm);
        self.register_standard(Moves::HAIL, apply_hail);

        // Screen moves
        self.register_standard(Moves::REFLECT, apply_reflect);
        self.register_standard(Moves::LIGHTSCREEN, apply_light_screen);
        self.register_standard(Moves::AURORAVEIL, apply_aurora_veil);

        // Hazard moves
        self.register_standard(Moves::SPIKES, apply_spikes);
        self.register_standard(Moves::TOXICSPIKES, apply_toxic_spikes);
        self.register_standard(Moves::STEALTHROCK, apply_stealth_rock);
        self.register_standard(Moves::STICKYWEB, apply_sticky_web);

        // Hazard removal moves
        self.register_standard(Moves::RAPIDSPIN, apply_rapid_spin);
        self.register_standard(Moves::DEFOG, apply_defog);

        // Multi-hit moves (use variable power registration for extended signature)
        self.register_variable_power(Moves::SURGINGSTRIKES, multi_hit::apply_surging_strikes);
        self.register_variable_power(Moves::DRAGONDARTS, multi_hit::apply_dragon_darts);
        self.register_variable_power(Moves::POPULATIONBOMB, multi_hit::apply_population_bomb);
        self.register_variable_power(Moves::SCALESHOT, multi_hit::apply_scale_shot);
        
        // Generic multi-hit moves that use apply_multi_hit_move
        self.register_variable_power(Moves::DOUBLESLAP, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::FURYATTACK, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::PINMISSILE, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::SPIKECANNON, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::BARRAGE, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::COMETPUNCH, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::BULLETSEED, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::ROCKBLAST, multi_hit::apply_multi_hit_move);
        self.register_variable_power(Moves::TAILSLAP, multi_hit::apply_multi_hit_move);

        // Recoil moves - now handled automatically via PS data

        // Drain moves - now handled automatically via PS data

        // Protection moves
        self.register_standard(Moves::PROTECT, apply_protect);
        self.register_standard(Moves::DETECT, apply_detect);
        self.register_standard(Moves::ENDURE, apply_endure);

        // Variable power moves (with branching support)
        self.register_variable_power(Moves::FACADE, variable_power::apply_facade);
        self.register_variable_power(Moves::HEX, variable_power::apply_hex);
        self.register_variable_power(Moves::GYROBALL, variable_power::apply_gyro_ball);
        self.register_variable_power(Moves::REVERSAL, variable_power::apply_reversal);
        self.register_variable_power(Moves::ACROBATICS, variable_power::apply_acrobatics);
        self.register_variable_power(Moves::WEATHERBALL, variable_power::apply_weather_ball);
        self.register_variable_power(Moves::AVALANCHE, variable_power::apply_avalanche);
        self.register_variable_power(Moves::ELECTROBALL, variable_power::apply_electroball);
        self.register_variable_power(Moves::ERUPTION, variable_power::apply_eruption);
        self.register_variable_power(Moves::WATERSPOUT, variable_power::apply_waterspout);
        self.register_variable_power(Moves::PUNISHMENT, variable_power::apply_punishment);
        self.register_variable_power(Moves::WAKEUPSLAP, variable_power::apply_wakeup_slap);
        self.register_variable_power(Moves::DRAGONENERGY, variable_power::apply_dragon_energy);
        self.register_variable_power(Moves::GRASSKNOT, variable_power::apply_grass_knot);
        self.register_variable_power(Moves::LOWKICK, variable_power::apply_low_kick);
        self.register_variable_power(Moves::HEATCRASH, variable_power::apply_heat_crash);
        self.register_variable_power(Moves::HEAVYSLAM, variable_power::apply_heavy_slam);
        self.register_variable_power(Moves::BARBBARRAGE, variable_power::apply_barb_barrage);
        self.register_variable_power(Moves::COLLISIONCOURSE, variable_power::apply_collision_course);
        self.register_variable_power(Moves::ELECTRODRIFT, variable_power::apply_electro_drift);
        self.register_variable_power(Moves::FREEZEDRY, variable_power::apply_freeze_dry);
        self.register_variable_power(Moves::HARDPRESS, variable_power::apply_hard_press);
        self.register_variable_power(Moves::HYDROSTEAM, variable_power::apply_hydro_steam);
        self.register_variable_power(Moves::LASTRESPECTS, variable_power::apply_last_respects);
        self.register_variable_power(Moves::POLTERGEIST, variable_power::apply_poltergeist);
        self.register_variable_power(Moves::STOREDPOWER, variable_power::apply_stored_power);
        self.register_variable_power(Moves::POWERTRIP, variable_power::apply_power_trip);
        self.register_variable_power(Moves::TERRAINPULSE, variable_power::apply_terrain_pulse);

        // Context-aware moves (need context parameter)
        self.register_context_aware(Moves::BOLTBEAK, variable_power::apply_boltbeak);
        self.register_context_aware(Moves::FISHIOUSREND, variable_power::apply_fishious_rend);
        self.register_context_aware(Moves::PURSUIT, variable_power::apply_pursuit);
        self.register_context_aware(Moves::SUCKERPUNCH, variable_power::apply_sucker_punch);
        self.register_context_aware(Moves::THUNDERCLAP, variable_power::apply_thunder_clap);
        self.register_context_aware(Moves::UPPERHAND, variable_power::apply_upper_hand);

        // Repository-aware moves (need repository parameter)
        self.register_repository_aware(Moves::MEFIRST, variable_power::apply_me_first);

        // Fixed damage moves (use extended registration for move data)
        self.register_standard(Moves::SEISMICTOSS, fixed_damage::apply_seismic_toss);
        self.register_standard(Moves::NIGHTSHADE, fixed_damage::apply_night_shade);
        self.register_standard(Moves::ENDEAVOR, fixed_damage::apply_endeavor);
        self.register_standard(Moves::FINALGAMBIT, fixed_damage::apply_final_gambit);
        self.register_standard(Moves::NATURESMADNESS, fixed_damage::apply_natures_madness);
        self.register_standard(Moves::RUINATION, fixed_damage::apply_ruination);
        self.register_standard(Moves::SUPERFANG, fixed_damage::apply_super_fang);

        // Self-targeting moves (both self-destruct and self-damage)
        self.register_variable_power(Moves::EXPLOSION, self_targeting::apply_explosion);
        self.register_variable_power(Moves::SELFDESTRUCT, self_targeting::apply_self_destruct);
        self.register_variable_power(Moves::MINDBLOWN, self_targeting::apply_mind_blown);

        // Special combat moves
        self.register_variable_power(Moves::BODYPRESS, apply_body_press);
        self.register_variable_power(Moves::FOULPLAY, apply_foul_play);
        self.register_variable_power(Moves::PHOTONGEYSER, apply_photon_geyser);
        self.register_variable_power(Moves::SKYDROP, apply_sky_drop);


        // Advanced hazards (mortalspin not implemented yet)

        // Secondary effects moves  
        self.register_extended(Moves::FLAMETHROWER, apply_flamethrower);
        self.register_extended(Moves::FIREBLAST, apply_fire_blast);
        self.register_extended(Moves::THUNDERBOLT, apply_thunderbolt);
        self.register_extended(Moves::ICEBEAM, apply_ice_beam);
        self.register_extended(Moves::SLUDGEBOMB, apply_sludge_bomb);
        self.register_extended(Moves::AIRSLASH, apply_air_slash);
        self.register_extended(Moves::IRONHEAD, apply_iron_head);
        self.register_extended(Moves::ROCKSLIDE, apply_rock_slide);

        // Counter moves
        self.register_standard(Moves::COUNTER, counter::apply_counter);
        self.register_standard(Moves::MIRRORCOAT, counter::apply_mirror_coat);
        self.register_standard(Moves::COMEUPPANCE, counter::apply_comeuppance);
        self.register_standard(Moves::METALBURST, counter::apply_metal_burst);

        // Additional healing moves  
        self.register_standard(Moves::PAINSPLIT, healing::apply_pain_split);

        // Strength Sap (special variable power move without branching)
        self.register_extended(Moves::STRENGTHSAP, variable_power::apply_strength_sap);
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
    /// registry.register_standard(Moves::THUNDERWAVE, apply_thunder_wave);
    /// registry.register_standard(Moves::SWORDSDANCE, apply_swords_dance);
    /// ```
    fn register_standard(&mut self, move_enum: Moves, effect_fn: MoveEffectFn) {
        self.standard_moves.insert(move_enum, effect_fn);
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
    /// registry.register_extended(Moves::FLAMETHROWER, apply_flamethrower);
    /// registry.register_extended(Moves::ACID, apply_acid); // Needs move data for stat reduction chance
    /// ```
    fn register_extended(&mut self, move_enum: Moves, effect_fn: ExtendedMoveEffectFn) {
        self.extended_moves.insert(move_enum, effect_fn);
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
    /// registry.register_variable_power(Moves::FACADE, apply_facade); // Double power if user has status
    /// registry.register_variable_power(Moves::DRAGONDARTS, apply_dragon_darts); // Multi-hit targeting
    /// ```
    fn register_variable_power(&mut self, move_enum: Moves, effect_fn: VariablePowerMoveEffectFn) {
        self.variable_power_moves.insert(move_enum, effect_fn);
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
    /// registry.register_context_aware(Moves::BOLTBEAK, apply_boltbeak); // Double power if user moves first
    /// registry.register_context_aware(Moves::SUCKERPUNCH, apply_sucker_punch); // Only works if opponent attacks
    /// ```
    fn register_context_aware(&mut self, move_enum: Moves, effect_fn: ContextAwareMoveEffectFn) {
        self.context_aware_moves.insert(move_enum, effect_fn);
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
    /// registry.register_repository_aware(Moves::MEFIRST, apply_me_first); // Needs to copy opponent's move
    /// registry.register_repository_aware(Moves::METRONOME, apply_metronome); // Needs random move lookup
    /// ```
    fn register_repository_aware(&mut self, move_enum: Moves, effect_fn: RepositoryAwareMoveEffectFn) {
        self.repository_aware_moves.insert(move_enum, effect_fn);
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
        let move_enum = move_data.name;
        
        // Try repository-aware moves first (most complex)
        if let Some(effect_fn) = self.repository_aware_moves.get(&move_enum) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, context, repository, branch_on_damage));
        }
        
        // Try context-aware moves
        if let Some(effect_fn) = self.context_aware_moves.get(&move_enum) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, context, branch_on_damage));
        }
        
        // Try variable power moves
        if let Some(effect_fn) = self.variable_power_moves.get(&move_enum) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation, branch_on_damage));
        }
        
        // Try extended moves
        if let Some(effect_fn) = self.extended_moves.get(&move_enum) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation));
        }
        
        // Try standard moves
        if let Some(effect_fn) = self.standard_moves.get(&move_enum) {
            return Ok(effect_fn(state, user_position, target_positions, generation));
        }
        
        // Move not found in any registry
        Err(crate::types::BattleError::InvalidMoveChoice { 
            reason: format!("Move '{:?}' is not registered in the move registry", move_data.name) 
        })
    }

    /// Check if a move is registered
    pub fn is_move_registered(&self, move_enum: &Moves) -> bool {
        self.standard_moves.contains_key(move_enum) 
            || self.extended_moves.contains_key(move_enum)
            || self.variable_power_moves.contains_key(move_enum)
            || self.context_aware_moves.contains_key(move_enum)
            || self.repository_aware_moves.contains_key(move_enum)
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
        assert!(registry.is_move_registered(&Moves::THUNDERWAVE));
        assert!(registry.is_move_registered(&Moves::SWORDSDANCE));
        assert!(registry.is_move_registered(&Moves::PROTECT));
        
        // Test non-existent move
        assert!(!registry.is_move_registered(&Moves::NONE));
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