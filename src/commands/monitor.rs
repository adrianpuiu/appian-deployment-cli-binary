use crate::{client::Client, Config, Result};
use colored::*;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub async fn execute(
    config: Config,
    deployment_uuid: String,
    kind: Option<String>,
    interval: u64,
    timeout: u64,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;
    
    info!("Monitoring deployment: {} with interval {}s, timeout {}s", deployment_uuid, interval, timeout);
    
    // Determine if this is an export or deployment based on kind parameter
    let operation_type = match kind.as_deref() {
        Some("export") => "export",
        Some("deployment") => "deployment",
        _ => "deployment", // Default to deployment
    };

    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(timeout);
    let interval_duration = Duration::from_secs(interval);

    println!("{}", format!("Monitoring {} operation: {}", operation_type, deployment_uuid).bold().cyan());
    println!("{}", format!("Interval: {}s, Timeout: {}s", interval, timeout).dimmed());
    println!();

    loop {
        if start_time.elapsed() > timeout_duration {
            return Err(crate::error::CliError::Timeout(format!(
                "Operation {} did not complete within {} seconds",
                deployment_uuid, timeout
            )));
        }

        // Get current status
        let status = if operation_type == "export" {
            let export_response = client.get_export_status(&deployment_uuid).await?;
            format!("{:?}", export_response.status)
        } else {
            let deployment_response = client.get_deployment_status(&deployment_uuid).await?;
            format!("{:?}", deployment_response.status)
        };

        let elapsed = start_time.elapsed().as_secs();
        print!("\r{}", format!("[{:4}s] Status: {}", elapsed, status).dimmed());
        
        // Check if operation is complete
        let is_complete = if operation_type == "export" {
            let export_response = client.get_export_status(&deployment_uuid).await?;
            export_response.status.is_terminal()
        } else {
            let deployment_response = client.get_deployment_status(&deployment_uuid).await?;
            deployment_response.status.is_terminal()
        };

        if is_complete {
            println!(); // Move to new line
            println!("{}", format!("✓ Operation {} completed after {} seconds", deployment_uuid, elapsed).green());
            
            // Print final status
            if format.as_deref() == Some("json") {
                if operation_type == "export" {
                    let export_response = client.get_export_status(&deployment_uuid).await?;
                    let json_output = serde_json::to_string_pretty(&export_response)?;
                    println!("{}", json_output);
                } else {
                    let deployment_response = client.get_deployment_status(&deployment_uuid).await?;
                    let json_output = serde_json::to_string_pretty(&deployment_response)?;
                    println!("{}", json_output);
                }
            }
            
            break;
        }

        sleep(interval_duration).await;
    }
    
    Ok(())
}