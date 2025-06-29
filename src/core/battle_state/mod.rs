//! # Modern Battle State System

use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::types::PokemonType;
use crate::core::instructions::{
    BattleInstruction, FieldInstruction, PokemonInstruction, PokemonStatus,
    StatsInstruction, StatusInstruction, Terrain, VolatileStatus, Weather,
};
use crate::core::move_choice::{MoveChoice, PokemonIndex};
use crate::generation::GenerationBattleMechanics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export MoveCategory for compatibility
pub use crate::core::instructions::MoveCategory;

// Re-export Pokemon-related types from pokemon module
mod pokemon;
pub use pokemon::*;

// Re-export Field-related types from field module
mod field;
pub use field::*;

// Re-export Side-related types from side module
mod side;
pub use side::*;


/// The main battle state with decomposed components
#[derive(Clone, Serialize)]
pub struct BattleState {
    /// The battle format determining rules and active Pokemon count
    pub format: BattleFormat,
    /// The two battle sides (always exactly 2)
    pub sides: [BattleSide; 2],
    /// Field conditions affecting the entire battlefield
    pub field: FieldConditions,
    /// Turn-related state information
    pub turn_info: TurnState,
    /// Generation-specific data repository
    #[serde(skip)]
    pub generation_repo: Arc<crate::data::generation_loader::GenerationRepository>,
    /// Game data repository  
    #[serde(skip)]
    pub game_data_repo: Arc<crate::data::GameDataRepository>,
}

impl std::fmt::Debug for BattleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BattleState")
            .field("format", &self.format)
            .field("sides", &self.sides)
            .field("field", &self.field)
            .field("turn_info", &self.turn_info)
            .field("generation_repo", &"<GenerationRepository>")
            .field("game_data_repo", &"<GameDataRepository>")
            .finish()
    }
}

impl<'de> serde::Deserialize<'de> for BattleState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BattleStateDeserialize {
            format: BattleFormat,
            sides: [BattleSide; 2],
            field: FieldConditions,
            turn_info: TurnState,
        }

        let data = BattleStateDeserialize::deserialize(deserializer)?;
        
        // Create default repositories during deserialization
        let generation_repo = Arc::new(
            crate::data::generation_loader::GenerationRepository::load_from_directory("data/ps-extracted")
                .map_err(|_| serde::de::Error::custom("Failed to load generation repository"))?
        );
        let game_data_repo = Arc::new(
            crate::data::GameDataRepository::from_path("data/ps-extracted")
                .map_err(|_| serde::de::Error::custom("Failed to load game data repository"))?
        );

        Ok(BattleState {
            format: data.format,
            sides: data.sides,
            field: data.field,
            turn_info: data.turn_info,
            generation_repo,
            game_data_repo,
        })
    }
}

impl Default for BattleState {
    fn default() -> Self {
        let generation_repo = Arc::new(
            crate::data::generation_loader::GenerationRepository::load_from_directory("data/ps-extracted")
                .expect("Failed to load generation-specific Pokemon data from data/ps-extracted")
        );
        let game_data_repo = Arc::new(
            crate::data::GameDataRepository::from_path("data/ps-extracted")
                .expect("Failed to load Pokemon data from data/ps-extracted")
        );
        Self::new(BattleFormat::gen9_ou(), generation_repo, game_data_repo)
    }
}

impl BattleState {
    /// Create a new battle state with the specified format
    pub fn new(
        format: BattleFormat,
        generation_repo: Arc<crate::data::generation_loader::GenerationRepository>,
        game_data_repo: Arc<crate::data::GameDataRepository>,
    ) -> Self {
        let side_one = BattleSide::new();
        let side_two = BattleSide::new();
        Self {
            format,
            sides: [side_one, side_two],
            field: FieldConditions::default(),
            turn_info: TurnState::default(),
            generation_repo,
            game_data_repo,
        }
    }

    /// Create a new battle state with teams from random team data
    pub fn new_with_teams(
        format: BattleFormat,
        team_one: Vec<crate::data::RandomPokemonSet>,
        team_two: Vec<crate::data::RandomPokemonSet>,
        generation_repo: Arc<crate::data::generation_loader::GenerationRepository>,
        game_data_repo: Arc<crate::data::GameDataRepository>,
    ) -> Self {
        let mut state = Self::new(format.clone(), generation_repo, game_data_repo.clone());

        // Convert and add Pokemon to each side
        for pokemon_set in team_one {
            let pokemon = pokemon_set.to_battle_pokemon(&game_data_repo);
            state.sides[0].add_pokemon(pokemon);
        }

        for pokemon_set in team_two {
            let pokemon = pokemon_set.to_battle_pokemon(&game_data_repo);
            state.sides[1].add_pokemon(pokemon);
        }

        // Set initial active Pokemon based on format
        let active_count = format.active_pokemon_count();
        for slot in 0..active_count {
            if slot < state.sides[0].pokemon.len() {
                state.sides[0].set_active_pokemon_at_slot(slot, Some(slot));
            }
            if slot < state.sides[1].pokemon.len() {
                state.sides[1].set_active_pokemon_at_slot(slot, Some(slot));
            }
        }

        state
    }

    /// Create a new battle state with pre-constructed Pokemon (for tests, direct team creation)
    pub fn new_with_pokemon(
        format: BattleFormat,
        team_one: Vec<Pokemon>,
        team_two: Vec<Pokemon>,
        generation_repo: Arc<crate::data::generation_loader::GenerationRepository>,
        game_data_repo: Arc<crate::data::GameDataRepository>,
    ) -> Self {
        let mut state = Self::new(format, generation_repo, game_data_repo);
        
        // Add Pokemon to each side
        for pokemon in team_one {
            state.sides[0].add_pokemon(pokemon);
        }
        
        for pokemon in team_two {
            state.sides[1].add_pokemon(pokemon);
        }
        
        // Set initial active Pokemon based on format
        let active_count = state.format.active_pokemon_count();
        for slot in 0..active_count {
            if slot < state.sides[0].pokemon.len() {
                state.sides[0].set_active_pokemon_at_slot(slot, Some(slot));
            }
            if slot < state.sides[1].pokemon.len() {
                state.sides[1].set_active_pokemon_at_slot(slot, Some(slot));
            }
        }
        
        state
    }

    /// Get a reference to a specific side
    pub fn get_side(&self, side_index: usize) -> Option<&BattleSide> {
        self.sides.get(side_index)
    }

    /// Get a mutable reference to a specific side
    pub fn get_side_mut(&mut self, side_index: usize) -> &mut BattleSide {
        &mut self.sides[side_index]
    }

    /// Check if Trick Room is active
    pub fn is_trick_room_active(&self) -> bool {
        self.field.global_effects.trick_room.is_some()
    }

    /// Check if Gravity is active
    pub fn is_gravity_active(&self) -> bool {
        self.field.global_effects.gravity.is_some()
    }

    /// Get the Pokemon at the specified position
    pub fn get_pokemon_at_position(&self, position: BattlePosition) -> Option<&Pokemon> {
        let side_index = match position.side {
            SideReference::SideOne => 0,
            SideReference::SideTwo => 1,
        };
        let side = self.get_side(side_index)?;
        side.get_active_pokemon_at_slot(position.slot)
    }

    /// Get a mutable reference to the Pokemon at the specified position
    pub fn get_pokemon_at_position_mut(
        &mut self,
        position: BattlePosition,
    ) -> Option<&mut Pokemon> {
        let side_index = match position.side {
            SideReference::SideOne => 0,
            SideReference::SideTwo => 1,
        };
        if side_index >= self.sides.len() {
            return None;
        }
        let side = &mut self.sides[side_index];
        side.get_active_pokemon_at_slot_mut(position.slot)
    }

    /// Get current weather
    pub fn weather(&self) -> Weather {
        self.field.weather.condition
    }

    /// Get current terrain
    pub fn terrain(&self) -> Terrain {
        self.field.terrain.condition
    }

    /// Get all active Pokemon positions in the battle
    pub fn get_all_active_positions(&self) -> Vec<BattlePosition> {
        let mut positions = Vec::new();
        
        // Add positions based on battle format
        match self.format.format_type {
            crate::core::battle_format::FormatType::Singles => {
                positions.push(BattlePosition { side: SideReference::SideOne, slot: 0 });
                positions.push(BattlePosition { side: SideReference::SideTwo, slot: 0 });
            }
            crate::core::battle_format::FormatType::Doubles | crate::core::battle_format::FormatType::Vgc => {
                for side in [SideReference::SideOne, SideReference::SideTwo] {
                    for slot in 0..2 {
                        let position = BattlePosition { side, slot };
                        if self.get_pokemon_at_position(position).is_some() {
                            positions.push(position);
                        }
                    }
                }
            }
            crate::core::battle_format::FormatType::Triples => {
                for side in [SideReference::SideOne, SideReference::SideTwo] {
                    for slot in 0..3 {
                        let position = BattlePosition { side, slot };
                        if self.get_pokemon_at_position(position).is_some() {
                            positions.push(position);
                        }
                    }
                }
            }
        }
        
        positions
    }

    /// Get a reference to a side by SideReference
    pub fn get_side_by_ref(&self, side_ref: SideReference) -> &BattleSide {
        match side_ref {
            SideReference::SideOne => &self.sides[0],
            SideReference::SideTwo => &self.sides[1],
        }
    }

    /// Get a mutable reference to a side by SideReference
    pub fn get_side_by_ref_mut(&mut self, side_ref: SideReference) -> &mut BattleSide {
        match side_ref {
            SideReference::SideOne => &mut self.sides[0],
            SideReference::SideTwo => &mut self.sides[1],
        }
    }

    /// Get generation mechanics
    pub fn get_generation_mechanics(&self) -> crate::generation::GenerationMechanics {
        self.format.generation.get_mechanics()
    }

    /// Get the generation for this battle
    pub fn get_generation(&self) -> crate::generation::Generation {
        self.format.generation
    }

    /// Check if a generation feature is available in this battle
    pub fn has_generation_feature(&self, feature: crate::generation::GenerationFeature) -> bool {
        self.format.generation.get_mechanics().has_feature(feature)
    }

    /// Check if a position is active (has a Pokemon that isn't fainted)
    pub fn is_position_active(&self, position: BattlePosition) -> bool {
        if let Some(pokemon) = self.get_pokemon_at_position(position) {
            pokemon.hp > 0
        } else {
            false
        }
    }

    /// Apply a list of battle instructions to modify the state
    pub fn apply_instructions(&mut self, instructions: &[BattleInstruction]) {
        for instruction in instructions {
            self.apply_single_instruction(instruction);
        }
    }

    /// Apply a single battle instruction
    pub fn apply_instruction(&mut self, instruction: &BattleInstruction) {
        self.apply_single_instruction(instruction);
    }

    /// Apply a single battle instruction (internal helper)
    fn apply_single_instruction(&mut self, instruction: &BattleInstruction) {
        match instruction {
            BattleInstruction::Pokemon(pokemon_instr) => {
                self.apply_pokemon_instruction(pokemon_instr);
            }
            BattleInstruction::Field(field_instr) => {
                self.apply_field_instruction(field_instr);
            }
            BattleInstruction::Status(status_instr) => {
                self.apply_status_instruction(status_instr);
            }
            BattleInstruction::Stats(stats_instr) => {
                self.apply_stats_instruction(stats_instr);
            }
        }
    }

    /// Apply Pokemon instruction (damage, healing, switching, etc.)
    fn apply_pokemon_instruction(&mut self, instruction: &PokemonInstruction) {
        let _ = self.apply_pokemon_instruction_with_substitute_info(instruction);
    }

    /// Apply Pokemon instruction with substitute information for effect blocking
    fn apply_pokemon_instruction_with_substitute_info(
        &mut self,
        instruction: &PokemonInstruction,
    ) -> Option<crate::engine::combat::core::SubstituteDamageResult> {
        use crate::engine::combat::core::SubstituteDamageResult;
        match instruction {
            PokemonInstruction::Damage { target, amount, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    // Check if Pokemon has a substitute
                    if pokemon.volatile_statuses.contains(VolatileStatus::Substitute) && pokemon.substitute_health > 0 {
                        // Damage goes to substitute first
                        let current_substitute_health = pokemon.substitute_health;
                        let remaining_substitute_health = current_substitute_health - amount;
                        
                        if remaining_substitute_health <= 0 {
                            // Substitute is broken, apply excess damage to Pokemon
                            pokemon.substitute_health = 0;
                            pokemon.volatile_statuses.remove(VolatileStatus::Substitute);
                            let excess_damage = amount - current_substitute_health;
                            let damage_to_pokemon = if excess_damage > 0 {
                                pokemon.hp = (pokemon.hp - excess_damage).max(0);
                                excess_damage
                            } else {
                                0
                            };
                            
                            return Some(SubstituteDamageResult {
                                hit_substitute: true,
                                substitute_broken: true,
                                damage_to_pokemon,
                            });
                        } else {
                            // Substitute absorbs all damage
                            pokemon.substitute_health = remaining_substitute_health;
                            
                            return Some(SubstituteDamageResult {
                                hit_substitute: true,
                                substitute_broken: false,
                                damage_to_pokemon: 0,
                            });
                        }
                    } else {
                        // No substitute or substitute broken, damage goes to Pokemon directly
                        pokemon.hp = (pokemon.hp - amount).max(0);
                        
                        return Some(SubstituteDamageResult {
                            hit_substitute: false,
                            substitute_broken: false,
                            damage_to_pokemon: *amount,
                        });
                    }
                }
            }
            PokemonInstruction::Heal { target, amount, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.hp = (pokemon.hp + amount).min(pokemon.max_hp);
                }
            }
            PokemonInstruction::MultiTargetDamage { target_damages, .. } => {
                for (target, damage) in target_damages {
                    if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                        // Apply damage directly to Pokemon HP
                        // Substitute handling should be done through explicit instructions
                        pokemon.hp = (pokemon.hp - damage).max(0);
                    }
                }
            }
            PokemonInstruction::Faint { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.hp = 0;
                    pokemon.status = PokemonStatus::None;
                    pokemon.volatile_statuses.clear();
                    // volatile_statuses.clear() already clears durations
                }
            }
            PokemonInstruction::Switch {
                position,
                new_pokemon,
                ..
            } => {
                let side_index = match position.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .set_active_pokemon_at_slot(position.slot, Some(*new_pokemon));
                }
            }
            PokemonInstruction::ChangeAbility {
                target,
                new_ability,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.ability = *new_ability;
                }
            }
            PokemonInstruction::ChangeItem {
                target, new_item, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.item = *new_item;
                }
            }
            PokemonInstruction::ChangeType {
                target, new_types, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.types = new_types.iter().filter_map(|type_str| PokemonType::from_normalized_str(type_str)).collect();
                }
            }
            PokemonInstruction::ToggleTerastallized {
                target,
                terastallized,
                tera_type,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.is_terastallized = *terastallized;
                    pokemon.tera_type = *tera_type;
                }
            }
            PokemonInstruction::ChangeSubstituteHealth {
                target, new_health, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.substitute_health = *new_health;
                }
            }
            PokemonInstruction::SetWish {
                target,
                heal_amount,
                turns_remaining,
                ..
            } => {
                let side_index = match target.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .wish_healing
                        .insert(target.slot, (*heal_amount, *turns_remaining));
                }
            }
            PokemonInstruction::DecrementWish { target, .. } => {
                let side_index = match target.side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    // First, check and update wish turns, storing heal amount for later use
                    let mut should_heal = false;
                    let mut heal_amount = 0;

                    if let Some((wish_heal_amount, turns)) =
                        self.sides[side_index].wish_healing.get_mut(&target.slot)
                    {
                        if *turns > 0 {
                            *turns -= 1;
                            if *turns == 0 {
                                should_heal = true;
                                heal_amount = *wish_heal_amount;
                            }
                        }
                    }

                    // Apply healing and cleanup separately to avoid borrowing conflicts
                    if should_heal {
                        if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                            pokemon.hp = (pokemon.hp + heal_amount).min(pokemon.max_hp);
                        }
                        self.sides[side_index].wish_healing.remove(&target.slot);
                    }
                }
            }
            PokemonInstruction::MultiTargetDamage { target_damages, .. } => {
                for (target, damage) in target_damages {
                    if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                        pokemon.hp = pokemon.hp.saturating_sub(*damage);
                        if pokemon.hp == 0 {
                            self.check_and_handle_faint(*target);
                        }
                    }
                }
            }
            PokemonInstruction::ChangeAbility { target, new_ability, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.ability = *new_ability;
                }
            }
            PokemonInstruction::ToggleAbility { target, suppressed, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.ability_suppressed = *suppressed;
                }
            }
            PokemonInstruction::ChangeItem { target, new_item, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.item = *new_item;
                }
            }
            PokemonInstruction::ChangeType { target, new_types, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    use crate::types::PokemonType;
                    use crate::types::from_string::FromNormalizedString;
                    pokemon.types = new_types.iter()
                        .filter_map(|type_str| PokemonType::from_normalized_str(&crate::utils::normalize_name(type_str)))
                        .collect();
                }
            }
            PokemonInstruction::FormeChange { target, new_forme, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.forme = Some(new_forme.clone());
                }
            }
            PokemonInstruction::ToggleTerastallized { target, terastallized, tera_type, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.is_terastallized = *terastallized;
                    if let Some(tera_type) = tera_type {
                        pokemon.tera_type = Some(*tera_type);
                    }
                }
            }
            PokemonInstruction::ChangeSubstituteHealth { target, new_health, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.substitute_health = *new_health;
                    if *new_health <= 0 {
                        pokemon.volatile_statuses.remove(crate::types::VolatileStatus::Substitute);
                    }
                }
            }
            PokemonInstruction::SetFutureSight { target, attacker_position, damage_amount, turns_remaining, move_name, .. } => {
                let side_index = match target.side {
                    crate::core::battle_format::SideReference::SideOne => 0,
                    crate::core::battle_format::SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index].future_sight.insert(
                        target.slot,
                        (*attacker_position, *damage_amount, *turns_remaining, move_name.clone())
                    );
                }
            }
            PokemonInstruction::DecrementFutureSight { target, .. } => {
                let side_index = match target.side {
                    crate::core::battle_format::SideReference::SideOne => 0,
                    crate::core::battle_format::SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    // First, get and update the future sight data
                    let should_apply_damage = if let Some((_, damage, turns, _)) = self.sides[side_index].future_sight.get_mut(&target.slot) {
                        *turns = turns.saturating_sub(1);
                        if *turns == 0 {
                            Some(*damage)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    // Then apply damage if needed
                    if let Some(damage) = should_apply_damage {
                        if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                            pokemon.hp = pokemon.hp.saturating_sub(damage);
                            if pokemon.hp == 0 {
                                self.check_and_handle_faint(*target);
                            }
                        }
                        self.sides[side_index].future_sight.remove(&target.slot);
                    }
                }
            }
            PokemonInstruction::ChangeDamageDealt { side_position, damage_amount, move_category, hit_substitute, .. } => {
                let side_index = match side_position.side {
                    crate::core::battle_format::SideReference::SideOne => 0,
                    crate::core::battle_format::SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index].last_damage_taken = *damage_amount;
                    self.sides[side_index].last_move_category = Some(*move_category);
                    self.sides[side_index].last_hit_substitute = *hit_substitute;
                }
            }
            PokemonInstruction::ItemTransfer { from, to, item, .. } => {
                // Get the item from the from Pokemon and transfer it to the to Pokemon
                let item_to_transfer = if let Some(from_pokemon) = self.get_pokemon_at_position_mut(*from) {
                    let item = from_pokemon.item.take();
                    item
                } else {
                    None
                };
                
                if let Some(to_pokemon) = self.get_pokemon_at_position_mut(*to) {
                    use crate::types::Items;
                    use crate::types::from_string::FromNormalizedString;
                    to_pokemon.item = Items::from_normalized_str(&crate::utils::normalize_name(item));
                }
            }
            PokemonInstruction::ForceSwitch { target, .. } => {
                // Mark the Pokemon as needing to switch out
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.must_switch = true;
                }
            }
            PokemonInstruction::DamageSubstitute { target, amount, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.substitute_health = pokemon.substitute_health.saturating_sub(*amount);
                    if pokemon.substitute_health <= 0 {
                        pokemon.volatile_statuses.remove(crate::types::VolatileStatus::Substitute);
                        pokemon.substitute_health = 0;
                    }
                }
            }
            PokemonInstruction::Message { .. } => {
                // Messages are for logging/debugging, no state change needed
            }
        }
        
        None
    }

    /// Apply Field instruction (weather, terrain, global effects, side conditions)
    fn apply_field_instruction(&mut self, instruction: &FieldInstruction) {
        match instruction {
            FieldInstruction::Weather {
                new_weather,
                turns,
                source,
                ..
            } => {
                self.field.weather.set(*new_weather, *turns, *source);
            }
            FieldInstruction::Terrain {
                new_terrain,
                turns,
                source,
                ..
            } => {
                self.field.terrain.set(*new_terrain, *turns, *source);
            }
            FieldInstruction::TrickRoom {
                active,
                turns,
                source,
                ..
            } => {
                if *active {
                    if let Some(turn_count) = turns {
                        self.field
                            .global_effects
                            .set_trick_room(*turn_count, *source);
                    }
                } else {
                    self.field.global_effects.clear_trick_room();
                }
            }
            FieldInstruction::Gravity {
                active,
                turns,
                source,
                ..
            } => {
                if *active {
                    if let Some(turn_count) = turns {
                        self.field.global_effects.set_gravity(*turn_count, *source);
                    }
                } else {
                    self.field.global_effects.clear_gravity();
                }
            }
            FieldInstruction::ApplySideCondition {
                side,
                condition,
                duration,
                ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index]
                        .side_conditions
                        .insert(*condition, *duration);
                }
            }
            FieldInstruction::RemoveSideCondition {
                side, condition, ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    self.sides[side_index].side_conditions.remove(condition);
                }
            }
            FieldInstruction::DecrementSideConditionDuration {
                side, condition, ..
            } => {
                let side_index = match side {
                    SideReference::SideOne => 0,
                    SideReference::SideTwo => 1,
                };
                if side_index < self.sides.len() {
                    if let Some(duration) =
                        self.sides[side_index].side_conditions.get_mut(condition)
                    {
                        if *duration > 0 {
                            *duration -= 1;
                            if *duration == 0 {
                                self.sides[side_index].side_conditions.remove(condition);
                            }
                        }
                    }
                }
            }
            FieldInstruction::DecrementWeatherTurns { .. } => {
                self.field.weather.decrement_turn();
            }
            FieldInstruction::DecrementTerrainTurns { .. } => {
                self.field.terrain.decrement_turn();
            }
            FieldInstruction::DecrementTrickRoomTurns { .. } => {
                self.field.global_effects.decrement_turn();
            }
            FieldInstruction::DecrementGravityTurns { .. } => {
                self.field.global_effects.decrement_turn();
            }
            FieldInstruction::ToggleForceSwitch { .. } => {
                // Force switch logic would be handled at a higher level
                // This is more of a metadata instruction for battle flow
            }
            FieldInstruction::ToggleBatonPassing { .. } => {
                // Baton passing logic would be handled at a higher level
                // This is more of a metadata instruction for switch mechanics
            }
            FieldInstruction::Message { .. } => {
                // Messages are for logging/debugging purposes and don't change state
                // Could be logged to a battle log if needed
            }
        }
    }

    /// Apply Status instruction (status conditions, volatile statuses)
    fn apply_status_instruction(&mut self, instruction: &StatusInstruction) {
        match instruction {
            StatusInstruction::Apply {
                target,
                status,
                duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = *status;
                    pokemon.status_duration = *duration;
                }
            }
            StatusInstruction::Remove { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::None;
                    pokemon.status_duration = None;
                }
            }
            StatusInstruction::ChangeDuration {
                target,
                new_duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status_duration = *new_duration;
                    if new_duration.is_none() || new_duration == &Some(0) {
                        pokemon.status = PokemonStatus::None;
                    }
                }
            }
            StatusInstruction::ApplyVolatile {
                target,
                status,
                duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.insert(*status);
                    if let Some(_dur) = duration {
                        // Duration tracking is handled within VolatileStatusStorage
                        // For now, just insert the status - duration logic can be added later
                    }
                }
            }
            StatusInstruction::RemoveVolatile { target, status, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.remove(*status);
                    // Duration removal is handled by VolatileStatusStorage
                }
            }
            StatusInstruction::ChangeVolatileDuration {
                target,
                status,
                new_duration,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(new_dur) = new_duration {
                        // Duration tracking is handled within VolatileStatusStorage
                        if *new_dur == 0 {
                            pokemon.volatile_statuses.remove(*status);
                        }
                    } else {
                        // Duration removal is handled by VolatileStatusStorage
                    }
                }
            }
            StatusInstruction::SetSleepTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::Sleep;
                    pokemon.status_duration = Some(*turns);
                }
            }
            StatusInstruction::SetRestTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::Sleep;
                    pokemon.status_duration = Some(*turns);
                    // Rest also heals to full HP
                    pokemon.hp = pokemon.max_hp;
                }
            }
            StatusInstruction::DecrementRestTurns { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(turns) = pokemon.status_duration {
                        if turns > 0 {
                            pokemon.status_duration = Some(turns - 1);
                            if turns - 1 == 0 {
                                pokemon.status = PokemonStatus::None;
                                pokemon.status_duration = None;
                            }
                        }
                    }
                }
            }
            StatusInstruction::DecrementPP {
                target,
                move_index,
                amount,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(move_data) = pokemon.get_move_mut(*move_index) {
                        move_data.pp = move_data.pp.saturating_sub(*amount);
                    }
                }
            }
            StatusInstruction::Apply { target, status, duration, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = *status;
                    pokemon.status_duration = *duration;
                }
            }
            StatusInstruction::Remove { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status = PokemonStatus::None;
                    pokemon.status_duration = None;
                }
            }
            StatusInstruction::ChangeDuration { target, new_duration, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.status_duration = *new_duration;
                }
            }
            StatusInstruction::ApplyVolatile { target, status, duration, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.insert(*status);
                    if let Some(duration) = duration {
                        pokemon.volatile_status_durations.insert(*status, *duration);
                    }
                }
            }
            StatusInstruction::RemoveVolatile { target, status, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.volatile_statuses.remove(*status);
                    pokemon.volatile_status_durations.remove(status);
                }
            }
            StatusInstruction::ChangeVolatileDuration { target, status, new_duration, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(new_duration) = new_duration {
                        pokemon.volatile_status_durations.insert(*status, *new_duration);
                    } else {
                        pokemon.volatile_status_durations.remove(status);
                    }
                }
            }
            StatusInstruction::SetSleepTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.sleep_turns = Some(*turns);
                }
            }
            StatusInstruction::SetRestTurns { target, turns, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.rest_turns = Some(*turns);
                }
            }
            StatusInstruction::DecrementRestTurns { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    if let Some(rest_turns) = pokemon.rest_turns {
                        pokemon.rest_turns = Some(rest_turns.saturating_sub(1));
                        if pokemon.rest_turns == Some(0) {
                            pokemon.rest_turns = None;
                            pokemon.status = PokemonStatus::None;
                            pokemon.status_duration = None;
                        }
                    }
                }
            }
            StatusInstruction::DisableMove { target, move_index, duration, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.disabled_moves.insert(*move_index, *duration);
                }
            }
            StatusInstruction::EnableMove { target, move_index, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.disabled_moves.remove(move_index);
                }
            }
            StatusInstruction::SetLastUsedMove { target, move_name, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.last_used_move = Some(move_name.clone());
                }
            }
            StatusInstruction::RestoreLastUsedMove { target, move_name, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.last_used_move = Some(move_name.clone());
                }
            }
        }
    }

    /// Apply Stats instruction (stat boosts, raw stat changes)
    fn apply_stats_instruction(&mut self, instruction: &StatsInstruction) {
        match instruction {
            StatsInstruction::BoostStats {
                target,
                stat_changes,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    for (&stat, &change) in stat_changes.iter() {
                        let current_boost = pokemon.stat_boosts.get_direct(stat);
                        let new_boost = (current_boost + change).clamp(-6, 6);
                        pokemon.stat_boosts.insert(stat, new_boost);
                    }
                }
            }
            StatsInstruction::ChangeAttack {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.attack = *new_value;
                }
            }
            StatsInstruction::ChangeDefense {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.defense = *new_value;
                }
            }
            StatsInstruction::ChangeSpecialAttack {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.special_attack = *new_value;
                }
            }
            StatsInstruction::ChangeSpecialDefense {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.special_defense = *new_value;
                }
            }
            StatsInstruction::ChangeSpeed {
                target, new_value, ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stats.speed = *new_value;
                }
            }
            StatsInstruction::ClearBoosts { target, .. } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    pokemon.stat_boosts.clear();
                }
            }
            StatsInstruction::CopyBoosts {
                target,
                source,
                stats_to_copy,
                ..
            } => {
                if let Some(source_pokemon) = self.get_pokemon_at_position(*source) {
                    // Copy the boosts to apply to target
                    let mut boosts_to_copy = HashMap::new();
                    for stat in stats_to_copy {
                        let boost = source_pokemon.stat_boosts.get_direct(*stat);
                        boosts_to_copy.insert(*stat, boost);
                    }

                    // Apply to target (need to get mutable reference after immutable one)
                    if let Some(target_pokemon) = self.get_pokemon_at_position_mut(*target) {
                        for (stat, boost) in boosts_to_copy {
                            target_pokemon.stat_boosts.insert(stat, boost);
                        }
                    }
                }
            }
            StatsInstruction::SwapBoosts {
                target1,
                target2,
                stats_to_swap,
                ..
            } => {
                // Collect boosts from both Pokemon
                let mut target1_boosts = HashMap::new();
                let mut target2_boosts = HashMap::new();

                if let Some(pokemon1) = self.get_pokemon_at_position(*target1) {
                    for stat in stats_to_swap {
                        target1_boosts
                            .insert(*stat, pokemon1.stat_boosts.get_direct(*stat));
                    }
                }

                if let Some(pokemon2) = self.get_pokemon_at_position(*target2) {
                    for stat in stats_to_swap {
                        target2_boosts
                            .insert(*stat, pokemon2.stat_boosts.get_direct(*stat));
                    }
                }

                // Apply swapped boosts
                if let Some(pokemon1) = self.get_pokemon_at_position_mut(*target1) {
                    for (stat, boost) in &target2_boosts {
                        pokemon1.stat_boosts.insert(*stat, *boost);
                    }
                }

                if let Some(pokemon2) = self.get_pokemon_at_position_mut(*target2) {
                    for (stat, boost) in &target1_boosts {
                        pokemon2.stat_boosts.insert(*stat, *boost);
                    }
                }
            }
            StatsInstruction::InvertBoosts {
                target,
                stats_to_invert,
                ..
            } => {
                if let Some(pokemon) = self.get_pokemon_at_position_mut(*target) {
                    for stat in stats_to_invert {
                        let boost = pokemon.stat_boosts.get_mut(*stat);
                        *boost = -*boost;
                    }
                }
            }
        }
    }

    /// Check if the battle has ended (all Pokemon fainted on one side)
    pub fn is_battle_over(&self) -> bool {
        let side_one_has_usable = self.sides[0].pokemon.iter().any(|p| p.hp > 0);
        let side_two_has_usable = self.sides[1].pokemon.iter().any(|p| p.hp > 0);

        !side_one_has_usable || !side_two_has_usable
    }

    /// Determine which side (0 or 1) has won, if any
    pub fn get_winner(&self) -> Option<usize> {
        let side_one_has_usable = self.sides[0].pokemon.iter().any(|p| p.hp > 0);
        let side_two_has_usable = self.sides[1].pokemon.iter().any(|p| p.hp > 0);

        match (side_one_has_usable, side_two_has_usable) {
            (false, true) => Some(1), // Side two wins
            (true, false) => Some(0), // Side one wins
            _ => None,                // Battle ongoing or tie
        }
    }

    /// Get all legal move options for both sides
    pub fn get_all_options(&self) -> (Vec<MoveChoice>, Vec<MoveChoice>) {
        let side_one_options = self.get_side_options(0);
        let side_two_options = self.get_side_options(1);
        (side_one_options, side_two_options)
    }

    /// Get move options for a specific side
    fn get_side_options(&self, side_index: usize) -> Vec<MoveChoice> {
        let mut options = Vec::new();

        if let Some(side) = self.get_side(side_index) {
            let active_count = self.format.active_pokemon_count();

            for slot in 0..active_count {
                if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                    if pokemon.hp > 0 {
                        // Add move options
                        for (move_index, move_data) in &pokemon.moves {
                            if move_data.pp > 0 {
                                let targets = self.get_valid_targets_for_move(
                                    side_index,
                                    slot,
                                    &move_data.target,
                                );
                                options.push(MoveChoice::new_move(*move_index, targets));
                            }
                        }

                        // Add switch options if there are benched Pokemon
                        for (i, bench_pokemon) in side.pokemon.iter().enumerate() {
                            if bench_pokemon.hp > 0
                                && !side.active_pokemon_indices.contains(&Some(i))
                            {
                                if let Some(pokemon_index) = PokemonIndex::from_index(i) {
                                    options.push(MoveChoice::new_switch(pokemon_index));
                                }
                            }
                        }
                    }
                }
            }
        }

        options
    }

    /// Get valid targets for a move based on its target type and format
    fn get_valid_targets_for_move(
        &self,
        user_side_index: usize,
        user_slot: usize,
        move_target: &crate::data::showdown_types::MoveTarget,
    ) -> Vec<BattlePosition> {
        let mut targets = Vec::new();
        let active_count = self.format.active_pokemon_count();
        let opponent_side_index = 1 - user_side_index;

        let user_side_ref = if user_side_index == 0 {
            SideReference::SideOne
        } else {
            SideReference::SideTwo
        };

        let opponent_side_ref = if opponent_side_index == 0 {
            SideReference::SideOne
        } else {
            SideReference::SideTwo
        };

        match move_target {
            crate::data::showdown_types::MoveTarget::Self_ => {
                // Target the user
                targets.push(BattlePosition {
                    side: user_side_ref,
                    slot: user_slot,
                });
            }
            crate::data::showdown_types::MoveTarget::Normal
            | crate::data::showdown_types::MoveTarget::AdjacentFoe => {
                // Target adjacent opponents
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
                // In singles, just return the first valid target
                if active_count == 1 && !targets.is_empty() {
                    targets.truncate(1);
                }
            }
            crate::data::showdown_types::MoveTarget::AllAdjacentFoes => {
                // All adjacent opponents (spread move)
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AllAdjacent => {
                // All adjacent Pokemon (including allies)
                for slot in 0..active_count {
                    if slot != user_slot {
                        let ally_position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(ally_position) {
                            targets.push(ally_position);
                        }
                    }
                    let opponent_position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(opponent_position) {
                        targets.push(opponent_position);
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AdjacentAlly => {
                // Adjacent allies only
                for slot in 0..active_count {
                    if slot != user_slot {
                        let position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::AdjacentAllyOrSelf => {
                // User or adjacent ally
                targets.push(BattlePosition {
                    side: user_side_ref,
                    slot: user_slot,
                });
                for slot in 0..active_count {
                    if slot != user_slot {
                        let position = BattlePosition {
                            side: user_side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::All
            | crate::data::showdown_types::MoveTarget::AllySide
            | crate::data::showdown_types::MoveTarget::FoeSide
            | crate::data::showdown_types::MoveTarget::AllyTeam => {
                // Field-wide moves don't need specific targets
                // Return empty vector as they affect the field/side itself
            }
            crate::data::showdown_types::MoveTarget::Any => {
                // Can target any active Pokemon
                for side_idx in 0..2 {
                    let side_ref = if side_idx == 0 {
                        SideReference::SideOne
                    } else {
                        SideReference::SideTwo
                    };
                    for slot in 0..active_count {
                        let position = BattlePosition {
                            side: side_ref,
                            slot,
                        };
                        if self.is_position_active(position) {
                            targets.push(position);
                        }
                    }
                }
            }
            crate::data::showdown_types::MoveTarget::RandomNormal => {
                // Random opponent - collect all valid opponents
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
            _ => {
                // For other target types, default to normal targeting
                for slot in 0..active_count {
                    let position = BattlePosition {
                        side: opponent_side_ref,
                        slot,
                    };
                    if self.is_position_active(position) {
                        targets.push(position);
                    }
                }
            }
        }

        targets
    }

    /// Get damage tracking information for counter moves
    pub fn get_damage_dealt(&self, side_index: usize) -> &DamageDealt {
        &self.sides[side_index].damage_dealt
    }

    /// Reset damage tracking at turn start
    pub fn reset_damage_dealt(&mut self) {
        for side in &mut self.sides {
            side.damage_dealt.reset();
        }
    }

    /// Advance turn counter and handle turn-based effects
    pub fn update_turn(&mut self) {
        self.turn_info.next_turn();

        // Reset ability triggered flags for all Pokemon
        for side in &mut self.sides {
            for pokemon in &mut side.pokemon {
                pokemon.ability_triggered_this_turn = false;
            }
        }

        // Decrement field effect durations
        self.field.weather.decrement_turn();
        self.field.terrain.decrement_turn();
        self.field.global_effects.decrement_turn();
    }

    /// Track that a position has used a move this turn
    pub fn track_move_used(&mut self, position: BattlePosition) {
        self.turn_info.mark_moved(position);
    }

    /// Track that a position has taken damage this turn
    pub fn track_damage_taken(
        &mut self,
        target: BattlePosition,
        attacker: BattlePosition,
        damage: i16,
        move_category: MoveCategory,
        is_direct: bool,
    ) {
        let damage_info = DamageInfo::new(damage, move_category, attacker, is_direct);
        self.turn_info.mark_damaged(target, damage_info);
    }

    /// Check if user took damage from a physical/special move and moved second this turn
    pub fn user_moved_after_taking_damage(&self, user_position: BattlePosition) -> bool {
        self.turn_info.took_damage_from_attack(user_position)
            && self.turn_info.has_moved(user_position)
    }

    /// Generate a human-readable string representation of the battle state
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Battle Turn {} ===\n", self.turn_info.number));
        output.push_str(&format!("Format: {}\n", self.format.name));
        output.push_str(&format!("Weather: {:?}\n", self.field.weather.condition));
        output.push_str(&format!("Terrain: {:?}\n", self.field.terrain.condition));

        if self.is_trick_room_active() {
            output.push_str("Trick Room is active\n");
        }
        if self.is_gravity_active() {
            output.push_str("Gravity is active\n");
        }

        output.push_str("\n--- Side One ---\n");
        output.push_str(&self.format_side(&self.sides[0], 0));

        output.push_str("\n--- Side Two ---\n");
        output.push_str(&self.format_side(&self.sides[1], 1));

        output
    }

    /// Format a side for pretty printing
    fn format_side(&self, side: &BattleSide, side_index: usize) -> String {
        let mut output = String::new();
        let active_count = self.format.active_pokemon_count();

        output.push_str("Active Pokemon:\n");
        for slot in 0..active_count {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                output.push_str(&format!(
                    "  Slot {}: {} ({}/{} HP)\n",
                    slot, pokemon.species, pokemon.hp, pokemon.max_hp
                ));

                if pokemon.status != PokemonStatus::None {
                    output.push_str(&format!("    Status: {:?}\n", pokemon.status));
                }

                if !pokemon.volatile_statuses.is_empty() {
                    output.push_str(&format!("    Volatile: {:?}\n", pokemon.volatile_statuses));
                }
            } else {
                output.push_str(&format!("  Slot {}: Empty\n", slot));
            }
        }

        if !side.side_conditions.is_empty() {
            output.push_str(&format!("Side Conditions: {:?}\n", side.side_conditions));
        }

        output
    }

    /// Check if a Pokemon fainted and handle faint-related effects
    fn check_and_handle_faint(&mut self, position: BattlePosition) {
        if let Some(pokemon) = self.get_pokemon_at_position(position) {
            if pokemon.hp <= 0 {
                // Pokemon has fainted - could trigger additional effects here
                // For now, just ensure HP is exactly 0
                if let Some(pokemon_mut) = self.get_pokemon_at_position_mut(position) {
                    pokemon_mut.hp = 0;
                }
            }
        }
    }
}