use crate::{client::Client, Config, Result};
use colored::*;
use tracing::info;


pub async fn execute(
    config: Config,
    uuids: Vec<String>,
    export_type: String,
    name: Option<String>,
    description: Option<String>,
    dry_run: bool,
    format: Option<String>,
) -> Result<()> {
    if uuids.is_empty() {
        return Err(crate::error::CliError::InvalidArgument(
            "At least one --uuid must be provided".to_string()
        ));
    }

    let export_type = export_type.to_lowercase();
    if export_type != "package" && export_type != "application" {
        return Err(crate::error::CliError::InvalidArgument(
            "--export-type must be 'package' or 'application'".to_string()
        ));
    }

    if export_type == "package" && uuids.len() != 1 {
        return Err(crate::error::CliError::InvalidArgument(
            "For export-type 'package', exactly one --uuid is required".to_string()
        ));
    }

    if dry_run {
        info!("Dry run mode - validating export parameters");
        println!("{}", "Dry run validation successful".green());
        println!("Export type: {}", export_type);
        println!("UUIDs: {:?}", uuids);
        println!("Name: {:?}", name);
        println!("Description: {:?}", description);
        return Ok(());
    }

    let client = Client::new(config)?;
    
    info!("Starting export operation");
    println!("{}", "Starting export...".cyan());
    
    // Parse UUIDs
    let parsed_uuids: Vec<uuid::Uuid> = uuids
        .iter()
        .map(|u| uuid::Uuid::parse_str(u)
            .map_err(|e| crate::error::CliError::InvalidArgument(format!("Invalid UUID provided: {}", e)))
        )
        .collect::<std::result::Result<Vec<_>, crate::error::CliError>>()?;

    let request = crate::models::ExportRequest {
        uuids: parsed_uuids,
        export_type: export_type.clone(),
        name,
        description,
    };

    let response = client.export_multipart(&request).await?;
    
    println!("{}", "Export initiated successfully".green());
    println!("Export UUID: {}", response.uuid.to_string().cyan());
    println!("Status: {}", format!("{:?}", response.status).yellow());
    println!("Details URL: {}", response.url);
    
    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&response)?;
            println!("{}", json_output);
        }
        _ => {
            println!("\n{}", "Export Details:".bold());
            println!("  {}: {}", "Export UUID".dimmed(), response.uuid);
            println!("  {}: {:?}", "Status".dimmed(), response.status);
            println!("  {}: {}", "Details URL".dimmed(), response.url);
        }
    }
    
    Ok(())
}