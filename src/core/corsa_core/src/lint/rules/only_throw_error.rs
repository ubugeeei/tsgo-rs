use super::super::{LintNode, RuleContext, RuleMessage, RustLintRule};
use crate::lint::helpers::is_error_like_node;

/// Type-aware rule that requires thrown values to be Error-like.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnlyThrowErrorRule;

const MESSAGES: &[RuleMessage] = &[RuleMessage {
    id: "unexpected",
    description: "Only Error-like values should be thrown.",
}];
const LISTENERS: &[&str] = &["ThrowStatement"];

impl RustLintRule for OnlyThrowErrorRule {
    fn name(&self) -> &'static str {
        "only-throw-error"
    }

    fn docs_description(&self) -> &'static str {
        "Require thrown values to be Error-like."
    }

    fn messages(&self) -> &'static [RuleMessage] {
        MESSAGES
    }

    fn listeners(&self) -> &'static [&'static str] {
        LISTENERS
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode) {
        if node.kind != "ThrowStatement" {
            return;
        }
        let Some(argument) = node.child("argument") else {
            return;
        };
        if !is_error_like_node(argument) {
            ctx.report("unexpected", node.range);
        }
    }
}
