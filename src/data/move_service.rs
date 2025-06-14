//! # Move Data Service
//! 
//! Service for managing move data integration between rustemon/PokeAPI and engine-specific enhancements.
//! This is adapted from V1's move_service.rs to work with Tapu Simu's V2 architecture.

use super::types::EngineMoveData;
use super::rustemon_client::RustemonClient;
use super::conversion::rustemon_move_to_engine;
// Remove unused import
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Engine-specific data that supplements rustemon data
#[derive(Debug, Clone, Default)]
pub struct EngineSpecificMoveData {
    pub priority: i8,
    pub drain: Option<f32>,
    pub recoil: Option<f32>,
    pub crash: Option<f32>,
    pub heal: Option<HealEffect>,
    pub boost: Option<StatBoostEffect>,
    pub secondaries: Option<Vec<SecondaryEffect>>,
    pub flags: MoveFlags,
}

/// Heal effect data
#[derive(Debug, Clone)]
pub struct HealEffect {
    pub target: HealTarget,
    pub amount: f32,
}

/// Stat boost effect data
#[derive(Debug, Clone)]
pub struct StatBoostEffect {
    pub target: BoostTarget,
    pub boosts: StatBoosts,
}

/// Secondary effect data
#[derive(Debug, Clone)]
pub struct SecondaryEffect {
    pub chance: f32,
    pub effect: SecondaryEffectType,
}

/// Move flags
#[derive(Debug, Clone, Default)]
pub struct MoveFlags {
    pub contact: bool,
    pub protect: bool,
    pub heal: bool,
    pub sound: bool,
    pub powder: bool,
    pub punch: bool,
    pub bite: bool,
    pub bullet: bool,
    pub wind: bool,
    pub dance: bool,
}

/// Heal target types
#[derive(Debug, Clone)]
pub enum HealTarget {
    User,
    Ally,
    All,
}

/// Boost target types
#[derive(Debug, Clone)]
pub enum BoostTarget {
    User,
    Target,
    All,
}

/// Stat boosts
#[derive(Debug, Clone)]
pub struct StatBoosts {
    pub attack: i8,
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

/// Secondary effect types
#[derive(Debug, Clone)]
pub enum SecondaryEffectType {
    Status(StatusEffect),
    Boost(StatBoostEffect),
    Heal(HealEffect),
}

/// Status effect data
#[derive(Debug, Clone)]
pub enum StatusEffect {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned,
    Sleep,
}

// Use Choices enum from choices module
pub use super::choices::Choices;

/// Service for managing move data from rustemon with engine-specific enhancements
pub struct MoveDataService {
    client: Arc<RwLock<RustemonClient>>,
    cache: Arc<RwLock<HashMap<Choices, EngineMoveData>>>,
    engine_data: Arc<RwLock<HashMap<Choices, EngineSpecificMoveData>>>,
}

impl MoveDataService {
    /// Create a new move data service
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = RustemonClient::new();
        Ok(Self {
            client: Arc::new(RwLock::new(client)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            engine_data: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get move data for a specific move, fetching from rustemon if needed
    pub async fn get_move_data(&self, move_id: Choices) -> Result<EngineMoveData, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(data) = cache.get(&move_id) {
                return Ok(data.clone());
            }
        }

        // Fetch from rustemon
        let move_name = move_id.to_rustemon_name();
        let rustemon_move = {
            let mut client = self.client.write().await;
            client.get_move(move_name).await?
        };
        let engine_move_data = rustemon_move_to_engine(rustemon_move);

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(move_id, engine_move_data.clone());
        }

        Ok(engine_move_data)
    }

    /// Register engine-specific data for a move
    pub async fn register_engine_data(&self, move_id: Choices, data: EngineSpecificMoveData) {
        let mut engine_data = self.engine_data.write().await;
        engine_data.insert(move_id, data);
    }

    /// Get engine-specific data for a move
    pub async fn get_engine_data(&self, move_id: Choices) -> EngineSpecificMoveData {
        let engine_data = self.engine_data.read().await;
        engine_data.get(&move_id).cloned().unwrap_or_default()
    }

    /// Create enhanced move data combining rustemon + engine-specific data
    pub async fn get_enhanced_move_data(&self, move_id: Choices) -> Result<EnhancedMoveData, Box<dyn std::error::Error>> {
        let base_data = self.get_move_data(move_id).await?;
        let engine_data = self.get_engine_data(move_id).await;

        Ok(EnhancedMoveData {
            base_data,
            engine_data,
        })
    }
}

/// Combined move data with both rustemon and engine-specific information
#[derive(Debug, Clone)]
pub struct EnhancedMoveData {
    pub base_data: EngineMoveData,
    pub engine_data: EngineSpecificMoveData,
}

/// Builder for engine-specific move data
pub struct EngineDataBuilder {
    data: EngineSpecificMoveData,
}

impl EngineDataBuilder {
    pub fn new() -> Self {
        Self {
            data: EngineSpecificMoveData::default(),
        }
    }

    pub fn priority(mut self, priority: i8) -> Self {
        self.data.priority = priority;
        self
    }

    pub fn drain(mut self, drain: f32) -> Self {
        self.data.drain = Some(drain);
        self
    }

    pub fn recoil(mut self, recoil: f32) -> Self {
        self.data.recoil = Some(recoil);
        self
    }

    pub fn flags(mut self, flags: MoveFlags) -> Self {
        self.data.flags = flags;
        self
    }

    pub fn build(self) -> EngineSpecificMoveData {
        self.data
    }
}

impl Default for EngineDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}