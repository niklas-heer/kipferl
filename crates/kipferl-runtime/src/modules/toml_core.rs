pub(super) fn loads(text: &str) -> Result<String, String> {
    let value: toml::Value =
        toml::from_str(text).map_err(|error| format!("invalid TOML: {error}"))?;
    serde_json::to_string(&value).map_err(|error| format!("unsupported TOML value: {error}"))
}

pub(super) fn dumps(data: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(data).map_err(|error| format!("unsupported TOML data: {error}"))?;
    toml::to_string_pretty(&value).map_err(|error| format!("could not encode TOML: {error}"))
}
