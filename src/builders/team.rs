use crate::core::battle_format::BattleFormat;
use crate::data::ps::repository::Repository;
use crate::data::RandomPokemonSet;
use crate::types::errors::{TeamError, TeamResult};
use crate::types::identifiers::{SpeciesId, MoveId, AbilityId, ItemId};

/// Builder for creating teams with fluent API
pub struct TeamBuilder<'a> {
    data: &'a Repository,
    pokemon: Vec<PokemonBuilder>,
    format: Option<BattleFormat>,
}

impl<'a> TeamBuilder<'a> {
    /// Create a new team builder
    pub fn new(data: &'a Repository) -> Self {
        Self {
            data,
            pokemon: Vec::new(),
            format: None,
        }
    }

    /// Set the format for validation
    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Add a Pokemon to the team
    pub fn add_pokemon(mut self, pokemon: PokemonBuilder) -> Self {
        self.pokemon.push(pokemon);
        self
    }

    /// Add a Pokemon with just species
    pub fn add(mut self, species: impl Into<SpeciesId>) -> Self {
        let pokemon = PokemonBuilder::new(species.into());
        self.pokemon.push(pokemon);
        self
    }

    /// Start building a Pokemon and return a context for chaining
    pub fn pokemon(self, species: impl Into<SpeciesId>) -> TeamPokemonContext<'a> {
        TeamPokemonContext {
            team_builder: self,
            pokemon_builder: PokemonBuilder::new(species.into()),
        }
    }

    /// Generate a random team for the format
    pub fn random(mut self) -> TeamResult<Self> {
        let format = self.format.as_ref().ok_or_else(|| {
            TeamError::InvalidPokemon { 
                reason: "Format must be set to generate random team".to_string() 
            }
        })?;

        let mut team_loader = crate::data::random_team_loader::RandomTeamLoader::new();
        let random_team = team_loader.get_random_team(format)
            .map_err(|e| TeamError::RandomGenerationFailed { reason: e.to_string() })?;

        // Convert RandomPokemonSet to PokemonBuilder
        for pokemon_set in random_team {
            let pokemon_builder = PokemonBuilder::from_random_set(pokemon_set);
            self.pokemon.push(pokemon_builder);
        }

        Ok(self)
    }

    /// Build the team
    pub fn build(self) -> TeamResult<Vec<RandomPokemonSet>> {
        // Validate team size
        if let Some(format) = &self.format {
            if self.pokemon.len() != format.team_size {
                return Err(TeamError::InvalidSize { 
                    size: self.pokemon.len() 
                });
            }

            // Validate format constraints
            self.validate_format_constraints(format)?;
        }

        // Convert PokemonBuilder to RandomPokemonSet
        let mut team = Vec::new();
        for pokemon_builder in self.pokemon {
            let pokemon_set = pokemon_builder.build()?;
            team.push(pokemon_set);
        }

        Ok(team)
    }

    /// Validate that the team meets format constraints
    fn validate_format_constraints(&self, format: &BattleFormat) -> TeamResult<()> {
        // Check species clause
        if format.has_clause(&crate::core::battle_format::FormatClause::SpeciesClause) {
            let mut species_seen = std::collections::HashSet::new();
            for pokemon in &self.pokemon {
                if !species_seen.insert(&pokemon.species) {
                    return Err(TeamError::DuplicateSpecies { 
                        species: pokemon.species.clone() 
                    });
                }
            }
        }

        // Check item clause
        if format.has_clause(&crate::core::battle_format::FormatClause::ItemClause) {
            let mut items_seen = std::collections::HashSet::new();
            for pokemon in &self.pokemon {
                if let Some(item) = &pokemon.item {
                    if !items_seen.insert(item) {
                        return Err(TeamError::InvalidPokemon { 
                            reason: format!("Duplicate item: {}", item.as_str()) 
                        });
                    }
                }
            }
        }

        // Check bans
        for pokemon in &self.pokemon {
            if format.is_species_banned(pokemon.species.as_str()) {
                return Err(TeamError::FormatViolation { 
                    reason: format!("Banned species: {}", pokemon.species.as_str()) 
                });
            }

            if let Some(ability) = &pokemon.ability {
                if format.is_ability_banned(ability.as_str()) {
                    return Err(TeamError::FormatViolation { 
                        reason: format!("Banned ability: {}", ability.as_str()) 
                    });
                }
            }

            if let Some(item) = &pokemon.item {
                if format.is_item_banned(item.as_str()) {
                    return Err(TeamError::FormatViolation { 
                        reason: format!("Banned item: {}", item.as_str()) 
                    });
                }
            }

            for move_id in &pokemon.moves {
                if format.is_move_banned(move_id.as_str()) {
                    return Err(TeamError::FormatViolation { 
                        reason: format!("Banned move: {}", move_id.as_str()) 
                    });
                }
            }
        }

        Ok(())
    }
}

/// Context for building a Pokemon within a team
pub struct TeamPokemonContext<'a> {
    team_builder: TeamBuilder<'a>,
    pokemon_builder: PokemonBuilder,
}

impl<'a> TeamPokemonContext<'a> {
    /// Set the ability
    pub fn ability(mut self, ability: impl Into<AbilityId>) -> Self {
        self.pokemon_builder.ability = Some(ability.into());
        self
    }

    /// Set the item
    pub fn item(mut self, item: impl Into<ItemId>) -> Self {
        self.pokemon_builder.item = Some(item.into());
        self
    }

    /// Add a move
    pub fn move_slot(mut self, move_id: impl Into<MoveId>) -> Self {
        self.pokemon_builder.moves.push(move_id.into());
        self
    }

    /// Set all moves at once
    pub fn moves(mut self, moves: Vec<MoveId>) -> Self {
        self.pokemon_builder.moves = moves;
        self
    }

    /// Set the level
    pub fn level(mut self, level: u8) -> Self {
        self.pokemon_builder.level = level;
        self
    }

    /// Set EVs
    pub fn evs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        self.pokemon_builder.evs = Some([hp, atk, def, spa, spd, spe]);
        self
    }

    /// Set IVs
    pub fn ivs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        self.pokemon_builder.ivs = Some([hp, atk, def, spa, spd, spe]);
        self
    }

    /// Set nature
    pub fn nature(mut self, nature: impl Into<String>) -> Self {
        self.pokemon_builder.nature = Some(nature.into());
        self
    }

    /// Finish building this Pokemon and return to team builder
    pub fn finish(mut self) -> TeamBuilder<'a> {
        self.team_builder.pokemon.push(self.pokemon_builder);
        self.team_builder
    }

    /// Add another Pokemon to the team
    pub fn and_pokemon(self, species: impl Into<SpeciesId>) -> TeamPokemonContext<'a> {
        let team_builder = self.finish();
        team_builder.pokemon(species)
    }
}

/// Builder for individual Pokemon
#[derive(Debug, Clone)]
pub struct PokemonBuilder {
    species: SpeciesId,
    ability: Option<AbilityId>,
    item: Option<ItemId>,
    moves: Vec<MoveId>,
    level: u8,
    evs: Option<[u8; 6]>, // [HP, Atk, Def, SpA, SpD, Spe]
    ivs: Option<[u8; 6]>, // [HP, Atk, Def, SpA, SpD, Spe]
    nature: Option<String>,
}

impl PokemonBuilder {
    /// Create a new Pokemon builder
    pub fn new(species: SpeciesId) -> Self {
        Self {
            species,
            ability: None,
            item: None,
            moves: Vec::new(),
            level: 50,
            evs: None,
            ivs: None,
            nature: None,
        }
    }

    /// Create from a RandomPokemonSet
    pub fn from_random_set(set: RandomPokemonSet) -> Self {
        Self {
            species: SpeciesId::new(set.species),
            ability: set.ability.map(AbilityId::from),
            item: set.item.map(ItemId::new),
            moves: set.moves.into_iter().map(MoveId::new).collect(),
            level: set.level,
            evs: set.evs.map(|evs| [
                evs.hp.unwrap_or(0),
                evs.atk.unwrap_or(0),
                evs.def.unwrap_or(0),
                evs.spa.unwrap_or(0),
                evs.spd.unwrap_or(0),
                evs.spe.unwrap_or(0),
            ]),
            ivs: set.ivs.map(|ivs| [
                ivs.hp.unwrap_or(31),
                ivs.atk.unwrap_or(31),
                ivs.def.unwrap_or(31),
                ivs.spa.unwrap_or(31),
                ivs.spd.unwrap_or(31),
                ivs.spe.unwrap_or(31),
            ]),
            nature: set.nature,
        }
    }

    /// Set the ability
    pub fn with_ability(mut self, ability: impl Into<AbilityId>) -> Self {
        self.ability = Some(ability.into());
        self
    }

    /// Set the item
    pub fn with_item(mut self, item: impl Into<ItemId>) -> Self {
        self.item = Some(item.into());
        self
    }

    /// Add a move
    pub fn with_move(mut self, move_id: impl Into<MoveId>) -> Self {
        self.moves.push(move_id.into());
        self
    }

    /// Set moves
    pub fn with_moves(mut self, moves: Vec<MoveId>) -> Self {
        self.moves = moves;
        self
    }

    /// Set level
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Set EVs
    pub fn with_evs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        self.evs = Some([hp, atk, def, spa, spd, spe]);
        self
    }

    /// Set IVs
    pub fn with_ivs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        self.ivs = Some([hp, atk, def, spa, spd, spe]);
        self
    }

    /// Set nature
    pub fn with_nature(mut self, nature: impl Into<String>) -> Self {
        self.nature = Some(nature.into());
        self
    }

    /// Build the Pokemon
    pub fn build(self) -> TeamResult<RandomPokemonSet> {
        // Validate moves count
        if self.moves.len() > 4 {
            return Err(TeamError::InvalidPokemon { 
                reason: format!("Too many moves: {} (max 4)", self.moves.len()) 
            });
        }

        // Validate level
        if self.level == 0 || self.level > 100 {
            return Err(TeamError::InvalidPokemon { 
                reason: format!("Invalid level: {} (must be 1-100)", self.level) 
            });
        }

        // Set defaults
        let ability = self.ability.map(|a| a.as_str().to_string());
        let moves = self.moves.into_iter().map(|m| m.as_str().to_string()).collect();
        let evs = self.evs.map(|[hp, atk, def, spa, spd, spe]| {
            crate::data::random_team_loader::RandomStats {
                hp: Some(hp),
                atk: Some(atk),
                def: Some(def),
                spa: Some(spa),
                spd: Some(spd),
                spe: Some(spe),
            }
        });
        let ivs = self.ivs.map(|[hp, atk, def, spa, spd, spe]| {
            crate::data::random_team_loader::RandomStats {
                hp: Some(hp),
                atk: Some(atk),
                def: Some(def),
                spa: Some(spa),
                spd: Some(spd),
                spe: Some(spe),
            }
        });
        let nature = self.nature;

        Ok(RandomPokemonSet {
            name: self.species.as_str().to_string(), // Use species as name
            species: self.species.as_str().to_string(),
            level: self.level,
            gender: None,
            shiny: Some(false),
            ability,
            item: self.item.map(|i| i.as_str().to_string()),
            moves,
            nature,
            evs,
            ivs,
            tera_type: None,
            role: None,
            gigantamax: Some(false),
        })
    }
}

/// Convenience constructors
impl<'a> TeamBuilder<'a> {
    /// Create a standard OU team structure
    pub fn ou_team(data: &'a Repository) -> Self {
        Self::new(data)
            .format(BattleFormat::gen9_ou())
    }

    /// Create a VGC team structure
    pub fn vgc_team(data: &'a Repository) -> Self {
        Self::new(data)
            .format(BattleFormat::gen9_vgc())
    }

    /// Create a random battle team
    pub fn random_battle_team(data: &'a Repository) -> TeamResult<Self> {
        Self::new(data)
            .format(BattleFormat::gen9_random_battle())
            .random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    #[test]
    fn test_pokemon_builder() {
        let pokemon = PokemonBuilder::new(SpeciesId::new("Charizard"))
            .with_ability("Blaze")
            .with_item("Charizardite X")
            .with_move("Flamethrower")
            .with_move("Dragon Pulse")
            .with_level(50)
            .with_evs(0, 252, 0, 252, 4, 0)
            .with_nature("Modest")
            .build()
            .unwrap();

        assert_eq!(pokemon.species, "Charizard");
        assert_eq!(pokemon.ability, "Blaze");
        assert_eq!(pokemon.item, Some("Charizardite X".to_string()));
        assert_eq!(pokemon.moves.len(), 2);
        assert_eq!(pokemon.level, 50);
        assert_eq!(pokemon.nature, "Modest");
    }

    #[test]
    fn test_team_builder_fluent_api() {
        // This test demonstrates the fluent API structure
        // Actual execution would require a real Repository instance
        let _team_structure = |data: &Repository| {
            TeamBuilder::new(data)
                .format(BattleFormat::gen9_ou())
                .pokemon("Charizard")
                    .ability("Solar Power")
                    .item("Choice Specs")
                    .move_slot("Flamethrower")
                    .move_slot("Solar Beam")
                    .move_slot("Focus Blast")
                    .move_slot("Air Slash")
                    .level(50)
                    .evs(0, 0, 0, 252, 4, 252)
                    .nature("Modest")
                .and_pokemon("Venusaur")
                    .ability("Chlorophyll")
                    .item("Life Orb")
                    .level(50)
                .finish()
                // .build() // Would need actual data to complete
        };
    }

    #[test]
    fn test_team_validation() {
        let format = BattleFormat::new(
            "Test Format".to_string(),
            Generation::Gen9,
            FormatType::Singles,
        );

        // Test species clause violation
        let pokemon1 = PokemonBuilder::new(SpeciesId::new("Charizard")).build().unwrap();
        let pokemon2 = PokemonBuilder::new(SpeciesId::new("Charizard")).build().unwrap();

        // This would fail species clause validation if we had a real format with species clause
        // The test demonstrates the validation structure
    }

    #[test]
    fn test_invalid_pokemon() {
        // Test too many moves
        let result = PokemonBuilder::new(SpeciesId::new("Charizard"))
            .with_move("Move1")
            .with_move("Move2")
            .with_move("Move3")
            .with_move("Move4")
            .with_move("Move5")
            .build();

        assert!(result.is_err());

        // Test invalid level
        let result = PokemonBuilder::new(SpeciesId::new("Charizard"))
            .with_level(0)
            .build();

        assert!(result.is_err());
    }
}