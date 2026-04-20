use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetupResponse {
    pub ok: bool,
    pub data_root: String,
    pub cache_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctorResponse {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UninstallRequest {
    pub purge_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UninstallResponse {
    pub ok: bool,
    pub cache_removed: bool,
    pub data_removed: bool,
    pub data_preserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRequest {
    pub version: Option<String>,
    pub force: bool,
}
