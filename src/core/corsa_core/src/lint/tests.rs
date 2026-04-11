use std::collections::BTreeMap;

use serde_json::json;

use super::{LintNode, LintRuleRegistry, TextRange};

#[test]
fn reports_array_delete_with_splice_suggestion() {
    let diagnostics = registry()
        .run_rule("no-array-delete", &array_delete_node())
        .unwrap();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_name, "no-array-delete");
    assert_eq!(diagnostics[0].message_id, "unexpected");
    assert_eq!(diagnostics[0].range, TextRange::new(0, 20));
    assert_eq!(diagnostics[0].suggestions.len(), 1);
    assert_eq!(diagnostics[0].suggestions[0].message_id, "useSplice");
    assert_eq!(diagnostics[0].suggestions[0].fixes.len(), 3);
    assert_eq!(
        diagnostics[0].suggestions[0].fixes[0].range,
        TextRange::new(0, 7)
    );
    assert_eq!(
        diagnostics[0].suggestions[0].fixes[1].range,
        TextRange::new(13, 14)
    );
    assert_eq!(
        diagnostics[0].suggestions[0].fixes[2].range,
        TextRange::new(19, 20)
    );
}

#[test]
fn ignores_non_array_member_delete() {
    let mut node = array_delete_node();
    node.children
        .get_mut("argument")
        .unwrap()
        .children
        .get_mut("object")
        .unwrap()
        .type_texts = vec!["{ value: number }".to_owned()];

    let diagnostics = registry().run_rule("no-array-delete", &node).unwrap();

    assert!(diagnostics.is_empty());
}

#[test]
fn reports_for_in_array() {
    let diagnostics = registry()
        .run_rule("no-for-in-array", &for_in_array_node("readonly string[]"))
        .unwrap();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_name, "no-for-in-array");
    assert_eq!(diagnostics[0].message_id, "unexpected");
}

#[test]
fn ignores_for_in_record() {
    let diagnostics = registry()
        .run_rule("no-for-in-array", &for_in_array_node("{ value: number }"))
        .unwrap();

    assert!(diagnostics.is_empty());
}

#[test]
fn reports_for_in_array_literal() {
    let mut node = for_in_array_node("");
    node.children.get_mut("right").unwrap().kind = "ArrayExpression".to_owned();

    let diagnostics = registry().run_rule("no-for-in-array", &node).unwrap();

    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn lists_default_rule_meta() {
    let registry = registry();
    assert_eq!(
        registry.rule_names().collect::<Vec<_>>(),
        vec![
            "no-array-delete",
            "no-for-in-array",
            "await-thenable",
            "no-implied-eval",
            "no-mixed-enums",
            "no-unsafe-unary-minus",
            "only-throw-error",
            "prefer-find",
            "prefer-includes",
            "prefer-regexp-exec",
            "use-unknown-in-catch-callback-variable"
        ]
    );
    let metas = registry.metas();
    assert_eq!(metas[0].name, "no-array-delete");
    assert_eq!(metas[0].listeners, vec!["UnaryExpression"]);
    assert_eq!(
        metas[0].messages.get("unexpected").unwrap(),
        "Do not delete elements from an array-like value."
    );
    assert_eq!(metas[1].name, "no-for-in-array");
    assert_eq!(metas[1].listeners, vec!["ForInStatement"]);
}

fn registry() -> LintRuleRegistry {
    LintRuleRegistry::with_default_type_aware_rules()
}

fn array_delete_node() -> LintNode {
    LintNode {
        kind: "UnaryExpression".to_owned(),
        range: TextRange::new(0, 20),
        text: None,
        type_texts: Vec::new(),
        property_names: Vec::new(),
        fields: BTreeMap::from([("operator".to_owned(), json!("delete"))]),
        children: BTreeMap::from([(
            "argument".to_owned(),
            LintNode {
                kind: "MemberExpression".to_owned(),
                range: TextRange::new(7, 20),
                text: None,
                type_texts: Vec::new(),
                property_names: Vec::new(),
                fields: BTreeMap::from([("computed".to_owned(), json!(true))]),
                children: BTreeMap::from([
                    (
                        "object".to_owned(),
                        LintNode {
                            kind: "Identifier".to_owned(),
                            range: TextRange::new(7, 13),
                            text: Some("values".to_owned()),
                            type_texts: vec!["number[]".to_owned()],
                            property_names: Vec::new(),
                            fields: BTreeMap::new(),
                            children: BTreeMap::new(),
                            child_lists: BTreeMap::new(),
                        },
                    ),
                    (
                        "property".to_owned(),
                        LintNode {
                            kind: "Identifier".to_owned(),
                            range: TextRange::new(14, 19),
                            text: Some("index".to_owned()),
                            type_texts: Vec::new(),
                            property_names: Vec::new(),
                            fields: BTreeMap::new(),
                            children: BTreeMap::new(),
                            child_lists: BTreeMap::new(),
                        },
                    ),
                ]),
                child_lists: BTreeMap::new(),
            },
        )]),
        child_lists: BTreeMap::new(),
    }
}

fn for_in_array_node(right_type_text: &str) -> LintNode {
    LintNode {
        kind: "ForInStatement".to_owned(),
        range: TextRange::new(0, 42),
        text: None,
        type_texts: Vec::new(),
        property_names: Vec::new(),
        fields: BTreeMap::new(),
        children: BTreeMap::from([(
            "right".to_owned(),
            LintNode {
                kind: "Identifier".to_owned(),
                range: TextRange::new(18, 24),
                text: Some("values".to_owned()),
                type_texts: vec![right_type_text.to_owned()],
                property_names: Vec::new(),
                fields: BTreeMap::new(),
                children: BTreeMap::new(),
                child_lists: BTreeMap::new(),
            },
        )]),
        child_lists: BTreeMap::new(),
    }
}
