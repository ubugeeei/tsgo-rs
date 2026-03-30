use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{NodeHandle, ProjectHandle, SignatureHandle, SymbolHandle, TypeHandle};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub use_case_sensitive_file_names: bool,
    pub current_directory: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub options: Value,
    pub file_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: ProjectHandle,
    pub config_file_name: String,
    pub compiler_options: Value,
    pub root_files: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolResponse {
    pub id: SymbolHandle,
    pub name: String,
    pub flags: u32,
    pub check_flags: u32,
    #[serde(default)]
    pub declarations: Vec<NodeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_declaration: Option<NodeHandle>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeResponse {
    pub id: TypeHandle,
    pub flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TypeHandle>,
    #[serde(default)]
    pub type_parameters: Vec<TypeHandle>,
    #[serde(default)]
    pub outer_type_parameters: Vec<TypeHandle>,
    #[serde(default)]
    pub local_type_parameters: Vec<TypeHandle>,
    #[serde(default)]
    pub element_flags: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<TypeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_type: Option<TypeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_type: Option<TypeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends_type: Option<TypeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_type: Option<TypeHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subst_constraint: Option<TypeHandle>,
    #[serde(default)]
    pub texts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<SymbolHandle>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureResponse {
    pub id: SignatureHandle,
    pub flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<NodeHandle>,
    #[serde(default)]
    pub type_parameters: Vec<TypeHandle>,
    #[serde(default)]
    pub parameters: Vec<SymbolHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this_parameter: Option<SymbolHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SignatureHandle>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypePredicateResponse {
    pub kind: i32,
    pub parameter_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<TypeResponse>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    pub key_type: TypeResponse,
    pub value_type: TypeResponse,
    #[serde(default)]
    pub is_readonly: bool,
}
