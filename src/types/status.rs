//! # Status Type System
//!
//! This module defines the status-related enums used throughout the battle system
//! for Pokemon status conditions and volatile statuses.

use serde::{Deserialize, Serialize};

/// Pokemon status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokemonStatus {
    None,
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned, // Toxic/Badly Poisoned
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
            5 => PokemonStatus::BadlyPoisoned,
            6 => PokemonStatus::Sleep,
            _ => PokemonStatus::None, // Default fallback
        }
    }
}

impl Default for PokemonStatus {
    fn default() -> Self {
        PokemonStatus::None
    }
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
    MicleBoost,
    CustapBoost,
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