//! # Tapu Simu UI Server
//! 
//! Web server binary for the tapu-simu testing UI.

use clap::Parser;
use tapu_simu::ui::server::start_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value_t = 3001)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🎮 Starting Tapu Simu Testing UI");
    println!("📁 Serving on port {}", args.port);
    println!("🌐 Open http://localhost:{} in your browser", args.port);
    
    start_server(args.port).await
}