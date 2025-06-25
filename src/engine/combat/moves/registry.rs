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
    apply_aromatherapy, apply_refresh, apply_rest, apply_sleep_talk
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
    apply_barrage, apply_fury_swipes, apply_comet_punch, apply_bullet_seed,
    apply_icicle_spear, apply_rock_blast, apply_tail_slap, apply_scale_shot
};

use super::damage::recoil::{
    apply_double_edge, apply_take_down, apply_submission, apply_brave_bird,
    apply_flare_blitz, apply_volt_tackle, apply_wood_hammer, apply_wild_charge,
    apply_head_smash
};

use super::damage::drain::{
    apply_absorb, apply_mega_drain, apply_giga_drain, apply_drain_punch,
    apply_leech_life, apply_horn_leech, apply_parabolic_charge
};

use super::special::protection::{
    apply_protect, apply_detect, apply_endure, apply_quick_guard, apply_wide_guard
};

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

/// Move registry that maps move names to their effect functions
pub struct MoveRegistry {
    /// Standard move effects
    standard_moves: std::collections::HashMap<String, MoveEffectFn>,
    /// Extended move effects that need move data
    extended_moves: std::collections::HashMap<String, ExtendedMoveEffectFn>,
}

impl MoveRegistry {
    /// Create a new move registry with all move effects registered
    pub fn new() -> Self {
        let mut registry = Self {
            standard_moves: std::collections::HashMap::new(),
            extended_moves: std::collections::HashMap::new(),
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
        self.register_standard("refresh", apply_refresh);
        self.register_standard("rest", apply_rest);
        self.register_standard("sleeptalk", apply_sleep_talk);

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

        // Multi-hit moves
        self.register_standard("doubleslap", apply_double_slap);
        self.register_standard("furyattack", apply_fury_attack);
        self.register_standard("pinmissile", apply_pin_missile);
        self.register_standard("spikecannon", apply_spike_cannon);
        self.register_standard("barrage", apply_barrage);
        self.register_standard("furyswipes", apply_fury_swipes);
        self.register_standard("cometpunch", apply_comet_punch);
        self.register_standard("bulletseed", apply_bullet_seed);
        self.register_standard("iciclespear", apply_icicle_spear);
        self.register_standard("rockblast", apply_rock_blast);
        self.register_standard("tailslap", apply_tail_slap);
        self.register_standard("scaleshot", apply_scale_shot);

        // Recoil moves
        self.register_standard("doubleedge", apply_double_edge);
        self.register_standard("takedown", apply_take_down);
        self.register_standard("submission", apply_submission);
        self.register_standard("bravebird", apply_brave_bird);
        self.register_standard("flareblitz", apply_flare_blitz);
        self.register_standard("volttackle", apply_volt_tackle);
        self.register_standard("woodhammer", apply_wood_hammer);
        self.register_standard("wildcharge", apply_wild_charge);
        self.register_standard("headsmash", apply_head_smash);

        // Drain moves
        self.register_standard("absorb", apply_absorb);
        self.register_standard("megadrain", apply_mega_drain);
        self.register_standard("gigadrain", apply_giga_drain);
        self.register_standard("drainpunch", apply_drain_punch);
        self.register_standard("leechlife", apply_leech_life);
        self.register_standard("hornleech", apply_horn_leech);
        self.register_standard("paraboliccharge", apply_parabolic_charge);

        // Protection moves
        self.register_standard("protect", apply_protect);
        self.register_standard("detect", apply_detect);
        self.register_standard("endure", apply_endure);
        self.register_standard("quickguard", apply_quick_guard);
        self.register_standard("wideguard", apply_wide_guard);
    }

    /// Register a standard move effect
    fn register_standard(&mut self, name: &str, effect_fn: MoveEffectFn) {
        self.standard_moves.insert(name.to_string(), effect_fn);
    }

    /// Register an extended move effect that needs move data
    fn register_extended(&mut self, name: &str, effect_fn: ExtendedMoveEffectFn) {
        self.extended_moves.insert(name.to_string(), effect_fn);
    }

    /// Apply move effects using the registry
    pub fn apply_move_effects(
        &self,
        state: &BattleState,
        move_data: &MoveData,
        user_position: BattlePosition,
        target_positions: &[BattlePosition],
        generation: &GenerationMechanics,
        _context: &MoveContext,
        _repository: &GameDataRepository,
        _branch_on_damage: bool,
    ) -> BattleResult<Vec<BattleInstructions>> {
        let move_name = normalize_name(&move_data.name);
        
        // Try standard moves first
        if let Some(effect_fn) = self.standard_moves.get(&move_name) {
            return Ok(effect_fn(state, user_position, target_positions, generation));
        }
        
        // Try extended moves
        if let Some(effect_fn) = self.extended_moves.get(&move_name) {
            return Ok(effect_fn(state, move_data, user_position, target_positions, generation));
        }
        
        // Fallback for unimplemented moves
        eprintln!("Warning: Move '{}' not implemented, no effect applied", move_data.name);
        Ok(vec![])
    }

    /// Check if a move is registered
    pub fn is_move_registered(&self, move_name: &str) -> bool {
        let normalized = normalize_name(move_name);
        self.standard_moves.contains_key(&normalized) || self.extended_moves.contains_key(&normalized)
    }

    /// Get the number of registered moves
    pub fn registered_move_count(&self) -> usize {
        self.standard_moves.len() + self.extended_moves.len()
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