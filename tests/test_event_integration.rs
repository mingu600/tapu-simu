use tapu_simu::dex::{ShowdownDex, Dex};
use tapu_simu::events::EventHandlerRegistry;
use std::path::Path;

#[test]
fn test_ability_event_handler_registration() {
    // Try to load real Pokemon Showdown data
    let dex = match ShowdownDex::new(Path::new(".")) {
        Ok(dex) => dex,
        Err(_) => {
            println!("Could not load Pokemon Showdown data, skipping test");
            return;
        }
    };
    
    // Test that Static ability has registered DamagingHit event
    if let Some(static_ability) = dex.get_ability("static") {
        assert!(static_ability.event_handlers.handles("DamagingHit"), 
                "Static ability should handle DamagingHit event");
        println!("✓ Static ability correctly handles DamagingHit");
    }
    
    // Test that Intimidate ability has registered SwitchIn event
    if let Some(intimidate_ability) = dex.get_ability("intimidate") {
        assert!(intimidate_ability.event_handlers.handles("SwitchIn"), 
                "Intimidate ability should handle SwitchIn event");
        println!("✓ Intimidate ability correctly handles SwitchIn");
    }
    
    // Test that a non-implemented ability doesn't handle events
    if let Some(other_ability) = dex.get_ability("levitate") {
        assert!(!other_ability.event_handlers.handles("DamagingHit"), 
                "Levitate ability should not handle DamagingHit event");
        println!("✓ Levitate ability correctly doesn't handle unimplemented events");
    }
}

#[test]
fn test_item_event_handler_registration() {
    // Try to load real Pokemon Showdown data
    let dex = match ShowdownDex::new(Path::new(".")) {
        Ok(dex) => dex,
        Err(_) => {
            println!("Could not load Pokemon Showdown data, skipping test");
            return;
        }
    };
    
    // Test that Leftovers item has registered Residual event
    if let Some(leftovers_item) = dex.get_item("leftovers") {
        assert!(leftovers_item.event_handlers.handles("Residual"), 
                "Leftovers item should handle Residual event");
        println!("✓ Leftovers item correctly handles Residual");
    }
    
    // Test that Choice Band item has registered ModifyAttack event
    if let Some(choice_band_item) = dex.get_item("choiceband") {
        assert!(choice_band_item.event_handlers.handles("ModifyAttack"), 
                "Choice Band item should handle ModifyAttack event");
        println!("✓ Choice Band item correctly handles ModifyAttack");
    }
}