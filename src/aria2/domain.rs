#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    version: String,
    enabled_features: Vec<String>,
}

impl VersionInfo {
    pub fn new(version: impl Into<String>, enabled_features: Vec<String>) -> Self {
        Self {
            version: version.into(),
            enabled_features,
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    #[cfg(test)]
    pub fn enabled_features(&self) -> &[String] {
        &self.enabled_features
    }
}
