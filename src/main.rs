mod asset_pipeline;
mod cli;
mod claude;
mod config;
mod cron;
mod daemon;
mod dag;
mod error;
mod hooks;
mod interview;
mod notifications;
mod orchestrator;
mod remote;
mod sandbox;
mod setup;
mod skills;
mod state;
mod tui;
mod types;
mod visual_qa;

use clap::Parser;
use cli::{Cli, Commands};
use config::BtcConfig;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> error::BtcResult<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("btc=debug")
    } else {
        EnvFilter::new("btc=info")
    };
    fmt().with_env_filter(filter).init();

    let project_dir = std::env::current_dir()?;
    let _config = BtcConfig::load(&project_dir).unwrap_or_default();

    match cli.command {
        Commands::Setup => {
            let wizard = setup::SetupWizard::new(project_dir);
            wizard.run()
        }
        Commands::New { description } => {
            let runner = interview::InterviewRunner::new(project_dir);
            let spec_path = runner.run(&description)?;
            println!("Spec written to: {}", spec_path.display());
            Ok(())
        }
        Commands::Plan { spec, consensus } => {
            tracing::info!("Generating plan (consensus: {})", consensus);
            println!("BTC plan — generating execution plan");
            let _ = spec;
            // TODO: Phase 5 — DagPlanner
            Ok(())
        }
        Commands::Run { plan, debug: debug_mode } => {
            tracing::info!("Executing plan (debug: {})", debug_mode);
            println!("BTC run — executing with multi-agent orchestration");
            let _ = plan;
            // TODO: Phase 5 — DagExecutor
            Ok(())
        }
        Commands::Daemon { action } => {
            tracing::info!("Daemon command: {:?}", action);
            println!("BTC daemon — managing background daemon");
            // TODO: Phase 6 — DaemonServer
            Ok(())
        }
        Commands::Cron { action } => {
            tracing::info!("Cron command: {:?}", action);
            println!("BTC cron — managing scheduled workflows");
            // TODO: Phase 6 — CronScheduler
            Ok(())
        }
        Commands::Status => {
            let daemon_socket = project_dir.join(".btc").join("daemon.sock");
            let client = daemon::DaemonClient::new(daemon_socket);
            if client.is_daemon_running() {
                println!("Daemon is running. Connect for live status.");
            } else {
                println!("Daemon offline. Use `btc daemon start` to start.");
                // Read last known state
                let state_dir = project_dir.join(".btc").join("state");
                if let Ok(state_mgr) = state::StateManager::new(state_dir) {
                    if let Ok(status) = state_mgr.read::<serde_json::Value>("orchestrator") {
                        println!(
                            "Last known state: {}",
                            serde_json::to_string_pretty(&status).unwrap_or_default()
                        );
                    }
                }
            }
            Ok(())
        }
        Commands::Debug { action } => {
            tracing::info!("Debug command: {:?}", action);
            println!("BTC debug");
            // TODO: Phase 7 — debug utilities
            Ok(())
        }
    }
}
