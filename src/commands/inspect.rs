use crate::{client::Client, Config, Result};
use colored::*;
use std::path::PathBuf;
use tracing::info;

pub async fn execute(
    config: Config,
    package_path: PathBuf,
    customization_file: Option<PathBuf>,
    admin_console_file: Option<PathBuf>,
    format: Option<String>,
) -> Result<()> {
    if !package_path.exists() {
        return Err(crate::error::CliError::FileSystem(format!(
            "Package file not found: {}",
            package_path.display()
        )));
    }
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

    // Perform a quick local validation to improve UX and use helper functions
    let validation = validate_package_file(&package_path).await?;
    if !validation.is_valid {
        return Err(crate::error::CliError::InvalidArgument(
            "Package file is invalid".to_string(),
        ));
    }

    let client = Client::new(config)?;
    info!("Inspecting package via API: {}", package_path.display());
    println!("{}", format!("Inspecting package: {}", package_path.display()).cyan());
    println!(
        "{} {}",
        "Package size:".dimmed(),
        format_bytes(validation.total_size).cyan()
    );
    if !validation.violations.is_empty() {
        // Show non-error validations as hints before sending to API
        let warnings: Vec<_> = validation
            .violations
            .iter()
            .filter(|v| matches!(v.severity, crate::models::ViolationSeverity::Warning))
            .collect();
        if !warnings.is_empty() {
            println!("{}", "Validation warnings:".yellow());
            for w in warnings {
                println!("  - {} ({})", w.message, w.code);
            }
        }
    }

    // Build InspectionRequest based on provided file names
    let package_file_name = package_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::error::CliError::InvalidArgument(
            "Invalid package file name".to_string()
        ))?;

    let customization_file_name = customization_file
        .as_ref()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()))
        .map(|s| s.to_string());
    let admin_console_file_name = admin_console_file
        .as_ref()
        .and_then(|p| p.file_name().and_then(|n| n.to_str()))
        .map(|s| s.to_string());

    let request_json = crate::models::InspectionRequest {
        admin_console_settings_file_name: admin_console_file_name,
        package_file_name: package_file_name.to_string(),
        customization_file_name: customization_file_name,
    };

    let response = client
        .inspect_package(
            &request_json,
            &package_path,
            customization_file.as_deref(),
            admin_console_file.as_deref(),
        )
        .await?;

    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&response)?;
            println!("{}", json_output);
        }
        _ => {
            println!("{}", "Inspection initiated:".bold().green());
            println!("  {}: {}", "UUID".dimmed(), response.uuid);
            println!("  {}: {}", "URL".dimmed(), response.url);
        }
    }

    Ok(())
}

async fn validate_package_file(package_path: &PathBuf) -> Result<crate::models::ValidationResult> {
    use std::fs;
    
    // Basic file validation
    let metadata = fs::metadata(package_path).map_err(|e| {
        crate::error::CliError::FileSystem(format!("Failed to read package file: {}", e))
    })?;
    
    let mut violations = Vec::new();
    let mut is_valid = true;
    
    // Check file size (basic validation)
    if metadata.len() == 0 {
        violations.push(crate::models::ValidationViolation {
            severity: crate::models::ViolationSeverity::Error,
            message: "Package file is empty".to_string(),
            code: "EMPTY_FILE".to_string(),
        });
        is_valid = false;
    }
    
    if metadata.len() > 100 * 1024 * 1024 { // 100MB limit
        violations.push(crate::models::ValidationViolation {
            severity: crate::models::ViolationSeverity::Warning,
            message: "Package file is very large (>100MB)".to_string(),
            code: "LARGE_FILE".to_string(),
        });
    }
    
    // Check file extension
    if let Some(ext) = package_path.extension() {
        if ext != "zip" {
            violations.push(crate::models::ValidationViolation {
                severity: crate::models::ViolationSeverity::Warning,
                message: "Package file should have .zip extension".to_string(),
                code: "WRONG_EXTENSION".to_string(),
            });
        }
    }
    
    Ok(crate::models::ValidationResult {
        is_valid,
        total_size: metadata.len(),
        violations,
    })
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}