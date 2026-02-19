use anyhow::{Context, Result};
use openapiv3::OpenAPI;

use crate::spec::{LoadedSpec, SpecVersion};

pub fn parse(content: &str) -> Result<LoadedSpec> {
    let api: OpenAPI = serde_yaml::from_str(content).context("failed to parse OpenAPI 3.x spec")?;

    let version = if api.openapi.starts_with("3.1") {
        SpecVersion::V31
    } else {
        SpecVersion::V30
    };

    let title = api.info.title.clone();
    let spec_version_str = api.openapi.clone();

    // Extract top-level schema names from components/schemas
    let schema_names: Vec<String> = api
        .components
        .as_ref()
        .map(|c| {
            let mut names: Vec<String> = c.schemas.keys().cloned().collect();
            names.sort();
            names
        })
        .unwrap_or_default();

    Ok(LoadedSpec {
        title,
        openapi_version: spec_version_str,
        version,
        schema_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PETSTORE: &str = include_str!("../../fixtures/petstore.yaml");

    #[test]
    fn parses_petstore_schema_names() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        assert_eq!(spec.title, "Petstore");
        // Should find Error, NewPet, Pet, Pets (sorted alphabetically)
        assert_eq!(spec.schema_names, vec!["Error", "NewPet", "Pet", "Pets"]);
    }

    #[test]
    fn detects_openapi_30() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        assert_eq!(spec.version, SpecVersion::V30);
    }
}
