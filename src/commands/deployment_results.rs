use crate::{client::Client, Config, Result};
use colored::*;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub async fn execute(
    config: Config,
    deployment_uuid: String,
    format: Option<String>,
    poll: bool,
) -> Result<()> {
    let client = Client::new(config)?;

    info!("Getting deployment results for: {}", deployment_uuid);

    if poll {
        println!("{}", "Polling until terminal status...".bold().cyan());
        let interval = Duration::from_secs(10);
        let timeout = Duration::from_secs(600); // 10 minutes
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(crate::error::CliError::Timeout(format!(
                    "Deployment {} did not reach a terminal status within {} seconds",
                    deployment_uuid,
                    timeout.as_secs()
                )));
            }

            let status = client.get_deployment_status(&deployment_uuid).await?;
            if status.status.is_terminal() {
                println!("{} {:?}", "Terminal status:".green().bold(), status.status);
                break;
            } else {
                println!("Status: {:?}{}", status.status, " (waiting)".dimmed());
            }

            sleep(interval).await;
        }
    }

    let results = client.get_deployment_results(&deployment_uuid).await?;

    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&results)?;
            println!("{}", json_output);
        }
        _ => {
            println!("{}", "Deployment Results:".bold().green());
            match results {
                crate::models::DeploymentResults::Import(import) => {
                    println!("  {}: {:?}", "Status".dimmed(), import.status);
                    println!("  {}: {}", "Deployment Log".dimmed(), import.summary.deployment_log_url);
                    println!("  {}:", "Admin Console Settings".dimmed());
                    println!(
                        "    total={}, imported={}, failed={}, skipped={}",
                        import.summary.admin_console_settings.total,
                        import.summary.admin_console_settings.imported,
                        import.summary.admin_console_settings.failed,
                        import.summary.admin_console_settings.skipped
                    );
                    println!("  {}:", "Objects".dimmed());
                    println!(
                        "    total={}, imported={}, failed={}, skipped={}",
                        import.summary.objects.total,
                        import.summary.objects.imported,
                        import.summary.objects.failed,
                        import.summary.objects.skipped
                    );
                    println!("  {}:", "Plugins".dimmed());
                    println!(
                        "    total={}, imported={}, skipped={}",
                        import.summary.plugins.total,
                        import.summary.plugins.imported,
                        import.summary.plugins.skipped
                    );
                    println!("  {}: {}", "Database Scripts".dimmed(), import.summary.database_scripts);
                }
                crate::models::DeploymentResults::Export(export) => {
                    println!("  {}: {:?}", "Status".dimmed(), export.status);
                    if let Some(url) = &export.deployment_log_url {
                        println!("  {}: {}", "Deployment Log".dimmed(), url);
                    }
                    if let Some(pkg) = &export.package_zip {
                        println!("  {}: {}", "Package Zip".dimmed(), pkg);
                    }
                    if let Some(plugins) = &export.plugins_zip {
                        println!("  {}: {}", "Plugins Zip".dimmed(), plugins);
                    }
                    if let Some(cf) = &export.customization_file {
                        println!("  {}: {}", "Customization File".dimmed(), cf);
                    }
                    if let Some(cft) = &export.customization_file_template {
                        println!("  {}: {}", "Customization File Template".dimmed(), cft);
                    }
                    if !export.database_scripts.is_empty() {
                        println!("  {}:", "Database Scripts".dimmed());
                        for s in &export.database_scripts {
                            println!(
                                "    • {} (order {}): {}",
                                s.file_name,
                                s.order_id,
                                s.url
                            );
                        }
                    }
                    if let Some(ds) = &export.data_source {
                        println!("  {}: {}", "Data Source".dimmed(), ds);
                    }
                }
            }
        }
    }

    Ok(())
}