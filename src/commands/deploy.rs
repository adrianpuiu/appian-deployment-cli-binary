use crate::{client::Client, Config, Result};
use colored::*;

use tracing::info;

pub async fn execute(
    config: Config,
    package_zip_name: std::path::PathBuf,
    name: String,
    description: Option<String>,
    dry_run: bool,
    rollback_on_failure: bool,
    customization_file: Option<std::path::PathBuf>,
    admin_console_file: Option<std::path::PathBuf>,
    plugins_file: Option<std::path::PathBuf>,
    data_source: Option<String>,
    database_scripts: Option<Vec<std::path::PathBuf>>,
    format: Option<String>,
) -> Result<()> {
    if !package_zip_name.exists() {
        return Err(crate::error::CliError::InvalidArgument(format!(
            "Package file not found: {}",
            package_zip_name.display()
        )));
    }

    if dry_run {
        info!("Dry run mode - validating deployment parameters");
        println!("{}", "Dry run validation successful".green());
        println!("Package: {}", package_zip_name.display());
        println!("Deployment name: {}", name);
        println!("Description: {:?}", description);
        println!("Rollback on failure: {}", rollback_on_failure);
        if let Some(ref cf) = customization_file { println!("Customization file: {}", cf.display()); }
        if let Some(ref acf) = admin_console_file { println!("Admin Console settings: {}", acf.display()); }
        if let Some(ref pf) = plugins_file { println!("Plugins file: {}", pf.display()); }
        if let Some(ref ds) = data_source { println!("Data source: {}", ds); }
        if let Some(ref scripts) = database_scripts {
            println!("Database scripts (order):");
            for (i, s) in scripts.iter().enumerate() { println!("  {}. {}", i+1, s.display()); }
        }
        return Ok(());
    }

    let client = Client::new(config)?;
    
    info!("Starting deployment: {} with package {}", name, package_zip_name.display());
    println!("{}", "Starting deployment...".cyan());
    
    let package_name = package_zip_name
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::CliError::InvalidArgument(
            "Invalid package file name".to_string()
        ))?;

    // Validate optional files
    if let Some(ref path) = customization_file {
        if !path.exists() {
            return Err(crate::error::CliError::InvalidArgument(format!(
                "Customization file not found: {}",
                path.display()
            )));
        }
    }
    if let Some(ref path) = admin_console_file {
        if !path.exists() {
            return Err(crate::error::CliError::InvalidArgument(format!(
                "Admin Console settings file not found: {}",
                path.display()
            )));
        }
    }
    if let Some(ref path) = plugins_file {
        if !path.exists() {
            return Err(crate::error::CliError::InvalidArgument(format!(
                "Plugins file not found: {}",
                path.display()
            )));
        }
    }
    if let Some(ref scripts) = database_scripts {
        for s in scripts {
            if !s.exists() {
                return Err(crate::error::CliError::InvalidArgument(format!(
                    "Database script not found: {}",
                    s.display()
                )));
            }
        }
    }

    // Build JSON request object per API v2
    let mut db_scripts_json: Vec<crate::models::DatabaseScript> = vec![];
    if let Some(ref scripts) = database_scripts {
        for (i, path) in scripts.iter().enumerate() {
            let fname = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| crate::error::CliError::InvalidArgument("Invalid database script file name".to_string()))?;
            db_scripts_json.push(crate::models::DatabaseScript {
                file_name: fname.to_string(),
                order_id: (i + 1).to_string(),
            });
        }
    }

    let customization_file_name = customization_file.as_ref().and_then(|p| p.file_name().and_then(|n| n.to_str())).map(|s| s.to_string());
    let admin_console_file_name = admin_console_file.as_ref().and_then(|p| p.file_name().and_then(|n| n.to_str())).map(|s| s.to_string());
    let plugins_file_name = plugins_file.as_ref().and_then(|p| p.file_name().and_then(|n| n.to_str())).map(|s| s.to_string());

    let request_json = crate::models::DeploymentRequest {
        name: name.clone(),
        description: description.clone(),
        admin_console_settings_file_name: admin_console_file_name,
        package_file_name: Some(package_name.to_string()),
        customization_file_name: customization_file_name,
        plugins_file_name: plugins_file_name,
        data_source,
        database_scripts: if db_scripts_json.is_empty() { None } else { Some(db_scripts_json) },
    };

    let response = client
        .deploy_package_multipart(
            &request_json,
            &package_zip_name,
            customization_file.as_deref(),
            admin_console_file.as_deref(),
            plugins_file.as_deref(),
            database_scripts.as_ref().map(|v| v.as_slice()),
        )
        .await?;
    
    println!("{}", "Deployment initiated successfully".green());
    println!("Deployment UUID: {}", response.uuid.to_string().cyan());
    println!("Status URL: {}", response.url);
    println!("Status: {}", response.status.yellow());
    
    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&response)?;
            println!("{}", json_output);
        }
        _ => {
            println!("\n{}", "Deployment Details:".bold());
            println!("  {}: {}", "Deployment UUID".dimmed(), response.uuid);
            println!("  {}: {}", "Status".dimmed(), response.status);
            println!("  {}: {}", "Results URL".dimmed(), response.url);
            println!("\n{}", "Use 'status' or 'monitor' commands to track progress".dimmed());
        }
    }
    
    Ok(())
}