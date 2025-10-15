use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;


mod client;
mod commands;
mod config;
mod error;
mod models;

use crate::config::{Config, CliOverrides};
use crate::error::Result;

#[derive(Parser)]
#[command(name = "appian-deployment-cli")]
#[command(about = "Appian Deployment CLI - Automate Appian deployments via REST API v2")]
#[command(version)]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Configuration file path")]
    config_file: Option<PathBuf>,

    #[arg(long, global = true, help = "Base URL for Appian API")]
    base_url: Option<String>,

    #[arg(long, global = true, help = "API key for authentication")]
    api_key: Option<String>,

    #[arg(long, global = true, help = "Enable verbose output")]
    verbose: bool,

    #[arg(long, global = true, help = "Suppress non-essential output")]
    quiet: bool,

    #[arg(long, global = true, help = "Output format (text or json)")]
    format: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "get_packages")]
    #[command(about = "List packages for applications")]
    GetPackages {
        #[arg(long, help = "Application UUID (repeatable)")]
        app_uuid: Vec<String>,
    },

    #[cfg(feature = "export")]
    #[command(about = "Export application to artifact zip")]
    Export {
        #[arg(long, value_delimiter = ',', help = "UUIDs to export (repeatable or comma-separated)")]
        uuids: Vec<String>,

        #[arg(long, default_value = "package", help = "Export type (package|application)")]
        export_type: String,

        #[arg(long, help = "Export name")]
        name: Option<String>,

        #[arg(long, help = "Export description")]
        description: Option<String>,

        #[arg(long, help = "Validate without execution")]
        dry_run: bool,
    },

    #[cfg(feature = "validate")]
    #[command(about = "Inspect package via API")]
    Inspect {
        #[arg(long, help = "Package zip file path")]
        package_zip_name: PathBuf,

        #[arg(long, help = "Import customization properties file (.properties)")]
        customization_file: Option<PathBuf>,

        #[arg(long, help = "Admin Console settings zip (.zip)")]
        admin_console_file: Option<PathBuf>,
    },

    #[cfg(feature = "validate")]
    #[command(name = "get-inspection", about = "Get inspection results by UUID")]
    GetInspection {
        #[arg(long, help = "Inspection UUID")]
        uuid: String,
    },

    #[cfg(feature = "deploy")]
    #[command(about = "Deploy package to target environment")]
    Deploy {
        #[arg(long, help = "Package zip file path")]
        package_zip_name: PathBuf,

        #[arg(long, help = "Deployment name")]
        name: String,

        #[arg(long, help = "Deployment description")]
        description: Option<String>,

        #[arg(long, help = "Plan-only deployment")]
        dry_run: bool,

        #[arg(long, default_value = "true", help = "Rollback on failure")]
        rollback_on_failure: bool,

        #[arg(long, help = "Import customization properties file (.properties)")]
        customization_file: Option<PathBuf>,

        #[arg(long, help = "Admin Console settings zip (.zip)")]
        admin_console_file: Option<PathBuf>,

        #[arg(long, help = "Plug-ins file (.zip)")]
        plugins_file: Option<PathBuf>,

        #[arg(long, help = "Data source name or UUID")]
        data_source: Option<String>,

        #[arg(long, value_delimiter = ',', help = "Comma-separated database scripts (.sql,.ddl) in execution order")]
        database_scripts: Option<Vec<PathBuf>>,
    },

    #[cfg(feature = "status")]
    #[command(about = "Check deployment status", alias = "get-deployment")]
    Status {
        #[arg(long, help = "Deployment UUID")]
        deployment_uuid: String,

        #[arg(long, help = "Operation kind (export or deployment)")]
        kind: Option<String>,
    },

    #[cfg(feature = "status")]
    #[command(about = "Retrieve deployment results", alias = "results")]
    GetDeploymentResults {
        #[arg(long, alias = "uuid", help = "Deployment UUID", value_name = "DEPLOYMENT_UUID")]
        deployment_uuid: String,

        #[arg(long, help = "Poll until terminal status before printing results")]
        poll: bool,
    },

    #[cfg(feature = "monitor")]
    #[command(about = "Monitor deployment until completion")]
    Monitor {
        #[arg(long, help = "Deployment UUID")]
        deployment_uuid: String,

        #[arg(long, help = "Operation kind (export or deployment)")]
        kind: Option<String>,

        #[arg(long, default_value = "10", help = "Polling interval in seconds")]
        interval_seconds: u64,

        #[arg(long, help = "Timeout in seconds")]
        timeout_seconds: Option<u64>,
    },

    #[cfg(feature = "download")]
    #[command(about = "Download exported artifact")]
    DownloadPackage {
        #[arg(long, help = "Deployment UUID")]
        deployment_uuid: String,

        #[arg(long, help = "Output directory or file")]
        output: Option<PathBuf>,

        #[arg(long, help = "Overwrite existing files")]
        overwrite: bool,
    },

    #[cfg(feature = "logs")]
    #[command(about = "Retrieve deployment logs")]
    Logs {
        #[arg(long, help = "Deployment UUID")]
        deployment_uuid: String,

        #[arg(long, help = "Stream logs until completion")]
        follow: bool,

        #[arg(long, help = "Number of lines to show from the end of logs")]
        tail: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> crate::error::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let _log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    setup_logging(&cli)?;

    info!("Appian Deployment CLI starting");

    let cli_overrides = CliOverrides {
        base_url: cli.base_url.clone(),
        api_key: cli.api_key.clone(),
    };
    let config = Config::load(cli.config_file.clone(), &cli_overrides)?;

    // Execute command
    match cli.command {
        #[cfg(feature = "get_packages")]
        Commands::GetPackages { app_uuid } => {
            commands::get_packages::execute(config, app_uuid, cli.format).await?;
        }
        #[cfg(feature = "export")]
        Commands::Export { 
            uuids,
            export_type,
            name,
            description,
            dry_run,
        } => {
            commands::export::execute(
                config,
                uuids,
                export_type,
                name,
                description,
                dry_run,
                cli.format,
            ).await?;
        }
        #[cfg(feature = "validate")]
        Commands::Inspect { package_zip_name, customization_file, admin_console_file } => {
            commands::inspect::execute(
                config,
                package_zip_name,
                customization_file,
                admin_console_file,
                cli.format,
            ).await?;
        }
        #[cfg(feature = "validate")]
        Commands::GetInspection { uuid } => {
            commands::inspection_results::execute(
                config,
                uuid,
                cli.format,
            ).await?;
        }
        #[cfg(feature = "deploy")]
        Commands::Deploy { 
            package_zip_name,
            name,
            description,
            dry_run,
            rollback_on_failure,
            customization_file,
            admin_console_file,
            plugins_file,
            data_source,
            database_scripts,
        } => {
            commands::deploy::execute(
                config,
                package_zip_name,
                name,
                description,
                dry_run,
                rollback_on_failure,
                customization_file,
                admin_console_file,
                plugins_file,
                data_source,
                database_scripts,
                cli.format,
            ).await?;
        }
        #[cfg(feature = "status")]
        Commands::Status { deployment_uuid, kind } => {
            commands::status::execute(config, deployment_uuid, kind, cli.format).await?;
        }
        #[cfg(feature = "status")]
        Commands::GetDeploymentResults { deployment_uuid, poll } => {
            commands::deployment_results::execute(config, deployment_uuid, cli.format, poll).await?;
        }
        #[cfg(feature = "monitor")]
        Commands::Monitor { 
            deployment_uuid,
            kind,
            interval_seconds,
            timeout_seconds,
        } => {
            commands::monitor::execute(
                config,
                deployment_uuid,
                kind,
                interval_seconds,
                timeout_seconds.unwrap_or(3600), // Default to 1 hour
                cli.format,
            ).await?;
        }
        #[cfg(feature = "download")]
        Commands::DownloadPackage { 
            deployment_uuid,
            output,
            overwrite,
        } => {
            commands::download_package::execute(
                config,
                deployment_uuid,
                output,
                overwrite,
                cli.format,
            ).await?;
        }
        #[cfg(feature = "logs")]
        Commands::Logs {
            deployment_uuid,
            follow,
            tail,
        } => {
            commands::logs::execute(
                config,
                deployment_uuid,
                follow,
                tail,
                cli.format,
            ).await?;
        }
    }

    Ok(())
}

fn setup_logging(cli: &Cli) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let filter = if cli.quiet {
        EnvFilter::new("error")
    } else if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_thread_ids(false).with_file(false).with_line_number(false))
        .with(filter)
        .init();
    
    Ok(())
}