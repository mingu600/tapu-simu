//! # Team Builder
//! 
//! Standardized team builder implementing the common Builder trait
//! with comprehensive validation and error handling.

use crate::core::battle_format::BattleFormat;
use crate::data::GameDataRepository;
use crate::data::RandomPokemonSet;
use crate::types::identifiers::{SpeciesId, AbilityId, MoveId, ItemId};
use super::traits::{Builder, BuilderError, ValidationContext, ValidatingBuilder};

/// Team builder with standardized interface
pub struct TeamBuilder<'a> {
    /// Data repository for validation
    data: &'a GameDataRepository,
    /// Pokemon on the team
    pokemon: Vec<PokemonBuilder>,
    /// Format for validation
    format: Option<BattleFormat>,
    /// Validation context
    validation_context: ValidationContext,
}

/// Builder for individual Pokemon
#[derive(Debug, Clone)]
pub struct PokemonBuilder {
    /// Species ID
    species: SpeciesId,
    /// Level (1-100)
    level: Option<u8>,
    /// Ability
    ability: Option<AbilityId>,
    /// Held item
    item: Option<ItemId>,
    /// Moves (up to 4)
    moves: Vec<MoveId>,
    /// Nature
    nature: Option<String>,
    /// EVs (Effort Values)
    evs: Option<EVsConfig>,
    /// IVs (Individual Values)
    ivs: Option<IVsConfig>,
}

/// EV configuration
#[derive(Debug, Clone, Default)]
pub struct EVsConfig {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}

/// IV configuration
#[derive(Debug, Clone)]
pub struct IVsConfig {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub special_attack: u8,
    pub special_defense: u8,
    pub speed: u8,
}

impl Default for IVsConfig {
    fn default() -> Self {
        Self {
            hp: 31,
            attack: 31,
            defense: 31,
            special_attack: 31,
            special_defense: 31,
            speed: 31,
        }
    }
}

impl<'a> TeamBuilder<'a> {
    /// Create a new modern team builder
    pub fn new(data: &'a GameDataRepository) -> Self {
        Self {
            data,
            pokemon: Vec::new(),
            format: None,
            validation_context: ValidationContext::default(),
        }
    }

    /// Set the format for validation
    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Add a Pokemon using the builder pattern
    pub fn pokemon(mut self, species: impl Into<SpeciesId>) -> TeamPokemonContext<'a> {
        TeamPokemonContext {
            team_builder: self,
            pokemon_builder: PokemonBuilder::new(species.into()),
        }
    }

    /// Add a pre-built Pokemon
    pub fn add_pokemon(mut self, pokemon: PokemonBuilder) -> Self {
        self.pokemon.push(pokemon);
        self
    }

    /// Set validation context
    pub fn validation_context(mut self, context: ValidationContext) -> Self {
        self.validation_context = context;
        self
    }

    /// Validate team size for current format
    fn validate_team_size(&self) -> Result<(), BuilderError> {
        let expected_size = match self.format.as_ref().map(|f| &f.format_type) {
            Some(crate::core::battle_format::FormatType::Singles) => 6,
            Some(crate::core::battle_format::FormatType::Doubles) => 6,
            Some(crate::core::battle_format::FormatType::Vgc) => 6,
            Some(crate::core::battle_format::FormatType::Triples) => 6,
            None => 6, // Default assumption
        };

        if self.pokemon.len() != expected_size {
            return Err(BuilderError::InvalidValue {
                field: "team_size".to_string(),
                value: self.pokemon.len().to_string(),
                reason: format!("Expected {} Pokemon, got {}", expected_size, self.pokemon.len()),
            });
        }

        Ok(())
    }

    /// Validate individual Pokemon
    fn validate_pokemon(&self) -> Result<(), BuilderError> {
        for (i, pokemon) in self.pokemon.iter().enumerate() {
            pokemon.validate(self.data)
                .map_err(|e| match e {
                    BuilderError::InvalidValue { field, value, reason } => {
                        BuilderError::InvalidValue {
                            field: format!("pokemon[{}].{}", i, field),
                            value,
                            reason,
                        }
                    }
                    other => other,
                })?;
        }
        Ok(())
    }
}

impl<'a> Builder<Vec<RandomPokemonSet>> for TeamBuilder<'a> {
    type Error = BuilderError;

    fn build(self) -> Result<Vec<RandomPokemonSet>, Self::Error> {
        // Validate first
        self.validate()?;

        // Convert PokemonBuilders to RandomPokemonSets
        let mut team = Vec::new();
        for pokemon_builder in self.pokemon {
            let pokemon_set = pokemon_builder.build(self.data)?;
            team.push(pokemon_set);
        }

        Ok(team)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate team size
        self.validate_team_size()?;

        // Validate individual Pokemon
        self.validate_pokemon()?;

        Ok(())
    }
}

impl<'a> ValidatingBuilder<Vec<RandomPokemonSet>> for TeamBuilder<'a> {
    type Context = ValidationContext;

    fn validate_aspect(&self, context: &Self::Context) -> Result<(), Self::Error> {
        if context.strict_mode {
            self.validate()?;
        } else {
            // Lenient validation - just check we have some Pokemon
            if self.pokemon.is_empty() {
                return Err(BuilderError::ValidationFailed {
                    reason: "Team must have at least one Pokemon".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl PokemonBuilder {
    /// Create a new Pokemon builder
    pub fn new(species: SpeciesId) -> Self {
        Self {
            species,
            level: None,
            ability: None,
            item: None,
            moves: Vec::new(),
            nature: None,
            evs: None,
            ivs: None,
        }
    }

    /// Set level
    pub fn level(mut self, level: u8) -> Self {
        self.level = Some(level);
        self
    }

    /// Set ability
    pub fn ability(mut self, ability: impl Into<AbilityId>) -> Self {
        self.ability = Some(ability.into());
        self
    }

    /// Set held item
    pub fn item(mut self, item: impl Into<ItemId>) -> Self {
        self.item = Some(item.into());
        self
    }

    /// Add a move
    pub fn move_slot(mut self, move_id: impl Into<MoveId>) -> Self {
        if self.moves.len() < 4 {
            self.moves.push(move_id.into());
        }
        self
    }

    /// Set nature
    pub fn nature(mut self, nature: impl Into<String>) -> Self {
        self.nature = Some(nature.into());
        self
    }

    /// Set EVs
    pub fn evs(mut self, evs: EVsConfig) -> Self {
        self.evs = Some(evs);
        self
    }

    /// Set IVs
    pub fn ivs(mut self, ivs: IVsConfig) -> Self {
        self.ivs = Some(ivs);
        self
    }

    /// Validate this Pokemon
    pub fn validate(&self, data: &GameDataRepository) -> Result<(), BuilderError> {
        // Validate level
        let level = self.level.unwrap_or(50);
        if level == 0 || level > 100 {
            return Err(BuilderError::InvalidValue {
                field: "level".to_string(),
                value: level.to_string(),
                reason: "Level must be between 1 and 100".to_string(),
            });
        }

        // Validate moves (max 4)
        if self.moves.len() > 4 {
            return Err(BuilderError::InvalidValue {
                field: "moves".to_string(),
                value: self.moves.len().to_string(),
                reason: "Pokemon can have at most 4 moves".to_string(),
            });
        }

        // Validate EVs total (max 510)
        if let Some(ref evs) = self.evs {
            let total = evs.hp as u16 + evs.attack as u16 + evs.defense as u16 
                      + evs.special_attack as u16 + evs.special_defense as u16 + evs.speed as u16;
            if total > 510 {
                return Err(BuilderError::InvalidValue {
                    field: "evs".to_string(),
                    value: total.to_string(),
                    reason: "Total EVs cannot exceed 510".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Build into a RandomPokemonSet
    pub fn build(self, _data: &GameDataRepository) -> Result<RandomPokemonSet, BuilderError> {
        self.validate(_data)?;

        // Convert to RandomPokemonSet
        // This is a simplified conversion - a real implementation would be more comprehensive
        Ok(RandomPokemonSet {
            name: self.species.as_str().to_string(),
            species: self.species.as_str().to_string(),
            level: self.level.unwrap_or(50),
            gender: None,
            shiny: None,
            ability: self.ability.map(|a| a.as_str().to_string()),
            item: self.item.map(|i| i.as_str().to_string()),
            moves: self.moves.into_iter().map(|m| m.as_str().to_string()).collect(),
            nature: self.nature,
            evs: None, // Simplified for now
            ivs: None, // Simplified for now
            gigantamax: None,
            role: None,
            tera_type: None,
        })
    }
}

/// Context for building Pokemon within a team
pub struct TeamPokemonContext<'a> {
    team_builder: TeamBuilder<'a>,
    pokemon_builder: PokemonBuilder,
}

impl<'a> TeamPokemonContext<'a> {
    /// Set level and continue building
    pub fn level(mut self, level: u8) -> Self {
        self.pokemon_builder = self.pokemon_builder.level(level);
        self
    }

    /// Set ability and continue building
    pub fn ability(mut self, ability: impl Into<AbilityId>) -> Self {
        self.pokemon_builder = self.pokemon_builder.ability(ability);
        self
    }

    /// Set item and continue building
    pub fn item(mut self, item: impl Into<ItemId>) -> Self {
        self.pokemon_builder = self.pokemon_builder.item(item);
        self
    }

    /// Add a move and continue building
    pub fn move_slot(mut self, move_id: impl Into<MoveId>) -> Self {
        self.pokemon_builder = self.pokemon_builder.move_slot(move_id);
        self
    }

    /// Set nature and continue building
    pub fn nature(mut self, nature: impl Into<String>) -> Self {
        self.pokemon_builder = self.pokemon_builder.nature(nature);
        self
    }

    /// Finish building this Pokemon and return to team builder
    pub fn finish(mut self) -> TeamBuilder<'a> {
        self.team_builder.pokemon.push(self.pokemon_builder);
        self.team_builder
    }

    /// Add another Pokemon to the team
    pub fn pokemon(self, species: impl Into<SpeciesId>) -> TeamPokemonContext<'a> {
        let mut team_builder = self.finish();
        team_builder.pokemon(species)
    }
}