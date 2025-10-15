use crate::{client::Client, Config, Result};
use colored::*;
use tracing::info;

pub async fn execute(
    config: Config,
    deployment_uuid: String,
    follow: bool,
    tail: Option<usize>,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;
    
    info!("Fetching logs for deployment: {}", deployment_uuid);
    
    if follow {
        println!("{}", "Following logs (streaming)...".yellow());
        println!("{}", "Press Ctrl+C to stop".dimmed());
        println!();
        
        // Stream logs (simplified implementation)
        // In a real implementation, this would use WebSocket or SSE
        stream_logs(&client, &deployment_uuid, format.clone()).await?;
    } else {
        // Fetch logs once
        let response = client.get_deployment_logs(&deployment_uuid, tail).await?;
        
        match format.as_deref() {
            Some("json") => {
                let json_output = serde_json::to_string_pretty(&response)?;
                println!("{}", json_output);
            }
            _ => {
                println!("{}", format!("Logs for deployment: {}", deployment_uuid).bold().green());
                println!("Total entries: {}", response.total.to_string().cyan());
                println!();
                
                if response.logs.is_empty() {
                    println!("{}", "No logs found.".yellow());
                } else {
                    for log_entry in &response.logs {
                        let level_color = match log_entry.level {
            crate::models::LogLevel::Error => "red",
            crate::models::LogLevel::Warn => "yellow", 
            crate::models::LogLevel::Info => "green",
            crate::models::LogLevel::Debug => "blue",
        };
                        
                        println!(
                            "{} {} {}",
                            log_entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
                            format!("[{:5}]", format!("{:?}", log_entry.level)).color(level_color),
                            log_entry.message
                        );
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn stream_logs(
    client: &Client,
    deployment_uuid: &str,
    _format: Option<String>,
) -> Result<()> {
    // Simplified streaming implementation
    // In a real implementation, this would use WebSocket or Server-Sent Events
    let mut last_log_count = 0;
    
    loop {
        let response = client.get_deployment_logs(deployment_uuid, None).await?;
        
        // Print only new logs
        let new_logs = &response.logs[last_log_count..];
        
        for log_entry in new_logs {
            let level_color = match log_entry.level {
                crate::models::LogLevel::Error => "red",
                crate::models::LogLevel::Warn => "yellow", 
                crate::models::LogLevel::Info => "green",
                crate::models::LogLevel::Debug => "blue",
            };
            
            println!(
                "{} {} {}",
                log_entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed(),
                format!("[{:5}]", format!("{:?}", log_entry.level)).color(level_color),
                log_entry.message
            );
        }
        
        last_log_count = response.logs.len();
        
        // Check if deployment is complete
        let status_response = client.get_deployment_status(deployment_uuid).await?;
        if status_response.status.is_terminal() {
            println!("\n{}", "Deployment completed. Log streaming stopped.".green());
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    Ok(())
}