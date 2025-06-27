use crate::core::battle_format::{BattleFormat, FormatType, FormatClause, BanList};
use crate::generation::Generation;
use crate::types::errors::{FormatError, FormatResult};
use crate::types::{PokemonName, Moves, Abilities, Items};

/// Builder for creating battle formats with fluent API
#[derive(Debug, Clone)]
pub struct FormatBuilder {
    name: Option<String>,
    generation: Option<Generation>,
    format_type: Option<FormatType>,
    team_size: Option<usize>,
    active_per_side: Option<usize>,
    clauses: Vec<FormatClause>,
    banned_species: Vec<PokemonName>,
    banned_moves: Vec<Moves>,
    banned_items: Vec<Items>,
    banned_abilities: Vec<Abilities>,
}

impl FormatBuilder {
    /// Create a new format builder
    pub fn new() -> Self {
        Self {
            name: None,
            generation: None,
            format_type: None,
            team_size: None,
            active_per_side: None,
            clauses: Vec::new(),
            banned_species: Vec::new(),
            banned_moves: Vec::new(),
            banned_items: Vec::new(),
            banned_abilities: Vec::new(),
        }
    }

    /// Set the format name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the generation
    pub fn generation(mut self, gen: Generation) -> Self {
        self.generation = Some(gen);
        self
    }

    /// Set to singles format
    pub fn singles(mut self) -> Self {
        self.format_type = Some(FormatType::Singles);
        if self.active_per_side.is_none() {
            self.active_per_side = Some(1);
        }
        self
    }

    /// Set to doubles format
    pub fn doubles(mut self) -> Self {
        self.format_type = Some(FormatType::Doubles);
        if self.active_per_side.is_none() {
            self.active_per_side = Some(2);
        }
        self
    }

    /// Set to VGC format
    pub fn vgc(mut self) -> Self {
        self.format_type = Some(FormatType::Vgc);
        if self.active_per_side.is_none() {
            self.active_per_side = Some(2);
        }
        self
    }

    /// Set to triples format
    pub fn triples(mut self) -> Self {
        self.format_type = Some(FormatType::Triples);
        if self.active_per_side.is_none() {
            self.active_per_side = Some(3);
        }
        self
    }

    /// Set custom team size
    pub fn team_size(mut self, size: usize) -> Self {
        self.team_size = Some(size);
        self
    }

    /// Set custom active Pokemon count
    pub fn active_count(mut self, count: usize) -> Self {
        self.active_per_side = Some(count);
        self
    }

    /// Add Sleep Clause
    pub fn sleep_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::SleepClause) {
            self.clauses.push(FormatClause::SleepClause);
        }
        self
    }

    /// Add Freeze Clause
    pub fn freeze_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::FreezeClause) {
            self.clauses.push(FormatClause::FreezeClause);
        }
        self
    }

    /// Add Species Clause
    pub fn species_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::SpeciesClause) {
            self.clauses.push(FormatClause::SpeciesClause);
        }
        self
    }

    /// Add Item Clause
    pub fn item_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::ItemClause) {
            self.clauses.push(FormatClause::ItemClause);
        }
        self
    }

    /// Add Evasion Clause
    pub fn evasion_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::EvasionClause) {
            self.clauses.push(FormatClause::EvasionClause);
        }
        self
    }

    /// Add OHKO Clause
    pub fn ohko_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::OhkoClause) {
            self.clauses.push(FormatClause::OhkoClause);
        }
        self
    }

    /// Add Moody Clause
    pub fn moody_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::MoodyClause) {
            self.clauses.push(FormatClause::MoodyClause);
        }
        self
    }

    /// Add Swagger Clause
    pub fn swagger_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::SwaggerClause) {
            self.clauses.push(FormatClause::SwaggerClause);
        }
        self
    }

    /// Add Baton Pass Clause
    pub fn baton_pass_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::BatonPassClause) {
            self.clauses.push(FormatClause::BatonPassClause);
        }
        self
    }

    /// Add Endless Battle Clause
    pub fn endless_battle_clause(mut self) -> Self {
        if !self.clauses.contains(&FormatClause::EndlessBattleClause) {
            self.clauses.push(FormatClause::EndlessBattleClause);
        }
        self
    }

    /// Add multiple clauses at once
    pub fn clauses(mut self, clauses: Vec<FormatClause>) -> Self {
        for clause in clauses {
            if !self.clauses.contains(&clause) {
                self.clauses.push(clause);
            }
        }
        self
    }

    /// Ban a Pokemon species
    pub fn ban_species(mut self, species: impl Into<PokemonName>) -> Self {
        let species = species.into();
        if !self.banned_species.contains(&species) {
            self.banned_species.push(species);
        }
        self
    }

    /// Ban multiple Pokemon species
    pub fn ban_species_list(mut self, species: Vec<PokemonName>) -> Self {
        for s in species {
            if !self.banned_species.contains(&s) {
                self.banned_species.push(s);
            }
        }
        self
    }

    /// Ban a move
    pub fn ban_move(mut self, move_id: impl Into<Moves>) -> Self {
        let move_id = move_id.into();
        if !self.banned_moves.contains(&move_id) {
            self.banned_moves.push(move_id);
        }
        self
    }

    /// Ban multiple moves
    pub fn ban_moves(mut self, moves: Vec<Moves>) -> Self {
        for m in moves {
            if !self.banned_moves.contains(&m) {
                self.banned_moves.push(m);
            }
        }
        self
    }

    /// Ban an item
    pub fn ban_item(mut self, item: impl Into<Items>) -> Self {
        let item = item.into();
        if !self.banned_items.contains(&item) {
            self.banned_items.push(item);
        }
        self
    }

    /// Ban multiple items
    pub fn ban_items(mut self, items: Vec<Items>) -> Self {
        for i in items {
            if !self.banned_items.contains(&i) {
                self.banned_items.push(i);
            }
        }
        self
    }

    /// Ban an ability
    pub fn ban_ability(mut self, ability: impl Into<Abilities>) -> Self {
        let ability = ability.into();
        if !self.banned_abilities.contains(&ability) {
            self.banned_abilities.push(ability);
        }
        self
    }

    /// Ban multiple abilities
    pub fn ban_abilities(mut self, abilities: Vec<Abilities>) -> Self {
        for a in abilities {
            if !self.banned_abilities.contains(&a) {
                self.banned_abilities.push(a);
            }
        }
        self
    }

    /// Add standard competitive clauses
    pub fn standard_clauses(mut self) -> Self {
        self = self.sleep_clause()
                .species_clause()
                .evasion_clause()
                .ohko_clause()
                .endless_battle_clause();
        self
    }

    /// Build the format
    pub fn build(self) -> FormatResult<BattleFormat> {
        let name = self.name.unwrap_or_else(|| "Custom Format".to_string());
        let generation = self.generation.unwrap_or(Generation::Gen9);
        let format_type = self.format_type.unwrap_or(FormatType::Singles);
        let team_size = self.team_size.unwrap_or(6);
        let active_per_side = self.active_per_side.unwrap_or(format_type.active_pokemon_count());

        // Validate settings
        if active_per_side > team_size {
            return Err(FormatError::InvalidTeamSize { 
                size: active_per_side, 
                expected: team_size 
            });
        }

        if active_per_side > 3 {
            return Err(FormatError::RuleViolation { 
                rule: "Maximum 3 active Pokemon per side".to_string() 
            });
        }

        // Create ban list
        let ban_list = BanList::new(
            self.banned_species.into_iter().map(|s| s.as_str().to_string()).collect(),
            self.banned_moves.into_iter().map(|m| m.as_str().to_string()).collect(),
            self.banned_items.into_iter().map(|i| i.as_str().to_string()).collect(),
            self.banned_abilities.into_iter().map(|a| a.as_str().to_string()).collect(),
        );

        let format = BattleFormat::new_with_settings(
            name,
            generation,
            format_type,
            team_size,
            active_per_side,
        )
        .with_clauses(self.clauses)
        .with_bans(ban_list);

        Ok(format)
    }
}

impl Default for FormatBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset format builders
impl FormatBuilder {
    /// Gen 9 OU format
    pub fn gen9_ou() -> Self {
        Self::new()
            .name("Gen 9 OU")
            .generation(Generation::Gen9)
            .singles()
            .standard_clauses()
    }

    /// Gen 9 VGC format
    pub fn gen9_vgc() -> Self {
        Self::new()
            .name("Gen 9 VGC 2024")
            .generation(Generation::Gen9)
            .doubles()
            .species_clause()
            .team_size(6)
            .active_count(2)
    }

    /// Gen 8 OU format
    pub fn gen8_ou() -> Self {
        Self::new()
            .name("Gen 8 OU")
            .generation(Generation::Gen8)
            .singles()
            .standard_clauses()
    }

    /// Gen 7 OU format
    pub fn gen7_ou() -> Self {
        Self::new()
            .name("Gen 7 OU")
            .generation(Generation::Gen7)
            .singles()
            .standard_clauses()
    }

    /// Gen 4 OU format
    pub fn gen4_ou() -> Self {
        Self::new()
            .name("Gen 4 OU")
            .generation(Generation::Gen4)
            .singles()
            .standard_clauses()
            .freeze_clause() // Gen 4 had freeze clause
    }

    /// Random Battle format
    pub fn random_battle(generation: Generation) -> Self {
        let name = format!("Gen {} Random Battle", generation as u8);
        Self::new()
            .name(name)
            .generation(generation)
            .singles()
            .species_clause() // Only species clause for random battles
    }

    /// Random Doubles format
    pub fn random_doubles(generation: Generation) -> Self {
        let name = format!("Gen {} Random Doubles", generation as u8);
        Self::new()
            .name(name)
            .generation(generation)
            .doubles()
            .species_clause()
    }

    /// Create a format with common Uber bans for OU
    pub fn ou_with_ubers_banned(generation: Generation) -> Self {
        let name = format!("Gen {} OU", generation as u8);
        let mut builder = Self::new()
            .name(name)
            .generation(generation)
            .singles()
            .standard_clauses();

        // Common Uber Pokemon (these would be more comprehensive in practice)
        let uber_species = match generation {
            Generation::Gen9 => vec![
                "Arceus", "Dialga", "Palkia", "Giratina", "Rayquaza", "Kyogre", "Groudon",
                "Mewtwo", "Mew", "Lugia", "Ho-Oh", "Calyrex-Ice", "Calyrex-Shadow",
                "Zacian", "Zamazenta", "Eternatus",
            ],
            Generation::Gen8 => vec![
                "Arceus", "Dialga", "Palkia", "Giratina", "Rayquaza", "Kyogre", "Groudon",
                "Mewtwo", "Lugia", "Ho-Oh", "Calyrex-Ice", "Calyrex-Shadow",
                "Zacian", "Zamazenta", "Eternatus",
            ],
            _ => vec![
                "Arceus", "Dialga", "Palkia", "Giratina", "Rayquaza", "Kyogre", "Groudon",
                "Mewtwo", "Lugia", "Ho-Oh",
            ],
        };

        for species in uber_species {
            if let Some(pokemon_name) = <crate::types::PokemonName as crate::types::FromNormalizedString>::from_normalized_str(species) {
                builder = builder.ban_species(pokemon_name);
            }
        }

        builder
    }
}

/// Extension trait for BattleFormat to add builder methods
impl BattleFormat {
    /// Create a format builder
    pub fn builder() -> FormatBuilder {
        FormatBuilder::new()
    }

    /// Create a custom format with a name
    pub fn custom(name: impl Into<String>) -> FormatBuilder {
        FormatBuilder::new().name(name)
    }
}

