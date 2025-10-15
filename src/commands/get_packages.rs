use crate::{client::Client, Config, Result};
use colored::*;
use serde_json;
use tracing::info;

pub async fn execute(
    config: Config,
    app_uuids: Vec<String>,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;
    
    info!("Fetching packages for applications: {:?}", app_uuids);
    
    let packages = client.get_packages(&app_uuids).await?;
    
    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&packages)?;
            println!("{}", json_output);
        }
        _ => {
            println!("{}", "Packages:".bold().green());
            println!("Total packages: {}", packages.len().to_string().cyan());
            println!();
            
            if packages.is_empty() {
                println!("{}", "No packages found.".yellow());
            } else {
                for package in &packages {
                    println!("{} {}", "•".cyan(), package.name.bold());
                    println!("  {}: {}", "Version".dimmed(), package.version);
                    println!("  {}: {}", "ID".dimmed(), package.id);
                    
                    if !package.dependencies.is_empty() {
                        println!("  {}: {}", "Dependencies".dimmed(), package.dependencies.join(", "));
                    }
                    
                    println!("  {}: {}", "Created".dimmed(), package.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    println!();
                }
            }
        }
    }
    
    Ok(())
}