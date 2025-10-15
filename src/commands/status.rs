use crate::{client::Client, Config, Result};
use colored::*;
use tracing::info;

pub async fn execute(
    config: Config,
    deployment_uuid: String,
    kind: Option<String>,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;
    
    info!("Getting status for deployment: {}", deployment_uuid);
    
    // Determine if this is an export or deployment based on kind parameter or auto-detection
    let operation_type = match kind.as_deref() {
        Some("export") => "export",
        Some("deployment") => "deployment",
        _ => {
            // Auto-detect based on UUID format or other heuristics
            // For now, default to deployment
            "deployment"
        }
    };

    let response = if operation_type == "export" {
        // Get export status
        let export_response = client.get_export_status(&deployment_uuid).await?;
        
        match format.as_deref() {
            Some("json") => {
                let json_output = serde_json::to_string_pretty(&export_response)?;
                println!("{}", json_output);
            }
            _ => {
                println!("{}", "Export Status:".bold().green());
                println!("  {}: {}", "Export UUID".dimmed(), export_response.uuid);
                println!("  {}: {:?}", "Status".dimmed(), export_response.status);
                println!("  {}: {}", "Details URL".dimmed(), export_response.url);
                
                if export_response.status.is_terminal() {
                    println!("\n{}", "Operation completed".green());
                } else {
                    println!("\n{}", "Operation in progress...".yellow());
                }
            }
        }
        
        return Ok(());
    } else {
        // Get deployment status
        client.get_deployment_status(&deployment_uuid).await?
    };

    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&response)?;
            println!("{}", json_output);
        }
        _ => {
            println!("{}", "Deployment Status:".bold().green());
            println!("  {}: {}", "Deployment ID".dimmed(), response.deployment_id);
            println!("  {}: {:?}", "Status".dimmed(), response.status);
            
            if let Some(current_step) = &response.current_step {
                println!("  {}: {}", "Current Step".dimmed(), current_step);
            }
            
            if !response.result_links.is_empty() {
                println!("  {}:", "Result Links".dimmed());
                for link in &response.result_links {
                    println!("    • {}", link);
                }
            }
            
            println!("  {}: {}", "Created".dimmed(), response.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("  {}: {}", "Updated".dimmed(), response.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
            
            if response.status.is_terminal() {
                println!("\n{}", "Deployment completed".green());
            } else {
                println!("\n{}", "Deployment in progress...".yellow());
            }
        }
    }
    
    Ok(())
}