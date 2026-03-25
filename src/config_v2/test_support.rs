use std::path::Path;

pub fn toml_string(value: impl AsRef<str>) -> String {
    toml::Value::String(value.as_ref().to_string()).to_string()
}

pub fn toml_path_source(path: impl AsRef<Path>) -> String {
    format!(
        "{{ path = {} }}",
        toml_string(path.as_ref().display().to_string())
    )
}

pub fn toml_string_secret(value: impl AsRef<str>) -> String {
    format!(r#"{{ type = "string", value = {} }}"#, toml_string(value))
}
