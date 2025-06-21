//! # Web Server
//! 
//! This module provides the web server for the tapu-simu testing UI.
//! It includes REST API endpoints and WebSocket support.

use axum::{
    extract::{Path, State as AxumState, WebSocketUpgrade},
    http::{Method, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::core::battle_format::BattleFormat;
use crate::core::battle_state::BattleState;
use crate::core::battle_format::FormatType;
use crate::generation::Generation;
use super::bridge::{
    EngineBridge, UIBattleState, UIBattleFormat, UIBattleSide, UIMoveChoice, 
    InstructionGenerationResponse, UIPokemon, UILegalOption
};
use super::pokemon_builder::PokemonBuilder;

/// Server state containing battle sessions
#[derive(Clone)]
pub struct ServerState {
    pub sessions: Arc<Mutex<HashMap<Uuid, BattleSession>>>,
    pub pokemon_builder: Arc<Mutex<PokemonBuilder>>,
}

/// A battle session
#[derive(Debug, Clone)]
pub struct BattleSession {
    pub id: Uuid,
    pub battle_state: BattleState,
    pub engine_bridge: EngineBridge,
    pub created_at: std::time::SystemTime,
}

/// Request to create a new battle session
#[derive(Debug, Deserialize)]
pub struct CreateBattleRequest {
    pub format: UIBattleFormat,
    pub side_one: UIBattleSide,
    pub side_two: UIBattleSide,
}

/// Response for battle session creation
#[derive(Debug, Serialize)]
pub struct CreateBattleResponse {
    pub session_id: Uuid,
    pub battle_state: UIBattleState,
}

/// Request to generate instructions
#[derive(Debug, Deserialize)]
pub struct GenerateInstructionsRequest {
    pub side_one_choice: UIMoveChoice,
    pub side_two_choice: UIMoveChoice,
}

/// Request to apply a specific instruction set
#[derive(Debug, Deserialize)]
pub struct ApplyInstructionSetRequest {
    pub instruction_set_index: usize,
    pub expected_turn_number: Option<u32>,
}

/// Request to preview a specific instruction set
#[derive(Debug, Deserialize)]
pub struct PreviewInstructionSetRequest {
    pub instruction_set_index: usize,
}

/// Request to update battle state directly
#[derive(Debug, Deserialize)]
pub struct UpdateBattleStateRequest {
    pub new_state: UIBattleState,
}

/// Response for legal options
#[derive(Debug, Serialize)]
pub struct LegalOptionsResponse {
    pub success: bool,
    pub error: Option<String>,
    pub side_one_options: Vec<UILegalOption>,
    pub side_two_options: Vec<UILegalOption>,
}

/// Request to create a Pokemon
#[derive(Debug, Deserialize)]
pub struct CreatePokemonRequest {
    pub species: String,
    pub level: Option<u8>,
}

/// Request to recalculate Pokemon stats
#[derive(Debug, Deserialize)]
pub struct RecalculatePokemonRequest {
    pub species: String,
    pub level: u8,
    pub ivs: [u8; 6],
    pub evs: [u8; 6],
    pub nature: String,
}

/// Request to create a fully customized Pokemon
#[derive(Debug, Deserialize)]
pub struct CreateCustomPokemonRequest {
    pub species: String,
    pub level: u8,
    pub ivs: [u8; 6],
    pub evs: [u8; 6],
    pub nature: String,
    pub ability: Option<String>,
    pub item: Option<String>,
    pub moves: Option<Vec<String>>,
    pub tera_type: Option<String>,
}

/// Query parameters for Pokemon data
#[derive(Debug, Deserialize)]
pub struct PokemonQuery {
    pub species: Option<String>,
}

impl ServerState {
    pub fn new() -> Self {
        println!("Creating ServerState...");
        println!("Current working directory: {:?}", std::env::current_dir().unwrap());
        
        let mut pokemon_builder = PokemonBuilder::new();
        
        // Try to load PS data, but don't fail if it's not available
        match pokemon_builder.load_data() {
            Ok(()) => {
                println!("Successfully loaded Pokemon data in ServerState!");
            }
            Err(e) => {
                eprintln!("Warning: Could not load PS data: {}. Using default data only.", e);
            }
        }

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pokemon_builder: Arc::new(Mutex::new(pokemon_builder)),
        }
    }
}

/// Create the main application router
pub fn create_app() -> Router {
    let state = ServerState::new();

    Router::new()
        // Frontend routes
        .route("/", get(serve_index))
        .route("/assets/*file", get(serve_static))
        
        // API routes
        .route("/api/battles", post(create_battle))
        .route("/api/battles/:session_id", get(get_battle))
        .route("/api/battles/:session_id/instructions", post(generate_instructions))
        .route("/api/battles/:session_id/apply", post(apply_instruction_set))
        .route("/api/battles/:session_id/preview", post(preview_instruction_set))
        .route("/api/battles/:session_id/legal-options", get(get_legal_options))
        .route("/api/battles/:session_id/state", get(get_battle_state))
        .route("/api/battles/:session_id/state", put(update_battle_state))
        
        // Pokemon data routes
        .route("/api/pokemon", get(get_pokemon_list))
        .route("/api/pokemon/create", post(create_pokemon))
        .route("/api/pokemon/recalculate", post(recalculate_pokemon_stats))
        .route("/api/pokemon/create-custom", post(create_custom_pokemon))
        .route("/api/pokemon/:species", get(get_pokemon_details))
        .route("/api/pokemon/:species/moves", get(get_pokemon_moves))
        .route("/api/pokemon/:species/abilities", get(get_pokemon_abilities))
        
        // Move and item data routes
        .route("/api/moves", get(get_move_list))
        .route("/api/moves/:move_name", get(get_move_details))
        .route("/api/items", get(get_item_list))
        
        // Preset routes
        .route("/api/presets/pokemon", get(get_pokemon_presets))
        .route("/api/presets/teams", get(get_preset_teams_list))
        .route("/api/presets/teams/:preset_name", get(get_preset_team))
        
        // WebSocket for real-time updates
        .route("/ws", get(websocket_handler))
        
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any)
        )
        .with_state(state)
}

/// Serve the main HTML page
async fn serve_index() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tapu Simu - Testing UI</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333; min-height: 100vh;
        }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { text-align: center; margin-bottom: 40px; color: white; }
        .header h1 { font-size: 3em; margin-bottom: 10px; }
        .header p { font-size: 1.2em; opacity: 0.9; }
        .main-content {
            background: white; border-radius: 20px; padding: 40px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }
        .features {
            display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 30px; margin-bottom: 40px;
        }
        .feature {
            padding: 30px; border-radius: 15px; background: #f8f9fa;
            border-left: 5px solid #667eea;
        }
        .feature h3 { color: #667eea; margin-bottom: 15px; font-size: 1.5em; }
        .feature p { line-height: 1.6; color: #666; }
        .api-status {
            text-align: center; padding: 20px; background: #e8f5e8;
            border-radius: 10px; margin-top: 30px;
        }
        .status-indicator {
            display: inline-block; width: 12px; height: 12px;
            background: #28a745; border-radius: 50%; margin-right: 10px;
        }
        .button {
            display: inline-block; padding: 12px 24px; background: #667eea;
            color: white; text-decoration: none; border-radius: 8px; margin: 10px;
            transition: all 0.3s ease;
        }
        .button:hover { background: #5a6fd8; transform: translateY(-2px); }
        .api-demo {
            margin-top: 30px; padding: 20px; background: #f8f9fa; border-radius: 10px;
        }
        .api-demo h3 { margin-bottom: 15px; }
        .api-endpoint {
            background: #e9ecef; padding: 10px; border-radius: 5px; margin: 10px 0;
            font-family: 'Courier New', monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌺 Tapu Simu</h1>
            <p>Format-aware Pokemon Battle Simulator Testing UI</p>
        </div>
        <div class="main-content">
            <div class="features">
                <div class="feature">
                    <h3>🏗️ Battle State Builder</h3>
                    <p>Easily create Pokemon teams with auto-populated defaults from PS data.</p>
                </div>
                <div class="feature">
                    <h3>⚔️ Move Selection Interface</h3>
                    <p>Format-aware move targeting with auto-resolution and priority preview.</p>
                </div>
                <div class="feature">
                    <h3>📊 Instruction Visualization</h3>
                    <p>View generated instructions with state changes and probability branches.</p>
                </div>
                <div class="feature">
                    <h3>🎮 Real-time Engine Integration</h3>
                    <p>Direct integration with tapu-simu engine for accurate battle mechanics.</p>
                </div>
            </div>
            <div class="api-status">
                <span class="status-indicator"></span>
                <strong>API Server Active</strong> - Ready for testing
            </div>
            <div class="api-demo">
                <h3>📡 Available API Endpoints</h3>
                <div class="api-endpoint">GET /api/pokemon - List all Pokemon species</div>
                <div class="api-endpoint">POST /api/battles - Create a new battle session</div>
                <div class="api-endpoint">POST /api/battles/{id}/instructions - Generate instructions</div>
                <div class="api-endpoint">GET /api/presets/teams/{name} - Get preset teams</div>
                <div style="text-align: center; margin-top: 20px;">
                    <a href="/api/pokemon" class="button">🔍 Test API</a>
                    <a href="/api/presets/pokemon" class="button">📋 View Presets</a>
                </div>
            </div>
        </div>
    </div>
    <script>
        async function testAPI() {
            try {
                const response = await fetch('/api/pokemon');
                const data = await response.json();
                console.log('Pokemon data:', data);
            } catch (error) {
                console.error('API test failed:', error);
            }
        }
        testAPI(); setInterval(testAPI, 5000);
    </script>
</body>
</html>"#)
}

/// Serve static assets
async fn serve_static(Path(_file): Path<String>) -> impl IntoResponse {
    // In a real implementation, this would serve files from the ui/dist directory
    (StatusCode::NOT_FOUND, "Static file serving not implemented")
}

/// Create a new battle session
async fn create_battle(
    AxumState(state): AxumState<ServerState>,
    Json(request): Json<CreateBattleRequest>,
) -> Result<Json<CreateBattleResponse>, StatusCode> {
    let session_id = Uuid::new_v4();

    // Convert format
    let format_type = match request.format.format_type.as_str() {
        "Singles" => FormatType::Singles,
        "Doubles" => FormatType::Doubles,
        "Vgc" => FormatType::Vgc,
        "Triples" => FormatType::Triples,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let generation = match request.format.generation.as_str() {
        "Gen9" => Generation::Gen9,
        "Gen8" => Generation::Gen8,
        "Gen7" => Generation::Gen7,
        "Gen6" => Generation::Gen6,
        "Gen5" => Generation::Gen5,
        "Gen4" => Generation::Gen4,
        _ => Generation::Gen9,
    };

    let battle_format = BattleFormat::new(request.format.name.clone(), generation, format_type);
    let engine_bridge = EngineBridge::new(battle_format.clone());

    // Create battle state
    let battle_state = match engine_bridge.create_battle_state(&request.format, &request.side_one, &request.side_two) {
        Ok(state) => state,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    let session = BattleSession {
        id: session_id,
        battle_state: battle_state.clone(),
        engine_bridge: engine_bridge.clone(),
        created_at: std::time::SystemTime::now(),
    };

    // Store session
    state.sessions.lock().unwrap().insert(session_id, session);

    Ok(Json(CreateBattleResponse {
        session_id,
        battle_state: engine_bridge.state_to_ui(&battle_state),
    }))
}

/// Get battle session details
async fn get_battle(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<UIBattleState>, StatusCode> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(session.engine_bridge.state_to_ui(&session.battle_state)))
}

/// Generate instructions for a battle
async fn generate_instructions(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<GenerateInstructionsRequest>,
) -> Result<Json<InstructionGenerationResponse>, StatusCode> {
    let mut sessions = state.sessions.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    let response = session.engine_bridge.generate_instructions(
        &mut session.battle_state,
        &request.side_one_choice,
        &request.side_two_choice,
    );

    Ok(Json(response))
}

/// Apply a specific instruction set to the battle state
async fn apply_instruction_set(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<ApplyInstructionSetRequest>,
) -> Result<Json<UIBattleState>, StatusCode> {
    let mut sessions = state.sessions.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    match session.engine_bridge.apply_instruction_set(
        &mut session.battle_state,
        request.instruction_set_index,
        request.expected_turn_number,
    ) {
        Ok(updated_state) => Ok(Json(updated_state)),
        Err(e) => {
            eprintln!("Failed to apply instruction set: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Preview a specific instruction set without applying it
async fn preview_instruction_set(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<PreviewInstructionSetRequest>,
) -> Result<Json<UIBattleState>, StatusCode> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    // Clone the battle state for preview
    let mut preview_state = session.battle_state.clone();
    
    match session.engine_bridge.apply_instruction_set(
        &mut preview_state,
        request.instruction_set_index,
        None, // Don't change turn number for previews
    ) {
        Ok(updated_state) => Ok(Json(updated_state)),
        Err(e) => {
            eprintln!("Failed to preview instruction set: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get legal options for both sides
async fn get_legal_options(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<LegalOptionsResponse>, StatusCode> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    match session.engine_bridge.get_all_legal_options(&session.battle_state) {
        Ok((side_one_options, side_two_options)) => {
            Ok(Json(LegalOptionsResponse {
                success: true,
                error: None,
                side_one_options,
                side_two_options,
            }))
        }
        Err(e) => {
            eprintln!("Failed to get legal options: {}", e);
            Ok(Json(LegalOptionsResponse {
                success: false,
                error: Some(e),
                side_one_options: vec![],
                side_two_options: vec![],
            }))
        }
    }
}

/// Get current battle state
async fn get_battle_state(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<UIBattleState>, StatusCode> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(session.engine_bridge.state_to_ui(&session.battle_state)))
}

/// Update battle state directly (used for going to previous turns)
async fn update_battle_state(
    AxumState(state): AxumState<ServerState>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<UpdateBattleStateRequest>,
) -> Result<Json<UIBattleState>, StatusCode> {
    let mut sessions = state.sessions.lock().unwrap();
    let session = sessions.get_mut(&session_id).ok_or(StatusCode::NOT_FOUND)?;
    
    // Convert UI state back to internal state
    match session.engine_bridge.ui_to_state(&request.new_state) {
        Ok(new_state) => {
            session.battle_state = new_state;
            Ok(Json(session.engine_bridge.state_to_ui(&session.battle_state)))
        }
        Err(e) => {
            eprintln!("Failed to convert UI state to internal state: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get list of available Pokemon
async fn get_pokemon_list(
    AxumState(state): AxumState<ServerState>,
) -> Json<Vec<String>> {
    let builder = state.pokemon_builder.lock().unwrap();
    Json(builder.get_species_list())
}

/// Create a Pokemon
async fn create_pokemon(
    AxumState(state): AxumState<ServerState>,
    Json(request): Json<CreatePokemonRequest>,
) -> Result<Json<UIPokemon>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    let level = request.level.unwrap_or(50);
    
    match builder.create_pokemon(&request.species, level) {
        Ok(pokemon) => Ok(Json(pokemon)),
        Err(e) => {
            eprintln!("Failed to create Pokemon {}: {}", request.species, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Recalculate Pokemon stats with custom IVs/EVs/nature
async fn recalculate_pokemon_stats(
    AxumState(state): AxumState<ServerState>,
    Json(request): Json<RecalculatePokemonRequest>,
) -> Result<Json<UIPokemon>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    
    match builder.create_pokemon_with_custom_stats(&request.species, request.level, &request.ivs, &request.evs, &request.nature) {
        Ok(pokemon) => Ok(Json(pokemon)),
        Err(e) => {
            eprintln!("Failed to recalculate Pokemon stats for {}: {}", request.species, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a fully customized Pokemon with moves, abilities, items
async fn create_custom_pokemon(
    AxumState(state): AxumState<ServerState>,
    Json(request): Json<CreateCustomPokemonRequest>,
) -> Result<Json<UIPokemon>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    
    match builder.create_fully_custom_pokemon(
        &request.species,
        request.level,
        &request.ivs,
        &request.evs,
        &request.nature,
        request.ability.as_deref(),
        request.item.as_deref(),
        request.moves.as_ref().map(|m| m.as_slice()),
        request.tera_type.as_deref(),
    ) {
        Ok(pokemon) => Ok(Json(pokemon)),
        Err(e) => {
            eprintln!("Failed to create custom Pokemon {}: {}", request.species, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get Pokemon details
async fn get_pokemon_details(
    AxumState(state): AxumState<ServerState>,
    Path(species): Path<String>,
) -> Result<Json<UIPokemon>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    
    match builder.create_pokemon(&species, 50) {
        Ok(pokemon) => Ok(Json(pokemon)),
        Err(e) => {
            eprintln!("Failed to get Pokemon details for {}: {}", species, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get moves for a Pokemon
async fn get_pokemon_moves(
    AxumState(state): AxumState<ServerState>,
    Path(species): Path<String>,
) -> Json<Vec<String>> {
    let builder = state.pokemon_builder.lock().unwrap();
    Json(builder.get_pokemon_moves(&species))
}

/// Get abilities for a Pokemon
async fn get_pokemon_abilities(
    AxumState(state): AxumState<ServerState>,
    Path(species): Path<String>,
) -> Json<Vec<String>> {
    let builder = state.pokemon_builder.lock().unwrap();
    Json(builder.get_pokemon_abilities(&species))
}

/// Get list of available moves
async fn get_move_list(
    AxumState(state): AxumState<ServerState>,
) -> Json<Vec<String>> {
    let builder = state.pokemon_builder.lock().unwrap();
    Json(builder.get_move_list())
}

/// Get move details
async fn get_move_details(
    AxumState(state): AxumState<ServerState>,
    Path(move_name): Path<String>,
) -> Result<Json<super::bridge::UIMove>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    
    match builder.create_move(&move_name) {
        Ok(move_data) => Ok(Json(move_data)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Get list of available items
async fn get_item_list(
    AxumState(state): AxumState<ServerState>,
) -> Json<Vec<String>> {
    let builder = state.pokemon_builder.lock().unwrap();
    Json(builder.get_item_list())
}

/// Get Pokemon presets
async fn get_pokemon_presets() -> Json<Vec<(String, String)>> {
    let presets = PokemonBuilder::get_suggested_pokemon()
        .into_iter()
        .map(|(name, desc)| (name.to_string(), desc.to_string()))
        .collect();
    Json(presets)
}

/// Get available preset teams
async fn get_preset_teams_list() -> Json<Vec<(String, String)>> {
    let teams = PokemonBuilder::get_preset_teams()
        .into_iter()
        .map(|(name, desc)| (name.to_string(), desc.to_string()))
        .collect();
    Json(teams)
}

/// Get a preset team
async fn get_preset_team(
    AxumState(state): AxumState<ServerState>,
    Path(preset_name): Path<String>,
) -> Result<Json<Vec<UIPokemon>>, StatusCode> {
    let builder = state.pokemon_builder.lock().unwrap();
    
    match builder.create_preset_team(&preset_name) {
        Ok(team) => Ok(Json(team)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    AxumState(_state): AxumState<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_websocket)
}

/// Handle WebSocket connections
async fn handle_websocket(mut socket: axum::extract::ws::WebSocket) {
    // For now, just echo messages
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if socket.send(msg).await.is_err() {
                break;
            }
        } else {
            break;
        }
    }
}

/// Start the web server
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🚀 Tapu Simu UI server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}