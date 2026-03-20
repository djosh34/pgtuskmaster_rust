use std::path::Path;
use crate::config_v2::types::{ConfigErrorV2, RuntimeConfigV2};

pub fn load_operator_config(path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    Err(ConfigErrorV2::Validation {
        field: "Not implemented",
        message: "Not implemented".to_string(),
    })
}

