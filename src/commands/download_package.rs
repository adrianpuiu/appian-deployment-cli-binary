use crate::{client::Client, Config, Result};
use colored::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

pub async fn execute(
    config: Config,
    deployment_uuid: String,
    output: Option<PathBuf>,
    overwrite: bool,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;
    
    info!("Downloading package: {}", deployment_uuid);
    
    // Determine output path
    let output_path = if let Some(path) = output {
        path
    } else {
        // Default to current directory with deployment UUID as filename
        PathBuf::from(format!("{}.zip", deployment_uuid))
    };

    // Check if file exists and overwrite is false
    if output_path.exists() && !overwrite {
        return Err(crate::error::CliError::FileSystem(format!(
            "File already exists: {}. Use --overwrite to replace.",
            output_path.display()
        )));
    }

    println!("{}", format!("Downloading package {}...", deployment_uuid).cyan());
    
    // Download the package
    let package_data = client.download_artifact(&deployment_uuid).await?;
    
    // Write to file
    let mut file = File::create(&output_path).map_err(|e| {
        crate::error::CliError::FileSystem(format!("Failed to create file: {}", e))
    })?;
    
    file.write_all(&package_data).map_err(|e| {
        crate::error::CliError::FileSystem(format!("Failed to write file: {}", e))
    })?;
    
    println!("{}", format!("✓ Package downloaded to: {}", output_path.display()).green());
    
    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::json!({
                "deployment_uuid": deployment_uuid,
                "output_path": output_path.to_string_lossy(),
                "size_bytes": package_data.len(),
                "success": true
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            println!("Package size: {} bytes", package_data.len().to_string().cyan());
        }
    }
    
    Ok(())
}