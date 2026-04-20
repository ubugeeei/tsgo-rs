use super::{
    AwaitThenableRule, LintDiagnostic, LintNode, NoArrayDeleteRule, NoForInArrayRule,
    NoImpliedEvalRule, NoMixedEnumsRule, NoUnsafeUnaryMinusRule, OnlyThrowErrorRule,
    PreferFindRule, PreferIncludesRule, PreferRegexpExecRule, RuleContext, RuleMeta, RustLintRule,
    UseUnknownInCatchCallbackVariableRule,
};

#[derive(Default)]
pub struct LintRuleRegistry {
    rules: Vec<Box<dyn RustLintRule>>,
}

impl LintRuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rule(mut self, rule: impl RustLintRule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn with_default_type_aware_rules() -> Self {
        Self::new()
            .with_rule(NoArrayDeleteRule)
            .with_rule(NoForInArrayRule)
            .with_rule(AwaitThenableRule)
            .with_rule(NoImpliedEvalRule)
            .with_rule(NoMixedEnumsRule)
            .with_rule(NoUnsafeUnaryMinusRule)
            .with_rule(OnlyThrowErrorRule)
            .with_rule(PreferFindRule)
            .with_rule(PreferIncludesRule)
            .with_rule(PreferRegexpExecRule)
            .with_rule(UseUnknownInCatchCallbackVariableRule)
    }

    pub fn rule_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.rules.iter().map(|rule| rule.name())
    }

    pub fn metas(&self) -> Vec<RuleMeta> {
        self.rules.iter().map(|rule| rule.meta()).collect()
    }

    pub fn run_rule(&self, rule_name: &str, node: &LintNode) -> Option<Vec<LintDiagnostic>> {
        let rule = self
            .rules
            .iter()
            .find(|candidate| candidate.name() == rule_name)?;
        let mut ctx = RuleContext::new(rule.as_ref());
        rule.check(&mut ctx, node);
        Some(ctx.finish())
    }
}

pub fn run_default_type_aware_rule(
    rule_name: &str,
    node: &LintNode,
) -> Option<Vec<LintDiagnostic>> {
    LintRuleRegistry::with_default_type_aware_rules().run_rule(rule_name, node)
}
