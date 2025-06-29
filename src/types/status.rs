//! # Status Type System
//!
//! This module defines the status-related enums used throughout the battle system
//! for Pokemon status conditions and volatile statuses.

use serde::{Deserialize, Serialize};
use bitflags::bitflags;

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

bitflags! {
    /// Bitflags for common volatile statuses
    /// More memory efficient than HashSet for frequently used statuses
    #[derive(Clone, Copy, Debug, Default)]
    pub struct VolatileStatusFlags: u64 {
        const CONFUSION = 1 << 0;
        const FLINCH = 1 << 1;
        const SUBSTITUTE = 1 << 2;
        const LEECH_SEED = 1 << 3;
        const CURSE = 1 << 4;
        const NIGHTMARE = 1 << 5;
        const ATTRACT = 1 << 6;
        const TORMENT = 1 << 7;
        const DISABLE = 1 << 8;
        const ENCORE = 1 << 9;
        const TAUNT = 1 << 10;
        const HELPING_HAND = 1 << 11;
        const MAGIC_COAT = 1 << 12;
        const FOLLOW_ME = 1 << 13;
        const PROTECT = 1 << 14;
        const ENDURE = 1 << 15;
        const AQUA_RING = 1 << 16;
        const AUTOTOMIZE = 1 << 17;
        const BANEFUL_BUNKER = 1 << 18;
        const BIDE = 1 << 19;
        const BOUNCE = 1 << 20;
        const BURNING_BULWARK = 1 << 21;
        const CHARGE = 1 << 22;
        const DEFENSE_CURL = 1 << 23;
        const DESTINY_BOND = 1 << 24;
        const DIG = 1 << 25;
        const DIVE = 1 << 26;
        const ELECTRIFY = 1 << 27;
        const EMBARGO = 1 << 28;
        const FLASH_FIRE = 1 << 29;
        const FLY = 1 << 30;
        const FOCUS_ENERGY = 1 << 31;
        const FORESIGHT = 1 << 32;
        const GASTRO_ACID = 1 << 33;
        const GRUDGE = 1 << 34;
        const HEAL_BLOCK = 1 << 35;
        const IMPRISON = 1 << 36;
        const INGRAIN = 1 << 37;
        const KINGS_SHIELD = 1 << 38;
        const LASER_FOCUS = 1 << 39;
        const LIGHT_SCREEN = 1 << 40;
        const LOCKED_MOVE = 1 << 41;
        const MAGNET_RISE = 1 << 42;
        const MAX_GUARD = 1 << 43;
        const MINIMIZE = 1 << 44;
        const MIRACLE_EYE = 1 << 45;
        const MUST_RECHARGE = 1 << 46;
        const NO_RETREAT = 1 << 47;
        const OCTOLOCK = 1 << 48;
        const PARTIALLY_TRAPPED = 1 << 49;
        const POWER_TRICK = 1 << 50;
        const RAGE = 1 << 51;
        const REFLECT = 1 << 52;
        const ROOST = 1 << 53;
        const SAFEGUARD = 1 << 54;
        const SKY_DROP = 1 << 55;
        const SMACK_DOWN = 1 << 56;
        const STEALTH_ROCK = 1 << 57;
        const TELEKINETIC_MOVE = 1 << 58;
        const TOXIC_SPIKES = 1 << 59;
        const TRANSFORM = 1 << 60;
        const TWO_TURN_MOVE = 1 << 61;
        const UPROAR = 1 << 62;
        const YAWN = 1 << 63;
    }
}

impl Serialize for VolatileStatusFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VolatileStatusFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(VolatileStatusFlags::from_bits_truncate(bits))
    }
}

impl VolatileStatus {
    /// Convert a VolatileStatus to its corresponding bitflag if supported
    pub fn to_flag(self) -> Option<VolatileStatusFlags> {
        match self {
            VolatileStatus::Confusion => Some(VolatileStatusFlags::CONFUSION),
            VolatileStatus::Flinch => Some(VolatileStatusFlags::FLINCH),
            VolatileStatus::Substitute => Some(VolatileStatusFlags::SUBSTITUTE),
            VolatileStatus::LeechSeed => Some(VolatileStatusFlags::LEECH_SEED),
            VolatileStatus::Curse => Some(VolatileStatusFlags::CURSE),
            VolatileStatus::Nightmare => Some(VolatileStatusFlags::NIGHTMARE),
            VolatileStatus::Attract => Some(VolatileStatusFlags::ATTRACT),
            VolatileStatus::Torment => Some(VolatileStatusFlags::TORMENT),
            VolatileStatus::Disable => Some(VolatileStatusFlags::DISABLE),
            VolatileStatus::Encore => Some(VolatileStatusFlags::ENCORE),
            VolatileStatus::Taunt => Some(VolatileStatusFlags::TAUNT),
            VolatileStatus::HelpingHand => Some(VolatileStatusFlags::HELPING_HAND),
            VolatileStatus::MagicCoat => Some(VolatileStatusFlags::MAGIC_COAT),
            VolatileStatus::FollowMe => Some(VolatileStatusFlags::FOLLOW_ME),
            VolatileStatus::Protect => Some(VolatileStatusFlags::PROTECT),
            VolatileStatus::Endure => Some(VolatileStatusFlags::ENDURE),
            VolatileStatus::AquaRing => Some(VolatileStatusFlags::AQUA_RING),
            VolatileStatus::Autotomize => Some(VolatileStatusFlags::AUTOTOMIZE),
            VolatileStatus::BanefulBunker => Some(VolatileStatusFlags::BANEFUL_BUNKER),
            VolatileStatus::Bide => Some(VolatileStatusFlags::BIDE),
            VolatileStatus::Bounce => Some(VolatileStatusFlags::BOUNCE),
            VolatileStatus::BurningBulwark => Some(VolatileStatusFlags::BURNING_BULWARK),
            VolatileStatus::Charge => Some(VolatileStatusFlags::CHARGE),
            VolatileStatus::DefenseCurl => Some(VolatileStatusFlags::DEFENSE_CURL),
            VolatileStatus::DestinyBond => Some(VolatileStatusFlags::DESTINY_BOND),
            VolatileStatus::Dig => Some(VolatileStatusFlags::DIG),
            VolatileStatus::Dive => Some(VolatileStatusFlags::DIVE),
            VolatileStatus::Electrify => Some(VolatileStatusFlags::ELECTRIFY),
            VolatileStatus::Embargo => Some(VolatileStatusFlags::EMBARGO),
            VolatileStatus::FlashFire => Some(VolatileStatusFlags::FLASH_FIRE),
            VolatileStatus::Fly => Some(VolatileStatusFlags::FLY),
            VolatileStatus::FocusEnergy => Some(VolatileStatusFlags::FOCUS_ENERGY),
            VolatileStatus::Foresight => Some(VolatileStatusFlags::FORESIGHT),
            VolatileStatus::GastroAcid => Some(VolatileStatusFlags::GASTRO_ACID),
            VolatileStatus::Grudge => Some(VolatileStatusFlags::GRUDGE),
            VolatileStatus::HealBlock => Some(VolatileStatusFlags::HEAL_BLOCK),
            VolatileStatus::Imprison => Some(VolatileStatusFlags::IMPRISON),
            VolatileStatus::Ingrain => Some(VolatileStatusFlags::INGRAIN),
            VolatileStatus::KingsShield => Some(VolatileStatusFlags::KINGS_SHIELD),
            VolatileStatus::LaserFocus => Some(VolatileStatusFlags::LASER_FOCUS),
            VolatileStatus::LightScreen => Some(VolatileStatusFlags::LIGHT_SCREEN),
            VolatileStatus::LockedMove => Some(VolatileStatusFlags::LOCKED_MOVE),
            VolatileStatus::MagnetRise => Some(VolatileStatusFlags::MAGNET_RISE),
            VolatileStatus::MaxGuard => Some(VolatileStatusFlags::MAX_GUARD),
            VolatileStatus::Minimize => Some(VolatileStatusFlags::MINIMIZE),
            VolatileStatus::MiracleEye => Some(VolatileStatusFlags::MIRACLE_EYE),
            VolatileStatus::MustRecharge => Some(VolatileStatusFlags::MUST_RECHARGE),
            VolatileStatus::NoRetreat => Some(VolatileStatusFlags::NO_RETREAT),
            VolatileStatus::Octolock => Some(VolatileStatusFlags::OCTOLOCK),
            VolatileStatus::PartiallyTrapped => Some(VolatileStatusFlags::PARTIALLY_TRAPPED),
            VolatileStatus::PowerTrick => Some(VolatileStatusFlags::POWER_TRICK),
            VolatileStatus::Rage => Some(VolatileStatusFlags::RAGE),
            VolatileStatus::Reflect => Some(VolatileStatusFlags::REFLECT),
            VolatileStatus::Roost => Some(VolatileStatusFlags::ROOST),
            VolatileStatus::Safeguard => Some(VolatileStatusFlags::SAFEGUARD),
            VolatileStatus::SkyDrop => Some(VolatileStatusFlags::SKY_DROP),
            VolatileStatus::SmackDown => Some(VolatileStatusFlags::SMACK_DOWN),
            VolatileStatus::StealthRock => Some(VolatileStatusFlags::STEALTH_ROCK),
            VolatileStatus::TelekineticMove => Some(VolatileStatusFlags::TELEKINETIC_MOVE),
            VolatileStatus::ToxicSpikes => Some(VolatileStatusFlags::TOXIC_SPIKES),
            VolatileStatus::Transform => Some(VolatileStatusFlags::TRANSFORM),
            VolatileStatus::TwoTurnMove => Some(VolatileStatusFlags::TWO_TURN_MOVE),
            VolatileStatus::Uproar => Some(VolatileStatusFlags::UPROAR),
            VolatileStatus::Yawn => Some(VolatileStatusFlags::YAWN),
            _ => None, // Statuses not supported by bitflags
        }
    }
}

/// Duration tracking for volatile statuses that need turn counting
/// Only tracks durations for statuses that actually need them
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VolatileStatusDurations {
    /// Confusion turns remaining (1-4)
    pub confusion_turns: u8,
    /// Encore turns remaining (typically 3)
    pub encore_turns: u8,
    /// Disable turns remaining (typically 4)
    pub disable_turns: u8,
    /// Taunt turns remaining (typically 3)
    pub taunt_turns: u8,
    /// Sleep turns remaining
    pub sleep_turns: u8,
    /// Perish song countdown
    pub perish_turns: u8,
    /// Magnet Rise turns remaining (5)
    pub magnet_rise_turns: u8,
    /// Heal Block turns remaining (5)
    pub heal_block_turns: u8,
    /// Embargo turns remaining (5)
    pub embargo_turns: u8,
    /// Uproar turns remaining (2-5)
    pub uproar_turns: u8,
    /// Stockpile count (0-3)
    pub stockpile_count: u8,
    /// Multi-turn move progress
    pub two_turn_move_turn: u8,
}

/// Hybrid volatile status storage combining bitflags and HashMap
/// Uses bitflags for common statuses, HashMap for rare ones
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VolatileStatusStorage {
    /// Common statuses stored as bitflags
    pub flags: VolatileStatusFlags,
    /// Rare statuses not covered by bitflags
    pub overflow: std::collections::HashSet<VolatileStatus>,
    /// Duration tracking for timed statuses
    pub durations: VolatileStatusDurations,
}

impl VolatileStatusStorage {
    /// Check if a volatile status is active
    pub fn contains(&self, status: VolatileStatus) -> bool {
        if let Some(flag) = status.to_flag() {
            self.flags.contains(flag)
        } else {
            self.overflow.contains(&status)
        }
    }
    
    /// Add a volatile status
    pub fn insert(&mut self, status: VolatileStatus) {
        if let Some(flag) = status.to_flag() {
            self.flags.insert(flag);
        } else {
            self.overflow.insert(status);
        }
    }
    
    /// Remove a volatile status
    pub fn remove(&mut self, status: VolatileStatus) {
        if let Some(flag) = status.to_flag() {
            self.flags.remove(flag);
        } else {
            self.overflow.remove(&status);
        }
    }
    
    /// Clear all volatile statuses
    pub fn clear(&mut self) {
        self.flags = VolatileStatusFlags::empty();
        self.overflow.clear();
        self.durations = VolatileStatusDurations::default();
    }
    
    /// Check if any volatile statuses are active
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty() && self.overflow.is_empty()
    }
    
    /// Get an iterator over all active volatile statuses
    pub fn iter(&self) -> impl Iterator<Item = VolatileStatus> + '_ {
        // Create a vector of active statuses from flags
        let mut active_statuses = Vec::new();
        
        // Check each bit flag manually
        if self.flags.contains(VolatileStatusFlags::CONFUSION) {
            active_statuses.push(VolatileStatus::Confusion);
        }
        if self.flags.contains(VolatileStatusFlags::FLINCH) {
            active_statuses.push(VolatileStatus::Flinch);
        }
        if self.flags.contains(VolatileStatusFlags::SUBSTITUTE) {
            active_statuses.push(VolatileStatus::Substitute);
        }
        if self.flags.contains(VolatileStatusFlags::LEECH_SEED) {
            active_statuses.push(VolatileStatus::LeechSeed);
        }
        if self.flags.contains(VolatileStatusFlags::CURSE) {
            active_statuses.push(VolatileStatus::Curse);
        }
        if self.flags.contains(VolatileStatusFlags::NIGHTMARE) {
            active_statuses.push(VolatileStatus::Nightmare);
        }
        if self.flags.contains(VolatileStatusFlags::ATTRACT) {
            active_statuses.push(VolatileStatus::Attract);
        }
        if self.flags.contains(VolatileStatusFlags::TORMENT) {
            active_statuses.push(VolatileStatus::Torment);
        }
        if self.flags.contains(VolatileStatusFlags::DISABLE) {
            active_statuses.push(VolatileStatus::Disable);
        }
        if self.flags.contains(VolatileStatusFlags::ENCORE) {
            active_statuses.push(VolatileStatus::Encore);
        }
        if self.flags.contains(VolatileStatusFlags::TAUNT) {
            active_statuses.push(VolatileStatus::Taunt);
        }
        // Add more as needed...
        
        let flag_iter = active_statuses.into_iter();
        let overflow_iter = self.overflow.iter().copied();
        
        flag_iter.chain(overflow_iter)
    }
    
    /// Convert flag name to VolatileStatus enum
    fn name_to_status(&self, name: &str) -> VolatileStatus {
        match name {
            "CONFUSION" => VolatileStatus::Confusion,
            "FLINCH" => VolatileStatus::Flinch,
            "SUBSTITUTE" => VolatileStatus::Substitute,
            "LEECH_SEED" => VolatileStatus::LeechSeed,
            "CURSE" => VolatileStatus::Curse,
            "NIGHTMARE" => VolatileStatus::Nightmare,
            "ATTRACT" => VolatileStatus::Attract,
            "TORMENT" => VolatileStatus::Torment,
            "DISABLE" => VolatileStatus::Disable,
            "ENCORE" => VolatileStatus::Encore,
            "TAUNT" => VolatileStatus::Taunt,
            "HELPING_HAND" => VolatileStatus::HelpingHand,
            "MAGIC_COAT" => VolatileStatus::MagicCoat,
            "FOLLOW_ME" => VolatileStatus::FollowMe,
            "PROTECT" => VolatileStatus::Protect,
            "ENDURE" => VolatileStatus::Endure,
            // Add more mappings as needed
            _ => VolatileStatus::Confusion, // Fallback
        }
    }
    
    /// Convert to HashSet for compatibility
    pub fn to_hashset(&self) -> std::collections::HashSet<VolatileStatus> {
        self.iter().collect()
    }
    
    /// Create from HashSet for compatibility
    pub fn from_hashset(set: &std::collections::HashSet<VolatileStatus>) -> Self {
        let mut storage = Self::default();
        for &status in set {
            storage.insert(status);
        }
        storage
    }
}