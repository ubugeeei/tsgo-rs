import { runDistributedOrchestratorExample } from "./node/distributed_orchestrator.ts";
import { runMinimalStartExample } from "./node/minimal_start.ts";
import { runMockClientExample } from "./node/mock_client.ts";
import { runRawCallsExample } from "./node/raw_calls.ts";
import { runUnsafeTypeFlowExample } from "./node/unsafe_type_flow.ts";
import { runVirtualDocumentExample } from "./node/virtual_document.ts";
import customRulesConfig from "./typescript_oxlint/custom_rules_config.ts";
import { typescriptOxlintCustomPlugin } from "./typescript_oxlint/custom_plugin.ts";
import { noStringPlusNumberRule } from "./typescript_oxlint/custom_rule.ts";
import nativeRulesConfig from "./typescript_oxlint/native_rules_config.ts";

function ruleCount(config: readonly unknown[]): number {
  return config.reduce<number>((count, entry) => {
    const rules = (entry as { rules?: Record<string, unknown> }).rules ?? {};
    return count + Object.keys(rules).length;
  }, 0);
}

const result = {
  customPluginRuleNames: Object.keys(
    (typescriptOxlintCustomPlugin as { rules?: Record<string, unknown> }).rules ?? {},
  ),
  customRuleDocs:
    (noStringPlusNumberRule as { meta?: { docs?: { description?: string } } }).meta?.docs
      ?.description ?? null,
  customRuleEntries: ruleCount(customRulesConfig),
  distributedOrchestrator: runDistributedOrchestratorExample(),
  minimalStart: runMinimalStartExample(),
  mockClient: runMockClientExample(),
  nativeRuleEntries: ruleCount(nativeRulesConfig),
  rawCalls: runRawCallsExample(),
  unsafeTypeFlow: runUnsafeTypeFlowExample(),
  virtualDocument: runVirtualDocumentExample(),
};

console.log(JSON.stringify(result, null, 2));
