//! # Battle State System
//! 
//! This module defines the core battle state representation for the V2 engine.
//! The state is format-aware and supports multiple active Pokemon per side.

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::core::instruction::{PokemonStatus, VolatileStatus, Weather, Terrain, SideCondition};
use crate::data::types::Stats;

// Re-export MoveCategory so other modules can import it from state
pub use crate::core::instruction::MoveCategory;
use crate::core::move_choice::MoveIndex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tracks damage dealt to a side for counter moves
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageDealt {
    /// Amount of damage dealt
    pub damage: i16,
    /// Category of the move that dealt damage
    pub move_category: MoveCategory,
    /// Whether the damage hit a substitute
    pub hit_substitute: bool,
}

impl DamageDealt {
    /// Create a new DamageDealt with default values
    pub fn new() -> Self {
        Self {
            damage: 0,
            move_category: MoveCategory::Physical,
            hit_substitute: false,
        }
    }

    /// Reset damage tracking (called at start of turn)
    pub fn reset(&mut self) {
        self.damage = 0;
        self.move_category = MoveCategory::Physical;
        self.hit_substitute = false;
    }

    /// Set damage information
    pub fn set_damage(&mut self, damage: i16, move_category: MoveCategory, hit_substitute: bool) {
        self.damage = damage;
        self.move_category = move_category;
        self.hit_substitute = hit_substitute;
    }
}

impl Default for DamageDealt {
    fn default() -> Self {
        Self::new()
    }
}

/// The main battle state containing all information about the current battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The battle format determining rules and active Pokemon count
    pub format: BattleFormat,
    /// Side one (player 1)
    pub side_one: BattleSide,
    /// Side two (player 2)
    pub side_two: BattleSide,
    /// Current weather condition
    pub weather: Weather,
    /// Weather duration (turns remaining)
    pub weather_turns_remaining: Option<u8>,
    /// Current terrain condition
    pub terrain: Terrain,
    /// Terrain duration (turns remaining)
    pub terrain_turns_remaining: Option<u8>,
    /// Current turn number
    pub turn: u32,
    /// Whether Trick Room is currently active
    pub trick_room_active: bool,
    /// Trick Room duration (turns remaining)
    pub trick_room_turns_remaining: Option<u8>,
    /// Whether Gravity is currently active
    pub gravity_active: bool,
    /// Gravity duration (turns remaining)
    pub gravity_turns_remaining: Option<u8>,
}

impl State {
    /// Create a new battle state with the specified format
    pub fn new(format: BattleFormat) -> Self {
        Self {
            format,
            side_one: BattleSide::new(),
            side_two: BattleSide::new(),
            weather: Weather::None,
            weather_turns_remaining: None,
            terrain: Terrain::None,
            terrain_turns_remaining: None,
            turn: 1,
            trick_room_active: false,
            trick_room_turns_remaining: None,
            gravity_active: false,
            gravity_turns_remaining: None,
        }
    }

    /// Create a new battle state with teams from random team data
    pub fn new_with_teams(
        format: BattleFormat,
        team_one: Vec<crate::data::RandomPokemonSet>,
        team_two: Vec<crate::data::RandomPokemonSet>,
    ) -> Self {
        let mut state = Self::new(format.clone());
        
        // Create factories for proper move and Pokemon data
        let move_factory = match crate::data::ps_move_factory::PSMoveFactory::new() {
            Ok(factory) => factory,
            Err(e) => {
                eprintln!("Warning: Failed to create move factory: {}. Using placeholder moves.", e);
                // Create a basic factory that will use fallbacks
                crate::data::ps_move_factory::PSMoveFactory::new().unwrap_or_else(|_| {
                    panic!("Could not create fallback move factory")
                })
            }
        };
        
        let pokemon_factory = match crate::data::ps_pokemon_factory::PSPokemonFactory::new() {
            Ok(factory) => factory,
            Err(e) => {
                eprintln!("Warning: Failed to create Pokemon factory: {}. Using placeholder Pokemon data.", e);
                // Create a basic factory that will use fallbacks
                crate::data::ps_pokemon_factory::PSPokemonFactory::new().unwrap_or_else(|_| {
                    panic!("Could not create fallback Pokemon factory")
                })
            }
        };
        
        // Convert and add Pokemon to each side
        for pokemon_set in team_one {
            let pokemon = pokemon_set.to_battle_pokemon(&move_factory, &pokemon_factory);
            state.side_one.add_pokemon(pokemon);
        }
        
        for pokemon_set in team_two {
            let pokemon = pokemon_set.to_battle_pokemon(&move_factory, &pokemon_factory);
            state.side_two.add_pokemon(pokemon);
        }
        
        // Set initial active Pokemon based on format
        let active_count = format.active_pokemon_count();
        for slot in 0..active_count {
            if slot < state.side_one.pokemon.len() {
                state.side_one.set_active_pokemon_at_slot(slot, Some(slot));
            }
            if slot < state.side_two.pokemon.len() {
                state.side_two.set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        state
    }

    /// Serialize the battle state to a compact string format
    /// Format: format/side1/side2/weather/terrain/turn/trick_room/gravity
    pub fn serialize(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}/{}/{}",
            self.format.serialize(),
            self.side_one.serialize(),
            self.side_two.serialize(),
            self.serialize_weather(),
            self.serialize_terrain(),
            self.turn,
            self.serialize_trick_room(),
            self.serialize_gravity()
        )
    }

    /// Deserialize a battle state from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let parts: Vec<&str> = serialized.split('/').collect();
        if parts.len() < 8 {
            return Err(format!("Invalid state format: expected 8 parts, got {}", parts.len()));
        }

        let format = BattleFormat::deserialize(parts[0])?;
        let side_one = BattleSide::deserialize(parts[1])?;
        let side_two = BattleSide::deserialize(parts[2])?;
        let (weather, weather_turns_remaining) = Self::deserialize_weather(parts[3])?;
        let (terrain, terrain_turns_remaining) = Self::deserialize_terrain(parts[4])?;
        let turn = parts[5].parse::<u32>()
            .map_err(|_| format!("Invalid turn number: {}", parts[5]))?;
        let (trick_room_active, trick_room_turns_remaining) = Self::deserialize_trick_room(parts[6])?;
        let (gravity_active, gravity_turns_remaining) = Self::deserialize_gravity(parts[7])?;

        Ok(Self {
            format,
            side_one,
            side_two,
            weather,
            weather_turns_remaining,
            terrain,
            terrain_turns_remaining,
            turn,
            trick_room_active,
            trick_room_turns_remaining,
            gravity_active,
            gravity_turns_remaining,
        })
    }

    /// Serialize weather condition
    fn serialize_weather(&self) -> String {
        match self.weather_turns_remaining {
            Some(turns) => format!("{}:{}", self.weather as u8, turns),
            None => format!("{}", self.weather as u8),
        }
    }

    /// Deserialize weather condition
    fn deserialize_weather(weather_str: &str) -> Result<(Weather, Option<u8>), String> {
        if weather_str.contains(':') {
            let parts: Vec<&str> = weather_str.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid weather format: {}", weather_str));
            }
            let weather_id = parts[0].parse::<u8>()
                .map_err(|_| format!("Invalid weather ID: {}", parts[0]))?;
            let turns = parts[1].parse::<u8>()
                .map_err(|_| format!("Invalid weather turns: {}", parts[1]))?;
            Ok((Weather::from(weather_id), Some(turns)))
        } else {
            let weather_id = weather_str.parse::<u8>()
                .map_err(|_| format!("Invalid weather ID: {}", weather_str))?;
            Ok((Weather::from(weather_id), None))
        }
    }

    /// Serialize terrain condition
    fn serialize_terrain(&self) -> String {
        match self.terrain_turns_remaining {
            Some(turns) => format!("{}:{}", self.terrain as u8, turns),
            None => format!("{}", self.terrain as u8),
        }
    }

    /// Deserialize terrain condition
    fn deserialize_terrain(terrain_str: &str) -> Result<(Terrain, Option<u8>), String> {
        if terrain_str.contains(':') {
            let parts: Vec<&str> = terrain_str.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid terrain format: {}", terrain_str));
            }
            let terrain_id = parts[0].parse::<u8>()
                .map_err(|_| format!("Invalid terrain ID: {}", parts[0]))?;
            let turns = parts[1].parse::<u8>()
                .map_err(|_| format!("Invalid terrain turns: {}", parts[1]))?;
            Ok((Terrain::from(terrain_id), Some(turns)))
        } else {
            let terrain_id = terrain_str.parse::<u8>()
                .map_err(|_| format!("Invalid terrain ID: {}", terrain_str))?;
            Ok((Terrain::from(terrain_id), None))
        }
    }

    /// Serialize trick room state
    fn serialize_trick_room(&self) -> String {
        if self.trick_room_active {
            match self.trick_room_turns_remaining {
                Some(turns) => format!("1:{}", turns),
                None => "1".to_string(),
            }
        } else {
            "0".to_string()
        }
    }

    /// Deserialize trick room state
    fn deserialize_trick_room(trick_room_str: &str) -> Result<(bool, Option<u8>), String> {
        if trick_room_str == "0" {
            Ok((false, None))
        } else if trick_room_str == "1" {
            Ok((true, None))
        } else if trick_room_str.starts_with("1:") {
            let turns_str = &trick_room_str[2..];
            let turns = turns_str.parse::<u8>()
                .map_err(|_| format!("Invalid trick room turns: {}", turns_str))?;
            Ok((true, Some(turns)))
        } else {
            Err(format!("Invalid trick room format: {}", trick_room_str))
        }
    }

    /// Serialize gravity condition
    fn serialize_gravity(&self) -> String {
        if self.gravity_active {
            match self.gravity_turns_remaining {
                Some(turns) => format!("1:{}", turns),
                None => "1".to_string(),
            }
        } else {
            "0".to_string()
        }
    }

    /// Deserialize gravity condition
    fn deserialize_gravity(gravity_str: &str) -> Result<(bool, Option<u8>), String> {
        if gravity_str == "0" {
            Ok((false, None))
        } else if gravity_str == "1" {
            Ok((true, None))
        } else if gravity_str.starts_with("1:") {
            let turns_str = &gravity_str[2..];
            let turns = turns_str.parse::<u8>()
                .map_err(|_| format!("Invalid gravity turns: {}", turns_str))?;
            Ok((true, Some(turns)))
        } else {
            Err(format!("Invalid gravity format: {}", gravity_str))
        }
    }

    /// Get a reference to the specified side
    pub fn get_side(&self, side_ref: SideReference) -> &BattleSide {
        match side_ref {
            SideReference::SideOne => &self.side_one,
            SideReference::SideTwo => &self.side_two,
        }
    }

    /// Get a mutable reference to the specified side
    pub fn get_side_mut(&mut self, side_ref: SideReference) -> &mut BattleSide {
        match side_ref {
            SideReference::SideOne => &mut self.side_one,
            SideReference::SideTwo => &mut self.side_two,
        }
    }

    /// Get the Pokemon at the specified position
    pub fn get_pokemon_at_position(&self, position: BattlePosition) -> Option<&Pokemon> {
        let side = self.get_side(position.side);
        side.get_active_pokemon_at_slot(position.slot)
    }

    /// Get a mutable reference to the Pokemon at the specified position
    pub fn get_pokemon_at_position_mut(&mut self, position: BattlePosition) -> Option<&mut Pokemon> {
        let side = self.get_side_mut(position.side);
        side.get_active_pokemon_at_slot_mut(position.slot)
    }

    /// Check if a position is currently active (has a Pokemon)
    pub fn is_position_active(&self, position: BattlePosition) -> bool {
        position.is_valid_for_format(&self.format)
            && self.get_pokemon_at_position(position).is_some()
    }

    /// Get all active Pokemon positions
    pub fn get_active_positions(&self) -> Vec<BattlePosition> {
        let mut positions = Vec::new();
        for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..self.format.active_pokemon_count() {
                let position = BattlePosition::new(side_ref, slot);
                if self.is_position_active(position) {
                    positions.push(position);
                }
            }
        }
        positions
    }

    /// Get the generation mechanics for this battle
    pub fn get_generation_mechanics(&self) -> crate::generation::GenerationMechanics {
        self.format.generation.get_mechanics()
    }

    /// Get the generation for this battle
    pub fn get_generation(&self) -> crate::generation::Generation {
        self.format.generation
    }

    /// Check if a generation feature is available in this battle
    pub fn has_generation_feature(&self, feature: crate::generation::GenerationFeature) -> bool {
        use crate::generation::GenerationBattleMechanics;
        self.get_generation_mechanics().has_feature(feature)
    }

    /// Check if the battle is over
    pub fn is_battle_over(&self) -> bool {
        self.side_one.is_defeated() || self.side_two.is_defeated()
    }

    /// Get the winner if the battle is over
    pub fn get_winner(&self) -> Option<SideReference> {
        if self.side_one.is_defeated() && !self.side_two.is_defeated() {
            Some(SideReference::SideTwo)
        } else if self.side_two.is_defeated() && !self.side_one.is_defeated() {
            Some(SideReference::SideOne)
        } else {
            None
        }
    }

    /// Apply a vector of instructions to the battle state
    pub fn apply_instructions(&mut self, instructions: &[crate::core::instruction::Instruction]) {
        for instruction in instructions {
            self.apply_instruction(instruction);
        }
    }

    /// Apply a single instruction to the battle state
    pub fn apply_instruction(&mut self, instruction: &crate::core::instruction::Instruction) {
        use crate::core::instruction::Instruction;
        
        match instruction {
            Instruction::PositionDamage(instr) => {
                self.apply_position_damage(instr.target_position, instr.damage_amount);
            }
            Instruction::PositionHeal(instr) => {
                self.apply_position_heal(instr.target_position, instr.heal_amount);
            }
            Instruction::ApplyStatus(instr) => {
                self.apply_status(instr.target_position, instr.status);
            }
            Instruction::RemoveStatus(instr) => {
                self.remove_status(instr.target_position);
            }
            Instruction::BoostStats(instr) => {
                self.apply_stat_boosts(instr.target_position, &instr.stat_boosts);
            }
            Instruction::ApplyVolatileStatus(instr) => {
                self.apply_volatile_status(instr.target_position, instr.volatile_status, instr.duration);
            }
            Instruction::RemoveVolatileStatus(instr) => {
                self.remove_volatile_status(instr.target_position, instr.volatile_status);
            }
            Instruction::ChangeVolatileStatusDuration(instr) => {
                self.change_volatile_status_duration(instr.target_position, instr.volatile_status, instr.duration_change);
            }
            Instruction::ChangeStatusDuration(instr) => {
                self.change_status_duration(instr.target_position, instr.duration_change);
            }
            Instruction::ChangeWeather(instr) => {
                self.change_weather(instr.weather, instr.duration);
            }
            Instruction::ChangeTerrain(instr) => {
                self.change_terrain(instr.terrain, instr.duration);
            }
            Instruction::SwitchPokemon(instr) => {
                self.switch_pokemon(instr.position, instr.previous_index, instr.next_index);
            }
            Instruction::ApplySideCondition(instr) => {
                self.apply_side_condition(instr.side, instr.condition, instr.duration);
            }
            Instruction::RemoveSideCondition(instr) => {
                self.remove_side_condition(instr.side, instr.condition);
            }
            Instruction::DecrementSideConditionDuration(instr) => {
                self.decrement_side_condition_duration(instr.side, instr.condition, instr.amount);
            }
            Instruction::DecrementWeatherTurns => {
                self.decrement_weather_turns();
            }
            Instruction::DecrementTerrainTurns => {
                self.decrement_terrain_turns();
            }
            
            // Move Management Instructions
            Instruction::DisableMove(instr) => {
                self.disable_move(instr.target_position, instr.move_index, instr.duration);
            }
            Instruction::EnableMove(instr) => {
                self.enable_move(instr.target_position, instr.move_index);
            }
            Instruction::DecrementPP(instr) => {
                self.decrement_pp(instr.target_position, instr.move_index, instr.amount);
            }
            Instruction::SetLastUsedMove(instr) => {
                self.set_last_used_move(instr.target_position, &instr.move_name, instr.move_id);
            }
            Instruction::RestoreLastUsedMove(instr) => {
                self.restore_last_used_move(instr.target_position, &instr.move_name);
            }
            
            // Pokemon Attribute Instructions
            Instruction::ChangeAbility(instr) => {
                self.change_ability(instr.target_position, &instr.new_ability);
            }
            Instruction::ToggleAbility(instr) => {
                self.toggle_ability(instr.target_position, instr.enabled);
            }
            Instruction::ChangeItem(instr) => {
                self.change_item(instr.target_position, instr.new_item.as_deref());
            }
            Instruction::RemoveItem(instr) => {
                self.remove_item(instr.target_position);
            }
            Instruction::GiveItem(instr) => {
                let _previous_item = self.give_item(instr.target_position, &instr.item);
                // Note: The previous_item should ideally be stored in the instruction for proper undo
            }
            Instruction::Faint(instr) => {
                let _previous_hp = self.faint_pokemon(instr.target_position);
                // Note: The previous_hp should ideally match instr.previous_hp for validation
            }
            Instruction::ToggleGravity(instr) => {
                self.toggle_gravity(instr.active, instr.duration);
            }
            Instruction::ChangeType(instr) => {
                self.change_type(instr.target_position, &instr.new_types);
            }
            Instruction::FormeChange(instr) => {
                self.forme_change(instr.target_position, &instr.new_forme);
            }
            Instruction::ToggleTerastallized(instr) => {
                self.toggle_terastallized(instr.target_position, instr.tera_type.as_deref());
            }
            
            // Advanced Field Effect Instructions
            Instruction::ToggleTrickRoom(instr) => {
                self.toggle_trick_room(instr.active, instr.duration);
            }
            Instruction::DecrementTrickRoomTurns => {
                self.decrement_trick_room_turns();
            }
            
            // Special Mechanic Instructions
            Instruction::SetWish(instr) => {
                self.set_wish(instr.target_position, instr.heal_amount, instr.turns_remaining);
            }
            Instruction::DecrementWish(instr) => {
                self.decrement_wish(instr.target_position);
            }
            Instruction::SetFutureSight(instr) => {
                self.set_future_sight(instr.target_position, instr.attacker_position, instr.damage_amount, instr.turns_remaining, &instr.move_name);
            }
            Instruction::DecrementFutureSight(instr) => {
                self.decrement_future_sight(instr.target_position);
            }
            Instruction::ChangeSubstituteHealth(instr) => {
                self.change_substitute_health(instr.target_position, instr.health_change);
            }
            
            // Sleep/Rest System Instructions
            Instruction::SetRestTurns(instr) => {
                self.set_rest_turns(instr.target_position, instr.turns);
            }
            Instruction::SetSleepTurns(instr) => {
                self.set_sleep_turns(instr.target_position, instr.turns);
            }
            Instruction::DecrementRestTurns(instr) => {
                self.decrement_rest_turns(instr.target_position);
            }
            
            // Battle State Management Instructions
            Instruction::ToggleBatonPassing(instr) => {
                self.toggle_baton_passing(instr.target_position, instr.active);
            }
            Instruction::ToggleShedTailing(instr) => {
                self.toggle_shed_tailing(instr.target_position, instr.active);
            }
            Instruction::ToggleSideOneForceSwitch => {
                self.toggle_side_force_switch(crate::core::battle_format::SideReference::SideOne);
            }
            Instruction::ToggleSideTwoForceSwitch => {
                self.toggle_side_force_switch(crate::core::battle_format::SideReference::SideTwo);
            }
            
            // Raw Stat Modification Instructions
            Instruction::ChangeAttack(instr) => {
                self.change_raw_attack(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeDefense(instr) => {
                self.change_raw_defense(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpecialAttack(instr) => {
                self.change_raw_special_attack(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpecialDefense(instr) => {
                self.change_raw_special_defense(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpeed(instr) => {
                self.change_raw_speed(instr.target_position, instr.stat_change);
            }
            
            // Switch Move Management Instructions
            Instruction::SetSideOneMoveSecondSwitchOutMove(instr) => {
                self.set_second_move_switch_out(crate::core::battle_format::SideReference::SideOne, &instr.new_choice);
            }
            Instruction::SetSideTwoMoveSecondSwitchOutMove(instr) => {
                self.set_second_move_switch_out(crate::core::battle_format::SideReference::SideTwo, &instr.new_choice);
            }
            
            // Damage Tracking Instructions
            Instruction::ChangeDamageDealt(instr) => {
                self.change_damage_dealt(instr.target_position, instr.damage_amount);
            }
            Instruction::ChangeDamageDealtMoveCategory(instr) => {
                self.change_damage_dealt_move_category(instr.target_position, instr.move_category);
            }
            Instruction::ToggleDamageDealtHitSubstitute(instr) => {
                self.toggle_damage_dealt_hit_substitute(instr.target_position, instr.hit_substitute);
            }
            
            // Multi-Target Instructions
            Instruction::MultiTargetDamage(instr) => {
                self.apply_multi_target_damage(&instr.target_damages);
            }
        }
    }

    /// Reverse multiple instructions in LIFO order
    pub fn reverse_instructions(&mut self, instructions: &[crate::core::instruction::Instruction]) {
        for instruction in instructions.iter().rev() {
            self.reverse_instruction(instruction);
        }
    }

    /// Reverse a single instruction to undo its effects
    pub fn reverse_instruction(&mut self, instruction: &crate::core::instruction::Instruction) {
        use crate::core::instruction::Instruction;
        
        match instruction {
            Instruction::PositionDamage(instr) => {
                if let Some(previous_hp) = instr.previous_hp {
                    self.set_pokemon_hp(instr.target_position, previous_hp);
                }
            }
            Instruction::PositionHeal(instr) => {
                if let Some(previous_hp) = instr.previous_hp {
                    self.set_pokemon_hp(instr.target_position, previous_hp);
                }
            }
            Instruction::ApplyStatus(instr) => {
                if let Some(previous_status) = instr.previous_status {
                    self.set_pokemon_status(instr.target_position, previous_status, instr.previous_status_duration.unwrap_or(None));
                }
            }
            Instruction::RemoveStatus(instr) => {
                if let Some(previous_status) = instr.previous_status {
                    self.set_pokemon_status(instr.target_position, previous_status, instr.previous_status_duration.unwrap_or(None));
                }
            }
            Instruction::BoostStats(instr) => {
                if let Some(ref previous_boosts) = instr.previous_boosts {
                    self.set_stat_boosts(instr.target_position, previous_boosts);
                }
            }
            Instruction::ApplyVolatileStatus(instr) => {
                self.reverse_apply_volatile_status(instr.target_position, instr.volatile_status, instr.duration);
            }
            Instruction::RemoveVolatileStatus(instr) => {
                self.reverse_remove_volatile_status(instr.target_position, instr.volatile_status);
            }
            Instruction::ChangeVolatileStatusDuration(instr) => {
                self.reverse_change_volatile_status_duration(instr.target_position, instr.volatile_status, instr.duration_change);
            }
            Instruction::ChangeStatusDuration(instr) => {
                self.reverse_change_status_duration(instr.target_position, instr.duration_change);
            }
            Instruction::ChangeWeather(instr) => {
                if let (Some(previous_weather), Some(previous_duration)) = (instr.previous_weather, instr.previous_duration) {
                    self.weather = previous_weather;
                    self.weather_turns_remaining = previous_duration;
                }
            }
            Instruction::ChangeTerrain(instr) => {
                if let (Some(previous_terrain), Some(previous_duration)) = (instr.previous_terrain, instr.previous_duration) {
                    self.terrain = previous_terrain;
                    self.terrain_turns_remaining = previous_duration;
                }
            }
            Instruction::SwitchPokemon(instr) => {
                // Reverse the switch by switching back
                self.switch_pokemon(instr.position, instr.next_index, instr.previous_index);
            }
            Instruction::ApplySideCondition(instr) => {
                self.reverse_apply_side_condition(instr.side, instr.condition);
            }
            Instruction::RemoveSideCondition(instr) => {
                self.reverse_remove_side_condition(instr.side, instr.condition);
            }
            Instruction::DecrementSideConditionDuration(instr) => {
                self.reverse_decrement_side_condition_duration(instr.side, instr.condition, instr.amount);
            }
            Instruction::DecrementWeatherTurns => {
                self.reverse_decrement_weather_turns();
            }
            Instruction::DecrementTerrainTurns => {
                self.reverse_decrement_terrain_turns();
            }
            
            // Move Management Instructions (simplified reversal - just apply opposite)
            Instruction::DisableMove(instr) => {
                self.enable_move(instr.target_position, instr.move_index);
            }
            Instruction::EnableMove(instr) => {
                self.disable_move(instr.target_position, instr.move_index, None);
            }
            Instruction::DecrementPP(instr) => {
                self.increment_pp(instr.target_position, instr.move_index, instr.amount);
            }
            Instruction::SetLastUsedMove(_instr) => {
                // Last used move reversal would need previous state stored
            }
            
            // Pokemon Attribute Instructions (would need previous values stored)
            Instruction::ChangeAbility(_instr) => {
                // Ability change reversal would need previous ability stored
            }
            Instruction::ChangeItem(_instr) => {
                // Item change reversal would need previous item stored
            }
            Instruction::ChangeType(_instr) => {
                // Type change reversal would need previous types stored
            }
            Instruction::FormeChange(_instr) => {
                // Forme change reversal would need previous forme stored
            }
            Instruction::ToggleTerastallized(instr) => {
                // Toggle terastallization (reverse the toggle)
                self.toggle_terastallized(instr.target_position, instr.tera_type.as_deref());
            }
            
            // Advanced Field Effect Instructions
            Instruction::ToggleTrickRoom(instr) => {
                // Reverse trick room toggle
                self.toggle_trick_room(!instr.active, None);
            }
            Instruction::DecrementTrickRoomTurns => {
                self.reverse_decrement_trick_room_turns();
            }
            
            // Special Mechanic Instructions (simplified - would need full previous state)
            Instruction::SetWish(_instr) => {
                // Wish reversal would need previous wish state
            }
            Instruction::DecrementWish(_instr) => {
                // Wish decrement reversal would need previous wish state
            }
            Instruction::SetFutureSight(_instr) => {
                // Future sight reversal would need previous state
            }
            Instruction::DecrementFutureSight(_instr) => {
                // Future sight decrement reversal would need previous state
            }
            Instruction::ChangeSubstituteHealth(_instr) => {
                // Substitute health reversal would need previous health
            }
            
            // Sleep/Rest System Instructions
            Instruction::SetRestTurns(_instr) => {
                // Rest turns reversal would need previous status
            }
            Instruction::SetSleepTurns(_instr) => {
                // Sleep turns reversal would need previous status
            }
            Instruction::DecrementRestTurns(_instr) => {
                // Rest decrement reversal would need previous status
            }
            
            // Battle State Management Instructions
            Instruction::ToggleBatonPassing(instr) => {
                self.toggle_baton_passing(instr.target_position, !instr.active);
            }
            Instruction::ToggleShedTailing(instr) => {
                self.toggle_shed_tailing(instr.target_position, !instr.active);
            }
            Instruction::ToggleSideOneForceSwitch => {
                // Side force switch reversal would need previous state
            }
            Instruction::ToggleSideTwoForceSwitch => {
                // Side force switch reversal would need previous state
            }
            
            // Raw Stat Modification Instructions
            Instruction::ChangeAttack(instr) => {
                self.reverse_change_attack(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeDefense(instr) => {
                self.reverse_change_defense(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpecialAttack(instr) => {
                self.reverse_change_special_attack(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpecialDefense(instr) => {
                self.reverse_change_special_defense(instr.target_position, instr.stat_change);
            }
            Instruction::ChangeSpeed(instr) => {
                self.reverse_change_speed(instr.target_position, instr.stat_change);
            }
            
            // Switch Move Management Instructions
            Instruction::SetSideOneMoveSecondSwitchOutMove(_instr) => {
                // Switch move reversal would need previous choice
            }
            Instruction::SetSideTwoMoveSecondSwitchOutMove(_instr) => {
                // Switch move reversal would need previous choice
            }
            
            // Damage Tracking Instructions
            Instruction::ChangeDamageDealt(instr) => {
                self.reverse_change_damage_dealt(instr.target_position, instr.damage_amount);
            }
            Instruction::ChangeDamageDealtMoveCategory(_instr) => {
                // Category reversal would need previous category
            }
            Instruction::ToggleDamageDealtHitSubstitute(instr) => {
                self.toggle_damage_dealt_hit_substitute(instr.target_position, !instr.hit_substitute);
            }
            
            // Multi-Target Instructions
            Instruction::MultiTargetDamage(instr) => {
                self.reverse_multi_target_damage(&instr.target_damages);
            }
            
            // Missing instruction patterns (TODO: implement proper reversal logic)
            Instruction::RestoreLastUsedMove(instr) => {
                // Reverse of restoring a move is to disable it again
                // Apply Disable status to reverse the restoration
                if let Some(pokemon) = self.get_pokemon_at_position_mut(instr.target_position) {
                    pokemon.volatile_statuses.insert(VolatileStatus::Disable);
                }
            }
            Instruction::ToggleAbility(instr) => {
                // Reverse the toggle operation
                self.toggle_ability(instr.target_position, !instr.enabled);
            }
            Instruction::RemoveItem(instr) => {
                // Restore the previous item if it existed
                if let Some(ref previous_item) = instr.previous_item {
                    let _restored_previous = self.give_item(instr.target_position, previous_item);
                }
            }
            Instruction::GiveItem(instr) => {
                // Restore the previous item state
                if let Some(ref previous_item) = instr.previous_item {
                    let _restored_previous = self.give_item(instr.target_position, previous_item);
                } else {
                    // No previous item, so remove the current item
                    self.remove_item(instr.target_position);
                }
            }
            Instruction::Faint(instr) => {
                // Restore the previous HP
                if let Some(pokemon) = self.get_pokemon_at_position_mut(instr.target_position) {
                    pokemon.hp = instr.previous_hp;
                }
            }
            Instruction::ToggleGravity(instr) => {
                // Reverse the gravity toggle
                self.toggle_gravity(!instr.active, instr.duration);
            }
        }
    }

    /// Apply damage to a Pokemon at the specified position
    fn apply_position_damage(&mut self, position: BattlePosition, damage: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.hp = (pokemon.hp - damage).max(0);
        }
    }

    /// Apply healing to a Pokemon at the specified position
    fn apply_position_heal(&mut self, position: BattlePosition, heal_amount: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.hp = (pokemon.hp + heal_amount).min(pokemon.max_hp);
        }
    }

    /// Apply a status condition to a Pokemon at the specified position
    fn apply_status(&mut self, position: BattlePosition, status: crate::core::instruction::PokemonStatus) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.status = status;
            // Set default duration for certain statuses
            pokemon.status_duration = match status {
                crate::core::instruction::PokemonStatus::Sleep => Some(1), // Will be randomized in actual implementation
                crate::core::instruction::PokemonStatus::Freeze => Some(1),
                _ => None,
            };
        }
    }

    /// Remove status condition from a Pokemon at the specified position
    fn remove_status(&mut self, position: BattlePosition) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.status = crate::core::instruction::PokemonStatus::None;
            pokemon.status_duration = None;
        }
    }

    /// Apply stat boosts to a Pokemon at the specified position
    fn apply_stat_boosts(&mut self, position: BattlePosition, boosts: &HashMap<crate::core::instruction::Stat, i8>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            for (stat, boost) in boosts {
                let current = pokemon.stat_boosts.get(stat).unwrap_or(&0);
                let new_boost = (current + boost).clamp(-6, 6);
                pokemon.stat_boosts.insert(*stat, new_boost);
            }
        }
    }

    /// Apply a volatile status to a Pokemon at the specified position
    fn apply_volatile_status(
        &mut self,
        position: BattlePosition,
        volatile_status: crate::core::instruction::VolatileStatus,
        duration: Option<u8>,
    ) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.volatile_statuses.insert(volatile_status);
            if let Some(dur) = duration {
                pokemon.volatile_status_durations.insert(volatile_status, dur);
            }
        }
    }

    /// Remove a volatile status from a Pokemon at the specified position
    fn remove_volatile_status(
        &mut self,
        position: BattlePosition,
        volatile_status: crate::core::instruction::VolatileStatus,
    ) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.volatile_statuses.remove(&volatile_status);
            pokemon.volatile_status_durations.remove(&volatile_status);
        }
    }

    /// Change volatile status duration for a Pokemon at the specified position
    fn change_volatile_status_duration(
        &mut self,
        position: BattlePosition,
        volatile_status: crate::core::instruction::VolatileStatus,
        duration_change: i8,
    ) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if let Some(current_duration) = pokemon.volatile_status_durations.get_mut(&volatile_status) {
                let new_duration = (*current_duration as i8 + duration_change).max(0) as u8;
                if new_duration > 0 {
                    *current_duration = new_duration;
                } else {
                    // If duration reaches 0, remove the volatile status
                    pokemon.volatile_statuses.remove(&volatile_status);
                    pokemon.volatile_status_durations.remove(&volatile_status);
                }
            }
        }
    }

    /// Change status condition duration for a Pokemon at the specified position
    fn change_status_duration(
        &mut self,
        position: BattlePosition,
        duration_change: i8,
    ) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if let Some(current_duration) = pokemon.status_duration {
                let new_duration = (current_duration as i8 + duration_change).max(0) as u8;
                if new_duration > 0 {
                    pokemon.status_duration = Some(new_duration);
                } else {
                    // If duration reaches 0, remove the status condition
                    pokemon.status = crate::core::instruction::PokemonStatus::None;
                    pokemon.status_duration = None;
                }
            }
        }
    }

    /// Change the weather condition
    fn change_weather(&mut self, weather: crate::core::instruction::Weather, duration: Option<u8>) {
        self.weather = weather;
        self.weather_turns_remaining = duration;
    }

    /// Change the terrain condition
    fn change_terrain(&mut self, terrain: crate::core::instruction::Terrain, duration: Option<u8>) {
        self.terrain = terrain;
        self.terrain_turns_remaining = duration;
    }

    /// Switch a Pokemon at the specified position
    fn switch_pokemon(&mut self, position: BattlePosition, previous_index: usize, next_index: usize) {
        if let Some(side) = self.get_side_mut_ref(position.side) {
            // Validate that the previous index matches the current active Pokemon
            if let Some(&Some(current_index)) = side.active_pokemon_indices.get(position.slot) {
                if current_index != previous_index {
                    eprintln!("Warning: Switch instruction previous_index {} doesn't match current active Pokemon {}", previous_index, current_index);
                }
            }
            
            // Perform the switch
            side.set_active_pokemon_at_slot(position.slot, Some(next_index));
        }
    }

    /// Apply a side condition
    fn apply_side_condition(
        &mut self,
        side_ref: crate::core::battle_format::SideReference,
        condition: crate::core::instruction::SideCondition,
        duration: Option<u8>,
    ) {
        let side = self.get_side_mut(side_ref);
        let new_value = match duration {
            Some(dur) => dur,
            None => {
                // Increment for stackable conditions like Spikes, Toxic Spikes
                let current_value = side.side_conditions.get(&condition).unwrap_or(&0);
                current_value + 1
            }
        };
        
        // Apply condition-specific limits
        let max_value = match condition {
            crate::core::instruction::SideCondition::Spikes => 3,
            crate::core::instruction::SideCondition::ToxicSpikes => 2,
            _ => 255, // Most conditions don't have limits or use raw duration values
        };
        
        side.side_conditions.insert(condition, new_value.min(max_value));
    }

    /// Remove a side condition
    fn remove_side_condition(
        &mut self,
        side_ref: crate::core::battle_format::SideReference,
        condition: crate::core::instruction::SideCondition,
    ) {
        let side = self.get_side_mut(side_ref);
        side.side_conditions.remove(&condition);
    }

    /// Decrement side condition duration
    fn decrement_side_condition_duration(
        &mut self,
        side_ref: crate::core::battle_format::SideReference,
        condition: crate::core::instruction::SideCondition,
        amount: u8,
    ) {
        let should_remove = {
            let side = self.get_side_mut(side_ref);
            if let Some(current_value) = side.side_conditions.get_mut(&condition) {
                if *current_value > amount {
                    *current_value -= amount;
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };
        
        if should_remove {
            let side = self.get_side_mut(side_ref);
            side.side_conditions.remove(&condition);
        }
    }

    /// Decrement weather turns remaining
    fn decrement_weather_turns(&mut self) {
        if let Some(turns) = self.weather_turns_remaining {
            if turns > 1 {
                self.weather_turns_remaining = Some(turns - 1);
            } else {
                self.weather = crate::core::instruction::Weather::None;
                self.weather_turns_remaining = None;
            }
        }
    }

    /// Decrement terrain turns remaining
    fn decrement_terrain_turns(&mut self) {
        if let Some(turns) = self.terrain_turns_remaining {
            if turns > 1 {
                self.terrain_turns_remaining = Some(turns - 1);
            } else {
                self.terrain = crate::core::instruction::Terrain::None;
                self.terrain_turns_remaining = None;
            }
        }
    }

    /// Helper method to get a mutable reference to a side by reference
    fn get_side_mut_ref(&mut self, side_ref: crate::core::battle_format::SideReference) -> Option<&mut BattleSide> {
        match side_ref {
            crate::core::battle_format::SideReference::SideOne => Some(&mut self.side_one),
            crate::core::battle_format::SideReference::SideTwo => Some(&mut self.side_two),
        }
    }

    // ========== Move Management Implementation ==========

    /// Disable a move at the specified position
    fn disable_move(&mut self, position: BattlePosition, move_index: u8, duration: Option<u8>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            let move_idx = match move_index {
                0 => MoveIndex::M0,
                1 => MoveIndex::M1,
                2 => MoveIndex::M2,
                3 => MoveIndex::M3,
                _ => return,
            };
            
            if let Some(move_data) = pokemon.moves.get_mut(&move_idx) {
                // For now, we'll track disabled status in volatile statuses
                // In a full implementation, moves would have a disabled field
                pokemon.volatile_statuses.insert(VolatileStatus::Disable);
                if let Some(dur) = duration {
                    pokemon.volatile_status_durations.insert(VolatileStatus::Disable, dur);
                }
            }
        }
    }

    /// Enable a move at the specified position
    fn enable_move(&mut self, position: BattlePosition, _move_index: u8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            // Remove disable status - in a full implementation this would enable specific moves
            pokemon.volatile_statuses.remove(&VolatileStatus::Disable);
            pokemon.volatile_status_durations.remove(&VolatileStatus::Disable);
        }
    }

    /// Decrement PP of a move at the specified position
    fn decrement_pp(&mut self, position: BattlePosition, move_index: u8, amount: u8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            let move_idx = match move_index {
                0 => MoveIndex::M0,
                1 => MoveIndex::M1,
                2 => MoveIndex::M2,
                3 => MoveIndex::M3,
                _ => return,
            };
            
            if let Some(move_data) = pokemon.moves.get_mut(&move_idx) {
                move_data.pp = move_data.pp.saturating_sub(amount);
            }
        }
    }

    /// Set the last used move at the specified position
    fn set_last_used_move(&mut self, _position: BattlePosition, _move_name: &str, _move_id: Option<u16>) {
        // In tapu-simu, we'd need to add a last_used_move field to Pokemon or BattleSide
        // For now, this is a placeholder that could be extended
        
        // Store in side conditions or add new field - this is a simplified implementation
        // A full implementation would track this per Pokemon or per side
    }

    // ========== Pokemon Attribute Implementation ==========

    /// Change ability at the specified position
    fn change_ability(&mut self, position: BattlePosition, new_ability: &str) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.ability = new_ability.to_string();
        }
    }

    /// Faint a Pokemon at the specified position
    fn faint_pokemon(&mut self, position: BattlePosition) -> i16 {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            let previous_hp = pokemon.hp;
            pokemon.hp = 0;
            previous_hp
        } else {
            0
        }
    }

    /// Toggle ability state at the specified position
    fn toggle_ability(&mut self, position: BattlePosition, enabled: bool) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if enabled {
                // Enable ability by removing GastroAcid volatile status
                pokemon.volatile_statuses.remove(&VolatileStatus::GastroAcid);
                pokemon.volatile_status_durations.remove(&VolatileStatus::GastroAcid);
            } else {
                // Disable ability by applying GastroAcid volatile status
                pokemon.volatile_statuses.insert(VolatileStatus::GastroAcid);
                // GastroAcid typically lasts until the Pokemon switches out (no duration)
            }
        }
    }

    /// Restore last used move (typically removes Disable status)
    fn restore_last_used_move(&mut self, position: BattlePosition, _move_name: &str) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            // Remove Disable volatile status to restore move usage
            // In a full implementation, this would restore the specific move
            // For now, we remove the general Disable status
            pokemon.volatile_statuses.remove(&VolatileStatus::Disable);
            pokemon.volatile_status_durations.remove(&VolatileStatus::Disable);
        }
    }

    /// Change item at the specified position
    fn change_item(&mut self, position: BattlePosition, new_item: Option<&str>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.item = new_item.map(|item| item.to_string());
        }
    }

    /// Remove item from Pokemon at the specified position
    fn remove_item(&mut self, position: BattlePosition) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.item = None;
        }
    }

    /// Give item to Pokemon at the specified position
    fn give_item(&mut self, position: BattlePosition, item: &str) -> Option<String> {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            let previous_item = pokemon.item.clone();
            pokemon.item = Some(item.to_string());
            previous_item
        } else {
            None
        }
    }

    /// Change types at the specified position
    fn change_type(&mut self, position: BattlePosition, new_types: &[String]) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.types = new_types.to_vec();
        }
    }

    /// Change forme at the specified position
    fn forme_change(&mut self, position: BattlePosition, new_forme: &str) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            // Update species to include forme
            pokemon.species = new_forme.to_string();
        }
    }

    /// Toggle terastallization at the specified position
    fn toggle_terastallized(&mut self, position: BattlePosition, tera_type: Option<&str>) {
        // Only allow Terastallization in Gen 9+
        let is_gen9_or_later = self.format.generation.number() >= 9;
        
        if is_gen9_or_later {
            if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
                pokemon.is_terastallized = !pokemon.is_terastallized;
                if let Some(_ttype) = tera_type {
                    // Convert string to tera type - simplified
                    pokemon.tera_type = Some(crate::core::move_choice::PokemonType::Normal);
                }
            }
        }
    }

    // ========== Advanced Field Effect Implementation ==========

    /// Toggle trick room
    fn toggle_trick_room(&mut self, active: bool, duration: Option<u8>) {
        self.trick_room_active = active;
        self.trick_room_turns_remaining = duration;
    }

    /// Decrement trick room turns remaining
    fn decrement_trick_room_turns(&mut self) {
        if let Some(turns) = self.trick_room_turns_remaining {
            if turns > 1 {
                self.trick_room_turns_remaining = Some(turns - 1);
            } else {
                self.trick_room_active = false;
                self.trick_room_turns_remaining = None;
            }
        }
    }

    /// Toggle gravity
    fn toggle_gravity(&mut self, active: bool, duration: Option<u8>) {
        self.gravity_active = active;
        self.gravity_turns_remaining = duration;
    }

    /// Decrement gravity turns remaining
    fn decrement_gravity_turns(&mut self) {
        if let Some(turns) = self.gravity_turns_remaining {
            if turns > 1 {
                self.gravity_turns_remaining = Some(turns - 1);
            } else {
                self.gravity_active = false;
                self.gravity_turns_remaining = None;
            }
        }
    }

    // ========== Special Mechanic Implementation ==========

    /// Set wish healing at the specified position
    fn set_wish(&mut self, position: BattlePosition, heal_amount: i16, turns_remaining: u8) {
        let side = self.get_side_mut(position.side);
        side.wish_healing.insert(position.slot, (heal_amount, turns_remaining));
    }

    /// Decrement wish counter at the specified position
    fn decrement_wish(&mut self, position: BattlePosition) {
        let should_activate = {
            let side = self.get_side_mut(position.side);
            if let Some((_heal_amount, turns)) = side.wish_healing.get_mut(&position.slot) {
                if *turns > 1 {
                    *turns -= 1;
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };
        
        if should_activate {
            // Get heal amount before removing from map
            let heal_amount = {
                let side = self.get_side(position.side);
                if let Some((heal_amount, _turns)) = side.wish_healing.get(&position.slot) {
                    *heal_amount
                } else {
                    0
                }
            };
            
            // Apply healing
            if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
                pokemon.hp = (pokemon.hp + heal_amount).min(pokemon.max_hp);
            }
            
            // Remove the wish
            let side = self.get_side_mut(position.side);
            side.wish_healing.remove(&position.slot);
        }
    }

    /// Set future sight attack at the specified position
    fn set_future_sight(&mut self, target_position: BattlePosition, attacker_position: BattlePosition, damage_amount: i16, turns_remaining: u8, move_name: &str) {
        let side = self.get_side_mut(target_position.side);
        side.future_sight_attacks.insert(
            target_position.slot,
            (attacker_position, damage_amount, turns_remaining, move_name.to_string())
        );
    }

    /// Decrement future sight counter at the specified position
    fn decrement_future_sight(&mut self, position: BattlePosition) {
        let should_activate = {
            let side = self.get_side_mut(position.side);
            if let Some((_attacker_pos, _damage, turns, _move_name)) = side.future_sight_attacks.get_mut(&position.slot) {
                if *turns > 1 {
                    *turns -= 1;
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };
        
        if should_activate {
            // Get damage amount before removing from map
            let damage = {
                let side = self.get_side(position.side);
                if let Some((_attacker_pos, damage, _turns, _move_name)) = side.future_sight_attacks.get(&position.slot) {
                    *damage
                } else {
                    0
                }
            };
            
            // Apply damage
            if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
                pokemon.hp = (pokemon.hp - damage).max(0);
            }
            
            // Remove the future sight attack
            let side = self.get_side_mut(position.side);
            side.future_sight_attacks.remove(&position.slot);
        }
    }

    /// Change substitute health at the specified position
    fn change_substitute_health(&mut self, position: BattlePosition, health_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.substitute_health = (pokemon.substitute_health + health_change).max(0);
            
            // If substitute health reaches 0, remove substitute status
            if pokemon.substitute_health <= 0 {
                pokemon.volatile_statuses.remove(&VolatileStatus::Substitute);
            }
        }
    }

    // ========== Sleep/Rest System Implementation ==========

    /// Set rest turns at the specified position
    fn set_rest_turns(&mut self, position: BattlePosition, turns: u8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            // For Rest, apply sleep status with specific duration
            pokemon.status = crate::core::instruction::PokemonStatus::Sleep;
            pokemon.status_duration = Some(turns);
        }
    }

    /// Set sleep turns at the specified position
    fn set_sleep_turns(&mut self, position: BattlePosition, turns: u8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.status = crate::core::instruction::PokemonStatus::Sleep;
            pokemon.status_duration = Some(turns);
        }
    }

    /// Decrement rest turns at the specified position
    fn decrement_rest_turns(&mut self, position: BattlePosition) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if let Some(current_duration) = pokemon.status_duration {
                if current_duration > 1 {
                    pokemon.status_duration = Some(current_duration - 1);
                } else {
                    // Rest sleep ends
                    pokemon.status = crate::core::instruction::PokemonStatus::None;
                    pokemon.status_duration = None;
                }
            }
        }
    }

    // ========== Battle State Management Implementation ==========

    /// Toggle baton passing state at the specified position
    fn toggle_baton_passing(&mut self, position: BattlePosition, active: bool) {
        // This would require additional state tracking fields
        // For now, use volatile status as placeholder
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if active {
                pokemon.volatile_statuses.insert(VolatileStatus::LockedMove);
            } else {
                pokemon.volatile_statuses.remove(&VolatileStatus::LockedMove);
            }
        }
    }

    /// Toggle shed tailing state at the specified position
    fn toggle_shed_tailing(&mut self, position: BattlePosition, active: bool) {
        // Similar to baton passing - would need specific state tracking
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if active {
                pokemon.volatile_statuses.insert(VolatileStatus::LockedMove);
            } else {
                pokemon.volatile_statuses.remove(&VolatileStatus::LockedMove);
            }
        }
    }

    /// Toggle force switch for a side
    fn toggle_side_force_switch(&mut self, side_ref: crate::core::battle_format::SideReference) {
        // This would require additional side-level state tracking
        // For now, this is a placeholder
    }

    // ========== Raw Stat Modification Implementation ==========

    /// Change raw attack stat at the specified position
    fn change_raw_attack(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.attack = (pokemon.stats.attack + stat_change).max(1);
        }
    }

    /// Change raw defense stat at the specified position
    fn change_raw_defense(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.defense = (pokemon.stats.defense + stat_change).max(1);
        }
    }

    /// Change raw special attack stat at the specified position
    fn change_raw_special_attack(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.special_attack = (pokemon.stats.special_attack + stat_change).max(1);
        }
    }

    /// Change raw special defense stat at the specified position
    fn change_raw_special_defense(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.special_defense = (pokemon.stats.special_defense + stat_change).max(1);
        }
    }

    /// Change raw speed stat at the specified position
    fn change_raw_speed(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.speed = (pokemon.stats.speed + stat_change).max(1);
        }
    }

    // ========== Switch Move Management Implementation ==========

    /// Set second move switch out for a side
    fn set_second_move_switch_out(&mut self, side_ref: crate::core::battle_format::SideReference, new_choice: &str) {
        // This would require additional side-level state tracking for switch moves
        // For now, this is a placeholder
    }

    // ========== Damage Tracking Implementation ==========

    /// Change damage dealt at the specified position
    fn change_damage_dealt(&mut self, position: BattlePosition, damage_amount: i16) {
        let side = self.get_side_mut(position.side);
        side.damage_dealt.damage = damage_amount;
    }

    /// Change damage dealt move category at the specified position
    fn change_damage_dealt_move_category(&mut self, position: BattlePosition, move_category: crate::core::instruction::MoveCategory) {
        let side = self.get_side_mut(position.side);
        side.damage_dealt.move_category = move_category;
    }

    /// Toggle damage dealt hit substitute at the specified position
    fn toggle_damage_dealt_hit_substitute(&mut self, position: BattlePosition, hit_substitute: bool) {
        let side = self.get_side_mut(position.side);
        side.damage_dealt.hit_substitute = hit_substitute;
    }

    /// Get damage dealt information for a side
    pub fn get_damage_dealt(&self, side: SideReference) -> &DamageDealt {
        &self.get_side(side).damage_dealt
    }

    /// Reset damage tracking for all sides (called at start of turn)
    pub fn reset_damage_tracking(&mut self) {
        self.side_one.damage_dealt.reset();
        self.side_two.damage_dealt.reset();
    }

    /// Set damage tracking for a side
    pub fn set_damage_tracking(&mut self, side: SideReference, damage: i16, move_category: MoveCategory, hit_substitute: bool) {
        let battle_side = self.get_side_mut(side);
        battle_side.damage_dealt.set_damage(damage, move_category, hit_substitute);
    }

    // ========== Multi-Target Implementation ==========

    /// Apply damage to multiple positions simultaneously
    fn apply_multi_target_damage(&mut self, target_damages: &[(BattlePosition, i16)]) {
        for (position, damage) in target_damages {
            self.apply_position_damage(*position, *damage);
        }
    }

    /// Get all legal options for both sides based on current battle state
    pub fn get_all_options(&self) -> (Vec<crate::MoveChoice>, Vec<crate::MoveChoice>) {
        let side_one_options = self.get_side_options(SideReference::SideOne);
        let side_two_options = self.get_side_options(SideReference::SideTwo);
        (side_one_options, side_two_options)
    }

    /// Get legal options for a specific side
    fn get_side_options(&self, side_ref: SideReference) -> Vec<crate::MoveChoice> {
        let mut options = Vec::new();
        let side = self.get_side(side_ref);
        
        // Check if we need to force a switch (fainted Pokemon)
        let active_pokemon = side.get_active_pokemon();
        let has_fainted_active = active_pokemon.iter().any(|p| p.is_fainted());
        
        if has_fainted_active {
            // Force switch for fainted Pokemon
            return self.get_switch_options(side_ref);
        }
        
        // Check for status-based restrictions
        if let Some(active_pokemon) = active_pokemon.get(0) {
            // Check for must recharge (moves like Hyper Beam)
            if active_pokemon.volatile_statuses.contains(&VolatileStatus::MustRecharge) {
                return vec![crate::MoveChoice::None];
            }
            
            // Check for encore (locked into repeating last move)
            if active_pokemon.volatile_statuses.contains(&VolatileStatus::Encore) {
                // For now, just add the first available move
                // In a real implementation, we'd track the last move used
                if let Some((move_index, _)) = active_pokemon.moves.iter().next() {
                    let target_positions = self.get_default_targets_for_move(side_ref, *move_index);
                    options.push(crate::MoveChoice::Move {
                        move_index: *move_index,
                        target_positions,
                    });
                    return options;
                }
            }
        }
        
        // Add available moves
        self.add_available_moves(&mut options, side_ref);
        
        // Add switch options if not trapped
        if !self.is_trapped(side_ref) {
            let mut switch_options = self.get_switch_options(side_ref);
            options.append(&mut switch_options);
        }
        
        // If no options available, add None as fallback
        if options.is_empty() {
            options.push(crate::MoveChoice::None);
        }
        
        options
    }
    
    /// Add available moves for a side
    fn add_available_moves(&self, options: &mut Vec<crate::MoveChoice>, side_ref: SideReference) {
        let side = self.get_side(side_ref);
        
        if let Some(active_pokemon) = side.get_active_pokemon().get(0) {
            // Check for taunt (prevents status moves)
            let has_taunt = active_pokemon.volatile_statuses.contains(&VolatileStatus::Taunt);
            
            for (move_index, move_data) in &active_pokemon.moves {
                // Skip moves with no PP
                if !move_data.has_pp() {
                    continue;
                }
                
                // Skip status moves if taunted
                if has_taunt && move_data.is_status_move() {
                    continue;
                }
                
                // Get valid target positions for this move
                let target_positions = self.get_valid_targets_for_move(side_ref, *move_index, move_data);
                
                if !target_positions.is_empty() {
                    options.push(crate::MoveChoice::Move {
                        move_index: *move_index,
                        target_positions,
                    });
                }
            }
        }
    }
    
    /// Get switch options for a side
    fn get_switch_options(&self, side_ref: SideReference) -> Vec<crate::MoveChoice> {
        let mut options = Vec::new();
        let side = self.get_side(side_ref);
        
        for (poke_idx, pokemon) in side.pokemon.iter().enumerate() {
            let pokemon_index = crate::core::move_choice::PokemonIndex::from_index(poke_idx).unwrap_or(crate::core::move_choice::PokemonIndex::P0);
            // Skip if Pokemon is fainted
            if pokemon.is_fainted() {
                continue;
            }
            
            // Skip if Pokemon is already active
            if side.active_pokemon_indices.contains(&Some(poke_idx)) {
                continue;
            }
            
            options.push(crate::MoveChoice::Switch(pokemon_index));
        }
        
        // If no valid switches, add None as fallback
        if options.is_empty() {
            options.push(crate::MoveChoice::None);
        }
        
        options
    }
    
    /// Check if a side is trapped (cannot switch)
    fn is_trapped(&self, side_ref: SideReference) -> bool {
        let side = self.get_side(side_ref);
        
        if let Some(active_pokemon) = side.get_active_pokemon().get(0) {
            // Check for trapping volatile statuses
            if active_pokemon.volatile_statuses.contains(&VolatileStatus::PartiallyTrapped) {
                return true;
            }
            
            // Check for locked move status
            if active_pokemon.volatile_statuses.contains(&VolatileStatus::LockedMove) {
                return true;
            }
            
            // Ghost types can always switch (immune to trapping)
            if active_pokemon.types.contains(&"Ghost".to_string()) {
                return false;
            }
            
            // Shed Shell item allows switching
            if let Some(item) = &active_pokemon.item {
                if item == "Shed Shell" {
                    return false;
                }
            }
            
            // Check opponent's trapping abilities
            let opponent_side = match side_ref {
                SideReference::SideOne => SideReference::SideTwo,
                SideReference::SideTwo => SideReference::SideOne,
            };
            
            let opponent_side_data = self.get_side(opponent_side);
            if let Some(opponent_pokemon) = opponent_side_data.get_active_pokemon().get(0) {
                // Shadow Tag traps all Pokemon except other Shadow Tag users
                if opponent_pokemon.ability == "Shadow Tag" && active_pokemon.ability != "Shadow Tag" {
                    return true;
                }
                
                // Arena Trap traps grounded Pokemon
                if opponent_pokemon.ability == "Arena Trap" {
                    // Check if Pokemon is grounded (not Flying type, no Levitate, no Air Balloon)
                    let is_flying = active_pokemon.types.contains(&"Flying".to_string());
                    let has_levitate = active_pokemon.ability == "Levitate";
                    let has_air_balloon = active_pokemon.item.as_ref().map_or(false, |item| item == "Air Balloon");
                    
                    if !is_flying && !has_levitate && !has_air_balloon {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    /// Get valid target positions for a move
    fn get_valid_targets_for_move(&self, user_side: SideReference, move_index: MoveIndex, move_data: &Move) -> Vec<BattlePosition> {
        use crate::data::ps_types::PSMoveTarget;
        
        match move_data.target {
            PSMoveTarget::Self_ => {
                // Target self
                vec![BattlePosition::new(user_side, 0)]
            },
            PSMoveTarget::Normal => {
                // Target single opponent
                let opponent_side = match user_side {
                    SideReference::SideOne => SideReference::SideTwo,
                    SideReference::SideTwo => SideReference::SideOne,
                };
                
                // Find first active opponent
                for slot in 0..self.format.active_pokemon_count() {
                    let position = BattlePosition::new(opponent_side, slot);
                    if self.is_position_active(position) {
                        return vec![position];
                    }
                }
                
                // Fallback to first slot
                vec![BattlePosition::new(opponent_side, 0)]
            },
            PSMoveTarget::AllAdjacent => {
                // Target all adjacent Pokemon (in doubles/triples)
                let mut targets = Vec::new();
                
                // Add opponents
                let opponent_side = match user_side {
                    SideReference::SideOne => SideReference::SideTwo,
                    SideReference::SideTwo => SideReference::SideOne,
                };
                
                for slot in 0..self.format.active_pokemon_count() {
                    let position = BattlePosition::new(opponent_side, slot);
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
                
                // In doubles/triples, also add ally
                if self.format.active_pokemon_count() > 1 {
                    for slot in 0..self.format.active_pokemon_count() {
                        let position = BattlePosition::new(user_side, slot);
                        if self.is_position_active(position) && position.slot != 0 { // Don't target self
                            targets.push(position);
                        }
                    }
                }
                
                if targets.is_empty() {
                    // Fallback
                    let opponent_side = match user_side {
                        SideReference::SideOne => SideReference::SideTwo,
                        SideReference::SideTwo => SideReference::SideOne,
                    };
                    targets.push(BattlePosition::new(opponent_side, 0));
                }
                
                targets
            },
            PSMoveTarget::AllAdjacentFoes => {
                // Target all adjacent foes
                let mut targets = Vec::new();
                let opponent_side = match user_side {
                    SideReference::SideOne => SideReference::SideTwo,
                    SideReference::SideTwo => SideReference::SideOne,
                };
                
                for slot in 0..self.format.active_pokemon_count() {
                    let position = BattlePosition::new(opponent_side, slot);
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
                
                if targets.is_empty() {
                    // Fallback
                    targets.push(BattlePosition::new(opponent_side, 0));
                }
                
                targets
            },
            _ => {
                // For other targets, default to single opponent
                let opponent_side = match user_side {
                    SideReference::SideOne => SideReference::SideTwo,
                    SideReference::SideTwo => SideReference::SideOne,
                };
                vec![BattlePosition::new(opponent_side, 0)]
            }
        }
    }
    
    /// Get default target positions for a move (used for encore, etc.)
    fn get_default_targets_for_move(&self, user_side: SideReference, move_index: MoveIndex) -> Vec<BattlePosition> {
        let side = self.get_side(user_side);
        if let Some(active_pokemon) = side.get_active_pokemon().get(0) {
            if let Some(move_data) = active_pokemon.get_move(move_index) {
                return self.get_valid_targets_for_move(user_side, move_index, move_data);
            }
        }
        
        // Fallback to opponent
        let opponent_side = match user_side {
            SideReference::SideOne => SideReference::SideTwo,
            SideReference::SideTwo => SideReference::SideOne,
        };
        vec![BattlePosition::new(opponent_side, 0)]
    }

    // ========== Helper Methods for Undo System ==========

    /// Set Pokemon HP directly (for undo operations)
    fn set_pokemon_hp(&mut self, position: BattlePosition, hp: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.hp = hp.max(0).min(pokemon.max_hp);
        }
    }

    /// Set Pokemon status directly (for undo operations)
    fn set_pokemon_status(&mut self, position: BattlePosition, status: crate::core::instruction::PokemonStatus, duration: Option<u8>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.status = status;
            pokemon.status_duration = duration;
        }
    }

    /// Set Pokemon stat boosts directly (for undo operations)
    fn set_stat_boosts(&mut self, position: BattlePosition, boosts: &std::collections::HashMap<crate::core::instruction::Stat, i8>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stat_boosts = boosts.clone();
        }
    }

    /// Reverse volatile status application
    fn reverse_apply_volatile_status(&mut self, position: BattlePosition, volatile_status: crate::core::instruction::VolatileStatus, _duration: Option<u8>) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.volatile_statuses.remove(&volatile_status);
            pokemon.volatile_status_durations.remove(&volatile_status);
        }
    }

    /// Reverse volatile status removal
    fn reverse_remove_volatile_status(&mut self, position: BattlePosition, volatile_status: crate::core::instruction::VolatileStatus) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.volatile_statuses.insert(volatile_status);
            // Duration would need to be stored in instruction for proper reversal
        }
    }

    /// Reverse volatile status duration change
    fn reverse_change_volatile_status_duration(&mut self, position: BattlePosition, volatile_status: crate::core::instruction::VolatileStatus, duration_change: i8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if let Some(current_duration) = pokemon.volatile_status_durations.get_mut(&volatile_status) {
                *current_duration = (*current_duration as i8 - duration_change).max(0) as u8;
            }
        }
    }

    /// Reverse status duration change
    fn reverse_change_status_duration(&mut self, position: BattlePosition, duration_change: i8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            if let Some(current_duration) = pokemon.status_duration {
                pokemon.status_duration = Some((current_duration as i8 - duration_change).max(0) as u8);
            }
        }
    }

    /// Reverse side condition application
    fn reverse_apply_side_condition(&mut self, side: crate::core::battle_format::SideReference, condition: crate::core::instruction::SideCondition) {
        let side_mut = self.get_side_mut(side);
        side_mut.side_conditions.remove(&condition);
    }

    /// Reverse side condition removal
    fn reverse_remove_side_condition(&mut self, side: crate::core::battle_format::SideReference, condition: crate::core::instruction::SideCondition) {
        let side_mut = self.get_side_mut(side);
        // Would need previous value stored in instruction for proper reversal
        side_mut.side_conditions.insert(condition, 1);
    }

    /// Reverse side condition duration decrement
    fn reverse_decrement_side_condition_duration(&mut self, side: crate::core::battle_format::SideReference, condition: crate::core::instruction::SideCondition, amount: u8) {
        let side_mut = self.get_side_mut(side);
        if let Some(current_duration) = side_mut.side_conditions.get_mut(&condition) {
            *current_duration += amount;
        }
    }

    /// Reverse weather turns decrement
    fn reverse_decrement_weather_turns(&mut self) {
        if let Some(current_turns) = self.weather_turns_remaining {
            self.weather_turns_remaining = Some(current_turns + 1);
        }
    }

    /// Reverse terrain turns decrement
    fn reverse_decrement_terrain_turns(&mut self) {
        if let Some(current_turns) = self.terrain_turns_remaining {
            self.terrain_turns_remaining = Some(current_turns + 1);
        }
    }

    /// Reverse trick room turns decrement
    fn reverse_decrement_trick_room_turns(&mut self) {
        if let Some(current_turns) = self.trick_room_turns_remaining {
            self.trick_room_turns_remaining = Some(current_turns + 1);
        }
    }

    /// Increment PP (reverse of decrement)
    fn increment_pp(&mut self, position: BattlePosition, move_index: u8, amount: u8) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            let move_idx = crate::core::move_choice::MoveIndex::from_u8(move_index).unwrap_or(crate::core::move_choice::MoveIndex::M0);
            if let Some(move_data) = pokemon.moves.get_mut(&move_idx) {
                move_data.pp = (move_data.pp + amount).min(move_data.max_pp);
            }
        }
    }

    /// Reverse raw attack change
    fn reverse_change_attack(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.attack = (pokemon.stats.attack - stat_change).max(1);
        }
    }

    /// Reverse raw defense change
    fn reverse_change_defense(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.defense = (pokemon.stats.defense - stat_change).max(1);
        }
    }

    /// Reverse raw special attack change
    fn reverse_change_special_attack(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.special_attack = (pokemon.stats.special_attack - stat_change).max(1);
        }
    }

    /// Reverse raw special defense change
    fn reverse_change_special_defense(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.special_defense = (pokemon.stats.special_defense - stat_change).max(1);
        }
    }

    /// Reverse raw speed change
    fn reverse_change_speed(&mut self, position: BattlePosition, stat_change: i16) {
        if let Some(pokemon) = self.get_pokemon_at_position_mut(position) {
            pokemon.stats.speed = (pokemon.stats.speed - stat_change).max(1);
        }
    }

    /// Reverse damage dealt change
    fn reverse_change_damage_dealt(&mut self, position: BattlePosition, damage_amount: i16) {
        let side = self.get_side_mut(position.side);
        side.damage_dealt.damage = damage_amount; // Set to previous value
    }

    /// Reverse multi-target damage
    fn reverse_multi_target_damage(&mut self, target_damages: &[(BattlePosition, i16)]) {
        for (position, damage) in target_damages {
            if let Some(pokemon) = self.get_pokemon_at_position_mut(*position) {
                pokemon.hp = (pokemon.hp + damage).min(pokemon.max_hp);
            }
        }
    }

    /// Pretty print the battle state for logging/debugging
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("=== Battle State (Turn {}) ===\n", self.turn));
        output.push_str(&format!("Format: {}\n", self.format));
        
        // Weather and terrain
        if self.weather != Weather::None {
            output.push_str(&format!("Weather: {:?}", self.weather));
            if let Some(turns) = self.weather_turns_remaining {
                output.push_str(&format!(" ({} turns)", turns));
            }
            output.push('\n');
        }
        
        if self.terrain != Terrain::None {
            output.push_str(&format!("Terrain: {:?}", self.terrain));
            if let Some(turns) = self.terrain_turns_remaining {
                output.push_str(&format!(" ({} turns)", turns));
            }
            output.push('\n');
        }
        
        if self.trick_room_active {
            output.push_str("Trick Room: Active");
            if let Some(turns) = self.trick_room_turns_remaining {
                output.push_str(&format!(" ({} turns)", turns));
            }
            output.push('\n');
        }
        
        // Side information
        let active_slots = self.format.active_pokemon_count();
        output.push_str("\n--- Side One ---\n");
        output.push_str(&self.side_one.pretty_print_with_format(active_slots));
        
        output.push_str("\n--- Side Two ---\n");
        output.push_str(&self.side_two.pretty_print_with_format(active_slots));
        
        output
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(BattleFormat::default())
    }
}

/// Represents one side of the battle (a player's team and active Pokemon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSide {
    /// All Pokemon on this side's team
    pub pokemon: Vec<Pokemon>,
    /// Indices of currently active Pokemon
    pub active_pokemon_indices: Vec<Option<usize>>,
    /// Side conditions affecting this side
    pub side_conditions: HashMap<SideCondition, u8>,
    /// Volatile statuses that affect the entire side
    pub side_volatile_statuses: HashSet<SideVolatileStatus>,
    /// Wish healing scheduled for specific slots (heal_amount, turns_remaining)
    pub wish_healing: HashMap<usize, (i16, u8)>,
    /// Future Sight attacks scheduled for specific slots (attacker_position, damage_amount, turns_remaining, move_name)
    pub future_sight_attacks: HashMap<usize, (BattlePosition, i16, u8, String)>,
    /// Damage tracking for counter moves
    pub damage_dealt: DamageDealt,
}

impl BattleSide {
    /// Create a new battle side
    pub fn new() -> Self {
        Self {
            pokemon: Vec::new(),
            active_pokemon_indices: vec![None; 3], // Max 3 for triples, unused slots ignored
            side_conditions: HashMap::new(),
            side_volatile_statuses: HashSet::new(),
            wish_healing: HashMap::new(),
            future_sight_attacks: HashMap::new(),
            damage_dealt: DamageDealt::new(),
        }
    }

    /// Serialize the battle side to a compact string format
    /// Format: pokemon_count|pokemon1|pokemon2|...|active_indices|side_conditions|side_volatile_statuses
    pub fn serialize(&self) -> String {
        let mut parts = Vec::new();
        
        // Pokemon count
        parts.push(self.pokemon.len().to_string());
        
        // All Pokemon
        for pokemon in &self.pokemon {
            parts.push(pokemon.serialize());
        }
        
        // Active Pokemon indices
        let active_indices = self.active_pokemon_indices.iter()
            .map(|opt| match opt {
                Some(index) => index.to_string(),
                None => "x".to_string(),
            })
            .collect::<Vec<_>>()
            .join(",");
        parts.push(active_indices);
        
        // Side conditions
        let side_conditions = self.side_conditions.iter()
            .map(|(condition, value)| format!("{}:{}", *condition as u8, value))
            .collect::<Vec<_>>()
            .join(",");
        parts.push(side_conditions);
        
        // Side volatile statuses
        let side_volatile_statuses = self.side_volatile_statuses.iter()
            .map(|status| (*status as u8).to_string())
            .collect::<Vec<_>>()
            .join(",");
        parts.push(side_volatile_statuses);
        
        parts.join("|")
    }

    /// Deserialize a battle side from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let parts: Vec<&str> = serialized.split('|').collect();
        if parts.len() < 4 {
            return Err(format!("Invalid side format: expected at least 4 parts, got {}", parts.len()));
        }

        let pokemon_count = parts[0].parse::<usize>()
            .map_err(|_| format!("Invalid pokemon count: {}", parts[0]))?;

        if parts.len() < 4 + pokemon_count {
            return Err(format!("Invalid side format: expected {} pokemon parts", pokemon_count));
        }

        // Deserialize Pokemon
        let mut pokemon = Vec::new();
        for i in 1..=pokemon_count {
            pokemon.push(Pokemon::deserialize(parts[i])?);
        }

        // Deserialize active indices
        let active_indices_str = parts[1 + pokemon_count];
        let active_pokemon_indices = if active_indices_str.is_empty() {
            vec![None; 3]
        } else {
            active_indices_str.split(',')
                .map(|s| if s == "x" { None } else { s.parse::<usize>().ok() })
                .collect()
        };

        // Deserialize side conditions
        let side_conditions_str = parts[2 + pokemon_count];
        let mut side_conditions = HashMap::new();
        if !side_conditions_str.is_empty() {
            for condition_str in side_conditions_str.split(',') {
                if let Some((condition_id_str, value_str)) = condition_str.split_once(':') {
                    let condition_id = condition_id_str.parse::<u8>()
                        .map_err(|_| format!("Invalid side condition ID: {}", condition_id_str))?;
                    let value = value_str.parse::<u8>()
                        .map_err(|_| format!("Invalid side condition value: {}", value_str))?;
                    side_conditions.insert(SideCondition::from(condition_id), value);
                }
            }
        }

        // Deserialize side volatile statuses
        let side_volatile_statuses_str = parts[3 + pokemon_count];
        let mut side_volatile_statuses = HashSet::new();
        if !side_volatile_statuses_str.is_empty() {
            for status_str in side_volatile_statuses_str.split(',') {
                let status_id = status_str.parse::<u8>()
                    .map_err(|_| format!("Invalid side volatile status ID: {}", status_str))?;
                let status = match status_id {
                    0 => SideVolatileStatus::TailWind,
                    1 => SideVolatileStatus::WideGuard,
                    2 => SideVolatileStatus::QuickGuard,
                    _ => SideVolatileStatus::TailWind,
                };
                side_volatile_statuses.insert(status);
            }
        }

        Ok(Self {
            pokemon,
            active_pokemon_indices,
            side_conditions,
            side_volatile_statuses,
            wish_healing: HashMap::new(), // TODO: Add serialization if needed
            future_sight_attacks: HashMap::new(), // TODO: Add serialization if needed
            damage_dealt: DamageDealt::new(),
        })
    }

    /// Get the active Pokemon at the specified slot
    pub fn get_active_pokemon_at_slot(&self, slot: usize) -> Option<&Pokemon> {
        if let Some(Some(index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get(*index)
        } else {
            None
        }
    }

    /// Get a mutable reference to the active Pokemon at the specified slot
    pub fn get_active_pokemon_at_slot_mut(&mut self, slot: usize) -> Option<&mut Pokemon> {
        if let Some(Some(index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get_mut(*index)
        } else {
            None
        }
    }

    /// Set the active Pokemon at the specified slot
    pub fn set_active_pokemon_at_slot(&mut self, slot: usize, pokemon_index: Option<usize>) {
        if slot < self.active_pokemon_indices.len() {
            self.active_pokemon_indices[slot] = pokemon_index;
        }
    }

    /// Get all active Pokemon
    pub fn get_active_pokemon(&self) -> Vec<&Pokemon> {
        self.active_pokemon_indices
            .iter()
            .filter_map(|index| index.and_then(|i| self.pokemon.get(i)))
            .collect()
    }

    /// Check if this side is defeated (all Pokemon have 0 HP)
    pub fn is_defeated(&self) -> bool {
        self.pokemon
            .iter()
            .all(|pokemon| pokemon.hp == 0)
    }

    /// Add a Pokemon to this side's team
    pub fn add_pokemon(&mut self, pokemon: Pokemon) {
        self.pokemon.push(pokemon);
    }

    /// Get the number of Pokemon on this side
    pub fn pokemon_count(&self) -> usize {
        self.pokemon.len()
    }

    /// Pretty print the side information for logging/debugging
    pub fn pretty_print(&self) -> String {
        self.pretty_print_with_format(3) // Default to 3 slots for backward compatibility
    }
    
    /// Pretty print the side information with format-aware slot display
    pub fn pretty_print_with_format(&self, max_active_slots: usize) -> String {
        let mut output = String::new();
        
        // Active Pokemon
        if !self.active_pokemon_indices.is_empty() {
            output.push_str("Active Pokemon:\n");
            let slots_to_show = max_active_slots.min(self.active_pokemon_indices.len());
            
            for slot in 0..slots_to_show {
                if let Some(pokemon_index_opt) = self.active_pokemon_indices.get(slot) {
                    if let Some(pokemon_index) = pokemon_index_opt {
                        if let Some(pokemon) = self.pokemon.get(*pokemon_index) {
                            output.push_str(&format!("  Slot {}: {}\n", slot, pokemon.pretty_print()));
                        } else {
                            output.push_str(&format!("  Slot {}: Invalid Pokemon index {}\n", slot, pokemon_index));
                        }
                    } else {
                        output.push_str(&format!("  Slot {}: Empty\n", slot));
                    }
                }
            }
        } else {
            output.push_str("No active Pokemon\n");
        }
        
        // Side conditions
        if !self.side_conditions.is_empty() {
            output.push_str("Side Conditions:\n");
            for condition in &self.side_conditions {
                output.push_str(&format!("  {:?}\n", condition));
            }
        }
        
        // Pokemon team
        output.push_str(&format!("Team ({} Pokemon):\n", self.pokemon.len()));
        for (i, pokemon) in self.pokemon.iter().enumerate() {
            let status_str = if pokemon.is_fainted() { " [FAINTED]" } else { "" };
            output.push_str(&format!("  {}: {} {}/{}{}\n", 
                i, 
                pokemon.species, 
                pokemon.hp,
                pokemon.max_hp,
                status_str
            ));
        }
        
        output
    }
}

impl Default for BattleSide {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a Pokemon in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Pokemon species name/ID
    pub species: String,
    /// Current HP
    pub hp: i16,
    /// Maximum HP
    pub max_hp: i16,
    /// Base stats
    pub stats: Stats,
    /// Current stat boosts (-6 to +6)
    pub stat_boosts: HashMap<crate::core::instruction::Stat, i8>,
    /// Current status condition
    pub status: PokemonStatus,
    /// Status duration (for sleep/freeze)
    pub status_duration: Option<u8>,
    /// Volatile statuses
    pub volatile_statuses: HashSet<VolatileStatus>,
    /// Volatile status durations (turns remaining for each status)
    pub volatile_status_durations: HashMap<VolatileStatus, u8>,
    /// Substitute health (when Substitute volatile status is active)
    pub substitute_health: i16,
    /// Current moves
    pub moves: HashMap<MoveIndex, Move>,
    /// Current ability
    pub ability: String,
    /// Held item
    pub item: Option<String>,
    /// Types (can change due to moves like Soak)
    pub types: Vec<String>,
    /// Level
    pub level: u8,
    /// Gender
    pub gender: Gender,
    /// Tera type (if Terastallized) - Gen 9+ only
    pub tera_type: Option<crate::core::move_choice::PokemonType>,
    /// Whether this Pokemon is Terastallized - Gen 9+ only
    pub is_terastallized: bool,
}

impl Pokemon {
    /// Create a new Pokemon with default values
    pub fn new(species: String) -> Self {
        Self {
            species,
            hp: 100,
            max_hp: 100,
            stats: Stats { hp: 100, attack: 100, defense: 100, special_attack: 100, special_defense: 100, speed: 100 },
            stat_boosts: HashMap::new(),
            status: PokemonStatus::None,
            status_duration: None,
            volatile_statuses: HashSet::new(),
            volatile_status_durations: HashMap::new(),
            substitute_health: 0,
            moves: HashMap::new(),
            ability: String::new(),
            item: None,
            types: vec!["Normal".to_string()],
            level: 50,
            gender: Gender::Unknown,
            tera_type: None,
            is_terastallized: false,
        }
    }

    /// Serialize the Pokemon to a compact string format
    /// Format: species,hp,max_hp,stats,stat_boosts,status,status_duration,volatile_statuses,ability,item,types,level,gender
    pub fn serialize(&self) -> String {
        let mut parts = Vec::new();
        
        // Basic info
        parts.push(self.species.clone());
        parts.push(self.hp.to_string());
        parts.push(self.max_hp.to_string());
        
        // Stats
        parts.push(format!("{}~{}~{}~{}~{}", 
            self.stats.attack, self.stats.defense, 
            self.stats.special_attack, self.stats.special_defense, 
            self.stats.speed));
        
        // Stat boosts
        let stat_boosts = self.stat_boosts.iter()
            .map(|(stat, boost)| format!("{}:{}", *stat as u8, boost))
            .collect::<Vec<_>>()
            .join("~");
        parts.push(stat_boosts);
        
        // Status
        parts.push((self.status as u8).to_string());
        parts.push(self.status_duration.map_or("x".to_string(), |d| d.to_string()));
        
        // Volatile statuses
        let volatile_statuses = self.volatile_statuses.iter()
            .map(|status| (*status as u8).to_string())
            .collect::<Vec<_>>()
            .join("~");
        parts.push(volatile_statuses);
        
        // Basic properties
        parts.push(self.ability.clone());
        parts.push(self.item.as_ref().unwrap_or(&"x".to_string()).clone());
        parts.push(self.types.join("~"));
        parts.push(self.level.to_string());
        parts.push((self.gender as u8).to_string());
        
        // Moves (serialize as count then move data)
        let moves_data = self.moves.iter()
            .map(|(index, move_data)| format!("{}#{}", *index as u8, move_data.serialize()))
            .collect::<Vec<_>>()
            .join("~");
        parts.push(moves_data);
        
        parts.join(",")
    }

    /// Deserialize a Pokemon from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let parts: Vec<&str> = serialized.split(',').collect();
        if parts.len() < 13 {
            return Err(format!("Invalid pokemon format: expected at least 13 parts, got {}", parts.len()));
        }

        let species = parts[0].to_string();
        let hp = parts[1].parse::<i16>()
            .map_err(|_| format!("Invalid HP: {}", parts[1]))?;
        let max_hp = parts[2].parse::<i16>()
            .map_err(|_| format!("Invalid max HP: {}", parts[2]))?;
        
        // Parse stats
        let stats_parts: Vec<&str> = parts[3].split('~').collect();
        if stats_parts.len() != 5 {
            return Err(format!("Invalid stats format: {}", parts[3]));
        }
        let stats = Stats {
            hp: max_hp, // Use max_hp as the base HP stat for deserialization
            attack: stats_parts[0].parse().map_err(|_| "Invalid attack stat")?,
            defense: stats_parts[1].parse().map_err(|_| "Invalid defense stat")?,
            special_attack: stats_parts[2].parse().map_err(|_| "Invalid special attack stat")?,
            special_defense: stats_parts[3].parse().map_err(|_| "Invalid special defense stat")?,
            speed: stats_parts[4].parse().map_err(|_| "Invalid speed stat")?,
        };
        
        // Parse stat boosts
        let mut stat_boosts = HashMap::new();
        if !parts[4].is_empty() {
            for boost_str in parts[4].split('~') {
                if let Some((stat_str, boost_str)) = boost_str.split_once(':') {
                    let stat_id = stat_str.parse::<u8>()
                        .map_err(|_| format!("Invalid stat ID: {}", stat_str))?;
                    let boost = boost_str.parse::<i8>()
                        .map_err(|_| format!("Invalid boost value: {}", boost_str))?;
                    stat_boosts.insert(crate::core::instruction::Stat::from(stat_id), boost);
                }
            }
        }
        
        // Parse status
        let status_id = parts[5].parse::<u8>()
            .map_err(|_| format!("Invalid status ID: {}", parts[5]))?;
        let status = PokemonStatus::from(status_id);
        
        let status_duration = if parts[6] == "x" {
            None
        } else {
            Some(parts[6].parse::<u8>()
                .map_err(|_| format!("Invalid status duration: {}", parts[6]))?)
        };
        
        // Parse volatile statuses
        let mut volatile_statuses = HashSet::new();
        if !parts[7].is_empty() {
            for status_str in parts[7].split('~') {
                let status_id = status_str.parse::<u8>()
                    .map_err(|_| format!("Invalid volatile status ID: {}", status_str))?;
                volatile_statuses.insert(VolatileStatus::from(status_id));
            }
        }
        
        let ability = parts[8].to_string();
        let item = if parts[9] == "x" { None } else { Some(parts[9].to_string()) };
        let types = parts[10].split('~').map(|s| s.to_string()).collect();
        let level = parts[11].parse::<u8>()
            .map_err(|_| format!("Invalid level: {}", parts[11]))?;
        let gender_id = parts[12].parse::<u8>()
            .map_err(|_| format!("Invalid gender ID: {}", parts[12]))?;
        let gender = match gender_id {
            0 => Gender::Male,
            1 => Gender::Female,
            _ => Gender::Unknown,
        };
        
        // Parse moves
        let mut moves = HashMap::new();
        if parts.len() > 13 && !parts[13].is_empty() {
            for move_str in parts[13].split('~') {
                if let Some((index_str, move_data_str)) = move_str.split_once('#') {
                    let index_id = index_str.parse::<u8>()
                        .map_err(|_| format!("Invalid move index: {}", index_str))?;
                    let move_index = match index_id {
                        0 => MoveIndex::M0,
                        1 => MoveIndex::M1,
                        2 => MoveIndex::M2,
                        3 => MoveIndex::M3,
                        _ => return Err(format!("Invalid move index: {}", index_id)),
                    };
                    let move_data = Move::deserialize(move_data_str)?;
                    moves.insert(move_index, move_data);
                }
            }
        }
        
        Ok(Self {
            species,
            hp,
            max_hp,
            stats,
            stat_boosts,
            status,
            status_duration,
            volatile_statuses,
            volatile_status_durations: HashMap::new(), // TODO: Add serialization if needed
            substitute_health: 0, // TODO: Add serialization if needed
            moves,
            ability,
            item,
            types,
            level,
            gender,
            tera_type: None, // TODO: Add serialization if needed
            is_terastallized: false, // TODO: Add serialization if needed
        })
    }

    /// Check if this Pokemon is fainted (HP = 0)
    pub fn is_fainted(&self) -> bool {
        self.hp == 0
    }

    /// Get the effective stat value including boosts
    pub fn get_effective_stat(&self, stat: crate::core::instruction::Stat) -> i16 {
        let base_value = match stat {
            crate::core::instruction::Stat::Hp => return self.max_hp, // HP doesn't get boosted
            crate::core::instruction::Stat::Attack => self.stats.attack,
            crate::core::instruction::Stat::Defense => self.stats.defense,
            crate::core::instruction::Stat::SpecialAttack => self.stats.special_attack,
            crate::core::instruction::Stat::SpecialDefense => self.stats.special_defense,
            crate::core::instruction::Stat::Speed => self.stats.speed,
            crate::core::instruction::Stat::Accuracy => 100, // Base accuracy
            crate::core::instruction::Stat::Evasion => 100,   // Base evasion
        };

        let boost = self.stat_boosts.get(&stat).copied().unwrap_or(0);
        let multiplier = if boost >= 0 {
            (2 + boost as i16) as f32 / 2.0
        } else {
            2.0 / (2 - boost as i16) as f32
        };

        (base_value as f32 * multiplier) as i16
    }

    /// Add a move to this Pokemon
    pub fn add_move(&mut self, index: MoveIndex, move_data: Move) {
        self.moves.insert(index, move_data);
    }

    /// Get a move by index
    pub fn get_move(&self, index: MoveIndex) -> Option<&Move> {
        self.moves.get(&index)
    }

    /// Pretty print the Pokemon information for logging/debugging
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{} (Lv{})", self.species, self.level));
        output.push_str(&format!(" HP: {}/{}", self.hp, self.max_hp));
        
        if self.status != PokemonStatus::None {
            output.push_str(&format!(" [{:?}", self.status));
            if let Some(duration) = self.status_duration {
                output.push_str(&format!(" {}t", duration));
            }
            output.push(']');
        }
        
        if !self.volatile_statuses.is_empty() {
            let statuses: Vec<String> = self.volatile_statuses.iter()
                .map(|s| format!("{:?}", s))
                .collect();
            output.push_str(&format!(" [{}]", statuses.join(", ")));
        }
        
        if !self.stat_boosts.is_empty() {
            let boosts: Vec<String> = self.stat_boosts.iter()
                .filter(|(_, &boost)| boost != 0)
                .map(|(stat, boost)| format!("{:?}: {:+}", stat, boost))
                .collect();
            if !boosts.is_empty() {
                output.push_str(&format!(" ({})", boosts.join(", ")));
            }
        }
        
        output
    }
}

/// Pokemon base stats

/// Pokemon gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

/// Side-wide volatile statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideVolatileStatus {
    TailWind,
    WideGuard,
    QuickGuard,
}

/// Represents a Pokemon's move in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    /// Move name/identifier
    pub name: String,
    /// Base power (0 for status moves)
    pub base_power: u8,
    /// Accuracy (1-100, 0 for never-miss moves)
    pub accuracy: u8,
    /// Move type
    pub move_type: String,
    /// Current PP
    pub pp: u8,
    /// Maximum PP
    pub max_pp: u8,
    /// Move target type (Pokemon Showdown format)
    pub target: crate::data::ps_types::PSMoveTarget,
    /// Move category
    pub category: MoveCategory,
    /// Move priority
    pub priority: i8,
}

impl Move {
    /// Create a new move with default values
    /// Note: This constructor should primarily be used for testing.
    /// Production code should use PSMoveFactory to get accurate move data.
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_power: 60,  // More reasonable default than 80
            accuracy: 100,
            move_type: "Normal".to_string(),
            pp: 15,          // More reasonable default than 20
            max_pp: 15,
            target: crate::data::ps_types::PSMoveTarget::Normal,
            category: MoveCategory::Physical,
            priority: 0,
        }
    }

    /// Serialize the move to a compact string format
    /// Format: name&power&accuracy&type&pp&max_pp&target&category&priority
    pub fn serialize(&self) -> String {
        format!("{}!{}!{}!{}!{}!{}!{}!{}!{}",
            self.name,
            self.base_power,
            self.accuracy,
            self.move_type,
            self.pp,
            self.max_pp,
            self.target.serialize(),
            self.category as u8,
            self.priority
        )
    }

    /// Deserialize a move from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let parts: Vec<&str> = serialized.split('!').collect();
        if parts.len() != 9 {
            return Err(format!("Invalid move format: expected 9 parts, got {}", parts.len()));
        }

        let name = parts[0].to_string();
        let base_power = parts[1].parse::<u8>()
            .map_err(|_| format!("Invalid base power: {}", parts[1]))?;
        let accuracy = parts[2].parse::<u8>()
            .map_err(|_| format!("Invalid accuracy: {}", parts[2]))?;
        let move_type = parts[3].to_string();
        let pp = parts[4].parse::<u8>()
            .map_err(|_| format!("Invalid PP: {}", parts[4]))?;
        let max_pp = parts[5].parse::<u8>()
            .map_err(|_| format!("Invalid max PP: {}", parts[5]))?;
        let target = crate::data::ps_types::PSMoveTarget::deserialize(parts[6])?;
        let category_id = parts[7].parse::<u8>()
            .map_err(|_| format!("Invalid category: {}", parts[7]))?;
        let category = match category_id {
            0 => MoveCategory::Physical,
            1 => MoveCategory::Special,
            2 => MoveCategory::Status,
            _ => return Err(format!("Invalid category ID: {}", category_id)),
        };
        let priority = parts[8].parse::<i8>()
            .map_err(|_| format!("Invalid priority: {}", parts[8]))?;

        Ok(Self {
            name,
            base_power,
            accuracy,
            move_type,
            pp,
            max_pp,
            target,
            category,
            priority,
        })
    }

    /// Create a new move with specific properties
    pub fn new_with_details(
        name: String,
        base_power: u8,
        accuracy: u8,
        move_type: String,
        pp: u8,
        target: crate::data::ps_types::PSMoveTarget,
        category: MoveCategory,
        priority: i8,
    ) -> Self {
        Self {
            name,
            base_power,
            accuracy,
            move_type,
            pp,
            max_pp: pp,
            target,
            category,
            priority,
        }
    }

    /// Check if this move is a status move (no base power)
    pub fn is_status_move(&self) -> bool {
        self.base_power == 0 || matches!(self.category, MoveCategory::Status)
    }

    /// Check if this move is a spread move
    pub fn is_spread_move(&self) -> bool {
        self.target.is_spread_move()
    }

    /// Check if this move can affect allies
    pub fn affects_allies(&self) -> bool {
        self.target.affects_allies()
    }

    /// Check if this move has PP remaining
    pub fn has_pp(&self) -> bool {
        self.pp > 0
    }

    /// Use PP for this move
    pub fn use_pp(&mut self) -> bool {
        if self.pp > 0 {
            self.pp -= 1;
            true
        } else {
            false
        }
    }

    /// Check if this move is a damaging move
    pub fn is_damaging(&self) -> bool {
        matches!(self.category, MoveCategory::Physical | MoveCategory::Special) && self.base_power > 0
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    #[test]
    fn test_state_creation() {
        let state = State::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        assert_eq!(state.format, BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        assert_eq!(state.turn, 1);
        assert_eq!(state.weather, Weather::None);
    }

    #[test]
    fn test_battle_side_active_pokemon() {
        let mut side = BattleSide::new();
        let pokemon = Pokemon::new("Pikachu".to_string());
        
        side.add_pokemon(pokemon);
        side.set_active_pokemon_at_slot(0, Some(0));

        assert!(side.get_active_pokemon_at_slot(0).is_some());
        assert!(side.get_active_pokemon_at_slot(1).is_none());
        assert_eq!(side.get_active_pokemon().len(), 1);
    }

    #[test]
    fn test_pokemon_stats() {
        let mut pokemon = Pokemon::new("Pikachu".to_string());
        pokemon.stats.attack = 120;
        
        // No boost
        assert_eq!(pokemon.get_effective_stat(crate::core::instruction::Stat::Attack), 120);
        
        // +1 boost
        pokemon.stat_boosts.insert(crate::core::instruction::Stat::Attack, 1);
        assert_eq!(pokemon.get_effective_stat(crate::core::instruction::Stat::Attack), 180);
        
        // -1 boost
        pokemon.stat_boosts.insert(crate::core::instruction::Stat::Attack, -1);
        assert_eq!(pokemon.get_effective_stat(crate::core::instruction::Stat::Attack), 80);
    }

    #[test]
    fn test_position_validity() {
        let state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let pos_valid = BattlePosition::new(SideReference::SideOne, 0);
        let pos_invalid = BattlePosition::new(SideReference::SideOne, 1);

        assert!(pos_valid.is_valid_for_format(&state.format));
        assert!(!pos_invalid.is_valid_for_format(&state.format));
    }

    #[test]
    fn test_state_serialization_basic() {
        let state = State::new(BattleFormat::gen9_ou());
        let serialized = state.serialize();
        let deserialized = State::deserialize(&serialized).unwrap();
        
        assert_eq!(state.format.name, deserialized.format.name);
        assert_eq!(state.format.generation, deserialized.format.generation);
        assert_eq!(state.format.format_type, deserialized.format.format_type);
        assert_eq!(state.turn, deserialized.turn);
        assert_eq!(state.weather, deserialized.weather);
        assert_eq!(state.terrain, deserialized.terrain);
        assert_eq!(state.trick_room_active, deserialized.trick_room_active);
    }

    #[test]
    fn test_pokemon_serialization() {
        let mut pokemon = Pokemon::new("Pikachu".to_string());
        pokemon.hp = 85;
        pokemon.max_hp = 100;
        pokemon.level = 50;
        pokemon.ability = "Static".to_string();
        pokemon.item = Some("Light Ball".to_string());
        pokemon.types = vec!["Electric".to_string()];
        
        let serialized = pokemon.serialize();
        let deserialized = Pokemon::deserialize(&serialized).unwrap();
        
        assert_eq!(pokemon.species, deserialized.species);
        assert_eq!(pokemon.hp, deserialized.hp);
        assert_eq!(pokemon.max_hp, deserialized.max_hp);
        assert_eq!(pokemon.level, deserialized.level);
        assert_eq!(pokemon.ability, deserialized.ability);
        assert_eq!(pokemon.item, deserialized.item);
        assert_eq!(pokemon.types, deserialized.types);
    }

    #[test]
    fn test_move_serialization() {
        let move_data = Move::new_with_details(
            "Thunderbolt".to_string(),
            90,
            100,
            "Electric".to_string(),
            15,
            crate::data::ps_types::PSMoveTarget::Normal,
            MoveCategory::Special,
            0,
        );
        
        let serialized = move_data.serialize();
        let deserialized = Move::deserialize(&serialized).unwrap();
        
        assert_eq!(move_data.name, deserialized.name);
        assert_eq!(move_data.base_power, deserialized.base_power);
        assert_eq!(move_data.accuracy, deserialized.accuracy);
        assert_eq!(move_data.move_type, deserialized.move_type);
        assert_eq!(move_data.pp, deserialized.pp);
        assert_eq!(move_data.max_pp, deserialized.max_pp);
        assert_eq!(move_data.priority, deserialized.priority);
    }

    #[test]
    fn test_battle_side_serialization() {
        let mut side = BattleSide::new();
        let pokemon1 = Pokemon::new("Charizard".to_string());
        let pokemon2 = Pokemon::new("Blastoise".to_string());
        
        side.add_pokemon(pokemon1);
        side.add_pokemon(pokemon2);
        side.set_active_pokemon_at_slot(0, Some(0));
        
        let serialized = side.serialize();
        let deserialized = BattleSide::deserialize(&serialized).unwrap();
        
        assert_eq!(side.pokemon.len(), deserialized.pokemon.len());
        assert_eq!(side.pokemon[0].species, deserialized.pokemon[0].species);
        assert_eq!(side.pokemon[1].species, deserialized.pokemon[1].species);
        assert_eq!(side.active_pokemon_indices[0], deserialized.active_pokemon_indices[0]);
    }

    #[test]
    fn test_enum_serialization() {
        // Test PokemonStatus
        assert_eq!(PokemonStatus::from(0), PokemonStatus::None);
        assert_eq!(PokemonStatus::from(1), PokemonStatus::Burn);
        assert_eq!(PokemonStatus::from(6), PokemonStatus::Sleep);
        
        // Test Weather
        assert_eq!(Weather::from(0), Weather::None);
        assert_eq!(Weather::from(1), Weather::Sun);
        assert_eq!(Weather::from(2), Weather::Rain);
        
        // Test Generation
        assert_eq!(Generation::from(9), Generation::Gen9);
        assert_eq!(Generation::from(4), Generation::Gen4);
        assert_eq!(Generation::from(255), Generation::Gen9); // Default fallback
    }

    #[test]
    fn test_complex_state_serialization() {
        let mut state = State::new(BattleFormat::vgc2024());
        state.turn = 5;
        state.weather = Weather::Rain;
        state.weather_turns_remaining = Some(3);
        state.terrain = Terrain::ElectricTerrain;
        state.terrain_turns_remaining = Some(2);
        state.trick_room_active = true;
        state.trick_room_turns_remaining = Some(4);
        
        // Add a Pokemon to side one
        let mut pokemon = Pokemon::new("Garchomp".to_string());
        pokemon.hp = 150;
        pokemon.max_hp = 200;
        pokemon.level = 50;
        state.side_one.add_pokemon(pokemon);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        let serialized = state.serialize();
        let deserialized = State::deserialize(&serialized).unwrap();
        
        assert_eq!(state.turn, deserialized.turn);
        assert_eq!(state.weather, deserialized.weather);
        assert_eq!(state.weather_turns_remaining, deserialized.weather_turns_remaining);
        assert_eq!(state.terrain, deserialized.terrain);
        assert_eq!(state.terrain_turns_remaining, deserialized.terrain_turns_remaining);
        assert_eq!(state.trick_room_active, deserialized.trick_room_active);
        assert_eq!(state.trick_room_turns_remaining, deserialized.trick_room_turns_remaining);
        assert_eq!(state.side_one.pokemon.len(), deserialized.side_one.pokemon.len());
        assert_eq!(state.side_one.pokemon[0].species, deserialized.side_one.pokemon[0].species);
    }
}