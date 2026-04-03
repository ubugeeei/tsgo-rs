use napi::Result;
use napi_derive::napi;
use serde::Deserialize;

use crate::util::parse_json;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnsafeTypeFlowInput {
    source_type_texts: Vec<String>,
    #[serde(default)]
    target_type_texts: Vec<String>,
}

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_assignment(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(tsgo_rs::utils::is_unsafe_assignment(
        input.source_type_texts.as_slice(),
        input.target_type_texts.as_slice(),
    ))
}

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_return(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(tsgo_rs::utils::is_unsafe_return(
        input.source_type_texts.as_slice(),
        input.target_type_texts.as_slice(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{is_unsafe_assignment, is_unsafe_return};

    #[test]
    fn flags_direct_any_assignment() {
        assert!(
            is_unsafe_assignment(
                r#"{"sourceTypeTexts":["any"],"targetTypeTexts":["string"]}"#.to_owned()
            )
            .unwrap()
        );
    }

    #[test]
    fn allows_unknown_targets() {
        assert!(
            !is_unsafe_assignment(
                r#"{"sourceTypeTexts":["any"],"targetTypeTexts":["unknown"]}"#.to_owned()
            )
            .unwrap()
        );
    }

    #[test]
    fn flags_generic_any_assignment() {
        assert!(
            is_unsafe_assignment(
                r#"{"sourceTypeTexts":["Set<any>"],"targetTypeTexts":["Set<string>"]}"#.to_owned()
            )
            .unwrap()
        );
    }

    #[test]
    fn flags_promise_any_returns() {
        assert!(
            is_unsafe_return(
                r#"{"sourceTypeTexts":["Promise<any>"],"targetTypeTexts":["Promise<string>"]}"#
                    .to_owned()
            )
            .unwrap()
        );
    }

    #[test]
    fn flags_unions_that_include_any() {
        assert!(
            is_unsafe_assignment(
                r#"{"sourceTypeTexts":["string | any"],"targetTypeTexts":["string"]}"#.to_owned()
            )
            .unwrap()
        );
    }

    #[test]
    fn inferred_targets_still_flag_any_flows() {
        assert!(is_unsafe_assignment(r#"{"sourceTypeTexts":["any[]"]}"#.to_owned()).unwrap());
    }

    #[test]
    fn keeps_specific_flows_allowed() {
        assert!(
            !is_unsafe_return(
                r#"{"sourceTypeTexts":["Promise<string>"],"targetTypeTexts":["Promise<string>"]}"#
                    .to_owned()
            )
            .unwrap()
        );
    }
}
