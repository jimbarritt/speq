/// The loaded, version-detected representation of an OpenAPI spec.
pub struct LoadedSpec {
    pub title: String,
    pub openapi_version: String,
    pub version: SpecVersion,
    pub schema_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecVersion {
    V20,
    V30,
    V31,
}

impl SpecVersion {
    pub fn label(&self) -> &'static str {
        match self {
            SpecVersion::V20 => "Swagger 2.0",
            SpecVersion::V30 => "OpenAPI 3.0",
            SpecVersion::V31 => "OpenAPI 3.1",
        }
    }
}
