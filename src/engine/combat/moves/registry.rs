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
use crate::engine::combat::move_context::{MoveExecutionContext, MoveEffectFn, adapt_simple_move, adapt_extended_move, adapt_variable_power_move, adapt_context_aware_move, adapt_repository_aware_move};

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

/// Legacy move effect dispatcher function types (for backward compatibility during transition)
type LegacyMoveEffectFn = fn(
    &BattleState,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
) -> Vec<BattleInstructions>;

type LegacyExtendedMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
) -> Vec<BattleInstructions>;

type LegacyVariablePowerMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
    bool, // branch_on_damage
) -> Vec<BattleInstructions>;

type LegacyContextAwareMoveEffectFn = fn(
    &BattleState,
    &MoveData,
    BattlePosition,
    &[BattlePosition],
    &GenerationMechanics,
    &MoveContext,
    bool, // branch_on_damage
) -> Vec<BattleInstructions>;

type LegacyRepositoryAwareMoveEffectFn = fn(
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
    /// Unified move effects using the new context system
    effects: std::collections::HashMap<Moves, MoveEffectFn>,
}

impl MoveRegistry {
    /// Create a new move registry with all move effects registered
    pub fn new() -> Self {
        let mut registry = Self {
            // Pre-allocate capacity based on expected total move count
            effects: std::collections::HashMap::with_capacity(200),
        };
        
        registry.register_all_moves();
        registry
    }

    /// Register all move effects in the registry
    fn register_all_moves(&mut self) {
        // Status effect moves (some using unified signatures)
        self.register(Moves::THUNDERWAVE, Box::new(crate::engine::combat::moves::status::status_effects::apply_thunder_wave_unified));
        self.register(Moves::SLEEPPOWDER, Box::new(crate::engine::combat::moves::status::status_effects::apply_sleep_powder_unified));
        self.register(Moves::TOXIC, adapt_simple_move(apply_toxic));
        self.register(Moves::WILLOWISP, adapt_simple_move(apply_will_o_wisp));
        self.register(Moves::STUNSPORE, adapt_simple_move(apply_stun_spore));
        self.register(Moves::POISONPOWDER, adapt_simple_move(apply_poison_powder));
        self.register(Moves::GLARE, adapt_simple_move(apply_glare));
        self.register(Moves::SPORE, adapt_simple_move(apply_spore));

        // Stat modifying moves (some using unified signatures)
        self.register(Moves::SWORDSDANCE, Box::new(crate::engine::combat::moves::status::stat_modifying::apply_swords_dance_unified));
        self.register(Moves::DRAGONDANCE, Box::new(crate::engine::combat::moves::status::stat_modifying::apply_dragon_dance_unified));
        self.register(Moves::NASTYPLOT, adapt_simple_move(apply_nasty_plot));
        self.register(Moves::AGILITY, adapt_simple_move(apply_agility));
        self.register(Moves::GROWL, adapt_simple_move(apply_growl));
        self.register(Moves::LEER, adapt_simple_move(apply_leer));
        self.register(Moves::TAILWHIP, adapt_simple_move(apply_tail_whip));
        self.register(Moves::STRINGSHOT, adapt_simple_move(apply_string_shot));
        self.register(Moves::ACID, adapt_extended_move(apply_acid));
        self.register(Moves::CHARM, adapt_simple_move(apply_charm));
        self.register(Moves::GROWTH, adapt_simple_move(apply_growth));
        self.register(Moves::FILLETAWAY, adapt_simple_move(apply_fillet_away));
        self.register(Moves::CLANGOROUSSOUL, adapt_simple_move(apply_clangorous_soul));

        // Healing moves
        self.register(Moves::RECOVER, adapt_simple_move(apply_recover));
        self.register(Moves::ROOST, adapt_simple_move(apply_roost));
        self.register(Moves::MOONLIGHT, adapt_simple_move(apply_moonlight));
        self.register(Moves::SYNTHESIS, adapt_simple_move(apply_synthesis));
        self.register(Moves::MORNINGSUN, adapt_simple_move(apply_morning_sun));
        self.register(Moves::SOFTBOILED, adapt_simple_move(apply_soft_boiled));
        self.register(Moves::WISH, adapt_simple_move(apply_wish));
        self.register(Moves::HEALBELL, adapt_simple_move(apply_heal_bell));
        self.register(Moves::AROMATHERAPY, adapt_simple_move(apply_aromatherapy));
        self.register(Moves::REST, adapt_simple_move(apply_rest));

        // Weather moves
        self.register(Moves::SUNNYDAY, adapt_simple_move(apply_sunny_day));
        self.register(Moves::RAINDANCE, adapt_simple_move(apply_rain_dance));
        self.register(Moves::SANDSTORM, adapt_simple_move(apply_sandstorm));
        self.register(Moves::HAIL, adapt_simple_move(apply_hail));

        // Screen moves
        self.register(Moves::REFLECT, adapt_simple_move(apply_reflect));
        self.register(Moves::LIGHTSCREEN, adapt_simple_move(apply_light_screen));
        self.register(Moves::AURORAVEIL, adapt_simple_move(apply_aurora_veil));

        // Hazard moves
        self.register(Moves::SPIKES, adapt_simple_move(apply_spikes));
        self.register(Moves::TOXICSPIKES, adapt_simple_move(apply_toxic_spikes));
        self.register(Moves::STEALTHROCK, adapt_simple_move(apply_stealth_rock));
        self.register(Moves::STICKYWEB, adapt_simple_move(apply_sticky_web));

        // Hazard removal moves
        self.register(Moves::RAPIDSPIN, adapt_simple_move(apply_rapid_spin));
        self.register(Moves::DEFOG, adapt_simple_move(apply_defog));

        // Multi-hit moves (use variable power adapter)
        self.register(Moves::SURGINGSTRIKES, adapt_variable_power_move(multi_hit::apply_surging_strikes));
        self.register(Moves::DRAGONDARTS, adapt_variable_power_move(multi_hit::apply_dragon_darts));
        self.register(Moves::POPULATIONBOMB, adapt_variable_power_move(multi_hit::apply_population_bomb));
        self.register(Moves::SCALESHOT, adapt_variable_power_move(multi_hit::apply_scale_shot));
        
        // Generic multi-hit moves that use apply_multi_hit_move
        self.register(Moves::DOUBLESLAP, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::FURYATTACK, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::PINMISSILE, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::SPIKECANNON, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::BARRAGE, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::COMETPUNCH, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::BULLETSEED, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::ROCKBLAST, adapt_variable_power_move(multi_hit::apply_multi_hit_move));
        self.register(Moves::TAILSLAP, adapt_variable_power_move(multi_hit::apply_multi_hit_move));

        // Recoil moves - now handled automatically via PS data

        // Drain moves - now handled automatically via PS data

        // Protection moves
        self.register(Moves::PROTECT, adapt_simple_move(apply_protect));
        self.register(Moves::DETECT, adapt_simple_move(apply_detect));
        self.register(Moves::ENDURE, adapt_simple_move(apply_endure));

        // Variable power moves (with branching support)
        self.register(Moves::FACADE, adapt_variable_power_move(variable_power::apply_facade));
        self.register(Moves::HEX, adapt_variable_power_move(variable_power::apply_hex));
        self.register(Moves::GYROBALL, adapt_variable_power_move(variable_power::apply_gyro_ball));
        self.register(Moves::REVERSAL, adapt_variable_power_move(variable_power::apply_reversal));
        self.register(Moves::ACROBATICS, adapt_variable_power_move(variable_power::apply_acrobatics));
        self.register(Moves::WEATHERBALL, adapt_variable_power_move(variable_power::apply_weather_ball));
        self.register(Moves::AVALANCHE, adapt_variable_power_move(variable_power::apply_avalanche));
        self.register(Moves::ELECTROBALL, adapt_variable_power_move(variable_power::apply_electroball));
        self.register(Moves::ERUPTION, Box::new(crate::engine::combat::moves::damage::variable_power::apply_eruption_unified));
        self.register(Moves::WATERSPOUT, Box::new(crate::engine::combat::moves::damage::variable_power::apply_waterspout_unified));
        self.register(Moves::PUNISHMENT, adapt_variable_power_move(variable_power::apply_punishment));
        self.register(Moves::WAKEUPSLAP, adapt_variable_power_move(variable_power::apply_wakeup_slap));
        self.register(Moves::DRAGONENERGY, adapt_variable_power_move(variable_power::apply_dragon_energy));
        self.register(Moves::GRASSKNOT, adapt_variable_power_move(variable_power::apply_grass_knot));
        self.register(Moves::LOWKICK, adapt_variable_power_move(variable_power::apply_low_kick));
        self.register(Moves::HEATCRASH, adapt_variable_power_move(variable_power::apply_heat_crash));
        self.register(Moves::HEAVYSLAM, adapt_variable_power_move(variable_power::apply_heavy_slam));
        self.register(Moves::BARBBARRAGE, adapt_variable_power_move(variable_power::apply_barb_barrage));
        self.register(Moves::COLLISIONCOURSE, adapt_variable_power_move(variable_power::apply_collision_course));
        self.register(Moves::ELECTRODRIFT, adapt_variable_power_move(variable_power::apply_electro_drift));
        self.register(Moves::FREEZEDRY, adapt_variable_power_move(variable_power::apply_freeze_dry));
        self.register(Moves::HARDPRESS, adapt_variable_power_move(variable_power::apply_hard_press));
        self.register(Moves::HYDROSTEAM, adapt_variable_power_move(variable_power::apply_hydro_steam));
        self.register(Moves::LASTRESPECTS, adapt_variable_power_move(variable_power::apply_last_respects));
        self.register(Moves::POLTERGEIST, adapt_variable_power_move(variable_power::apply_poltergeist));
        self.register(Moves::STOREDPOWER, adapt_variable_power_move(variable_power::apply_stored_power));
        self.register(Moves::POWERTRIP, adapt_variable_power_move(variable_power::apply_power_trip));
        self.register(Moves::TERRAINPULSE, adapt_variable_power_move(variable_power::apply_terrain_pulse));

        // Context-aware moves (need context parameter)
        self.register(Moves::BOLTBEAK, adapt_context_aware_move(variable_power::apply_boltbeak));
        self.register(Moves::FISHIOUSREND, adapt_context_aware_move(variable_power::apply_fishious_rend));
        self.register(Moves::PURSUIT, adapt_context_aware_move(variable_power::apply_pursuit));
        self.register(Moves::SUCKERPUNCH, adapt_context_aware_move(variable_power::apply_sucker_punch));
        self.register(Moves::THUNDERCLAP, adapt_context_aware_move(variable_power::apply_thunder_clap));
        self.register(Moves::UPPERHAND, adapt_context_aware_move(variable_power::apply_upper_hand));

        // Repository-aware moves (need repository parameter)
        self.register(Moves::MEFIRST, adapt_repository_aware_move(variable_power::apply_me_first));

        // Fixed damage moves
        self.register(Moves::SEISMICTOSS, adapt_simple_move(fixed_damage::apply_seismic_toss));
        self.register(Moves::NIGHTSHADE, adapt_simple_move(fixed_damage::apply_night_shade));
        self.register(Moves::ENDEAVOR, adapt_simple_move(fixed_damage::apply_endeavor));
        self.register(Moves::FINALGAMBIT, adapt_simple_move(fixed_damage::apply_final_gambit));
        self.register(Moves::NATURESMADNESS, adapt_simple_move(fixed_damage::apply_natures_madness));
        self.register(Moves::RUINATION, adapt_simple_move(fixed_damage::apply_ruination));
        self.register(Moves::SUPERFANG, adapt_simple_move(fixed_damage::apply_super_fang));

        // Self-targeting moves (both self-destruct and self-damage)
        self.register(Moves::EXPLOSION, adapt_variable_power_move(self_targeting::apply_explosion));
        self.register(Moves::SELFDESTRUCT, adapt_variable_power_move(self_targeting::apply_self_destruct));
        self.register(Moves::MINDBLOWN, adapt_variable_power_move(self_targeting::apply_mind_blown));

        // Special combat moves
        self.register(Moves::BODYPRESS, adapt_variable_power_move(apply_body_press));
        self.register(Moves::FOULPLAY, adapt_variable_power_move(apply_foul_play));
        self.register(Moves::PHOTONGEYSER, adapt_variable_power_move(apply_photon_geyser));
        self.register(Moves::SKYDROP, adapt_variable_power_move(apply_sky_drop));

        // Advanced hazards (mortalspin not implemented yet)

        // Secondary effects moves  
        self.register(Moves::FLAMETHROWER, adapt_extended_move(apply_flamethrower));
        self.register(Moves::FIREBLAST, adapt_extended_move(apply_fire_blast));
        self.register(Moves::THUNDERBOLT, adapt_extended_move(apply_thunderbolt));
        self.register(Moves::ICEBEAM, adapt_extended_move(apply_ice_beam));
        self.register(Moves::SLUDGEBOMB, adapt_extended_move(apply_sludge_bomb));
        self.register(Moves::AIRSLASH, adapt_extended_move(apply_air_slash));
        self.register(Moves::IRONHEAD, adapt_extended_move(apply_iron_head));
        self.register(Moves::ROCKSLIDE, adapt_extended_move(apply_rock_slide));

        // Counter moves
        self.register(Moves::COUNTER, adapt_simple_move(counter::apply_counter));
        self.register(Moves::MIRRORCOAT, adapt_simple_move(counter::apply_mirror_coat));
        self.register(Moves::COMEUPPANCE, adapt_simple_move(counter::apply_comeuppance));
        self.register(Moves::METALBURST, adapt_simple_move(counter::apply_metal_burst));

        // Additional healing moves  
        self.register(Moves::PAINSPLIT, adapt_simple_move(healing::apply_pain_split));

        // Strength Sap (special variable power move without branching)
        self.register(Moves::STRENGTHSAP, adapt_extended_move(variable_power::apply_strength_sap));
    }

    /// Register a move effect with the unified signature
    ///
    /// This is the primary registration method that all moves should use.
    /// It accepts the unified MoveEffectFn signature that provides access to all
    /// battle context through MoveExecutionContext.
    ///
    /// ## Function signature:
    /// ```rust
    /// fn move_effect(ctx: &mut MoveExecutionContext) -> Vec<BattleInstructions>
    /// ```
    ///
    /// ## Example:
    /// ```rust
    /// registry.register(Moves::THUNDERWAVE, my_unified_move_effect);
    /// registry.register(Moves::FACADE, adapt_variable_power_move(original_facade));
    /// ```
    pub fn register(&mut self, move_enum: Moves, effect_fn: MoveEffectFn) {
        self.effects.insert(move_enum, effect_fn);
    }

    /// Get a move effect by name
    ///
    /// Returns the unified move effect function for the given move, if registered.
    pub fn get_effect(&self, move_name: &Moves) -> Option<&MoveEffectFn> {
        self.effects.get(move_name)
    }

    /// Apply move effects using the unified registry
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
        
        if let Some(effect_fn) = self.effects.get(&move_enum) {
            let mut move_context = MoveExecutionContext::new(
                state,
                move_data,
                user_position,
                target_positions,
                generation,
                context,
                repository,
                branch_on_damage,
            );
            Ok(effect_fn(&mut move_context))
        } else {
            // Move not found in registry
            Err(crate::types::BattleError::InvalidMoveChoice { 
                reason: format!("Move '{:?}' is not registered in the move registry", move_data.name) 
            })
        }
    }

    /// Check if a move is registered
    pub fn is_move_registered(&self, move_enum: &Moves) -> bool {
        self.effects.contains_key(move_enum)
    }

    /// Get the number of registered moves
    pub fn registered_move_count(&self) -> usize {
        self.effects.len()
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