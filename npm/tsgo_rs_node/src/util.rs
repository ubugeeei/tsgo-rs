use napi::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::fmt::Display;
use tsgo_rs::api::{ApiMode, ApiSpawnConfig};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnOptions {
    pub executable: String,
    pub cwd: Option<String>,
    pub mode: Option<String>,
}

pub fn build_spawn_config(options: SpawnOptions) -> Result<ApiSpawnConfig> {
    let mut config = ApiSpawnConfig::new(options.executable);
    if let Some(cwd) = options.cwd {
        config = config.with_cwd(cwd);
    }
    if let Some(mode) = options.mode {
        config = config.with_mode(parse_mode(mode.as_str())?);
    }
    Ok(config)
}

pub fn parse_json<T>(value: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(value).map_err(into_napi_error)
}

pub fn parse_optional_json(value: Option<String>) -> Result<Value> {
    match value {
        Some(value) => parse_json(value.as_str()),
        None => Ok(Value::Null),
    }
}

pub fn to_json<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(into_napi_error)
}

pub fn into_napi_error(error: impl Display) -> Error {
    Error::from_reason(error.to_string())
}

fn parse_mode(mode: &str) -> Result<ApiMode> {
    match mode {
        "jsonrpc" => Ok(ApiMode::AsyncJsonRpcStdio),
        "msgpack" => Ok(ApiMode::SyncMsgpackStdio),
        _ => Err(Error::from_reason("unknown tsgo api mode".to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::{SpawnOptions, build_spawn_config, parse_json, parse_optional_json};
    use serde_json::json;
    use tsgo_rs::api::ApiMode;

    #[test]
    fn parse_optional_json_defaults_to_null() {
        assert_eq!(parse_optional_json(None).unwrap(), json!(null));
    }

    #[test]
    fn spawn_config_defaults_to_msgpack() {
        let options = parse_json::<SpawnOptions>(r#"{"executable":"./tsgo"}"#).unwrap();
        let config = build_spawn_config(options).unwrap();
        assert_eq!(config.mode, ApiMode::SyncMsgpackStdio);
    }

    #[test]
    fn spawn_config_accepts_jsonrpc_mode() {
        let options =
            parse_json::<SpawnOptions>(r#"{"executable":"./tsgo","mode":"jsonrpc"}"#).unwrap();
        let config = build_spawn_config(options).unwrap();
        assert_eq!(config.mode, ApiMode::AsyncJsonRpcStdio);
    }
}
