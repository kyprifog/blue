//! Realm configuration (realm.yaml)
//!
//! Defines the top-level realm configuration including governance and trust settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::RealmError;

/// Top-level realm configuration stored in realm.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConfig {
    /// Realm name (unique identifier)
    pub name: String,

    /// Realm version (semver)
    pub version: String,

    /// When the realm was created
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    /// Governance settings
    #[serde(default)]
    pub governance: Governance,

    /// Trust settings
    #[serde(default)]
    pub trust: TrustConfig,
}

impl RealmConfig {
    /// Create a new realm with defaults
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            governance: Governance::default(),
            trust: TrustConfig::default(),
        }
    }

    /// Load from a YAML file
    pub fn load(path: &Path) -> Result<Self, RealmError> {
        let content = std::fs::read_to_string(path).map_err(|e| RealmError::ReadFile {
            path: path.display().to_string(),
            source: e,
        })?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Save to a YAML file
    pub fn save(&self, path: &Path) -> Result<(), RealmError> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content).map_err(|e| RealmError::WriteFile {
            path: path.display().to_string(),
            source: e,
        })?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), RealmError> {
        // Validate version is valid semver
        semver::Version::parse(&self.version)?;
        Ok(())
    }
}

/// Governance settings for the realm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Governance {
    /// How new repos can join
    #[serde(default)]
    pub admission: AdmissionPolicy,

    /// Who can approve new repos (email addresses)
    #[serde(default)]
    pub approvers: Vec<String>,

    /// Policy for breaking changes
    #[serde(default)]
    pub breaking_changes: BreakingChangePolicy,
}

impl Default for Governance {
    fn default() -> Self {
        Self {
            admission: AdmissionPolicy::Approval,
            approvers: Vec::new(),
            breaking_changes: BreakingChangePolicy::default(),
        }
    }
}

/// How new repos can join the realm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AdmissionPolicy {
    /// Anyone can join
    Open,

    /// Requires approval from an approver
    #[default]
    Approval,

    /// Only explicitly invited repos can join
    InviteOnly,
}

/// Policy for breaking changes to contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangePolicy {
    /// Whether breaking changes require approval
    #[serde(default = "default_true")]
    pub require_approval: bool,

    /// Grace period in days before breaking changes take effect
    #[serde(default = "default_grace_period")]
    pub grace_period_days: u32,
}

impl Default for BreakingChangePolicy {
    fn default() -> Self {
        Self {
            require_approval: true,
            grace_period_days: 14,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_grace_period() -> u32 {
    14
}

/// Trust configuration for the realm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    /// Trust model
    #[serde(default)]
    pub mode: TrustMode,

    /// Whether commits must be signed
    #[serde(default)]
    pub require_signed_commits: bool,

    /// File path patterns to permission groups
    /// e.g., "repos/{repo}.yaml" -> ["repo_maintainers"]
    #[serde(default)]
    pub permissions: HashMap<String, Vec<String>>,
}

impl Default for TrustConfig {
    fn default() -> Self {
        let mut permissions = HashMap::new();
        permissions.insert(
            "repos/{repo}.yaml".to_string(),
            vec!["repo_maintainers".to_string()],
        );
        permissions.insert(
            "domains/{domain}/domain.yaml".to_string(),
            vec!["domain_owners".to_string()],
        );
        permissions.insert(
            "domains/{domain}/contracts/{name}.yaml".to_string(),
            vec!["contract_owner".to_string()],
        );
        permissions.insert(
            "domains/{domain}/bindings/{repo}.yaml".to_string(),
            vec!["repo_maintainers".to_string()],
        );

        Self {
            mode: TrustMode::Collaborative,
            require_signed_commits: false,
            permissions,
        }
    }
}

/// Trust model for the realm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TrustMode {
    /// All participants are equal peers
    #[default]
    Collaborative,

    /// One party provides, others consume
    VendorCustomer,

    /// Loose coordination between independent parties
    Federation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_config_new() {
        let config = RealmConfig::new("test-realm");
        assert_eq!(config.name, "test-realm");
        assert_eq!(config.version, "1.0.0");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_realm_config_yaml_roundtrip() {
        let config = RealmConfig::new("letemcook");
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: RealmConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, config.name);
    }

    #[test]
    fn test_governance_defaults() {
        let gov = Governance::default();
        assert_eq!(gov.admission, AdmissionPolicy::Approval);
        assert!(gov.breaking_changes.require_approval);
        assert_eq!(gov.breaking_changes.grace_period_days, 14);
    }

    #[test]
    fn test_trust_config_defaults() {
        let trust = TrustConfig::default();
        assert_eq!(trust.mode, TrustMode::Collaborative);
        assert!(!trust.require_signed_commits);
        assert!(trust.permissions.contains_key("repos/{repo}.yaml"));
    }

    #[test]
    fn test_admission_policy_serde() {
        let yaml = "invite-only";
        let policy: AdmissionPolicy = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(policy, AdmissionPolicy::InviteOnly);
    }
}
