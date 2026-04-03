use std::fmt;

use serde::{Deserialize, Serialize};

use super::split::{split_top_level_owned, split_type_text_owned};

/// Classification bucket for a rendered TypeScript type text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeTextKind {
    Any,
    Bigint,
    Boolean,
    Nullish,
    Number,
    Regexp,
    String,
    Unknown,
    Other,
}

impl TypeTextKind {
    /// Returns the stable lowercase label used across bindings.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Bigint => "bigint",
            Self::Boolean => "boolean",
            Self::Nullish => "nullish",
            Self::Number => "number",
            Self::Regexp => "regexp",
            Self::String => "string",
            Self::Unknown => "unknown",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for TypeTextKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returns a coarse-grained classification for a rendered type text.
pub fn classify_type_text(text: Option<&str>) -> TypeTextKind {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return TypeTextKind::Other;
    };
    match text {
        "any" => TypeTextKind::Any,
        "unknown" | "never" => TypeTextKind::Unknown,
        "string" => TypeTextKind::String,
        "number" => TypeTextKind::Number,
        "bigint" => TypeTextKind::Bigint,
        "boolean" | "true" | "false" => TypeTextKind::Boolean,
        "null" | "undefined" => TypeTextKind::Nullish,
        _ if is_string_literal(text) => TypeTextKind::String,
        _ if is_number_literal(text) => TypeTextKind::Number,
        _ if is_bigint_literal(text) => TypeTextKind::Bigint,
        _ if text.contains("null |") || text.contains("| null") => TypeTextKind::Nullish,
        _ if text.contains("RegExp") => TypeTextKind::Regexp,
        _ => TypeTextKind::Other,
    }
}

/// Splits a type text at top-level occurrences of `delimiter`.
pub fn split_top_level_type_text(text: &str, delimiter: char) -> Vec<String> {
    split_top_level_owned(text, delimiter)
}

/// Splits a type text at top-level union and intersection boundaries.
pub fn split_type_text(text: &str) -> Vec<String> {
    split_type_text_owned(text)
}

pub fn is_string_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::String)
}

pub fn is_number_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Number)
}

pub fn is_bigint_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Bigint)
}

pub fn is_any_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Any)
}

pub fn is_unknown_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Unknown)
}

pub fn is_array_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref().trim();
        text.ends_with("[]")
            || text.starts_with("Array<")
            || text.starts_with("ReadonlyArray<")
            || (text.starts_with('[') && text.ends_with(']'))
    })
}

pub fn is_promise_like_type_texts<T: AsRef<str>, P: AsRef<str>>(
    type_texts: &[T],
    property_names: &[P],
) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref();
        text.contains("Promise") || text.contains("Thenable")
    }) || property_names.iter().any(|name| name.as_ref() == "then")
}

pub fn is_error_like_type_texts<T: AsRef<str>, P: AsRef<str>>(
    type_texts: &[T],
    property_names: &[P],
) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref().trim();
        text == "Error" || text.ends_with("Error")
    }) || {
        let mut has_message = false;
        let mut has_name = false;
        for property_name in property_names {
            match property_name.as_ref() {
                "message" => has_message = true,
                "name" => has_name = true,
                _ => {}
            }
            if has_message && has_name {
                return true;
            }
        }
        false
    }
}

fn matches_kind<T: AsRef<str>>(type_texts: &[T], kind: TypeTextKind) -> bool {
    type_texts
        .iter()
        .any(|text| classify_type_text(Some(text.as_ref())) == kind)
}

pub(crate) fn is_string_literal(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
        || (text.starts_with('`') && text.ends_with('`'))
}

pub(crate) fn is_number_literal(text: &str) -> bool {
    text.parse::<f64>().is_ok()
}

pub(crate) fn is_bigint_literal(text: &str) -> bool {
    text.ends_with('n') && text[..text.len() - 1].parse::<i128>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        TypeTextKind, classify_type_text, is_array_like_type_texts, is_error_like_type_texts,
        is_promise_like_type_texts, split_top_level_type_text, split_type_text,
    };

    #[test]
    fn classifies_type_texts() {
        assert_eq!(classify_type_text(Some("'value'")), TypeTextKind::String);
        assert_eq!(classify_type_text(Some("42")), TypeTextKind::Number);
        assert_eq!(classify_type_text(Some("42n")), TypeTextKind::Bigint);
        assert_eq!(classify_type_text(Some("boolean")), TypeTextKind::Boolean);
        assert_eq!(
            classify_type_text(Some("null | string")),
            TypeTextKind::Nullish
        );
        assert_eq!(classify_type_text(Some("RegExp")), TypeTextKind::Regexp);
        assert_eq!(classify_type_text(None), TypeTextKind::Other);
    }

    #[test]
    fn splits_type_texts_at_top_level_only() {
        assert_eq!(
            split_top_level_type_text("Promise<string | number> | null", '|'),
            vec!["Promise<string | number>", "null"]
        );
        assert_eq!(
            split_type_text("string | Promise<Array<number>> & undefined"),
            vec!["string", "Promise<Array<number>>", "undefined"]
        );
    }

    #[test]
    fn detects_array_promise_and_error_shapes() {
        assert!(is_array_like_type_texts(&["ReadonlyArray<string>"]));
        assert!(is_promise_like_type_texts(
            &["Promise<string>"],
            &[] as &[&str]
        ));
        assert!(is_promise_like_type_texts(
            &[] as &[&str],
            &["then", "catch"]
        ));
        assert!(is_error_like_type_texts(&["TypeError"], &[] as &[&str]));
        assert!(is_error_like_type_texts(
            &[] as &[&str],
            &["message", "name"]
        ));
    }
}
