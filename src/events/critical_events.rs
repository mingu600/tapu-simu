//! Critical Pokemon Showdown Events Implementation
//! 
//! This module implements the exact event system from Pokemon Showdown with
//! complete fidelity, including all priority handling, return value semantics,
//! and special cases.

use crate::events::{EventContext, EventSystem, RelayVar, RelayContainer, EventResult, EventTarget, EventSource};
use crate::errors::BattleResult;
use crate::pokemon::PokemonRef;
use crate::battle_state::BattleState;
use crate::prng::PRNG;
use std::collections::HashMap;

/// Critical events that form the backbone of Pokemon Showdown's battle system
/// These must be implemented with exact PS fidelity
impl EventSystem {
    
    /// BeforeMove event - Called before any move is executed
    /// Used for: Sleep/paralysis/freeze checks, choice locking, taunt, etc.
    /// Return semantics: false = move fails, null = move fails silently, undefined = continue
    pub fn run_before_move(
        &mut self,
        pokemon_ref: PokemonRef,
        target: Option<PokemonRef>,
        move_id: &str,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<bool> {
        let relay_container = RelayContainer::new(RelayVar::Bool(true));
        
        let result = self.run_event(
            "BeforeMove",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // PS return semantics: false or null = move blocked
        match result.value {
            RelayVar::Bool(can_move) => Ok(can_move),
            _ => Ok(false), // Default to blocked if unexpected type
        }
    }
    
    /// TryHit event - Core move hit validation
    /// Used for: Immunity checks, protect/detect, type immunity, wonder guard
    /// Priority: Left-to-right order (as per PS)
    /// Return semantics: false = miss with message, null = miss silently, 0 = hit substitute
    pub fn run_try_hit(
        &mut self,
        target: PokemonRef,
        source: PokemonRef,
        move_id: &str,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<TryHitResult> {
        let relay_container = RelayContainer::new(RelayVar::Bool(true));
        
        let result = self.run_event(
            "TryHit",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // Convert PS return values to our enum
        match result.value {
            RelayVar::Bool(false) => Ok(TryHitResult::Miss),
            RelayVar::Number(0.0) => Ok(TryHitResult::HitSubstitute),
            RelayVar::None => Ok(TryHitResult::MissSilent),
            _ => Ok(TryHitResult::Hit),
        }
    }
    
    /// BasePower event - Modify move base power
    /// Used for: Technician, Adaptability, terrain/weather boosts, rivalry, etc.
    /// Return semantics: Number = new base power, undefined = no change
    pub fn run_base_power(
        &mut self,
        source: PokemonRef,
        target: PokemonRef,
        move_id: &str,
        base_power: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::BasePower(base_power));
        
        let result = self.run_event(
            "BasePower",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // Return modified base power or original if unchanged
        Ok(result.value.as_base_power().unwrap_or(base_power))
    }
    
    /// ModifyDamage event - Final damage modifications
    /// Used for: Type effectiveness, STAB, weather effects, abilities like Filter
    /// Return semantics: Number = new damage, undefined = no change
    pub fn run_modify_damage(
        &mut self,
        target: PokemonRef,
        source: PokemonRef,
        move_id: &str,
        damage: u32,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u32> {
        let relay_container = RelayContainer::new(RelayVar::Damage(damage));
        
        let result = self.run_event(
            "ModifyDamage",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_damage().unwrap_or(damage))
    }
    
    /// DamagingHit event - After damage is dealt but before secondary effects
    /// Used for: Contact abilities (Static, Flame Body), King's Rock, etc.
    /// Priority: Left-to-right order
    /// Return semantics: Usually ignored, used for side effects
    pub fn run_damaging_hit(
        &mut self,
        target: PokemonRef,
        source: PokemonRef,
        move_id: &str,
        damage: u32,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<()> {
        let relay_container = RelayContainer::new(RelayVar::Damage(damage));
        
        let _result = self.run_event(
            "DamagingHit",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // DamagingHit typically doesn't return values, just triggers side effects
        Ok(())
    }
    
    /// ModifySTAB event - Modify Same Type Attack Bonus
    /// Used for: Adaptability (2x instead of 1.5x)
    /// Return semantics: Number = new STAB multiplier, undefined = no change
    pub fn run_modify_stab(
        &mut self,
        source: PokemonRef,
        move_id: &str,
        move_type: crate::types::Type,
        stab: f32,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<f32> {
        let relay_container = RelayContainer::new(RelayVar::StabMultiplier(stab));
        
        let result = self.run_event(
            "ModifySTAB",
            Some(EventTarget::Pokemon(source)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stab_multiplier().unwrap_or(stab))
    }
    
    /// Effectiveness event - Modify type effectiveness
    /// Used for: Wonder Guard, Scrappy, Ring Target, etc.
    /// Return semantics: Number = new effectiveness, undefined = no change
    pub fn run_effectiveness(
        &mut self,
        target: PokemonRef,
        source: PokemonRef,
        move_type: crate::types::Type,
        effectiveness: f32,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<f32> {
        let relay_container = RelayContainer::new(RelayVar::TypeEffectiveness(effectiveness));
        
        let result = self.run_event(
            "Effectiveness",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_type_effectiveness().unwrap_or(effectiveness))
    }
    
    /// SwitchIn event - Called when Pokemon enters battle
    /// Used for: Intimidate, Download, Trace, weather setting abilities, etc.
    /// Return semantics: Usually ignored, used for side effects
    pub fn run_switch_in(
        &mut self,
        pokemon_ref: PokemonRef,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<()> {
        let relay_container = RelayContainer::new(RelayVar::None);
        
        let _result = self.run_event(
            "SwitchIn",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(())
    }
    
    /// SwitchOut event - Called when Pokemon leaves battle
    /// Used for: Natural Cure, Regenerator, etc.
    /// Return semantics: Usually ignored, used for side effects
    pub fn run_switch_out(
        &mut self,
        pokemon_ref: PokemonRef,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<()> {
        let relay_container = RelayContainer::new(RelayVar::None);
        
        let _result = self.run_event(
            "SwitchOut",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(())
    }
    
    /// TurnEnd event - Called at the end of each turn
    /// Used for: Leftovers, status damage, weather effects, etc.
    /// Return semantics: Usually ignored, used for side effects
    pub fn run_turn_end(
        &mut self,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<()> {
        let relay_container = RelayContainer::new(RelayVar::None);
        
        // Run for the entire battle field
        let _result = self.run_event(
            "TurnEnd",
            Some(EventTarget::Field),
            None,
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(())
    }
    
    /// Residual event - Called for ongoing effects
    /// Used for: Leftovers healing, Poison damage, etc.
    /// Return semantics: Usually ignored, used for side effects
    pub fn run_residual(
        &mut self,
        pokemon_ref: PokemonRef,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<()> {
        let relay_container = RelayContainer::new(RelayVar::None);
        
        let _result = self.run_event(
            "Residual",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(())
    }
    
    /// ModifyMove event - Modify move properties before execution
    /// Used for: Pixilate, Aerilate, Normalize, etc.
    /// Return semantics: Move object = modified move, undefined = no change
    pub fn run_modify_move(
        &mut self,
        source: PokemonRef,
        target: Option<PokemonRef>,
        move_data: &crate::pokemon::MoveData,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<Option<crate::pokemon::MoveData>> {
        let relay_container = RelayContainer::new(RelayVar::move_data(move_data.clone()));
        
        let result = self.run_event(
            "ModifyMove",
            target.map(EventTarget::Pokemon).or(Some(EventTarget::Pokemon(source))),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_data.id.clone(), move_data.name.clone(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // Return modified move if it was changed
        match result.value.as_move() {
            Some(modified_move) => Ok(Some(modified_move.clone())),
            None => Ok(None),
        }
    }
    
    /// Immunity event - Check for type/move immunity
    /// Used for: Levitate, Flash Fire absorption, Soundproof, etc.
    /// Return semantics: false = not immune, true = immune
    pub fn run_immunity(
        &mut self,
        target: PokemonRef,
        source: PokemonRef,
        move_type: crate::types::Type,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<bool> {
        let relay_container = RelayContainer::new(RelayVar::Bool(false));
        
        let result = self.run_event(
            "Immunity",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        // Return true if immune, false if not
        Ok(result.value.as_bool().unwrap_or(false))
    }
}

/// Result of TryHit event processing
#[derive(Debug, Clone, PartialEq)]
pub enum TryHitResult {
    /// Move hits normally
    Hit,
    /// Move misses with message
    Miss,
    /// Move misses silently
    MissSilent,
    /// Move hits substitute instead
    HitSubstitute,
}

/// Pokemon Showdown stat modification events
impl EventSystem {
    /// ModifyAtk event - Modify Attack stat
    pub fn run_modify_attack(
        &mut self,
        pokemon_ref: PokemonRef,
        attack: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::StatValue(attack));
        
        let result = self.run_event(
            "ModifyAtk",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stat_value().unwrap_or(attack))
    }
    
    /// ModifyDef event - Modify Defense stat
    pub fn run_modify_defense(
        &mut self,
        pokemon_ref: PokemonRef,
        defense: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::StatValue(defense));
        
        let result = self.run_event(
            "ModifyDef",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stat_value().unwrap_or(defense))
    }
    
    /// ModifySpA event - Modify Special Attack stat
    pub fn run_modify_special_attack(
        &mut self,
        pokemon_ref: PokemonRef,
        special_attack: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::StatValue(special_attack));
        
        let result = self.run_event(
            "ModifySpA",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stat_value().unwrap_or(special_attack))
    }
    
    /// ModifySpD event - Modify Special Defense stat
    pub fn run_modify_special_defense(
        &mut self,
        pokemon_ref: PokemonRef,
        special_defense: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::StatValue(special_defense));
        
        let result = self.run_event(
            "ModifySpD",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stat_value().unwrap_or(special_defense))
    }
    
    /// ModifySpeed event - Modify Speed stat
    pub fn run_modify_speed(
        &mut self,
        pokemon_ref: PokemonRef,
        speed: u16,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u16> {
        let relay_container = RelayContainer::new(RelayVar::StatValue(speed));
        
        let result = self.run_event(
            "ModifySpeed",
            Some(EventTarget::Pokemon(pokemon_ref)),
            Some(EventSource::Pokemon(pokemon_ref)),
            None,
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_stat_value().unwrap_or(speed))
    }
}

/// Pokemon Showdown secondary events
impl EventSystem {
    /// ModifyAccuracy event - Modify move accuracy
    pub fn run_modify_accuracy(
        &mut self,
        source: PokemonRef,
        target: PokemonRef,
        move_id: &str,
        accuracy: u8,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u8> {
        let relay_container = RelayContainer::new(RelayVar::Accuracy(accuracy));
        
        let result = self.run_event(
            "ModifyAccuracy",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_accuracy().unwrap_or(accuracy))
    }
    
    /// ModifyCritRatio event - Modify critical hit ratio
    pub fn run_modify_crit_ratio(
        &mut self,
        source: PokemonRef,
        target: PokemonRef,
        move_id: &str,
        crit_ratio: u8,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<u8> {
        let relay_container = RelayContainer::new(RelayVar::CritRatio(crit_ratio));
        
        let result = self.run_event(
            "ModifyCritRatio",
            Some(EventTarget::Pokemon(target)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_crit_ratio().unwrap_or(crit_ratio))
    }
    
    /// ModifyPriority event - Modify move priority
    pub fn run_modify_priority(
        &mut self,
        source: PokemonRef,
        move_id: &str,
        priority: i8,
        battle_state: &mut BattleState,
        prng: &mut PRNG,
        turn: u32,
    ) -> BattleResult<i8> {
        let relay_container = RelayContainer::new(RelayVar::Priority(priority));
        
        let result = self.run_event(
            "ModifyPriority",
            Some(EventTarget::Pokemon(source)),
            Some(EventSource::Pokemon(source)),
            Some(crate::events::EffectData::move_effect(move_id.to_string(), move_id.to_string(), Some(format!("{}:{}", source.side, source.position)))),
            relay_container,
            battle_state,
            prng,
            turn,
        )?;
        
        Ok(result.value.as_priority().unwrap_or(priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle_state::BattleState;
    use crate::format::BattleFormat;
    use crate::side::SideId;
    
    #[test]
    fn test_critical_events_integration() {
        let mut event_system = EventSystem::new();
        let mut battle_state = BattleState::new(BattleFormat::Singles);
        let mut prng = crate::prng::PRNG::new(Some([1, 2, 3, 4]));
        
        let pokemon_ref = PokemonRef { side: SideId::P1, position: 0 };
        
        // Test BeforeMove event
        let can_move = event_system.run_before_move(
            pokemon_ref,
            Some(pokemon_ref),
            "tackle",
            &mut battle_state,
            &mut prng,
            1,
        );
        assert!(can_move.is_ok());
        
        // Test TryHit event
        let hit_result = event_system.run_try_hit(
            pokemon_ref,
            pokemon_ref,
            "tackle",
            &mut battle_state,
            &mut prng,
            1,
        );
        assert!(hit_result.is_ok());
        
        // Test BasePower event
        let base_power = event_system.run_base_power(
            pokemon_ref,
            pokemon_ref,
            "tackle",
            40,
            &mut battle_state,
            &mut prng,
            1,
        );
        assert!(base_power.is_ok());
        assert_eq!(base_power.unwrap(), 40); // Should be unchanged without handlers
    }
    
    #[test]
    fn test_stat_modification_events() {
        let mut event_system = EventSystem::new();
        let mut battle_state = BattleState::new(BattleFormat::Singles);
        let mut prng = crate::prng::PRNG::new(Some([1, 2, 3, 4]));
        
        let pokemon_ref = PokemonRef { side: SideId::P1, position: 0 };
        
        // Test stat modification events
        let attack = event_system.run_modify_attack(pokemon_ref, 100, &mut battle_state, &mut prng, 1);
        assert!(attack.is_ok());
        assert_eq!(attack.unwrap(), 100);
        
        let speed = event_system.run_modify_speed(pokemon_ref, 80, &mut battle_state, &mut prng, 1);
        assert!(speed.is_ok());
        assert_eq!(speed.unwrap(), 80);
    }
}