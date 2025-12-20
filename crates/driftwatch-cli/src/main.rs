use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod adapters;
mod api;
mod commands;

use commands::{auth, project, run};

#[derive(Parser)]
#[command(name = "driftwatch")]
#[command(about = "CLI tool for benchmark submission and management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(
        long,
        env = "DRIFTWATCH_API_URL",
        default_value = "http://localhost:4000",
        global = true
    )]
    api_url: String,
}

#[derive(Subcommand)]
enum Commands {
    Serve(ServeArgs),
    Auth {
        #[command(subcommand)]
        command: auth::AuthCommands,
    },
    Project {
        #[command(subcommand)]
        command: project::ProjectCommands,
    },
    Run(run::RunArgs),
}

#[derive(Args)]
struct ServeArgs {
    #[arg(short, long, env = "PORT", default_value = "4000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info".into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            driftwatch_api::serve(Some(args.port)).await
        }
        Commands::Auth { command } => {
            init_cli_tracing();
            auth::handle(command).await
        }
        Commands::Project { command } => {
            init_cli_tracing();
            project::handle(command, &cli.api_url).await
        }
        Commands::Run(args) => {
            init_cli_tracing();
            run::handle(args, &cli.api_url).await
        }
    }
}

fn init_cli_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "driftwatch=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();
}
