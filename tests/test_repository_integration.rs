//! Integration test to validate repository-based weight and item lookups

use std::path::Path;
use tapu_simu::data::ps::repository::Repository;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_loading() {
        let repo_path = Path::new("data/ps-extracted");
        if !repo_path.exists() {
            println!("Skipping test - PS data not available");
            return;
        }
        
        let repository = Repository::from_path(repo_path);
        assert!(repository.is_ok(), "Repository should load successfully");
        
        let repo = repository.unwrap();
        let stats = repo.stats();
        
        println!("Repository loaded with {} Pokemon, {} items, {} moves, {} abilities", 
                 stats.pokemon_count, stats.item_count, stats.move_count, stats.ability_count);
        
        assert!(stats.pokemon_count > 0, "Should have Pokemon data");
        assert!(stats.item_count > 0, "Should have item data");
    }

    #[test]
    fn test_pokemon_weight_lookup() {
        let repo_path = Path::new("data/ps-extracted");
        if !repo_path.exists() {
            println!("Skipping test - PS data not available");
            return;
        }
        
        let repository = Repository::from_path(repo_path).expect("Repository should load");
        
        // Test some common Pokemon weights
        let pikachu_weight = repository.get_pokemon_weight("pikachu");
        let charizard_weight = repository.get_pokemon_weight("charizard");
        let bulbasaur_weight = repository.get_pokemon_weight("bulbasaur");
        
        println!("Pikachu weight: {:?}", pikachu_weight);
        println!("Charizard weight: {:?}", charizard_weight);
        println!("Bulbasaur weight: {:?}", bulbasaur_weight);
        
        // Verify weights are reasonable (Pikachu should be light, Charizard heavy)
        if let Some(pika_weight) = pikachu_weight {
            assert!(pika_weight > 0.0 && pika_weight < 10.0, "Pikachu should be light");
        }
        
        if let Some(char_weight) = charizard_weight {
            assert!(char_weight > 50.0 && char_weight < 200.0, "Charizard should be heavy");
        }
    }

    #[test]
    fn test_item_fling_power_lookup() {
        let repo_path = Path::new("data/ps-extracted");
        if !repo_path.exists() {
            println!("Skipping test - PS data not available");
            return;
        }
        
        let repository = Repository::from_path(repo_path).expect("Repository should load");
        
        // Test some items with known fling powers
        let flame_orb_power = repository.get_item_fling_power("flameorb");
        let toxic_orb_power = repository.get_item_fling_power("toxicorb");
        let iron_ball_power = repository.get_item_fling_power("ironball");
        
        println!("Flame Orb fling power: {:?}", flame_orb_power);
        println!("Toxic Orb fling power: {:?}", toxic_orb_power);
        println!("Iron Ball fling power: {:?}", iron_ball_power);
        
        // Verify fling powers are reasonable
        if let Some(power) = flame_orb_power {
            assert!(power > 0 && power <= 130, "Fling power should be reasonable");
        }
    }

    #[test]
    fn test_item_fling_capability() {
        let repo_path = Path::new("data/ps-extracted");
        if !repo_path.exists() {
            println!("Skipping test - PS data not available");
            return;
        }
        
        let repository = Repository::from_path(repo_path).expect("Repository should load");
        
        // Test regular items (should be flingable)
        let choice_band_flingable = repository.can_item_be_flung("choiceband");
        let leftovers_flingable = repository.can_item_be_flung("leftovers");
        
        // Test orbs (some should be unflingable)
        let red_orb_flingable = repository.can_item_be_flung("redorb");
        let blue_orb_flingable = repository.can_item_be_flung("blueorb");
        let flame_orb_flingable = repository.can_item_be_flung("flameorb");
        
        println!("Choice Band flingable: {}", choice_band_flingable);
        println!("Leftovers flingable: {}", leftovers_flingable);
        println!("Red Orb flingable: {}", red_orb_flingable);
        println!("Blue Orb flingable: {}", blue_orb_flingable);
        println!("Flame Orb flingable: {}", flame_orb_flingable);
        
        // Flame Orb should be flingable (normal battle item)
        // Red/Blue Orb should likely be unflingable (key items)
        if flame_orb_flingable {
            println!("✅ Flame Orb is correctly flingable");
        }
        
        if !red_orb_flingable {
            println!("✅ Red Orb is correctly unflingable");
        }
    }
}