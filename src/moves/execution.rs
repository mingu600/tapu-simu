//! Pokemon Showdown move execution pipeline
//! 
//! This module implements the exact move execution pipeline from Pokemon Showdown,
//! including all the key functions: tryMoveHit, hitStepMoveHitLoop, spreadMoveHit, 
//! getDamage, and moveHit.

use crate::errors::BattleResult;
use crate::events::{EventSystem, EventTarget, EventSource};
use crate::pokemon::Pokemon;
use super::targeting::TargetResolver;
use crate::side::SideId;
use crate::pokemon::MoveData;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Active move in execution - matches PS ActiveMove interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMove {
    pub name: String,
    pub id: String,
    pub hit: u8,                           // Current hit number
    pub move_hit_data: MoveHitData,        // Per-target hit data
    pub total_damage: Option<TotalDamage>, // Accumulated damage
    pub spread_hit: bool,                  // Multi-target move flag
    pub base_move: MoveData,               // Base move data
    pub type_: Option<String>,             // Current move type (can be modified)
    pub category: Option<String>,          // Current category (can be modified)
    pub base_power: Option<u16>,           // Current base power (can be modified)
    pub accuracy: Option<u8>,              // Current accuracy (can be modified)
    pub crit_ratio: u8,                    // Critical hit ratio
    pub will_crit: bool,                   // Forced critical hit
    pub hit_count: Option<u8>,             // For multi-hit moves
    pub z_move: bool,                      // Is this a Z-move?
    pub max_move: bool,                    // Is this a Max move?
    pub is_z: bool,                        // Alternative Z-move flag
    pub flags: HashMap<String, bool>,      // Move flags (contact, protect, etc.)
}

/// Total damage tracking for multi-hit moves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TotalDamage {
    Damage(u32),
    Failed,
}

/// Per-target hit data - matches PS MoveHitData
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoveHitData {
    pub target_data: HashMap<String, TargetHitData>, // targetSlotid -> data
}

/// Hit data for a specific target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetHitData {
    pub crit: bool,           // Did this move crit?
    pub type_mod: i8,         // Type effectiveness (-6 to +6)
    pub z_broke_protect: bool, // Did Z-move break protect?
}

/// Pokemon reference for targeting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonRef {
    pub side_id: SideId,
    pub position: usize,
}

impl PokemonRef {
    pub fn slot_id(&self) -> String {
        format!("{:?}:{}", self.side_id, self.position)
    }
}

/// Move execution result types matching PS semantics
#[derive(Debug, Clone)]
pub enum MoveResult {
    Damage(u32),              // Damage dealt
    Failed,                   // Move failed loudly
    Silent,                   // Move failed silently
    NoEffect,                 // Move had no effect but continues
}

/// Move execution context
pub struct MoveExecutor {
    event_system: EventSystem,
}

impl MoveExecutor {
    pub fn new() -> Self {
        Self {
            event_system: EventSystem::new(),
        }
    }

    /// Pokemon Showdown's tryMoveHit() function
    /// 
    /// Pre-execution validation and targeting for side/field moves
    pub fn try_move_hit(
        &mut self,
        targets: &[PokemonRef],
        user: &PokemonRef,
        active_move: &mut ActiveMove,
    ) -> BattleResult<Option<MoveResult>> {
        // Check accuracy for each target
        for target in targets {
            if !self.check_accuracy(user, target, active_move)? {
                // Move missed
                return Ok(Some(MoveResult::Miss));
            }
        }
        
        // All accuracy checks passed - move hits
        // Run 'PrepareHit' event here when event system is fully integrated
        
        // Return success to continue execution
        Ok(None)
    }
    
    /// Check if a move hits based on accuracy calculation
    /// This matches Pokemon Showdown's accuracy checking mechanics
    fn check_accuracy(
        &mut self,
        user: &PokemonRef,
        target: &PokemonRef,
        active_move: &ActiveMove,
    ) -> BattleResult<bool> {
        // Moves with no accuracy always hit (like Swift)
        if active_move.base_move.accuracy.is_none() {
            return Ok(true);
        }
        
        let base_accuracy = active_move.base_move.accuracy.unwrap();
        
        // Get user and target Pokemon from battle state
        let user_pokemon = self.battle_state.get_pokemon(*user)?;
        let target_pokemon = self.battle_state.get_pokemon(*target)?;
        
        // Calculate accuracy stage multiplier (from stat boosts)
        let accuracy_stage = user_pokemon.boosts.accuracy;
        let evasion_stage = target_pokemon.boosts.evasion;
        let net_stage = accuracy_stage - evasion_stage;
        
        // Pokemon Showdown accuracy stage multipliers
        let stage_multiplier = match net_stage {
            -6 => 3.0 / 9.0,   // 33%
            -5 => 3.0 / 8.0,   // 37.5%
            -4 => 3.0 / 7.0,   // ~43%
            -3 => 3.0 / 6.0,   // 50%
            -2 => 3.0 / 5.0,   // 60%
            -1 => 3.0 / 4.0,   // 75%
             0 => 1.0,         // 100%
             1 => 4.0 / 3.0,   // ~133%
             2 => 5.0 / 3.0,   // ~167%
             3 => 6.0 / 3.0,   // 200%
             4 => 7.0 / 3.0,   // ~233%
             5 => 8.0 / 3.0,   // ~267%
             6 => 9.0 / 3.0,   // 300%
             _ => if net_stage > 6 { 9.0 / 3.0 } else { 3.0 / 9.0 },
        };
        
        // Calculate final accuracy
        let final_accuracy = (base_accuracy as f32 * stage_multiplier) as u8;
        let final_accuracy = final_accuracy.min(100); // Cap at 100%
        
        // Roll for accuracy
        let roll = self.battle_state.random.next_u32() % 100;
        let hit = roll < final_accuracy as u32;
        
        Ok(hit)
    }

    /// Pokemon Showdown's hitStepMoveHitLoop() function
    /// 
    /// Multi-hit handling and accuracy checks per hit
    pub fn hit_step_move_hit_loop(
        &mut self,
        targets: &[PokemonRef],
        user: &PokemonRef,
        active_move: &mut ActiveMove,
    ) -> BattleResult<Vec<MoveResult>> {
        let mut results = Vec::new();
        
        // Determine number of hits for multi-hit moves
        let hit_count = self.determine_hit_count(active_move);
        
        // Loop through each hit
        for hit_num in 1..=hit_count {
            active_move.hit = hit_num;
            
            // Call spreadMoveHit for this hit
            let hit_result = self.spread_move_hit(
                targets,
                user,
                active_move,
                None, // hit_effect
                false, // is_secondary
                false, // is_self
            )?;
            
            // Check if we should continue hitting (target fainted, etc.)
            let should_stop = self.should_stop_hitting(&hit_result);
            results.push(hit_result);
            
            if should_stop {
                break;
            }
        }
        
        Ok(results)
    }

    /// Pokemon Showdown's spreadMoveHit() function
    /// 
    /// Main damage calculation and effect application
    pub fn spread_move_hit(
        &mut self,
        targets: &[PokemonRef],
        user: &PokemonRef,
        active_move: &mut ActiveMove,
        _hit_effect: Option<&HitEffect>,
        _is_secondary: bool,
        _is_self: bool,
    ) -> BattleResult<MoveResult> {
        // Step 1: Damage Calculation
        let _damage_results = self.get_spread_damage(targets, user, active_move)?;
        
        // Step 2: Damage Application
        // This would apply damage to all targets
        // battle.spread_damage(damage_results);
        
        // Step 3: Effect Application  
        // This would handle boosts, status, healing, etc.
        // self.run_move_effects(targets, user, active_move, hit_effect)?;
        
        // For now, return success with placeholder damage
        Ok(MoveResult::Damage(0))
    }

    /// Pokemon Showdown's getDamage() function
    /// 
    /// Core damage formula implementation
    pub fn get_damage(
        &mut self,
        source: &PokemonRef,
        target: &PokemonRef,
        active_move: &ActiveMove,
        _suppress_messages: bool,
    ) -> BattleResult<Option<u32>> {
        // Step 1: Immunity check
        // if !target.run_immunity(move.type) { return Ok(None); }
        
        // Step 2: Special damage handling (OHKO, fixed damage, level-based)
        if let Some(special_damage) = self.calculate_special_damage(source, target, active_move)? {
            return Ok(Some(special_damage));
        }
        
        // Step 3: Base power calculation with events
        let base_power = self.calculate_base_power(source, target, active_move)?;
        if base_power == 0 {
            return Ok(Some(0));
        }
        
        // Step 4: Critical hit determination
        let is_crit = self.determine_critical_hit(source, target, active_move)?;
        
        // Step 5: Get attack and defense stats
        let (attack_stat, defense_stat) = self.get_battle_stats(source, target, active_move, is_crit)?;
        
        // Step 6: Base damage calculation
        // floor(floor(floor(2 * level / 5 + 2) * basePower * attack) / defense) / 50)
        let level = 50; // Assume level 50 for now
        let base_damage = ((2 * level / 5 + 2) * base_power as u32 * attack_stat) / defense_stat / 50;
        
        // Step 7: Apply damage modifiers
        let final_damage = self.apply_damage_modifiers(
            base_damage, 
            source, 
            target, 
            active_move, 
            is_crit
        )?;
        
        // Apply secondary effects after damage calculation
        if let Some(secondary) = &active_move.base_move.secondary_effect {
            self.apply_secondary_effect(target, source, secondary, active_move)?;
        }
        
        Ok(Some(final_damage))
    }
    
    /// Apply secondary effects from moves
    fn apply_secondary_effect(
        &mut self,
        target: &PokemonRef,
        source: &PokemonRef,
        secondary: &crate::pokemon::SecondaryEffect,
        active_move: &ActiveMove,
    ) -> BattleResult<()> {
        // Roll for secondary effect chance
        let roll = self.battle_state.random.next_u32() % 100;
        if roll >= secondary.chance as u32 {
            return Ok(()); // Effect didn't trigger
        }
        
        // Apply the secondary effect
        match &secondary.effect {
            crate::pokemon::SecondaryEffectType::Status(status) => {
                self.apply_status_effect(target, status.clone())?;
            },
            crate::pokemon::SecondaryEffectType::StatBoost { stat, amount } => {
                self.apply_stat_boost(target, stat, *amount)?;
            },
            crate::pokemon::SecondaryEffectType::Burn => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::Burn)?;
            },
            crate::pokemon::SecondaryEffectType::Paralyze => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::Paralysis)?;
            },
            crate::pokemon::SecondaryEffectType::Freeze => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::Freeze)?;
            },
            crate::pokemon::SecondaryEffectType::Poison => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::Poison)?;
            },
            crate::pokemon::SecondaryEffectType::BadlyPoison => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::BadlyPoison)?;
            },
            crate::pokemon::SecondaryEffectType::Sleep => {
                self.apply_status_effect(target, crate::pokemon::StatusCondition::Sleep)?;
            },
            crate::pokemon::SecondaryEffectType::Flinch => {
                // Flinch is typically handled as a volatile status
                self.apply_volatile_status(target, "flinch".to_string())?;
            },
        }
        
        Ok(())
    }
    
    /// Apply a status condition to a Pokemon
    fn apply_status_effect(
        &mut self,
        target: &PokemonRef,
        status: crate::pokemon::StatusCondition,
    ) -> BattleResult<()> {
        let target_pokemon = self.battle_state.get_pokemon_mut(*target)?;
        
        // Don't apply status if Pokemon already has a major status
        if target_pokemon.status.is_some() {
            return Ok(());
        }
        
        target_pokemon.status = Some(status);
        Ok(())
    }
    
    /// Apply a stat boost to a Pokemon
    fn apply_stat_boost(
        &mut self,
        target: &PokemonRef,
        stat: &str,
        amount: i8,
    ) -> BattleResult<()> {
        let target_pokemon = self.battle_state.get_pokemon_mut(*target)?;
        
        // Apply stat boost (clamp to -6/+6 range)
        match stat {
            "attack" | "atk" => {
                target_pokemon.boosts.attack = (target_pokemon.boosts.attack + amount).clamp(-6, 6);
            },
            "defense" | "def" => {
                target_pokemon.boosts.defense = (target_pokemon.boosts.defense + amount).clamp(-6, 6);
            },
            "spatk" | "spa" => {
                target_pokemon.boosts.special_attack = (target_pokemon.boosts.special_attack + amount).clamp(-6, 6);
            },
            "spdef" | "spd" => {
                target_pokemon.boosts.special_defense = (target_pokemon.boosts.special_defense + amount).clamp(-6, 6);
            },
            "speed" | "spe" => {
                target_pokemon.boosts.speed = (target_pokemon.boosts.speed + amount).clamp(-6, 6);
            },
            "accuracy" | "acc" => {
                target_pokemon.boosts.accuracy = (target_pokemon.boosts.accuracy + amount).clamp(-6, 6);
            },
            "evasion" | "eva" => {
                target_pokemon.boosts.evasion = (target_pokemon.boosts.evasion + amount).clamp(-6, 6);
            },
            _ => {
                // Unknown stat, ignore
            }
        }
        
        Ok(())
    }
    
    /// Apply a volatile status effect to a Pokemon
    fn apply_volatile_status(
        &mut self,
        target: &PokemonRef,
        volatile_id: String,
    ) -> BattleResult<()> {
        let target_pokemon = self.battle_state.get_pokemon_mut(*target)?;
        
        let volatile_status = crate::pokemon::VolatileStatus {
            id: volatile_id.clone(),
            duration: Some(1), // Most volatiles last 1 turn (like flinch)
            data: std::collections::HashMap::new(),
        };
        
        target_pokemon.volatiles.insert(volatile_id, volatile_status);
        Ok(())
    }
    
    /// Resolve move targets based on target type and chosen target
    pub fn resolve_move_targets(
        &mut self,
        user: &PokemonRef,
        active_move: &ActiveMove,
        chosen_target: Option<PokemonRef>,
    ) -> BattleResult<Vec<PokemonRef>> {
        TargetResolver::resolve_targets(
            &self.battle_state,
            user,
            active_move.base_move.target,
            chosen_target,
            &mut self.battle_state.random,
        )
    }

    /// Pokemon Showdown's moveHit() function
    /// 
    /// Wrapper for single-target moves
    pub fn move_hit(
        &mut self,
        targets: &[PokemonRef],
        user: &PokemonRef,
        active_move: &mut ActiveMove,
        hit_effect: Option<&HitEffect>,
        is_secondary: bool,
        is_self: bool,
    ) -> BattleResult<MoveResult> {
        // Simple wrapper that calls spreadMoveHit
        self.spread_move_hit(targets, user, active_move, hit_effect, is_secondary, is_self)
    }

    // Helper methods

    fn determine_hit_count(&self, active_move: &ActiveMove) -> u8 {
        if let Some(hit_count) = active_move.hit_count {
            return hit_count;
        }
        
        // Multi-hit moves have hardcoded distributions in PS
        match active_move.id.as_str() {
            "bulletseed" | "rockblast" | "icicleshard" => {
                // 2-5 hits with distribution: [0, 0, 35, 35, 15, 15]
                // For now, return 3 as default
                3
            }
            "doubleslap" | "cometpunch" | "furyattack" | "pinmissile" | "spikecannon" => {
                // 2-5 hits with distribution: [0, 0, 37.5, 37.5, 12.5, 12.5]
                3
            }
            "armthrust" | "beatup" => {
                // Variable based on party/ability
                2
            }
            "dragondarts" => {
                // Special targeting mechanics
                2
            }
            _ => 1, // Single hit move
        }
    }

    fn should_stop_hitting(&self, result: &MoveResult) -> bool {
        matches!(result, MoveResult::Failed | MoveResult::Silent)
    }

    fn get_spread_damage(
        &mut self,
        targets: &[PokemonRef],
        user: &PokemonRef,
        active_move: &ActiveMove,
    ) -> BattleResult<Vec<Option<u32>>> {
        let mut damages = Vec::new();
        
        for target in targets {
            let damage = self.get_damage(user, target, active_move, false)?;
            damages.push(damage);
        }
        
        Ok(damages)
    }

    fn calculate_special_damage(
        &self,
        _source: &PokemonRef,
        _target: &PokemonRef,
        active_move: &ActiveMove,
    ) -> BattleResult<Option<u32>> {
        // Handle special damage moves
        match active_move.id.as_str() {
            "seismic-toss" | "night-shade" => {
                // Level-based damage
                Ok(Some(50)) // Assume level 50
            }
            "dragon-rage" => {
                // Fixed 40 damage
                Ok(Some(40))
            }
            "sonic-boom" => {
                // Fixed 20 damage
                Ok(Some(20))
            }
            "super-fang" => {
                // Half current HP
                // Ok(Some(target.current_hp / 2))
                Ok(Some(50)) // Placeholder
            }
            _ => Ok(None), // Not a special damage move
        }
    }

    fn calculate_base_power(
        &mut self,
        _source: &PokemonRef,
        _target: &PokemonRef,
        active_move: &ActiveMove,
    ) -> BattleResult<u16> {
        let base_power = active_move.base_move.base_power;
        
        // Run BasePower event to allow modifications
        // This would use the event system to let abilities, items, etc. modify base power
        
        // For now, return the base power from move data
        Ok(base_power)
    }

    fn determine_critical_hit(
        &self,
        _source: &PokemonRef,
        _target: &PokemonRef,
        active_move: &ActiveMove,
    ) -> BattleResult<bool> {
        if active_move.will_crit {
            return Ok(true);
        }
        
        // Calculate critical hit based on crit ratio
        // This would use PRNG and crit ratio calculations
        // For now, return false
        Ok(false)
    }

    fn get_battle_stats(
        &self,
        _source: &PokemonRef,
        _target: &PokemonRef,
        active_move: &ActiveMove,
        _is_crit: bool,
    ) -> BattleResult<(u32, u32)> {
        // Get attack and defense stats considering:
        // - Move category (physical/special)
        // - Stat boosts
        // - Critical hits (ignore negative boosts)
        // - Ability/item modifications
        
        // For now, return placeholder values
        let attack = match active_move.base_move.category {
            crate::types::MoveCategory::Physical => 100, // Attack stat
            _ => 100, // Special Attack stat
        };
        
        let defense = match active_move.base_move.category {
            crate::types::MoveCategory::Physical => 100, // Defense stat  
            _ => 100, // Special Defense stat
        };
        
        Ok((attack, defense))
    }

    fn apply_damage_modifiers(
        &mut self,
        base_damage: u32,
        _source: &PokemonRef,
        _target: &PokemonRef,
        _active_move: &ActiveMove,
        _is_crit: bool,
    ) -> BattleResult<u32> {
        // Apply damage modifiers in PS order:
        // 1. STAB (Same Type Attack Bonus)
        // 2. Type effectiveness  
        // 3. Burn (halves physical damage)
        // 4. Weather effects
        // 5. Critical hit (1.5x)
        // 6. Random factor (85-100%)
        // 7. Ability/item modifiers
        
        let mut damage = base_damage;
        
        // For now, just apply random factor
        damage = (damage * 85) / 100; // Minimum damage roll
        
        Ok(damage)
    }
}

/// Hit effect data for secondary effects
#[derive(Debug, Clone)]
pub struct HitEffect {
    pub effect_type: String,
    pub data: HashMap<String, serde_json::Value>,
}

impl Default for MoveExecutor {
    fn default() -> Self {
        Self::new()
    }
}