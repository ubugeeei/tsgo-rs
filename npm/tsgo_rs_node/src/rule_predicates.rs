use napi::Result;
use napi_derive::napi;
use serde::Deserialize;
use std::collections::BTreeSet;

use crate::util::parse_json;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnsafeTypeFlowInput {
    source_type_texts: Vec<String>,
    #[serde(default)]
    target_type_texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SimpleType {
    Any,
    Unknown,
    Never,
    Primitive(String),
    Array(Box<SimpleType>),
    Tuple(Vec<SimpleType>),
    Generic { base: String, args: Vec<SimpleType> },
    Union(Vec<SimpleType>),
    Intersection(Vec<SimpleType>),
    Other(String),
}

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_assignment(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(has_unsafe_any_flow(
        input.source_type_texts.as_slice(),
        input.target_type_texts.as_slice(),
    ))
}

#[allow(dead_code)]
#[napi]
pub fn is_unsafe_return(input_json: String) -> Result<bool> {
    let input = parse_json::<UnsafeTypeFlowInput>(input_json.as_str())?;
    Ok(has_unsafe_any_flow(
        input.source_type_texts.as_slice(),
        input.target_type_texts.as_slice(),
    ))
}

fn has_unsafe_any_flow(source_texts: &[String], target_texts: &[String]) -> bool {
    let sources = parse_type_texts(source_texts);
    if sources.is_empty() {
        return false;
    }
    let targets = parse_type_texts(target_texts);
    if targets.is_empty() {
        return sources.iter().any(contains_any_like);
    }
    sources.iter().any(|source| {
        targets
            .iter()
            .filter(|target| !is_permissive_target(target))
            .any(|target| is_unsafe_flow(source, target))
    })
}

fn parse_type_texts(texts: &[String]) -> Vec<SimpleType> {
    let mut unique = BTreeSet::new();
    for text in texts {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            unique.insert(trimmed.to_owned());
        }
    }
    unique
        .into_iter()
        .map(|text| parse_type_text(text.as_str()))
        .collect()
}

fn parse_type_text(text: &str) -> SimpleType {
    let text = strip_wrapping_parens(text.trim());
    if let Some(parts) = split_top_level(text, '|') {
        return SimpleType::Union(parts.iter().map(|part| parse_type_text(part)).collect());
    }
    if let Some(parts) = split_top_level(text, '&') {
        return SimpleType::Intersection(parts.iter().map(|part| parse_type_text(part)).collect());
    }
    if let Some(stripped) = text.strip_suffix("[]") {
        return SimpleType::Array(Box::new(parse_type_text(stripped)));
    }
    if text.starts_with('[') && text.ends_with(']') && is_wrapped_by(text, '[', ']') {
        let inner = &text[1..text.len() - 1];
        let items = split_top_level_list(inner, ',')
            .into_iter()
            .map(parse_type_text)
            .collect();
        return SimpleType::Tuple(items);
    }
    if let Some((base, args)) = split_generic(text) {
        return SimpleType::Generic {
            base: base.to_owned(),
            args: split_top_level_list(args, ',')
                .into_iter()
                .map(parse_type_text)
                .collect(),
        };
    }
    match text {
        "any" => SimpleType::Any,
        "unknown" => SimpleType::Unknown,
        "never" => SimpleType::Never,
        "string" | "number" | "boolean" | "bigint" | "symbol" | "null" | "undefined" => {
            SimpleType::Primitive(text.to_owned())
        }
        "true" | "false" => SimpleType::Primitive("boolean".to_owned()),
        _ if is_string_literal(text) => SimpleType::Primitive("string".to_owned()),
        _ if is_number_literal(text) => SimpleType::Primitive("number".to_owned()),
        _ if is_bigint_literal(text) => SimpleType::Primitive("bigint".to_owned()),
        _ => SimpleType::Other(text.to_owned()),
    }
}

fn is_unsafe_flow(source: &SimpleType, target: &SimpleType) -> bool {
    if is_permissive_target(target) {
        return false;
    }
    match source {
        SimpleType::Any => true,
        SimpleType::Union(types) | SimpleType::Intersection(types) => {
            types.iter().any(|member| is_unsafe_flow(member, target))
        }
        SimpleType::Array(source_item) => match target {
            SimpleType::Array(target_item) => is_unsafe_flow(source_item, target_item),
            SimpleType::Generic { base, args }
                if is_array_like_base(base.as_str()) && args.len() == 1 =>
            {
                is_unsafe_flow(source_item, &args[0])
            }
            _ => false,
        },
        SimpleType::Tuple(source_items) => match target {
            SimpleType::Tuple(target_items) => source_items
                .iter()
                .zip(target_items.iter())
                .any(|(source_item, target_item)| is_unsafe_flow(source_item, target_item)),
            SimpleType::Array(target_item) => source_items
                .iter()
                .any(|source_item| is_unsafe_flow(source_item, target_item)),
            SimpleType::Generic { base, args }
                if is_array_like_base(base.as_str()) && args.len() == 1 =>
            {
                source_items
                    .iter()
                    .any(|source_item| is_unsafe_flow(source_item, &args[0]))
            }
            _ => false,
        },
        SimpleType::Generic {
            base: source_base,
            args: source_args,
        } => match target {
            SimpleType::Generic {
                base: target_base,
                args: target_args,
            } if same_container_family(source_base.as_str(), target_base.as_str())
                && source_args.len() == target_args.len() =>
            {
                source_args
                    .iter()
                    .zip(target_args.iter())
                    .any(|(source_arg, target_arg)| is_unsafe_flow(source_arg, target_arg))
            }
            SimpleType::Array(target_item)
                if is_array_like_base(source_base.as_str()) && source_args.len() == 1 =>
            {
                is_unsafe_flow(&source_args[0], target_item)
            }
            _ => false,
        },
        _ => false,
    }
}

fn contains_any_like(ty: &SimpleType) -> bool {
    match ty {
        SimpleType::Any => true,
        SimpleType::Array(inner) => contains_any_like(inner),
        SimpleType::Tuple(items) | SimpleType::Union(items) | SimpleType::Intersection(items) => {
            items.iter().any(contains_any_like)
        }
        SimpleType::Generic { args, .. } => args.iter().any(contains_any_like),
        _ => false,
    }
}

fn is_permissive_target(ty: &SimpleType) -> bool {
    match ty {
        SimpleType::Any | SimpleType::Unknown | SimpleType::Never => true,
        SimpleType::Union(types) => types.iter().any(is_permissive_target),
        _ => false,
    }
}

fn same_container_family(left: &str, right: &str) -> bool {
    left == right
        || (is_array_like_base(left) && is_array_like_base(right))
        || (is_promise_like_base(left) && is_promise_like_base(right))
}

fn is_array_like_base(base: &str) -> bool {
    matches!(base, "Array" | "ReadonlyArray")
}

fn is_promise_like_base(base: &str) -> bool {
    matches!(base, "Promise" | "PromiseLike")
}

fn split_generic(text: &str) -> Option<(&str, &str)> {
    let mut angle_depth = 0usize;
    let mut square_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut generic_start = None;
    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '<' if square_depth == 0 && paren_depth == 0 && brace_depth == 0 => {
                if angle_depth == 0 {
                    generic_start = Some(index);
                }
                angle_depth += 1;
            }
            '>' if angle_depth > 0 => angle_depth -= 1,
            '[' => square_depth += 1,
            ']' if square_depth > 0 => square_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            _ => {}
        }
    }
    let start = generic_start?;
    if angle_depth != 0 || !text.ends_with('>') || start == 0 {
        return None;
    }
    let base = text[..start].trim();
    let args = &text[start + 1..text.len() - 1];
    if base.is_empty() {
        return None;
    }
    Some((base, args))
}

fn split_top_level(text: &str, delimiter: char) -> Option<Vec<&str>> {
    let parts = split_top_level_list(text, delimiter);
    (parts.len() > 1).then_some(parts)
}

fn split_top_level_list(text: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut angle_depth = 0usize;
    let mut square_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '<' => angle_depth += 1,
            '>' if angle_depth > 0 => angle_depth -= 1,
            '[' => square_depth += 1,
            ']' if square_depth > 0 => square_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            _ if ch == delimiter
                && angle_depth == 0
                && square_depth == 0
                && paren_depth == 0
                && brace_depth == 0 =>
            {
                parts.push(text[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts
}

fn strip_wrapping_parens(text: &str) -> &str {
    let mut current = text.trim();
    while current.starts_with('(') && current.ends_with(')') && is_wrapped_by(current, '(', ')') {
        current = current[1..current.len() - 1].trim();
    }
    current
}

fn is_wrapped_by(text: &str, open: char, close: char) -> bool {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth -= 1;
                if depth == 0 && index + ch.len_utf8() != text.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn is_string_literal(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
        || (text.starts_with('`') && text.ends_with('`'))
}

fn is_number_literal(text: &str) -> bool {
    text.parse::<f64>().is_ok()
}

fn is_bigint_literal(text: &str) -> bool {
    text.ends_with('n') && text[..text.len() - 1].parse::<i128>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::{SimpleType, has_unsafe_any_flow, parse_type_text};

    #[test]
    fn parses_nested_containers() {
        assert_eq!(
            parse_type_text("Promise<Array<any>>"),
            SimpleType::Generic {
                base: "Promise".to_owned(),
                args: vec![SimpleType::Generic {
                    base: "Array".to_owned(),
                    args: vec![SimpleType::Any],
                }],
            }
        );
    }

    #[test]
    fn flags_direct_any_assignment() {
        assert!(has_unsafe_any_flow(
            &[String::from("any")],
            &[String::from("string")]
        ));
    }

    #[test]
    fn allows_unknown_targets() {
        assert!(!has_unsafe_any_flow(
            &[String::from("any")],
            &[String::from("unknown")]
        ));
    }

    #[test]
    fn flags_generic_any_assignment() {
        assert!(has_unsafe_any_flow(
            &[String::from("Set<any>")],
            &[String::from("Set<string>")]
        ));
    }

    #[test]
    fn flags_promise_any_returns() {
        assert!(has_unsafe_any_flow(
            &[String::from("Promise<any>")],
            &[String::from("Promise<string>")]
        ));
    }

    #[test]
    fn flags_unions_that_include_any() {
        assert!(has_unsafe_any_flow(
            &[String::from("string | any")],
            &[String::from("string")]
        ));
    }

    #[test]
    fn inferred_targets_still_flag_any_flows() {
        assert!(has_unsafe_any_flow(&[String::from("any[]")], &[]));
    }

    #[test]
    fn keeps_specific_flows_allowed() {
        assert!(!has_unsafe_any_flow(
            &[String::from("Promise<string>")],
            &[String::from("Promise<string>")]
        ));
    }
}
