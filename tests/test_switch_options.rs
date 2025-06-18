// Integration test for switch options
use tapu_simu::{State, BattleFormat, generation::Generation, battle_format::FormatType};
use tapu_simu::ui::bridge::EngineBridge;
use tapu_simu::state::Pokemon;

#[test]
fn test_switch_options_available() {
    // Create a basic battle format
    let format = BattleFormat::new("Test Singles".to_string(), Generation::Gen9, FormatType::Singles);
    
    // Create a battle state with multiple Pokemon
    let mut state = State::new(format.clone());
    
    // Add Pokemon to side one
    let mut pokemon1 = Pokemon::new("Pikachu".to_string());
    pokemon1.hp = 100;
    pokemon1.max_hp = 100;
    
    let mut pokemon2 = Pokemon::new("Charizard".to_string());
    pokemon2.hp = 150;
    pokemon2.max_hp = 150;
    
    let mut pokemon3 = Pokemon::new("Blastoise".to_string());
    pokemon3.hp = 160;
    pokemon3.max_hp = 160;
    
    state.side_one.add_pokemon(pokemon1);
    state.side_one.add_pokemon(pokemon2);
    state.side_one.add_pokemon(pokemon3);
    
    // Set active Pokemon (first one)
    state.side_one.active_pokemon_indices = vec![Some(0)];
    
    // Add Pokemon to side two
    let mut pokemon4 = Pokemon::new("Venusaur".to_string());
    pokemon4.hp = 140;
    pokemon4.max_hp = 140;
    
    state.side_two.add_pokemon(pokemon4);
    state.side_two.active_pokemon_indices = vec![Some(0)];
    
    // Create engine bridge
    let bridge = EngineBridge::new(format);
    
    // Get legal options
    let (side_one_options, side_two_options) = bridge.get_all_legal_options(&state).expect("Should get legal options");
    
    println!("Side One Legal Options:");
    for (i, option) in side_one_options.iter().enumerate() {
        println!("  {}: {} (type: {})", i, option.display_name, option.choice_type);
    }
    
    // Side one should have switch options since it has multiple Pokemon
    let switch_options: Vec<_> = side_one_options.iter()
        .filter(|opt| opt.choice_type == "switch")
        .collect();
    
    println!("Found {} switch options for side one", switch_options.len());
    
    // Should have 2 switch options (Pokemon 2 and 3, since Pokemon 1 is active)
    assert!(switch_options.len() >= 2, "Side one should have at least 2 switch options");
    
    // Verify switch options display correctly
    for switch_opt in &switch_options {
        assert!(switch_opt.display_name.contains("Switch to Pokemon"));
        assert_eq!(switch_opt.choice_type, "switch");
        assert!(switch_opt.move_choice.pokemon_index.is_some());
    }
}

#[test]
fn test_fainted_pokemon_switch_options() {
    // Create a basic battle format
    let format = BattleFormat::new("Test Singles".to_string(), Generation::Gen9, FormatType::Singles);
    
    // Create a battle state with multiple Pokemon
    let mut state = State::new(format.clone());
    
    // Add Pokemon to side one - first one fainted
    let mut pokemon1 = Pokemon::new("Pikachu".to_string());
    pokemon1.hp = 0; // Fainted
    pokemon1.max_hp = 100;
    
    let mut pokemon2 = Pokemon::new("Charizard".to_string());
    pokemon2.hp = 150;
    pokemon2.max_hp = 150;
    
    let mut pokemon3 = Pokemon::new("Blastoise".to_string());
    pokemon3.hp = 160;
    pokemon3.max_hp = 160;
    
    state.side_one.add_pokemon(pokemon1);
    state.side_one.add_pokemon(pokemon2);
    state.side_one.add_pokemon(pokemon3);
    
    // Set active Pokemon (first one - fainted)
    state.side_one.active_pokemon_indices = vec![Some(0)];
    
    // Add Pokemon to side two
    let mut pokemon4 = Pokemon::new("Venusaur".to_string());
    pokemon4.hp = 140;
    pokemon4.max_hp = 140;
    
    state.side_two.add_pokemon(pokemon4);
    state.side_two.active_pokemon_indices = vec![Some(0)];
    
    // Create engine bridge
    let bridge = EngineBridge::new(format);
    
    // Get legal options
    let (side_one_options, side_two_options) = bridge.get_all_legal_options(&state).expect("Should get legal options");
    
    println!("Side One Legal Options (with fainted active):");
    for (i, option) in side_one_options.iter().enumerate() {
        println!("  {}: {} (type: {})", i, option.display_name, option.choice_type);
    }
    
    // Side one should only have switch options when active Pokemon is fainted
    let switch_options: Vec<_> = side_one_options.iter()
        .filter(|opt| opt.choice_type == "switch")
        .collect();
    
    let move_options: Vec<_> = side_one_options.iter()
        .filter(|opt| opt.choice_type == "move")
        .collect();
    
    println!("Found {} switch options, {} move options", switch_options.len(), move_options.len());
    
    // With fainted Pokemon, should force switch (only switch options available)
    assert!(switch_options.len() >= 2, "Should have switch options for non-fainted Pokemon");
}