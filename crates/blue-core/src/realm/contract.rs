//! Contract definitions for cross-repo coordination
//!
//! Contracts define the schema and values shared between repos.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::RealmError;

/// A contract defining shared data between repos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract name (unique within domain)
    pub name: String,

    /// Contract version (semver)
    pub version: String,

    /// Which repo owns this contract (only owner can modify)
    pub owner: String,

    /// Compatibility settings
    #[serde(default)]
    pub compatibility: Compatibility,

    /// JSON Schema for the contract value
    pub schema: serde_json::Value,

    /// The actual contract value
    pub value: ContractValue,

    /// Validation configuration
    #[serde(default)]
    pub validation: Option<ValidationConfig>,

    /// Evolution history
    #[serde(default)]
    pub evolution: Vec<EvolutionEntry>,
}

impl Contract {
    /// Create a new contract
    pub fn new(name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            owner: owner.into(),
            compatibility: Compatibility::default(),
            schema: serde_json::json!({
                "type": "object"
            }),
            value: ContractValue::default(),
            validation: None,
            evolution: vec![EvolutionEntry {
                version: "1.0.0".to_string(),
                changes: vec!["Initial version".to_string()],
                compatible: true,
            }],
        }
    }

    /// Load from a YAML file
    pub fn load(path: &Path) -> Result<Self, RealmError> {
        let content = std::fs::read_to_string(path).map_err(|e| RealmError::ReadFile {
            path: path.display().to_string(),
            source: e,
        })?;
        let contract: Self = serde_yaml::from_str(&content)?;
        Ok(contract)
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

    /// Validate the contract
    pub fn validate(&self) -> Result<(), RealmError> {
        // Validate version is valid semver
        semver::Version::parse(&self.version)?;

        // Validate all evolution entries have valid versions
        for entry in &self.evolution {
            semver::Version::parse(&entry.version)?;
        }

        Ok(())
    }

    /// Check if this contract can be modified by a given repo
    pub fn can_modify(&self, repo: &str) -> bool {
        self.owner == repo
    }

    /// Check if a version upgrade is compatible
    pub fn is_compatible_upgrade(&self, new_version: &str) -> Result<bool, RealmError> {
        let current = semver::Version::parse(&self.version)?;
        let new = semver::Version::parse(new_version)?;

        // Major version bump = breaking change
        if new.major > current.major {
            return Ok(false);
        }

        // Same major, any minor/patch = compatible if backwards compatible
        Ok(self.compatibility.backwards)
    }
}

/// The actual value of a contract
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractValue {
    /// Read paths (for S3-style contracts)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub read: Vec<String>,

    /// Write paths
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub write: Vec<String>,

    /// Delete paths
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delete: Vec<String>,

    /// Additional fields (for flexibility)
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Compatibility settings for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compatibility {
    /// New version readable by old importers
    #[serde(default = "default_true")]
    pub backwards: bool,

    /// Old version readable by new importers
    #[serde(default)]
    pub forwards: bool,
}

impl Default for Compatibility {
    fn default() -> Self {
        Self {
            backwards: true,
            forwards: false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Validation configuration for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Script to run on export (validates exporter's code matches contract)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exporter: Option<String>,

    /// Script to run on import (validates importer's bindings)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importer: Option<String>,

    /// Scripts that only run in CI
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ci_only: Vec<String>,
}

/// An entry in the contract evolution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    /// Version this entry describes
    pub version: String,

    /// What changed in this version
    pub changes: Vec<String>,

    /// Whether this version is compatible with the previous
    #[serde(default = "default_true")]
    pub compatible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_new() {
        let contract = Contract::new("s3-permissions", "aperture");
        assert_eq!(contract.name, "s3-permissions");
        assert_eq!(contract.owner, "aperture");
        assert_eq!(contract.version, "1.0.0");
        assert!(contract.can_modify("aperture"));
        assert!(!contract.can_modify("fungal"));
    }

    #[test]
    fn test_contract_yaml_roundtrip() {
        let mut contract = Contract::new("s3-permissions", "aperture");
        contract.value.read = vec!["jobs/*/masks/*".to_string()];
        contract.value.write = vec!["jobs/*/*/manifest.json".to_string()];

        let yaml = serde_yaml::to_string(&contract).unwrap();
        let parsed: Contract = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.name, contract.name);
        assert_eq!(parsed.value.read, contract.value.read);
    }

    #[test]
    fn test_compatibility_check() {
        let contract = Contract::new("test", "owner");

        // Minor bump should be compatible
        assert!(contract.is_compatible_upgrade("1.1.0").unwrap());

        // Major bump should be incompatible
        assert!(!contract.is_compatible_upgrade("2.0.0").unwrap());
    }

    #[test]
    fn test_contract_value_extra_fields() {
        let yaml = r#"
read:
  - "path/a"
write:
  - "path/b"
custom_field: "custom_value"
"#;
        let value: ContractValue = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(value.read, vec!["path/a"]);
        assert!(value.extra.contains_key("custom_field"));
    }
}
