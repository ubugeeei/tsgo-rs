#![allow(dead_code)]

use napi::Result;
use napi_derive::napi;

use crate::util::{into_napi_error, parse_json, to_json};

#[napi]
pub fn run_native_lint_rule(rule_name: String, node_json: String) -> Result<String> {
    let node = parse_json::<corsa::lint::LintNode>(node_json.as_str())?;
    let Some(diagnostics) = corsa::lint::run_default_type_aware_rule(rule_name.as_str(), &node)
    else {
        return Err(into_napi_error(format!(
            "unknown native lint rule: {rule_name}"
        )));
    };
    to_json(&diagnostics)
}

#[napi]
pub fn native_lint_rule_metas_json() -> Result<String> {
    to_json(&corsa::lint::LintRuleRegistry::with_default_type_aware_rules().metas())
}
