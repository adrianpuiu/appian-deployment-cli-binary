use crate::config::Config;
use crate::error::{CliError, Result};
use crate::models::*;
use anyhow::Context;
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, error, info};


pub struct Client {
    http_client: HttpClient,
    config: Config,
}

impl Client {
    pub fn new(config: Config) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Client {
            http_client,
            config,
        })
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = self.config.get_api_url(path);
        debug!("Building {} request to {}", method, url);
        
        self.http_client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("appian-api-key", &self.config.api_key)
            .header("Accept", "application/json")
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let url = response.url().to_string();
        
        debug!("Response status: {} from {}", status, url);

        if status.is_success() {
            let json_result = response.json::<T>().await;
            json_result.map_err(|e| CliError::Api {
                status: 500,
                message: format!("Failed to parse response JSON: {}", e),
            })
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("API error {}: {}", status, error_text);
            
            match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    Err(CliError::Authentication(format!("Authentication failed: {}", error_text)))
                }
                StatusCode::NOT_FOUND => {
                    Err(CliError::Api {
                        status: status.as_u16(),
                        message: format!("Resource not found: {}", error_text),
                    })
                }
                StatusCode::REQUEST_TIMEOUT => {
                    Err(CliError::Timeout(format!("Request timeout: {}", error_text)))
                }
                _ if status.is_server_error() => {
                    Err(CliError::Api {
                        status: status.as_u16(),
                        message: format!("Server error: {}", error_text),
                    })
                }
                _ => {
                    Err(CliError::Api {
                        status: status.as_u16(),
                        message: error_text,
                    })
                }
            }
        }
    }

    #[cfg(feature = "get_packages")]
    pub async fn get_packages(&self, app_uuids: &[String]) -> Result<Vec<Package>> {
        info!("Fetching packages for applications: {:?}", app_uuids);
        
        let mut request = self.build_request(reqwest::Method::GET, "/deployment/v2/packages");
        
        if !app_uuids.is_empty() {
            let uuids_param = app_uuids.join(",");
            request = request.query(&[("app_uuids", uuids_param)]);
        }

        let response = request.send().await.context("Failed to send request")?;
        let response: PackageListResponse = self.handle_response(response).await?;
        Ok(response.packages)
    }

    #[cfg(feature = "export")]
    pub async fn export_multipart(&self, request: &ExportRequest) -> Result<ExportResponse> {
        use reqwest::multipart::{Form, Part};

        info!("Initiating export: exportType={}, uuids={:?}", request.export_type, request.uuids);

        // Build JSON part
        let json_str = serde_json::to_string(request)
            .context("Failed to serialize export request JSON")?;
        let json_part = Part::text(json_str)
            .mime_str("application/json")
            .ok();

        let mut form = Form::new();
        if let Some(part) = json_part {
            form = form.part("json", part);
        }

        let response = self
            .build_request(reqwest::Method::POST, "/suite/deployment-management/v2/deployments")
            .header("Action-Type", "export")
            .multipart(form)
            .send()
            .await
            .context("Failed to send export request")?;

        self.handle_response(response).await
    }

    #[cfg(feature = "deploy")]
    #[allow(dead_code)]
    pub async fn deploy_package(
        &self,
        package_zip_name: &str,
        name: &str,
        description: Option<&str>,
        _rollback_on_failure: bool,
    ) -> Result<DeployResponse> {
        info!("Deploying package: {}, name: {}", package_zip_name, name);
        
        let request_body = serde_json::json!({
            "name": name,
            "description": description,
            "packageFileName": package_zip_name,
        });

        let response = self
            .build_request(reqwest::Method::POST, "/deployment/v2/deployments")
            .header("Action-Type", "import")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send deploy request")?;

        self.handle_response(response).await
    }

    #[cfg(feature = "deploy")]
    pub async fn deploy_package_multipart(
        &self,
        request: &DeploymentRequest,
        package_file: &std::path::Path,
        customization_file: Option<&std::path::Path>,
        admin_console_file: Option<&std::path::Path>,
        plugins_file: Option<&std::path::Path>,
        database_scripts: Option<&[std::path::PathBuf]>,
    ) -> Result<DeployResponse> {
        use reqwest::multipart::{Form, Part};

        info!("Deploying (multipart) package: {}", request.name);

        // Build JSON part
        let json_str = serde_json::to_string(request)
            .context("Failed to serialize deployment request JSON")?;
        let json_part = Part::text(json_str)
            .mime_str("application/json")
            .ok();

        let mut form = Form::new();
        if let Some(part) = json_part {
            form = form.part("json", part);
        }

        // Attach files
        let pkg_name = package_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("package.zip")
            .to_string();
        let pkg_bytes = std::fs::read(package_file)
            .context("Failed to read package file for upload")?;
        let pkg_part = Part::bytes(pkg_bytes).file_name(pkg_name);
        form = form.part("packageFileName", pkg_part);

        if let Some(path) = customization_file {
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("customization.properties").to_string();
            let bytes = std::fs::read(path).context("Failed to read customization file for upload")?;
            let part = Part::bytes(bytes).file_name(fname);
            form = form.part("customizationFileName", part);
        }

        if let Some(path) = admin_console_file {
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("admin-console-settings.zip").to_string();
            let bytes = std::fs::read(path).context("Failed to read Admin Console settings file for upload")?;
            let part = Part::bytes(bytes).file_name(fname);
            form = form.part("adminConsoleSettingsFileName", part);
        }

        if let Some(path) = plugins_file {
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("plugins.zip").to_string();
            let bytes = std::fs::read(path).context("Failed to read plugins file for upload")?;
            let part = Part::bytes(bytes).file_name(fname);
            form = form.part("pluginsFileName", part);
        }

        if let Some(scripts) = database_scripts {
            for (idx, script_path) in scripts.iter().enumerate() {
                let key = format!("databaseScript{}", idx + 1);
                let fname = script_path.file_name().and_then(|n| n.to_str()).unwrap_or("script.sql").to_string();
                let bytes = std::fs::read(script_path).context("Failed to read database script file for upload")?;
                let part = Part::bytes(bytes).file_name(fname);
                form = form.part(key, part);
            }
        }

        let response = self
            .build_request(reqwest::Method::POST, "/deployment/v2/deployments")
            .header("Action-Type", "import")
            .multipart(form)
            .send()
            .await
            .context("Failed to send multipart deploy request")?;

        self.handle_response(response).await
    }

    #[cfg(any(feature = "status", feature = "monitor"))]
    pub async fn get_deployment_status(&self, deployment_uuid: &str) -> Result<DeploymentStatusResponse> {
        debug!("Getting deployment status for: {}", deployment_uuid);
        
        let path = format!("/deployment/v2/deployments/{}", deployment_uuid);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .context("Failed to get deployment status")?;

        self.handle_response(response).await
    }

    #[cfg(any(feature = "export", feature = "monitor"))]
    pub async fn get_export_status(&self, export_uuid: &str) -> Result<ExportResponse> {
        debug!("Getting export status for: {}", export_uuid);
        
        let path = format!("/suite/deployment-management/v2/deployments/{}", export_uuid);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .context("Failed to get export status")?;

        self.handle_response(response).await
    }

    #[cfg(any(feature = "status", feature = "monitor", feature = "download", feature = "logs", feature = "deploy"))]
    pub async fn get_deployment_results(&self, deployment_uuid: &str) -> Result<crate::models::DeploymentResults> {
        debug!("Getting deployment results for: {}", deployment_uuid);

        let path = format!("/suite/deployment-management/v2/deployments/{}", deployment_uuid);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .context("Failed to get deployment results")?;

        self.handle_response(response).await
    }

    #[cfg(feature = "logs")]
    pub async fn get_deployment_logs(
        &self,
        deployment_id: &str,
        tail: Option<usize>,
    ) -> Result<LogsResponse> {
        debug!("Getting deployment logs for: {}", deployment_id);
        
        let path = format!("/deployment/v2/deployments/{}/log", deployment_id);
        let mut request = self.build_request(reqwest::Method::GET, &path);
        
        if let Some(tail_param) = tail {
            request = request.query(&[("tail", tail_param.to_string())]);
        }

        let response = request.send().await.context("Failed to get deployment logs")?;
        self.handle_response(response).await
    }

    #[cfg(feature = "download")]
    pub async fn download_artifact(&self, artifact_id: &str) -> Result<Vec<u8>> {
        info!("Downloading artifact: {}", artifact_id);
        
        let path = format!("/deployment/v2/artifacts/{}", artifact_id);
        let request = self.build_request(reqwest::Method::GET, &path);
        let response = request.send().await.context("Failed to download artifact")?;
        
        if !response.status().is_success() {
            return Err(CliError::Api {
                status: response.status().as_u16(),
                message: format!("Failed to download artifact: {}", response.status()),
            });
        }
        
        let bytes = response.bytes().await.context("Failed to read response bytes")?;
        info!("Artifact downloaded successfully: {} bytes", bytes.len());
        Ok(bytes.to_vec())
    }

    #[cfg(feature = "validate")]
    #[allow(dead_code)]
    pub async fn validate_package(&self, package_path: &std::path::Path) -> Result<ValidationResult> {
        info!("Validating package: {}", package_path.display());
        
        if !package_path.exists() {
            return Err(CliError::FileSystem(format!("Package not found: {}", package_path.display())));
        }

        // For now, return a basic validation result
        // In a real implementation, this would call the actual validation API
        Ok(ValidationResult {
            is_valid: true,
            total_size: 0,
            violations: vec![],
        })
    }

    #[cfg(feature = "validate")]
    pub async fn inspect_package(
        &self,
        request: &InspectionRequest,
        package_file: &std::path::Path,
        customization_file: Option<&std::path::Path>,
        admin_console_file: Option<&std::path::Path>,
    ) -> Result<InspectionResponse> {
        use reqwest::multipart::{Form, Part};

        info!("Initiating inspection for package: {}", request.package_file_name);

        // Build JSON part
        let json_str = serde_json::to_string(request)
            .context("Failed to serialize inspection request JSON")?;
        let json_part = Part::text(json_str)
            .mime_str("application/json")
            .ok();

        let mut form = Form::new();
        if let Some(part) = json_part {
            form = form.part("json", part);
        }

        // Attach files with arbitrary keys as allowed by API
        let pkg_name = package_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("package.zip")
            .to_string();
        let pkg_bytes = std::fs::read(package_file)
            .context("Failed to read package file for upload")?;
        let pkg_part = Part::bytes(pkg_bytes).file_name(pkg_name);
        form = form.part("zipFile", pkg_part);

        if let Some(path) = customization_file {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("customization.properties")
                .to_string();
            let bytes = std::fs::read(path)
                .context("Failed to read customization file for upload")?;
            let part = Part::bytes(bytes).file_name(fname);
            form = form.part("ICF", part);
        }

        if let Some(path) = admin_console_file {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("admin-console-settings.zip")
                .to_string();
            let bytes = std::fs::read(path)
                .context("Failed to read Admin Console settings file for upload")?;
            let part = Part::bytes(bytes).file_name(fname);
            form = form.part("adminConsole", part);
        }

        let response = self
            .build_request(reqwest::Method::POST, "/suite/deployment-management/v2/inspections")
            .multipart(form)
            .send()
            .await
            .context("Failed to send inspection request")?;

        self.handle_response(response).await
    }

    #[cfg(feature = "validate")]
    pub async fn get_inspection_results(&self, inspection_uuid: &str) -> Result<InspectionResults> {
        debug!("Getting inspection results for: {}", inspection_uuid);

        let path = format!("/suite/deployment-management/v2/inspections/{}", inspection_uuid);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .await
            .context("Failed to get inspection results")?;

        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = Config {
            base_url: "https://test.example.com".to_string(),
            api_key: "test-key".to_string(),
            timeout_seconds: 30,
            logging: crate::config::LoggingConfig::default(),
            download: crate::config::DownloadConfig::default(),
            monitor: crate::config::MonitorConfig::default(),
        };

        let client = Client::new(config).unwrap();
        assert_eq!(client.config.base_url, "https://test.example.com");
    }
}