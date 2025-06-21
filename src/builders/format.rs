use crate::core::battle_format::{BattleFormat, FormatType, FormatClause, BanList};
use crate::generation::Generation;
use crate::types::errors::{FormatError, FormatResult};
use crate::types::identifiers::{SpeciesId, MoveId, AbilityId, ItemId};

/// Builder for creating battle formats with fluent API
#[derive(Debug, Clone)]
pub struct FormatBuilder {
    name: Option<String>,
    generation: Option<Generation>,
    format_type: Option<FormatType>,
    team_size: Option<usize>,
    active_per_side: Option<usize>,
    clauses: Vec<FormatClause>,
    banned_species: Vec<SpeciesId>,
    banned_moves: Vec<MoveId>,
    banned_items: Vec<ItemId>,
    banned_abilities: Vec<AbilityId>,
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
    pub fn ban_species(mut self, species: impl Into<SpeciesId>) -> Self {
        let species = species.into();
        if !self.banned_species.contains(&species) {
            self.banned_species.push(species);
        }
        self
    }

    /// Ban multiple Pokemon species
    pub fn ban_species_list(mut self, species: Vec<SpeciesId>) -> Self {
        for s in species {
            if !self.banned_species.contains(&s) {
                self.banned_species.push(s);
            }
        }
        self
    }

    /// Ban a move
    pub fn ban_move(mut self, move_id: impl Into<MoveId>) -> Self {
        let move_id = move_id.into();
        if !self.banned_moves.contains(&move_id) {
            self.banned_moves.push(move_id);
        }
        self
    }

    /// Ban multiple moves
    pub fn ban_moves(mut self, moves: Vec<MoveId>) -> Self {
        for m in moves {
            if !self.banned_moves.contains(&m) {
                self.banned_moves.push(m);
            }
        }
        self
    }

    /// Ban an item
    pub fn ban_item(mut self, item: impl Into<ItemId>) -> Self {
        let item = item.into();
        if !self.banned_items.contains(&item) {
            self.banned_items.push(item);
        }
        self
    }

    /// Ban multiple items
    pub fn ban_items(mut self, items: Vec<ItemId>) -> Self {
        for i in items {
            if !self.banned_items.contains(&i) {
                self.banned_items.push(i);
            }
        }
        self
    }

    /// Ban an ability
    pub fn ban_ability(mut self, ability: impl Into<AbilityId>) -> Self {
        let ability = ability.into();
        if !self.banned_abilities.contains(&ability) {
            self.banned_abilities.push(ability);
        }
        self
    }

    /// Ban multiple abilities
    pub fn ban_abilities(mut self, abilities: Vec<AbilityId>) -> Self {
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
            builder = builder.ban_species(species);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_builder_singles() {
        let format = FormatBuilder::new()
            .name("Test Singles")
            .generation(Generation::Gen9)
            .singles()
            .standard_clauses()
            .build()
            .unwrap();

        assert_eq!(format.name, "Test Singles");
        assert_eq!(format.generation, Generation::Gen9);
        assert_eq!(format.format_type, FormatType::Singles);
        assert_eq!(format.active_per_side, 1);
        assert!(format.has_clause(&FormatClause::SleepClause));
        assert!(format.has_clause(&FormatClause::SpeciesClause));
    }

    #[test]
    fn test_format_builder_doubles() {
        let format = FormatBuilder::new()
            .name("Test Doubles")
            .generation(Generation::Gen8)
            .doubles()
            .species_clause()
            .build()
            .unwrap();

        assert_eq!(format.format_type, FormatType::Doubles);
        assert_eq!(format.active_per_side, 2);
        assert!(format.has_clause(&FormatClause::SpeciesClause));
    }

    #[test]
    fn test_format_builder_bans() {
        let format = FormatBuilder::new()
            .name("Test Format")
            .generation(Generation::Gen9)
            .singles()
            .ban_species("Mewtwo")
            .ban_move("Spore")
            .ban_item("Soul Dew")
            .ban_ability("Arena Trap")
            .build()
            .unwrap();

        assert!(format.is_species_banned("mewtwo"));
        assert!(format.is_move_banned("spore"));
        assert!(format.is_item_banned("soul dew"));
        assert!(format.is_ability_banned("arena trap"));
    }

    #[test]
    fn test_preset_formats() {
        let gen9_ou = FormatBuilder::gen9_ou().build().unwrap();
        assert_eq!(gen9_ou.name, "Gen 9 OU");
        assert_eq!(gen9_ou.generation, Generation::Gen9);
        assert_eq!(gen9_ou.format_type, FormatType::Singles);

        let gen9_vgc = FormatBuilder::gen9_vgc().build().unwrap();
        assert_eq!(gen9_vgc.format_type, FormatType::Doubles);
        assert_eq!(gen9_vgc.active_per_side, 2);

        let random = FormatBuilder::random_battle(Generation::Gen8).build().unwrap();
        assert!(random.name.contains("Random Battle"));
        assert_eq!(random.generation, Generation::Gen8);
    }

    #[test]
    fn test_validation_errors() {
        // Test invalid active count
        let result = FormatBuilder::new()
            .name("Invalid Format")
            .team_size(3)
            .active_count(5)
            .build();

        assert!(result.is_err());

        // Test too many active Pokemon
        let result = FormatBuilder::new()
            .name("Too Many Active")
            .active_count(5)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_fluent_api() {
        let format = BattleFormat::custom("My Format")
            .generation(Generation::Gen9)
            .doubles()
            .standard_clauses()
            .ban_species("Arceus")
            .ban_move("Spore")
            .build()
            .unwrap();

        assert_eq!(format.name, "My Format");
        assert!(format.is_species_banned("arceus"));
        assert!(format.is_move_banned("spore"));
    }
}