use super::{LintDiagnostic, LintNode, LintSuggestion, RuleMessage, RuleMeta, TextRange};

/// A Rust-authored type-aware lint rule.
///
/// The host adapter owns AST traversal and type lookups, then sends compact
/// [`LintNode`] facts into the Rust rule. This keeps the final public surface as
/// an Oxlint JS plugin while allowing common rules to live on the Rust hot path.
pub trait RustLintRule: Send + Sync {
    fn name(&self) -> &'static str;

    fn docs_description(&self) -> &'static str;

    fn messages(&self) -> &'static [RuleMessage];

    fn listeners(&self) -> &'static [&'static str];

    fn has_suggestions(&self) -> bool {
        false
    }

    fn requires_type_texts(&self) -> bool {
        true
    }

    fn check(&self, ctx: &mut RuleContext<'_>, node: &LintNode);

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            name: self.name().to_owned(),
            docs_description: self.docs_description().to_owned(),
            messages: self
                .messages()
                .iter()
                .map(|message| (message.id.to_owned(), message.description.to_owned()))
                .collect(),
            has_suggestions: self.has_suggestions(),
            listeners: self
                .listeners()
                .iter()
                .map(|listener| (*listener).to_owned())
                .collect(),
            requires_type_texts: self.requires_type_texts(),
        }
    }
}

pub struct RuleContext<'a> {
    rule: &'a dyn RustLintRule,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'a> RuleContext<'a> {
    pub(crate) fn new(rule: &'a dyn RustLintRule) -> Self {
        Self {
            rule,
            diagnostics: Vec::new(),
        }
    }

    pub fn report(&mut self, message_id: &'static str, range: TextRange) {
        self.report_with_suggestions(message_id, range, Vec::new());
    }

    pub fn report_with_suggestions(
        &mut self,
        message_id: &'static str,
        range: TextRange,
        suggestions: Vec<LintSuggestion>,
    ) {
        self.diagnostics.push(LintDiagnostic {
            rule_name: self.rule.name().to_owned(),
            message_id: message_id.to_owned(),
            message: self.message(message_id),
            range,
            suggestions,
        });
    }

    pub(crate) fn finish(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }

    fn message(&self, message_id: &str) -> String {
        self.rule
            .messages()
            .iter()
            .find(|message| message.id == message_id)
            .map(|message| message.description)
            .unwrap_or(message_id)
            .to_owned()
    }
}
