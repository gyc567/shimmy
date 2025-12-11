//! Shimmy Vision Licensing Module
//!
//! Keygen-based licensing for vision features.
//! Handles license validation, caching, and usage metering.

#[cfg(feature = "vision")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "vision")]
use std::collections::HashMap;
#[cfg(feature = "vision")]
use std::path::PathBuf;
#[cfg(feature = "vision")]
use std::sync::Arc;
#[cfg(feature = "vision")]
use tokio::sync::RwLock;

/// License validation response from Keygen
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidation {
    pub valid: bool,
    pub entitlements: HashMap<String, serde_json::Value>,
    pub expires_at: Option<String>,
    pub meta: HashMap<String, serde_json::Value>,
}

/// Cached license information
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicense {
    pub key: String,
    pub validation: LicenseValidation,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Usage tracking for metering
#[cfg(feature = "vision")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub requests_today: u32,
    pub requests_this_month: u32,
    pub last_reset: chrono::DateTime<chrono::Utc>,
}

/// Vision licensing manager
#[cfg(feature = "vision")]
#[derive(Debug, Clone)]
pub struct VisionLicenseManager {
    cache: Arc<RwLock<Option<CachedLicense>>>,
    usage: Arc<RwLock<UsageStats>>,
    cache_path: PathBuf,
    usage_path: PathBuf,
}

#[cfg(feature = "vision")]
impl VisionLicenseManager {
    /// Create a new license manager
    pub fn new() -> Self {
        let cache_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("shimmy")
            .join("vision");

        std::fs::create_dir_all(&cache_dir).ok();

        Self {
            cache: Arc::new(RwLock::new(None)),
            usage: Arc::new(RwLock::new(UsageStats {
                requests_today: 0,
                requests_this_month: 0,
                last_reset: chrono::Utc::now(),
            })),
            cache_path: cache_dir.join("license_cache.json"),
            usage_path: cache_dir.join("usage_stats.json"),
        }
    }

    /// Load cached license and usage data
    pub async fn load_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load cached license
        if self.cache_path.exists() {
            let data = tokio::fs::read_to_string(&self.cache_path).await?;
            let cached: CachedLicense = serde_json::from_str(&data)?;
            *self.cache.write().await = Some(cached);
        }

        // Load usage stats
        if self.usage_path.exists() {
            let data = tokio::fs::read_to_string(&self.usage_path).await?;
            let usage: UsageStats = serde_json::from_str(&data)?;
            *self.usage.write().await = usage;
        }

        Ok(())
    }

    /// Validate a license key with Keygen
    pub async fn validate_license(
        &self,
        license_key: &str,
    ) -> Result<LicenseValidation, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.as_ref() {
            if cached.key == license_key {
                // Check if still valid (with 24h grace period)
                let now = chrono::Utc::now();
                if let Some(expires) = cached.expires_at {
                    if now < expires + chrono::Duration::hours(24) {
                        return Ok(cached.validation.clone());
                    }
                } else if (now - cached.cached_at) < chrono::Duration::hours(24) {
                    return Ok(cached.validation.clone());
                }
            }
        }

        // Validate with Keygen API
        let validation = self.call_keygen_validate(license_key).await?;

        // Cache the result
        let cached = CachedLicense {
            key: license_key.to_string(),
            validation: validation.clone(),
            cached_at: chrono::Utc::now(),
            expires_at: validation
                .expires_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        };

        // Save to disk
        let data = serde_json::to_string_pretty(&cached)?;
        tokio::fs::write(&self.cache_path, &data).await?;

        *self.cache.write().await = Some(cached);

        Ok(validation)
    }

    /// Check if vision is allowed for the given license
    pub async fn check_vision_access(
        &self,
        license_key: Option<&str>,
    ) -> Result<(), VisionLicenseError> {
        // Allow development mode bypass
        if std::env::var("SHIMMY_VISION_DEV_MODE").is_ok() {
            return Ok(());
        }

        let Some(key) = license_key else {
            return Err(VisionLicenseError::MissingLicense);
        };

        let validation = self
            .validate_license(key)
            .await
            .map_err(|e| VisionLicenseError::ValidationFailed(e.to_string()))?;

        if !validation.valid {
            return Err(VisionLicenseError::InvalidLicense);
        }

        // Check vision entitlement
        if let Some(vision_enabled) = validation.entitlements.get("vision") {
            if !vision_enabled.as_bool().unwrap_or(false) {
                return Err(VisionLicenseError::FeatureNotEnabled);
            }
        } else {
            return Err(VisionLicenseError::FeatureNotEnabled);
        }

        // Check usage limits
        let usage = self.usage.read().await;
        if let Some(monthly_cap) = validation.entitlements.get("monthly_cap") {
            if let Some(cap) = monthly_cap.as_u64() {
                if usage.requests_this_month >= cap as u32 {
                    return Err(VisionLicenseError::UsageLimitExceeded);
                }
            }
        }

        Ok(())
    }

    /// Record a vision request for metering
    pub async fn record_usage(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut usage = self.usage.write().await;
        let now = chrono::Utc::now();

        // Reset counters if needed
        if (now - usage.last_reset).num_days() >= 1 {
            usage.requests_today = 0;
        }
        if (now - usage.last_reset).num_days() >= 30 {
            usage.requests_this_month = 0;
            usage.last_reset = now;
        }

        usage.requests_today += 1;
        usage.requests_this_month += 1;

        // Save to disk
        let data = serde_json::to_string_pretty(&*usage)?;
        tokio::fs::write(&self.usage_path, &data).await?;

        Ok(())
    }

    /// Call Keygen API to validate license
    async fn call_keygen_validate(
        &self,
        license_key: &str,
    ) -> Result<LicenseValidation, Box<dyn std::error::Error>> {
        let account_id = std::env::var("KEYGEN_ACCOUNT_ID")
            .map_err(|_| "KEYGEN_ACCOUNT_ID environment variable not set")?;
        let api_key = std::env::var("KEYGEN_API_KEY")
            .map_err(|_| "KEYGEN_API_KEY environment variable not set")?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.keygen.sh/v1/accounts/{}/licenses/actions/validate-key",
            account_id
        );

        #[derive(Serialize)]
        struct ValidateRequest {
            meta: ValidateMeta,
        }

        #[derive(Serialize)]
        struct ValidateMeta {
            key: String,
        }

        #[derive(Deserialize)]
        struct ValidateResponse {
            meta: ValidateResponseMeta,
            data: Option<serde_json::Value>,
        }

        #[derive(Deserialize)]
        struct ValidateResponseMeta {
            valid: bool,
            code: String,
            #[serde(default)]
            detail: Option<String>,
        }

        let request_body = ValidateRequest {
            meta: ValidateMeta {
                key: license_key.to_string(),
            },
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Keygen API error: {}", response.status()).into());
        }

        let validate_response: ValidateResponse = response.json().await?;

        // Extract entitlements from the license data if available
        let mut entitlements = if let Some(data) = validate_response.data {
            if let Some(entitlements) = data
                .get("relationships")
                .and_then(|rels| rels.get("entitlements"))
                .and_then(|ents| ents.get("data"))
                .and_then(|ents_data| ents_data.as_array())
            {
                let mut ents = HashMap::new();
                for ent in entitlements {
                    if let Some(code) = ent
                        .get("attributes")
                        .and_then(|attrs| attrs.get("code"))
                        .and_then(|c| c.as_str())
                    {
                        ents.insert(code.to_string(), serde_json::Value::Bool(true));
                    }
                }
                ents
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Add default vision entitlement if not present but license is valid
        if validate_response.meta.valid && !entitlements.contains_key("vision") {
            entitlements.insert("vision".to_string(), serde_json::Value::Bool(true));
        }

        // Add default monthly cap if not present
        if !entitlements.contains_key("monthly_cap") {
            entitlements.insert(
                "monthly_cap".to_string(),
                serde_json::Value::Number(1000.into()),
            );
        }

        Ok(LicenseValidation {
            valid: validate_response.meta.valid,
            entitlements,
            expires_at: None, // Keygen doesn't return expiry in validate-key response
            meta: {
                let mut meta = HashMap::new();
                meta.insert(
                    "code".to_string(),
                    serde_json::Value::String(validate_response.meta.code),
                );
                if let Some(detail) = validate_response.meta.detail {
                    meta.insert("detail".to_string(), serde_json::Value::String(detail));
                }
                meta
            },
        })
    }
}

#[cfg(feature = "vision")]
impl Default for VisionLicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// License-related errors
#[cfg(feature = "vision")]
#[derive(Debug, thiserror::Error)]
pub enum VisionLicenseError {
    #[error("No license key provided")]
    MissingLicense,

    #[error("License validation failed: {0}")]
    ValidationFailed(String),

    #[error("Invalid or expired license")]
    InvalidLicense,

    #[error("Vision feature not enabled for this license")]
    FeatureNotEnabled,

    #[error("Monthly usage limit exceeded")]
    UsageLimitExceeded,
}

#[cfg(feature = "vision")]
impl VisionLicenseError {
    /// Convert to HTTP status code
    pub fn to_status_code(&self) -> axum::http::StatusCode {
        match self {
            VisionLicenseError::MissingLicense => axum::http::StatusCode::PAYMENT_REQUIRED,
            VisionLicenseError::ValidationFailed(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            VisionLicenseError::InvalidLicense => axum::http::StatusCode::FORBIDDEN,
            VisionLicenseError::FeatureNotEnabled => axum::http::StatusCode::FORBIDDEN,
            VisionLicenseError::UsageLimitExceeded => axum::http::StatusCode::PAYMENT_REQUIRED,
        }
    }

    /// Convert to JSON error response
    pub fn to_json_error(&self) -> serde_json::Value {
        serde_json::json!({
            "error": "License validation failed",
            "code": match self {
                VisionLicenseError::MissingLicense => "MISSING_LICENSE",
                VisionLicenseError::ValidationFailed(_) => "VALIDATION_ERROR",
                VisionLicenseError::InvalidLicense => "INVALID_LICENSE",
                VisionLicenseError::FeatureNotEnabled => "FEATURE_DISABLED",
                VisionLicenseError::UsageLimitExceeded => "USAGE_LIMIT_EXCEEDED",
            },
            "message": self.to_string()
        })
    }
}

/// Stub implementation for when vision is disabled
#[cfg(not(feature = "vision"))]
pub fn check_vision_license(_license: Option<&str>) -> Result<(), &'static str> {
    Err("Vision feature not enabled")
}
