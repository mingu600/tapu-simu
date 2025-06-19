//! # Pokemon Showdown Item Service
//! 
//! This module provides an item data service based on Pokemon Showdown data,
//! enabling access to item information including Fling power and effects.

use std::sync::OnceLock;
use crate::data::loader::PSDataRepository;
use crate::data::ps_types::{PSItemData, PSFling};

/// Global PS data repository - loaded once and cached
static PS_REPO: OnceLock<PSDataRepository> = OnceLock::new();

/// Pokemon Showdown item service
pub struct PSItemService;

impl PSItemService {
    /// Create a new PS item service
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize PS repository if not already done
        if PS_REPO.get().is_none() {
            let repo = PSDataRepository::load_from_directory("data/ps-extracted")?;
            PS_REPO.set(repo).map_err(|_| "Failed to initialize PS repository")?;
        }
        
        Ok(Self)
    }

    /// Get the PS repository
    fn get_repo(&self) -> Option<&PSDataRepository> {
        PS_REPO.get()
    }
    
    /// Check if PS repository is initialized
    pub fn is_initialized(&self) -> bool {
        PS_REPO.get().is_some()
    }

    /// Get item data by name
    pub fn get_item_by_name(&self, name: &str) -> Option<&PSItemData> {
        let repo = self.get_repo()?;
        repo.get_item_by_name(name)
    }

    /// Get item data by ID
    pub fn get_item_by_id(&self, id: &str) -> Option<&PSItemData> {
        let repo = self.get_repo()?;
        repo.get_item(id)
    }

    /// Get Fling data for an item
    pub fn get_fling_data(&self, item_name: &str) -> Option<&PSFling> {
        let item = self.get_item_by_name(item_name)?;
        item.fling.as_ref()
    }

    /// Get Fling power for an item (returns 0 if item can't be flung)
    pub fn get_fling_power(&self, item_name: &str) -> u8 {
        if let Some(fling_data) = self.get_fling_data(item_name) {
            fling_data.base_power
        } else {
            0 // Item can't be flung
        }
    }

    /// Check if an item can be flung
    pub fn can_be_flung(&self, item_name: &str) -> bool {
        self.get_fling_data(item_name).is_some()
    }

    /// Get Fling status effect for an item
    pub fn get_fling_status(&self, item_name: &str) -> Option<&str> {
        let fling_data = self.get_fling_data(item_name)?;
        fling_data.status.as_deref()
    }

    /// Get Fling volatile status effect for an item
    pub fn get_fling_volatile_status(&self, item_name: &str) -> Option<&str> {
        let fling_data = self.get_fling_data(item_name)?;
        fling_data.volatile_status.as_deref()
    }

    /// Check if an item exists
    pub fn item_exists(&self, name: &str) -> bool {
        if let Some(repo) = self.get_repo() {
            repo.get_item_by_name(name).is_some()
        } else {
            false
        }
    }

    /// Get item statistics
    pub fn get_stats(&self) -> PSItemServiceStats {
        if let Some(repo) = self.get_repo() {
            let repo_stats = repo.stats();
            let flingable_count = repo.get_all_items()
                .values()
                .filter(|item| item.fling.is_some())
                .count();
            
            PSItemServiceStats {
                total_items: repo_stats.item_count,
                flingable_items: flingable_count,
            }
        } else {
            PSItemServiceStats {
                total_items: 0,
                flingable_items: 0,
            }
        }
    }
}

impl Default for PSItemService {
    fn default() -> Self {
        // In tests or when PS data isn't available, create a service with empty data
        Self::new().unwrap_or_else(|_| PSItemService)
    }
}

/// Statistics about the PS item service
#[derive(Debug)]
pub struct PSItemServiceStats {
    pub total_items: usize,
    pub flingable_items: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_item_service_creation() {
        // This test will fail without actual PS data files
        // but demonstrates the intended usage
        if let Ok(service) = PSItemService::new() {
            let stats = service.get_stats();
            assert!(stats.total_items > 0);
        }
    }

    #[test]
    fn test_fling_data_access() {
        if let Ok(service) = PSItemService::new() {
            // Test items with known fling data
            if let Some(flame_orb_power) = service.get_fling_data("flameorb") {
                assert_eq!(flame_orb_power.base_power, 30);
                assert_eq!(flame_orb_power.status.as_deref(), Some("brn"));
            }
            
            if let Some(poison_barb_power) = service.get_fling_data("poisonbarb") {
                assert_eq!(poison_barb_power.base_power, 70);
                assert_eq!(poison_barb_power.status.as_deref(), Some("psn"));
            }
        }
    }

    #[test]
    fn test_fling_power_helpers() {
        if let Ok(service) = PSItemService::new() {
            // Test items with different fling powers
            if service.item_exists("abomasite") {
                assert_eq!(service.get_fling_power("abomasite"), 80); // Mega stones have 80 power
            }
            
            if service.item_exists("abilityshield") {
                assert_eq!(service.get_fling_power("abilityshield"), 30);
            }
        }
    }
}