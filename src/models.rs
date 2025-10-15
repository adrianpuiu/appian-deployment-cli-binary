use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageListResponse {
    pub packages: Vec<Package>,
    pub total: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    #[serde(rename = "uuids")]
    pub uuids: Vec<Uuid>,
    #[serde(rename = "exportType")]
    pub export_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub uuid: Uuid,
    pub url: String,
    pub status: ExportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportStatus {
    InProgress,
    Completed,
    CompletedWithErrors,
    // v2 API may return this more specific variant
    CompletedWithExportErrors,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseScript {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "adminConsoleSettingsFileName", skip_serializing_if = "Option::is_none")]
    pub admin_console_settings_file_name: Option<String>,
    #[serde(rename = "packageFileName", skip_serializing_if = "Option::is_none")]
    pub package_file_name: Option<String>,
    #[serde(rename = "customizationFileName", skip_serializing_if = "Option::is_none")]
    pub customization_file_name: Option<String>,
    #[serde(rename = "pluginsFileName", skip_serializing_if = "Option::is_none")]
    pub plugins_file_name: Option<String>,
    #[serde(rename = "dataSource", skip_serializing_if = "Option::is_none")]
    pub data_source: Option<String>,
    #[serde(rename = "databaseScripts", skip_serializing_if = "Option::is_none")]
    pub database_scripts: Option<Vec<DatabaseScript>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResponse {
    pub uuid: Uuid,
    pub url: String,
    pub status: String,
}

// Deployment results (API: GET /deployments/<uuid>)
// This endpoint returns different shapes depending on whether the operation was an import or export.
// We model both and use an untagged enum to deserialize accordingly.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImportDeploymentStatus {
    InProgress,
    Completed,
    CompletedWithImportErrors,
    CompletedWithPublishErrors,
    Failed,
    PendingReview,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConsoleSettingsSummary {
    pub total: u32,
    pub imported: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsSummary {
    pub total: u32,
    pub imported: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsSummary {
    pub total: u32,
    pub imported: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    #[serde(rename = "databaseScripts")]
    pub database_scripts: u32,
    #[serde(rename = "adminConsoleSettings")]
    pub admin_console_settings: AdminConsoleSettingsSummary,
    #[serde(rename = "plugins")]
    pub plugins: PluginsSummary,
    #[serde(rename = "objects")]
    pub objects: ObjectsSummary,
    #[serde(rename = "deploymentLogUrl")]
    pub deployment_log_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeploymentResults {
    pub summary: ImportSummary,
    pub status: ImportDeploymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDatabaseScript {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "orderId")]
    pub order_id: i32,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportDeploymentResults {
    #[serde(rename = "packageZip")]
    pub package_zip: Option<String>,
    #[serde(rename = "dataSource")]
    pub data_source: Option<String>,
    #[serde(rename = "databaseScripts")]
    pub database_scripts: Vec<ExportedDatabaseScript>,
    #[serde(rename = "pluginsZip")]
    pub plugins_zip: Option<String>,
    #[serde(rename = "customizationFile")]
    pub customization_file: Option<String>,
    #[serde(rename = "customizationFileTemplate")]
    pub customization_file_template: Option<String>,
    #[serde(rename = "deploymentLogUrl")]
    pub deployment_log_url: Option<String>,
    pub status: ExportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeploymentResults {
    Import(ImportDeploymentResults),
    Export(ExportDeploymentResults),
}

// Inspection models (API: POST /inspections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRequest {
    #[serde(rename = "adminConsoleSettingsFileName", skip_serializing_if = "Option::is_none")]
    pub admin_console_settings_file_name: Option<String>,
    #[serde(rename = "packageFileName")]
    pub package_file_name: String,
    #[serde(rename = "customizationFileName", skip_serializing_if = "Option::is_none")]
    pub customization_file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResponse {
    pub uuid: Uuid,
    pub url: String,
}

// Inspection results (API: GET /inspections/<uuid>)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InspectionOperationStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCountSummary {
    pub total: u32,
    pub imported: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionErrorEntry {
    #[serde(rename = "errorMessage")]
    pub error_message: String,
    #[serde(rename = "objectName")]
    pub object_name: String,
    #[serde(rename = "objectUuid")]
    pub object_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionWarningEntry {
    #[serde(rename = "warningMessage")]
    pub warning_message: String,
    #[serde(rename = "objectName")]
    pub object_name: String,
    #[serde(rename = "objectUuid")]
    pub object_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionProblemsSummary {
    #[serde(rename = "totalErrors")]
    pub total_errors: u32,
    #[serde(rename = "totalWarnings")]
    pub total_warnings: u32,
    #[serde(default)]
    pub errors: Vec<InspectionErrorEntry>,
    #[serde(default)]
    pub warnings: Vec<InspectionWarningEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionSummary {
    #[serde(rename = "adminConsoleSettingsExpected")]
    pub admin_console_settings_expected: InspectionCountSummary,
    #[serde(rename = "objectsExpected")]
    pub objects_expected: InspectionCountSummary,
    pub problems: InspectionProblemsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResults {
    pub summary: InspectionSummary,
    pub status: InspectionOperationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeploymentStatus {
    InProgress,
    Succeeded,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatusResponse {
    #[serde(rename = "deploymentId")]
    pub deployment_id: Uuid,
    pub status: DeploymentStatus,
    #[serde(rename = "currentStep")]
    pub current_step: Option<String>,
    #[serde(rename = "resultLinks")]
    pub result_links: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
    pub total: i32,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub total_size: u64,
    pub violations: Vec<ValidationViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub severity: ViolationSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatus {
    pub id: Uuid,
    pub status: String,
    #[serde(rename = "operationType")]
    pub operation_type: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl DeploymentStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, DeploymentStatus::Succeeded | DeploymentStatus::Failed | DeploymentStatus::RolledBack)
    }
}

impl ExportStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExportStatus::Completed
                | ExportStatus::CompletedWithErrors
                | ExportStatus::CompletedWithExportErrors
                | ExportStatus::Failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_terminal() {
        assert!(!DeploymentStatus::InProgress.is_terminal());
        assert!(DeploymentStatus::Succeeded.is_terminal());
        assert!(DeploymentStatus::Failed.is_terminal());
        assert!(DeploymentStatus::RolledBack.is_terminal());

        assert!(!ExportStatus::InProgress.is_terminal());
        assert!(ExportStatus::Completed.is_terminal());
        assert!(ExportStatus::CompletedWithErrors.is_terminal());
        assert!(ExportStatus::CompletedWithExportErrors.is_terminal());
        assert!(ExportStatus::Failed.is_terminal());
    }
}