import { spawnSync } from "node:child_process";

interface ExampleCommand {
  readonly args: readonly string[];
  readonly name: string;
}

const smokeExamples: readonly ExampleCommand[] = [
  { name: "minimal_start", args: ["run", "-p", "tsgo_rs", "--example", "minimal_start"] },
  { name: "virtual_document", args: ["run", "-p", "tsgo_rs", "--example", "virtual_document"] },
  { name: "mock_client", args: ["run", "-p", "tsgo_rs", "--example", "mock_client"] },
  {
    name: "filesystem_callbacks",
    args: ["run", "-p", "tsgo_rs", "--example", "filesystem_callbacks"],
  },
  { name: "lsp_overlay", args: ["run", "-p", "tsgo_rs", "--example", "lsp_overlay"] },
  {
    name: "orchestrator_cache",
    args: ["run", "-p", "tsgo_rs", "--example", "orchestrator_cache"],
  },
  { name: "observer_events", args: ["run", "-p", "tsgo_rs", "--example", "observer_events"] },
];

const realExamples: readonly ExampleCommand[] = [
  { name: "real_snapshot", args: ["run", "-p", "tsgo_rs", "--example", "real_snapshot"] },
];

const experimentalExamples: readonly ExampleCommand[] = [
  {
    name: "distributed_orchestrator",
    args: [
      "run",
      "-p",
      "tsgo_rs",
      "--features",
      "experimental-distributed",
      "--example",
      "distributed_orchestrator",
    ],
  },
];

const groups = {
  experimental: experimentalExamples,
  real: realExamples,
  smoke: smokeExamples,
} satisfies Record<string, readonly ExampleCommand[]>;

const group = process.argv[2] ?? "smoke";
const examples = groups[group as keyof typeof groups];

if (!examples) {
  console.error(
    `unknown rust example group: ${group}. Expected one of ${Object.keys(groups).join(", ")}`,
  );
  process.exit(1);
}

for (const example of examples) {
  console.error(`==> cargo ${example.args.join(" ")} (${example.name})`);
  const result = spawnSync("cargo", [...example.args], {
    cwd: process.cwd(),
    stdio: "inherit",
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
