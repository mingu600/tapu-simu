//! # Move Factory
//! 
//! Factory for creating moves using rustemon data with engine-specific enhancements.
//! This is adapted from V1's move_factory.rs for Tapu Simu's V2 architecture.

use super::move_service::{MoveDataService, Choices, EngineDataBuilder, MoveFlags, EnhancedMoveData};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Factory for creating moves using rustemon data with engine-specific enhancements
pub struct MoveFactory {
    service: Arc<MoveDataService>,
    initialized: OnceCell<()>,
}

impl MoveFactory {
    /// Create a new move factory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let service = MoveDataService::new()?;
        Ok(Self {
            service: Arc::new(service),
            initialized: OnceCell::new(),
        })
    }

    /// Initialize the factory with engine-specific move data
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialized.get_or_try_init(|| async {
            self.register_engine_specific_data().await
        }).await?;
        Ok(())
    }

    /// Get enhanced move data for a move
    pub async fn get_move(&self, move_id: Choices) -> Result<EnhancedMoveData, Box<dyn std::error::Error>> {
        self.initialize().await?;
        self.service.get_enhanced_move_data(move_id).await
    }

    /// Register engine-specific data for important moves
    async fn register_engine_specific_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Register Absorb as a draining move
        self.service.register_engine_data(
            Choices::ABSORB,
            EngineDataBuilder::new()
                .drain(0.5)
                .flags(MoveFlags {
                    heal: true,
                    protect: true,
                    ..Default::default()
                })
                .build()
        ).await;

        // Register Thunderbolt with paralysis chance
        // TODO: Add secondary effects support

        // Add more moves as needed...

        Ok(())
    }
}

impl Default for MoveFactory {
    fn default() -> Self {
        Self::new().expect("Failed to create MoveFactory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_move_factory_creation() {
        let factory = MoveFactory::new().unwrap();
        assert!(factory.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_absorb_move_data() {
        let factory = MoveFactory::new().unwrap();
        factory.initialize().await.unwrap();
        
        let move_data = factory.get_move(Choices::ABSORB).await.unwrap();
        assert_eq!(move_data.base_data.name, "absorb");
        assert_eq!(move_data.engine_data.drain, Some(0.5));
        assert!(move_data.engine_data.flags.heal);
    }
}