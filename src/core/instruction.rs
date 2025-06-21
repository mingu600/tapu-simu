//! # Battle Instruction System
//! 
//! This module provides re-exports of the modern instruction system and 
//! defines the enums used by the instruction system.

use serde::{Deserialize, Serialize};

// Re-export the modern instruction types
pub use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction, FieldInstruction, StatusInstruction, StatsInstruction};



/// Pokemon status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonStatus {
    None,
    Burn,
    Freeze,
    Paralysis,
    Poison,
    Toxic,
    Sleep,
}

impl From<u8> for PokemonStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => PokemonStatus::None,
            1 => PokemonStatus::Burn,
            2 => PokemonStatus::Freeze,
            3 => PokemonStatus::Paralysis,
            4 => PokemonStatus::Poison,
            5 => PokemonStatus::Toxic,
            6 => PokemonStatus::Sleep,
            _ => PokemonStatus::None, // Default fallback
        }
    }
}

/// Pokemon stats that can be boosted/lowered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Hp,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
}

impl From<u8> for Stat {
    fn from(value: u8) -> Self {
        match value {
            0 => Stat::Hp,
            1 => Stat::Attack,
            2 => Stat::Defense,
            3 => Stat::SpecialAttack,
            4 => Stat::SpecialDefense,
            5 => Stat::Speed,
            6 => Stat::Accuracy,
            7 => Stat::Evasion,
            _ => Stat::Hp, // Default fallback
        }
    }
}

/// Move categories for damage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

/// Volatile status conditions (comprehensive coverage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolatileStatus {
    // Basic volatile statuses
    Confusion,
    Flinch,
    Substitute,
    LeechSeed,
    Curse,
    Nightmare,
    Attract,
    Torment,
    Disable,
    Encore,
    Taunt,
    HelpingHand,
    MagicCoat,
    FollowMe,
    Protect,
    Endure,
    
    // Extended volatile statuses
    AquaRing,
    Autotomize,
    BanefulBunker,
    Bide,
    Bounce,
    BurningBulwark,
    Charge,
    DefenseCurl,
    DestinyBond,
    Dig,
    Dive,
    Electrify,
    Electroshot,
    Embargo,
    FlashFire,
    Fly,
    FocusEnergy,
    Foresight,
    FreezeeShock,
    GastroAcid,
    Geomancy,
    GlaiveRush,
    Grudge,
    HealBlock,
    IceBurn,
    Imprison,
    Ingrain,
    KingsShield,
    LaserFocus,
    LightScreen,
    LockedMove,
    MagnetRise,
    MaxGuard,
    MeteorBeam,
    Minimize,
    MiracleEye,
    MustRecharge,
    NoRetreat,
    Octolock,
    PartiallyTrapped,
    Perish1,
    Perish2,
    Perish3,
    PowerTrick,
    Protect2,
    Rage,
    Reflect,
    Roost,
    Safeguard,
    SkyDrop,
    SmackDown,
    SpikesL1,
    SpikesL2,
    SpikesL3,
    StealthRock,
    Stockpile1,
    Stockpile2,
    Stockpile3,
    TelekineticMove,
    ToxicSpikes,
    Transform,
    TwoTurnMove,
    Uproar,
    WishHealing,
    Yawn,
    Telekinesis,
    MustSwitch,
}

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    None,
    Hail,
    Rain,
    Sandstorm,
    Sand, // Alias for Sandstorm
    Snow,
    Sun,
    HarshSunlight,
    HarshSun, // Alias for HarshSunlight
    HeavyRain,
    StrongWinds,
}

impl From<u8> for Weather {
    fn from(value: u8) -> Self {
        match value {
            0 => Weather::None,
            1 => Weather::Hail,
            2 => Weather::Rain,
            3 => Weather::Sandstorm,
            4 => Weather::Sand,
            5 => Weather::Snow,
            6 => Weather::Sun,
            7 => Weather::HarshSunlight,
            8 => Weather::HarshSun,
            9 => Weather::HeavyRain,
            10 => Weather::StrongWinds,
            _ => Weather::None, // Default fallback
        }
    }
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    None,
    Electric,
    ElectricTerrain, // Alias for Electric
    Grassy,
    GrassyTerrain, // Alias for Grassy
    Misty,
    MistyTerrain, // Alias for Misty
    Psychic,
    PsychicTerrain, // Alias for Psychic
}

impl From<u8> for Terrain {
    fn from(value: u8) -> Self {
        match value {
            0 => Terrain::None,
            1 => Terrain::Electric,
            2 => Terrain::ElectricTerrain,
            3 => Terrain::Grassy,
            4 => Terrain::GrassyTerrain,
            5 => Terrain::Misty,
            6 => Terrain::MistyTerrain,
            7 => Terrain::Psychic,
            8 => Terrain::PsychicTerrain,
            _ => Terrain::None, // Default fallback
        }
    }
}

/// Side conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideCondition {
    Reflect,
    LightScreen,
    AuroraVeil,
    Mist,
    Safeguard,
    Tailwind,
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
    Wish,
    FutureSight,
    DoomDesire,
    HealingWish,
    LunarDance,
    CraftyShield,
    MatBlock,
    QuickGuard,
    WideGuard,
}

impl From<u8> for SideCondition {
    fn from(value: u8) -> Self {
        match value {
            0 => SideCondition::Reflect,
            1 => SideCondition::LightScreen,
            2 => SideCondition::AuroraVeil,
            3 => SideCondition::Mist,
            4 => SideCondition::Safeguard,
            5 => SideCondition::Tailwind,
            6 => SideCondition::Spikes,
            7 => SideCondition::ToxicSpikes,
            8 => SideCondition::StealthRock,
            9 => SideCondition::StickyWeb,
            10 => SideCondition::Wish,
            11 => SideCondition::FutureSight,
            12 => SideCondition::DoomDesire,
            13 => SideCondition::HealingWish,
            14 => SideCondition::LunarDance,
            15 => SideCondition::CraftyShield,
            16 => SideCondition::MatBlock,
            17 => SideCondition::QuickGuard,
            18 => SideCondition::WideGuard,
            _ => SideCondition::Reflect, // Default fallback
        }
    }
}

impl From<u8> for VolatileStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => VolatileStatus::Confusion,
            1 => VolatileStatus::Flinch,
            2 => VolatileStatus::Substitute,
            3 => VolatileStatus::LeechSeed,
            4 => VolatileStatus::Curse,
            5 => VolatileStatus::Nightmare,
            6 => VolatileStatus::Attract,
            7 => VolatileStatus::Torment,
            8 => VolatileStatus::Disable,
            9 => VolatileStatus::Encore,
            10 => VolatileStatus::Taunt,
            11 => VolatileStatus::HelpingHand,
            12 => VolatileStatus::MagicCoat,
            13 => VolatileStatus::FollowMe,
            14 => VolatileStatus::Protect,
            15 => VolatileStatus::Endure,
            _ => VolatileStatus::Confusion, // Default fallback
        }
    }
}

impl Default for PokemonStatus {
    fn default() -> Self {
        PokemonStatus::None
    }
}

impl Default for Weather {
    fn default() -> Self {
        Weather::None
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain::None
    }
}

