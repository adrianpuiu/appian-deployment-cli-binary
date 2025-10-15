use anyhow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Network error: {0}")]
    #[allow(dead_code)]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Configuration(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Validation error: {0}")]
    #[allow(dead_code)]
    Validation(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Deployment failed: {0}")]
    #[allow(dead_code)]
    DeploymentFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Unknown error: {0}")]
    #[allow(dead_code)]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, CliError>;

impl CliError {
    #[allow(dead_code)]
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Network(_) => 3,
            CliError::Authentication(_) => 4,
            CliError::Configuration(_) => 2,
            CliError::Api { status, .. } => {
                if *status >= 500 {
                    5
                } else {
                    1
                }
            }
            CliError::FileSystem(_) => 1,
            CliError::Validation(_) => 2,
            CliError::Timeout(_) => 6,
            CliError::DeploymentFailed(_) => 5,
            CliError::InvalidArgument(_) => 2,
            CliError::Io(_) => 1,
            CliError::Serialization(_) => 2,
            CliError::UrlParse(_) => 2,
            CliError::Unknown(_) => 1,
            CliError::Anyhow(_) => 1,
        }
    }
}

#[allow(dead_code)]
pub fn redact_sensitive_info(input: &str) -> String {
    let mut result = input.to_string();
    
    // Redact API keys (common patterns)
    result = regex::Regex::new(r#"(?i)(api[_-]?key|apikey|token)["']?\s*[:=]\s*["']?[a-zA-Z0-9_-]{20,}["']?"#)
        .unwrap()
        .replace_all(&result, "$1=***REDACTED***")
        .to_string();
    
    // Redact URLs with embedded credentials
    result = regex::Regex::new(r"(?i)(https?://)[a-zA-Z0-9_-]+:[^@]+@")
        .unwrap()
        .replace_all(&result, "${1}***:***@")
        .to_string();
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(CliError::Network("test".to_string()).exit_code(), 3);
        assert_eq!(CliError::Authentication("test".to_string()).exit_code(), 4);
        assert_eq!(CliError::Configuration("test".to_string()).exit_code(), 2);
        assert_eq!(CliError::Api { status: 500, message: "test".to_string() }.exit_code(), 5);
        assert_eq!(CliError::Api { status: 400, message: "test".to_string() }.exit_code(), 1);
        assert_eq!(CliError::Timeout("test".to_string()).exit_code(), 6);
    }

    #[test]
    fn test_redact_sensitive_info() {
        let input = r#"{
            "api_key": "sk-1234567890abcdef1234567890abcdef",
            "url": "https://user:password@example.com",
            "normal": "value"
        }"#;
        
        let redacted = redact_sensitive_info(input);
        assert!(redacted.contains("***REDACTED***"));
        assert!(redacted.contains("***:***@"));
        assert!(redacted.contains("\"normal\": \"value\""));
    }
}