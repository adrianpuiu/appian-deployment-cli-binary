use crate::{client::Client, Config, Result};
use colored::*;
use tracing::info;

pub async fn execute(
    config: Config,
    uuid: String,
    format: Option<String>,
) -> Result<()> {
    let client = Client::new(config)?;

    info!("Getting inspection results for: {}", uuid);
    println!("{}", format!("Fetching inspection results for {}...", uuid).cyan());

    let results = client.get_inspection_results(&uuid).await?;

    match format.as_deref() {
        Some("json") => {
            let json_output = serde_json::to_string_pretty(&results)?;
            println!("{}", json_output);
        }
        _ => {
            println!("{}", "Inspection Results:".bold().green());
            println!("  {}: {:?}", "Status".dimmed(), results.status);

            let admin = &results.summary.admin_console_settings_expected;
            println!("{}", "  Admin Console Settings:".bold());
            println!("    {}: {}", "Total".dimmed(), admin.total);
            println!("    {}: {}", "Imported".dimmed(), admin.imported);
            println!("    {}: {}", "Failed".dimmed(), admin.failed);
            println!("    {}: {}", "Skipped".dimmed(), admin.skipped);

            let objs = &results.summary.objects_expected;
            println!("{}", "  Package Objects:".bold());
            println!("    {}: {}", "Total".dimmed(), objs.total);
            println!("    {}: {}", "Imported".dimmed(), objs.imported);
            println!("    {}: {}", "Failed".dimmed(), objs.failed);
            println!("    {}: {}", "Skipped".dimmed(), objs.skipped);

            let probs = &results.summary.problems;
            println!("{}", "  Problems:".bold());
            println!("    {}: {}", "Total Errors".dimmed(), probs.total_errors);
            println!("    {}: {}", "Total Warnings".dimmed(), probs.total_warnings);

            if !probs.errors.is_empty() {
                println!("{}", "    Errors:".bold());
                for e in &probs.errors {
                    println!("      • {}", e.object_name.bold());
                    println!("        {}: {}", "UUID".dimmed(), e.object_uuid);
                    println!("        {}: {}", "Message".dimmed(), e.error_message);
                }
            }

            if !probs.warnings.is_empty() {
                println!("{}", "    Warnings:".bold());
                for w in &probs.warnings {
                    println!("      • {}", w.object_name.bold());
                    println!("        {}: {}", "UUID".dimmed(), w.object_uuid);
                    println!("        {}: {}", "Message".dimmed(), w.warning_message);
                }
            }
        }
    }

    Ok(())
}