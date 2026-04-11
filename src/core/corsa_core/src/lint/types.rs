use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// UTF-8 byte range used by Oxlint-compatible rule diagnostics and fixes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }
}

/// Serializable AST/type facts passed from the Oxlint JS plugin boundary into
/// Rust-authored rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintNode {
    pub kind: String,
    pub range: TextRange,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub type_texts: Vec<String>,
    #[serde(default)]
    pub property_names: Vec<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
    #[serde(default)]
    pub children: BTreeMap<String, LintNode>,
    #[serde(default)]
    pub child_lists: BTreeMap<String, Vec<LintNode>>,
}

impl LintNode {
    pub fn child(&self, key: &str) -> Option<&Self> {
        self.children.get(key)
    }

    pub fn child_list(&self, key: &str) -> Option<&[Self]> {
        self.child_lists.get(key).map(Vec::as_slice)
    }

    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(Value::as_str)
    }

    pub fn field_bool(&self, key: &str) -> Option<bool> {
        self.fields.get(key).and_then(Value::as_bool)
    }

    pub fn field_f64(&self, key: &str) -> Option<f64> {
        self.fields.get(key).and_then(Value::as_f64)
    }

    pub fn field_stringish(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFix {
    pub range: TextRange,
    pub replacement_text: String,
}

impl LintFix {
    pub fn replace_range(range: TextRange, replacement_text: impl Into<String>) -> Self {
        Self {
            range,
            replacement_text: replacement_text.into(),
        }
    }

    pub fn remove_range(range: TextRange) -> Self {
        Self::replace_range(range, "")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintSuggestion {
    pub message_id: String,
    pub message: String,
    pub fixes: Vec<LintFix>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintDiagnostic {
    pub rule_name: String,
    pub message_id: String,
    pub message: String,
    pub range: TextRange,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<LintSuggestion>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleMessage {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMeta {
    pub name: String,
    pub docs_description: String,
    pub messages: BTreeMap<String, String>,
    pub has_suggestions: bool,
    pub listeners: Vec<String>,
    pub requires_type_texts: bool,
}
