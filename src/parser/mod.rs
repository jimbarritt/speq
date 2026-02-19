pub mod v3;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::spec::LoadedSpec;

/// Detect OpenAPI version and parse the spec from YAML/JSON bytes.
pub fn parse_spec(content: &str) -> Result<LoadedSpec> {
    // Parse as generic JSON value first for version detection.
    // serde_yaml can deserialise both YAML and JSON.
    let raw: Value = serde_yaml::from_str(content).context("failed to parse spec as YAML/JSON")?;

    if let Some(version) = raw.get("swagger").and_then(|v| v.as_str()) {
        if version.starts_with("2.") {
            bail!("Swagger 2.0 support is not yet implemented");
        }
    }

    if let Some(version) = raw.get("openapi").and_then(|v| v.as_str()) {
        if version.starts_with("3.") {
            return v3::parse(content);
        }
    }

    bail!("Cannot determine OpenAPI version from spec (expected 'openapi: 3.x' or 'swagger: 2.0')")
}
