//! # Testing UI Module
//! 
//! This module provides a web-based testing interface for the tapu-simu engine.
//! It includes a REST API server, WebSocket support, and engine integration.

pub mod server;
pub mod bridge;
pub mod pokemon_builder;

pub use server::*;
pub use bridge::*;
pub use pokemon_builder::*;