//! # Battle Instruction System
//! 
//! This module defines the unified instruction system for the V2 engine.
//! All instructions are position-aware and designed for multi-format support.
//! Optional undo support is built into each instruction type.

use crate::core::battle_format::{BattlePosition, SideReference};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single battle instruction that modifies the game state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    /// Deal damage to a specific position
    PositionDamage(PositionDamageInstruction),
    /// Heal a specific position
    PositionHeal(PositionHealInstruction),
    /// Deal damage to multiple positions simultaneously
    MultiTargetDamage(MultiTargetDamageInstruction),
    /// Apply a status condition to a position
    ApplyStatus(ApplyStatusInstruction),
    /// Remove a status condition from a position
    RemoveStatus(RemoveStatusInstruction),
    /// Boost stats at a position
    BoostStats(BoostStatsInstruction),
    /// Switch Pokemon at a position
    SwitchPokemon(SwitchInstruction),
    /// Apply volatile status to a position
    ApplyVolatileStatus(ApplyVolatileStatusInstruction),
    /// Remove volatile status from a position
    RemoveVolatileStatus(RemoveVolatileStatusInstruction),
    /// Change volatile status duration
    ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction),
    /// Change status condition duration
    ChangeStatusDuration(ChangeStatusDurationInstruction),
    /// Change weather conditions
    ChangeWeather(ChangeWeatherInstruction),
    /// Change terrain conditions
    ChangeTerrain(ChangeTerrainInstruction),
    /// Apply side condition
    ApplySideCondition(ApplySideConditionInstruction),
    /// Remove side condition
    RemoveSideCondition(RemoveSideConditionInstruction),
    /// Decrement side condition duration
    DecrementSideConditionDuration(DecrementSideConditionDurationInstruction),
    
    // ========== Move Management Instructions ==========
    /// Disable a move
    DisableMove(DisableMoveInstruction),
    /// Enable a move
    EnableMove(EnableMoveInstruction),
    /// Decrement PP of a move
    DecrementPP(DecrementPPInstruction),
    /// Set the last used move
    SetLastUsedMove(SetLastUsedMoveInstruction),
    /// Restore last used move (for Disable)
    RestoreLastUsedMove(RestoreLastUsedMoveInstruction),
    
    // ========== Pokemon Attribute Instructions ==========
    /// Change Pokemon ability
    ChangeAbility(ChangeAbilityInstruction),
    /// Toggle ability state
    ToggleAbility(ToggleAbilityInstruction),
    /// Change Pokemon held item
    ChangeItem(ChangeItemInstruction),
    /// Remove Pokemon held item
    RemoveItem(RemoveItemInstruction),
    /// Give Pokemon an item
    GiveItem(GiveItemInstruction),
    /// Change Pokemon types
    ChangeType(ChangeTypeInstruction),
    /// Change Pokemon forme
    FormeChange(FormeChangeInstruction),
    /// Faint a Pokemon
    Faint(FaintInstruction),
    /// Toggle terastallization state
    ToggleTerastallized(ToggleTerastallizedInstruction),
    
    // ========== Advanced Field Effect Instructions ==========
    /// Toggle trick room
    ToggleTrickRoom(ToggleTrickRoomInstruction),
    /// Toggle gravity
    ToggleGravity(ToggleGravityInstruction),
    /// Decrement weather turns remaining
    DecrementWeatherTurns,
    /// Decrement terrain turns remaining
    DecrementTerrainTurns,
    /// Decrement trick room turns remaining
    DecrementTrickRoomTurns,
    
    // ========== Special Mechanic Instructions ==========
    /// Set wish healing
    SetWish(SetWishInstruction),
    /// Decrement wish counter
    DecrementWish(DecrementWishInstruction),
    /// Set future sight attack
    SetFutureSight(SetFutureSightInstruction),
    /// Decrement future sight counter
    DecrementFutureSight(DecrementFutureSightInstruction),
    /// Change substitute health
    ChangeSubstituteHealth(ChangeSubstituteHealthInstruction),
    
    // ========== Sleep/Rest System Instructions ==========
    /// Set rest turns
    SetRestTurns(SetRestTurnsInstruction),
    /// Set sleep turns
    SetSleepTurns(SetSleepTurnsInstruction),
    /// Decrement rest turns
    DecrementRestTurns(DecrementRestTurnsInstruction),
    
    // ========== Battle State Management Instructions ==========
    /// Toggle baton passing state
    ToggleBatonPassing(ToggleBatonPassingInstruction),
    /// Toggle shed tail state
    ToggleShedTailing(ToggleShedTailingInstruction),
    /// Toggle force switch for side one
    ToggleSideOneForceSwitch,
    /// Toggle force switch for side two
    ToggleSideTwoForceSwitch,
    
    // ========== Raw Stat Modification Instructions ==========
    /// Change raw attack stat (not boosts)
    ChangeAttack(ChangeStatInstruction),
    /// Change raw defense stat (not boosts)
    ChangeDefense(ChangeStatInstruction),
    /// Change raw special attack stat (not boosts)
    ChangeSpecialAttack(ChangeStatInstruction),
    /// Change raw special defense stat (not boosts)
    ChangeSpecialDefense(ChangeStatInstruction),
    /// Change raw speed stat (not boosts)
    ChangeSpeed(ChangeStatInstruction),
    
    // ========== Switch Move Management Instructions ==========
    /// Set side one's second switch out move
    SetSideOneMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    /// Set side two's second switch out move
    SetSideTwoMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    
    // ========== Damage Tracking Instructions ==========
    /// Track damage dealt
    ChangeDamageDealt(ChangeDamageDealtInstruction),
    /// Track move category of damage dealt
    ChangeDamageDealtMoveCategory(ChangeDamageDealtMoveCategoryInstruction),
    /// Track if damage hit substitute
    ToggleDamageDealtHitSubstitute(ToggleDamageDealtHitSubstituteInstruction),
}

impl Instruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            Instruction::PositionDamage(instr) => vec![instr.target_position],
            Instruction::PositionHeal(instr) => vec![instr.target_position],
            Instruction::MultiTargetDamage(instr) => {
                instr.target_damages.iter().map(|(pos, _)| *pos).collect()
            }
            Instruction::ApplyStatus(instr) => vec![instr.target_position],
            Instruction::RemoveStatus(instr) => vec![instr.target_position],
            Instruction::BoostStats(instr) => vec![instr.target_position],
            Instruction::SwitchPokemon(instr) => vec![instr.position],
            Instruction::ApplyVolatileStatus(instr) => vec![instr.target_position],
            Instruction::RemoveVolatileStatus(instr) => vec![instr.target_position],
            Instruction::ChangeVolatileStatusDuration(instr) => vec![instr.target_position],
            Instruction::ChangeStatusDuration(instr) => vec![instr.target_position],
            Instruction::ChangeWeather(_) => vec![], // Weather affects all
            Instruction::ChangeTerrain(_) => vec![], // Terrain affects all
            Instruction::ApplySideCondition(_) => vec![], // Side-wide
            Instruction::RemoveSideCondition(_) => vec![],
            Instruction::DecrementSideConditionDuration(_) => vec![],
            
            // Move Management Instructions
            Instruction::DisableMove(instr) => vec![instr.target_position],
            Instruction::EnableMove(instr) => vec![instr.target_position],
            Instruction::DecrementPP(instr) => vec![instr.target_position],
            Instruction::SetLastUsedMove(instr) => vec![instr.target_position],
            Instruction::RestoreLastUsedMove(instr) => vec![instr.target_position],
            
            // Pokemon Attribute Instructions
            Instruction::ChangeAbility(instr) => vec![instr.target_position],
            Instruction::ToggleAbility(instr) => vec![instr.target_position],
            Instruction::ChangeItem(instr) => vec![instr.target_position],
            Instruction::RemoveItem(instr) => vec![instr.target_position],
            Instruction::GiveItem(instr) => vec![instr.target_position],
            Instruction::ChangeType(instr) => vec![instr.target_position],
            Instruction::FormeChange(instr) => vec![instr.target_position],
            Instruction::Faint(instr) => vec![instr.target_position],
            Instruction::ToggleTerastallized(instr) => vec![instr.target_position],
            
            // Advanced Field Effect Instructions
            Instruction::ToggleTrickRoom(_) => vec![], // Affects all
            Instruction::ToggleGravity(_) => vec![], // Affects all
            Instruction::DecrementWeatherTurns => vec![], // Affects all
            Instruction::DecrementTerrainTurns => vec![], // Affects all
            Instruction::DecrementTrickRoomTurns => vec![], // Affects all
            
            // Special Mechanic Instructions
            Instruction::SetWish(instr) => vec![instr.target_position],
            Instruction::DecrementWish(instr) => vec![instr.target_position],
            Instruction::SetFutureSight(instr) => vec![instr.target_position],
            Instruction::DecrementFutureSight(instr) => vec![instr.target_position],
            Instruction::ChangeSubstituteHealth(instr) => vec![instr.target_position],
            
            // Sleep/Rest System Instructions
            Instruction::SetRestTurns(instr) => vec![instr.target_position],
            Instruction::SetSleepTurns(instr) => vec![instr.target_position],
            Instruction::DecrementRestTurns(instr) => vec![instr.target_position],
            
            // Battle State Management Instructions
            Instruction::ToggleBatonPassing(instr) => vec![instr.target_position],
            Instruction::ToggleShedTailing(instr) => vec![instr.target_position],
            Instruction::ToggleSideOneForceSwitch => vec![], // Side-wide
            Instruction::ToggleSideTwoForceSwitch => vec![], // Side-wide
            
            // Raw Stat Modification Instructions
            Instruction::ChangeAttack(instr) => vec![instr.target_position],
            Instruction::ChangeDefense(instr) => vec![instr.target_position],
            Instruction::ChangeSpecialAttack(instr) => vec![instr.target_position],
            Instruction::ChangeSpecialDefense(instr) => vec![instr.target_position],
            Instruction::ChangeSpeed(instr) => vec![instr.target_position],
            
            // Switch Move Management Instructions
            Instruction::SetSideOneMoveSecondSwitchOutMove(_) => vec![], // Side-wide effect
            Instruction::SetSideTwoMoveSecondSwitchOutMove(_) => vec![], // Side-wide effect
            
            // Damage Tracking Instructions
            Instruction::ChangeDamageDealt(instr) => vec![instr.target_position],
            Instruction::ChangeDamageDealtMoveCategory(instr) => vec![instr.target_position],
            Instruction::ToggleDamageDealtHitSubstitute(instr) => vec![instr.target_position],
        }
    }
}

/// Deal damage to a specific battle position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionDamageInstruction {
    pub target_position: BattlePosition,
    pub damage_amount: i16,
    pub previous_hp: Option<i16>,
}

/// Heal a specific battle position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionHealInstruction {
    pub target_position: BattlePosition,
    pub heal_amount: i16,
    pub previous_hp: Option<i16>,
}

/// Deal damage to multiple positions with potentially different amounts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiTargetDamageInstruction {
    pub target_damages: Vec<(BattlePosition, i16)>,
    pub previous_hps: Vec<(BattlePosition, i16)>,
}

/// Apply a status condition to a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyStatusInstruction {
    pub target_position: BattlePosition,
    pub status: PokemonStatus,
    pub previous_status: Option<PokemonStatus>,
    pub previous_status_duration: Option<Option<u8>>,
}

/// Remove a status condition from a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveStatusInstruction {
    pub target_position: BattlePosition,
    pub previous_status: Option<PokemonStatus>,
    pub previous_status_duration: Option<Option<u8>>,
}

/// Boost stats at a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoostStatsInstruction {
    pub target_position: BattlePosition,
    pub stat_boosts: HashMap<Stat, i8>,
    pub previous_boosts: Option<HashMap<Stat, i8>>,
}

/// Switch Pokemon at a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchInstruction {
    pub position: BattlePosition,
    pub previous_index: usize,
    pub next_index: usize,
}

/// Apply volatile status to a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyVolatileStatusInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
    pub duration: Option<u8>,
}

/// Remove volatile status from a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveVolatileStatusInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
}

/// Change volatile status duration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeVolatileStatusDurationInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
    pub duration_change: i8,
}

/// Change status condition duration (Sleep, Freeze)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStatusDurationInstruction {
    pub target_position: BattlePosition,
    pub duration_change: i8,
}

/// Change weather conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeWeatherInstruction {
    pub weather: Weather,
    pub duration: Option<u8>,
    pub previous_weather: Option<Weather>,
    pub previous_duration: Option<Option<u8>>,
}

/// Change terrain conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeTerrainInstruction {
    pub terrain: Terrain,
    pub duration: Option<u8>,
    pub previous_terrain: Option<Terrain>,
    pub previous_duration: Option<Option<u8>>,
}

/// Apply side condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplySideConditionInstruction {
    pub side: SideReference,
    pub condition: SideCondition,
    pub duration: Option<u8>,
}

/// Remove side condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveSideConditionInstruction {
    pub side: SideReference,
    pub condition: SideCondition,
}

/// Decrement side condition duration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementSideConditionDurationInstruction {
    pub side: SideReference,
    pub condition: SideCondition,
    pub amount: u8,
}

// ========== Move Management Instruction Structs ==========

/// Disable a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisableMoveInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
    pub duration: Option<u8>,
}

/// Enable a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnableMoveInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
}

/// Decrement PP of a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementPPInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
    pub amount: u8,
}

/// Set the last used move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLastUsedMoveInstruction {
    pub target_position: BattlePosition,
    pub move_name: String,
    pub move_id: Option<u16>,
}

/// Restore last used move (for Disable)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreLastUsedMoveInstruction {
    pub target_position: BattlePosition,
    pub move_name: String,
}

// ========== Pokemon Attribute Instruction Structs ==========

/// Change Pokemon ability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeAbilityInstruction {
    pub target_position: BattlePosition,
    pub new_ability: String,
    pub previous_ability: Option<String>,
}

/// Toggle ability state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleAbilityInstruction {
    pub target_position: BattlePosition,
    pub enabled: bool,
}

/// Change Pokemon held item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeItemInstruction {
    pub target_position: BattlePosition,
    pub new_item: Option<String>,
    pub previous_item: Option<String>,
}

/// Remove Pokemon held item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveItemInstruction {
    pub target_position: BattlePosition,
    pub previous_item: Option<String>,
}

/// Give Pokemon an item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GiveItemInstruction {
    pub target_position: BattlePosition,
    pub item: String,
}

/// Change Pokemon types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeTypeInstruction {
    pub target_position: BattlePosition,
    pub new_types: Vec<String>,
    pub previous_types: Option<Vec<String>>,
}

/// Change Pokemon forme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormeChangeInstruction {
    pub target_position: BattlePosition,
    pub new_forme: String,
    pub previous_forme: Option<String>,
}

/// Faint a Pokemon
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaintInstruction {
    pub target_position: BattlePosition,
}

/// Toggle terastallization state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleTerastallizedInstruction {
    pub target_position: BattlePosition,
    pub tera_type: Option<String>,
}

// ========== Advanced Field Effect Instruction Structs ==========

/// Toggle trick room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleTrickRoomInstruction {
    pub active: bool,
    pub duration: Option<u8>,
}

/// Toggle gravity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleGravityInstruction {
    pub active: bool,
    pub duration: Option<u8>,
}

// ========== Special Mechanic Instruction Structs ==========

/// Set wish healing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetWishInstruction {
    pub target_position: BattlePosition,
    pub heal_amount: i16,
    pub turns_remaining: u8,
}

/// Decrement wish counter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementWishInstruction {
    pub target_position: BattlePosition,
}

/// Set future sight attack
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFutureSightInstruction {
    pub target_position: BattlePosition,
    pub attacker_position: BattlePosition,
    pub damage_amount: i16,
    pub turns_remaining: u8,
    pub move_name: String,
}

/// Decrement future sight counter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementFutureSightInstruction {
    pub target_position: BattlePosition,
}

/// Change substitute health
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeSubstituteHealthInstruction {
    pub target_position: BattlePosition,
    pub health_change: i16,
    pub new_health: i16,
}

// ========== Sleep/Rest System Instruction Structs ==========

/// Set rest turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetRestTurnsInstruction {
    pub target_position: BattlePosition,
    pub turns: u8,
}

/// Set sleep turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetSleepTurnsInstruction {
    pub target_position: BattlePosition,
    pub turns: u8,
}

/// Decrement rest turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementRestTurnsInstruction {
    pub target_position: BattlePosition,
}

// ========== Battle State Management Instruction Structs ==========

/// Toggle baton passing state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleBatonPassingInstruction {
    pub target_position: BattlePosition,
    pub active: bool,
}

/// Toggle shed tail state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleShedTailingInstruction {
    pub target_position: BattlePosition,
    pub active: bool,
}

// ========== Damage Tracking Instruction Structs ==========

/// Track damage dealt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeDamageDealtInstruction {
    pub target_position: BattlePosition,
    pub damage_amount: i16,
}

/// Track move category of damage dealt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeDamageDealtMoveCategoryInstruction {
    pub target_position: BattlePosition,
    pub move_category: MoveCategory,
}

/// Track if damage hit substitute
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleDamageDealtHitSubstituteInstruction {
    pub target_position: BattlePosition,
    pub hit_substitute: bool,
}

// ========== Raw Stat Modification Instruction Structs ==========

/// Change raw stats (not boosts) - used for moves like Power Split, Guard Swap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStatInstruction {
    pub target_position: BattlePosition,
    pub stat_change: i16,
}

// ========== Switch Move Management Instruction Structs ==========

/// Set second switch out move (for moves like Baton Pass, U-turn)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetSecondMoveSwitchOutMoveInstruction {
    pub side: SideReference,
    pub previous_choice: Option<String>,
    pub new_choice: String,
}

/// A collection of instructions with probability and affected positions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateInstructions {
    /// Probability of this instruction set occurring (0.0 to 100.0)
    pub percentage: f32,
    /// List of instructions to execute
    pub instruction_list: Vec<Instruction>,
    /// All positions affected by these instructions
    pub affected_positions: Vec<BattlePosition>,
}

impl StateInstructions {
    /// Create a new StateInstructions with calculated affected positions
    pub fn new(percentage: f32, instruction_list: Vec<Instruction>) -> Self {
        let mut affected_positions = Vec::new();
        for instruction in &instruction_list {
            affected_positions.extend(instruction.affected_positions());
        }
        
        // Remove duplicates and sort for consistency
        affected_positions.sort();
        affected_positions.dedup();

        Self {
            percentage,
            instruction_list,
            affected_positions,
        }
    }

    /// Create empty instructions
    pub fn empty() -> Self {
        Self {
            percentage: 100.0,
            instruction_list: Vec::new(),
            affected_positions: Vec::new(),
        }
    }
}

impl Default for StateInstructions {
    fn default() -> Self {
        Self::empty()
    }
}

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
    Perish4,
    PhantomForce,
    Powder,
    PowerShift,
    PowerTrick,
    ProtosynthesisAttack,
    ProtosynthesisDefense,
    ProtosynthesisSpecialAttack,
    ProtosynthesisSpecialDefense,
    ProtosynthesisSpeed,
    QuarkDriveAttack,
    QuarkDriveDefense,
    QuarkDriveSpecialAttack,
    QuarkDriveSpecialDefense,
    QuarkDriveSpeed,
    Rage,
    RagePowder,
    RazorWind,
    Reflect,
    Roost,
    SaltCure,
    ShadowForce,
    SkullBash,
    SkyAttack,
    SkyDrop,
    SilkTrap,
    SlowStart,
    SmackDown,
    Snatch,
    SolarBeam,
    SolarBlade,
    SparklingAria,
    SpikyShield,
    Spotlight,
    Stockpile,
    SyrupBomb,
    TarShot,
    Telekinesis,
    ThroatChop,
    Truant,
    TypeChange,
    Unburden,
    Uproar,
    Yawn,
    
    // Multi-turn charging moves
    ChargingTurn,
    Recharging,
    TwoTurnMove,
    
    // Protection moves
    BanefulBunkerProtect,
    KingsShieldProtect,
    MaxGuardProtect,
    SpikyShieldProtect,
    SilkTrapProtect,
    
    // Stat modification volatile statuses
    AtkBoost1,
    AtkBoost2,
    AtkBoost3,
    DefBoost1,
    DefBoost2,
    DefBoost3,
    SpaBoost1,
    SpaBoost2,
    SpaBoost3,
    SpdBoost1,
    SpdBoost2,
    SpdBoost3,
    SpeBoost1,
    SpeBoost2,
    SpeBoost3,
    
    // Toxic counter for tracking progressive toxic damage
    ToxicCount,
}

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    None,
    Sun,
    Rain,
    Sand,
    Hail,
    Snow,
    HarshSun,
    HeavyRain,
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    None,
    ElectricTerrain,
    GrassyTerrain,
    MistyTerrain,
    PsychicTerrain,
}

/// Side conditions that affect an entire side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideCondition {
    Reflect,
    LightScreen,
    Mist,
    Safeguard,
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
    TailWind,
    WideGuard,
    QuickGuard,
    AuroraVeil,
}

// From trait implementations for enum conversions
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
            _ => PokemonStatus::None,
        }
    }
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
            _ => Stat::Hp,
        }
    }
}

impl From<u8> for Weather {
    fn from(value: u8) -> Self {
        match value {
            0 => Weather::None,
            1 => Weather::Sun,
            2 => Weather::Rain,
            3 => Weather::Sand,
            4 => Weather::Hail,
            5 => Weather::Snow,
            6 => Weather::HarshSun,
            7 => Weather::HeavyRain,
            _ => Weather::None,
        }
    }
}

impl From<u8> for Terrain {
    fn from(value: u8) -> Self {
        match value {
            0 => Terrain::None,
            1 => Terrain::ElectricTerrain,
            2 => Terrain::GrassyTerrain,
            3 => Terrain::MistyTerrain,
            4 => Terrain::PsychicTerrain,
            _ => Terrain::None,
        }
    }
}

impl From<u8> for SideCondition {
    fn from(value: u8) -> Self {
        match value {
            0 => SideCondition::Reflect,
            1 => SideCondition::LightScreen,
            2 => SideCondition::Mist,
            3 => SideCondition::Safeguard,
            4 => SideCondition::Spikes,
            5 => SideCondition::ToxicSpikes,
            6 => SideCondition::StealthRock,
            7 => SideCondition::StickyWeb,
            8 => SideCondition::TailWind,
            9 => SideCondition::WideGuard,
            10 => SideCondition::QuickGuard,
            11 => SideCondition::AuroraVeil,
            _ => SideCondition::Reflect,
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
            16 => VolatileStatus::AquaRing,
            17 => VolatileStatus::Autotomize,
            18 => VolatileStatus::BanefulBunker,
            19 => VolatileStatus::Bide,
            20 => VolatileStatus::Bounce,
            21 => VolatileStatus::BurningBulwark,
            22 => VolatileStatus::Charge,
            23 => VolatileStatus::DefenseCurl,
            24 => VolatileStatus::DestinyBond,
            25 => VolatileStatus::Dig,
            26 => VolatileStatus::Dive,
            27 => VolatileStatus::Electrify,
            28 => VolatileStatus::Electroshot,
            29 => VolatileStatus::Embargo,
            30 => VolatileStatus::FlashFire,
            31 => VolatileStatus::Fly,
            32 => VolatileStatus::FocusEnergy,
            33 => VolatileStatus::Foresight,
            34 => VolatileStatus::FreezeeShock,
            35 => VolatileStatus::GastroAcid,
            36 => VolatileStatus::Geomancy,
            37 => VolatileStatus::GlaiveRush,
            38 => VolatileStatus::Grudge,
            39 => VolatileStatus::HealBlock,
            40 => VolatileStatus::IceBurn,
            41 => VolatileStatus::Imprison,
            42 => VolatileStatus::Ingrain,
            43 => VolatileStatus::KingsShield,
            44 => VolatileStatus::LaserFocus,
            45 => VolatileStatus::LightScreen,
            46 => VolatileStatus::LockedMove,
            47 => VolatileStatus::MagnetRise,
            48 => VolatileStatus::MaxGuard,
            49 => VolatileStatus::MeteorBeam,
            50 => VolatileStatus::Minimize,
            51 => VolatileStatus::MiracleEye,
            52 => VolatileStatus::MustRecharge,
            53 => VolatileStatus::NoRetreat,
            54 => VolatileStatus::Octolock,
            55 => VolatileStatus::PartiallyTrapped,
            56 => VolatileStatus::Perish1,
            57 => VolatileStatus::Perish2,
            58 => VolatileStatus::Perish3,
            59 => VolatileStatus::Perish4,
            60 => VolatileStatus::PhantomForce,
            61 => VolatileStatus::Powder,
            62 => VolatileStatus::PowerShift,
            63 => VolatileStatus::PowerTrick,
            64 => VolatileStatus::ProtosynthesisAttack,
            65 => VolatileStatus::ProtosynthesisDefense,
            66 => VolatileStatus::ProtosynthesisSpecialAttack,
            67 => VolatileStatus::ProtosynthesisSpecialDefense,
            68 => VolatileStatus::ProtosynthesisSpeed,
            69 => VolatileStatus::QuarkDriveAttack,
            70 => VolatileStatus::QuarkDriveDefense,
            71 => VolatileStatus::QuarkDriveSpecialAttack,
            72 => VolatileStatus::QuarkDriveSpecialDefense,
            73 => VolatileStatus::QuarkDriveSpeed,
            74 => VolatileStatus::Rage,
            75 => VolatileStatus::RagePowder,
            76 => VolatileStatus::RazorWind,
            77 => VolatileStatus::Reflect,
            78 => VolatileStatus::Roost,
            79 => VolatileStatus::SaltCure,
            80 => VolatileStatus::ShadowForce,
            81 => VolatileStatus::SkullBash,
            82 => VolatileStatus::SkyAttack,
            83 => VolatileStatus::SkyDrop,
            84 => VolatileStatus::SilkTrap,
            85 => VolatileStatus::SlowStart,
            86 => VolatileStatus::SmackDown,
            87 => VolatileStatus::Snatch,
            88 => VolatileStatus::SolarBeam,
            89 => VolatileStatus::SolarBlade,
            90 => VolatileStatus::SparklingAria,
            91 => VolatileStatus::SpikyShield,
            92 => VolatileStatus::Spotlight,
            93 => VolatileStatus::Stockpile,
            94 => VolatileStatus::SyrupBomb,
            95 => VolatileStatus::TarShot,
            96 => VolatileStatus::Telekinesis,
            97 => VolatileStatus::ThroatChop,
            98 => VolatileStatus::Truant,
            99 => VolatileStatus::TypeChange,
            100 => VolatileStatus::Unburden,
            101 => VolatileStatus::Uproar,
            102 => VolatileStatus::Yawn,
            103 => VolatileStatus::ChargingTurn,
            104 => VolatileStatus::Recharging,
            105 => VolatileStatus::TwoTurnMove,
            106 => VolatileStatus::BanefulBunkerProtect,
            107 => VolatileStatus::KingsShieldProtect,
            108 => VolatileStatus::MaxGuardProtect,
            109 => VolatileStatus::SpikyShieldProtect,
            110 => VolatileStatus::SilkTrapProtect,
            111 => VolatileStatus::AtkBoost1,
            112 => VolatileStatus::AtkBoost2,
            113 => VolatileStatus::AtkBoost3,
            114 => VolatileStatus::DefBoost1,
            115 => VolatileStatus::DefBoost2,
            116 => VolatileStatus::DefBoost3,
            117 => VolatileStatus::SpaBoost1,
            118 => VolatileStatus::SpaBoost2,
            119 => VolatileStatus::SpaBoost3,
            120 => VolatileStatus::SpdBoost1,
            121 => VolatileStatus::SpdBoost2,
            122 => VolatileStatus::SpdBoost3,
            123 => VolatileStatus::SpeBoost1,
            124 => VolatileStatus::SpeBoost2,
            125 => VolatileStatus::SpeBoost3,
            126 => VolatileStatus::ToxicCount,
            _ => VolatileStatus::Confusion,
        }
    }
}