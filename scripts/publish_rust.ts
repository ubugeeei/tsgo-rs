import {
  assertCommandSucceeded,
  fail,
  rootDir,
  runCommandCapture,
  sleep,
} from "./shared.ts";

const crateNames = [
  "tsgo_rs_core",
  "tsgo_rs_runtime",
  "tsgo_rs_jsonrpc",
  "tsgo_rs_client",
  "tsgo_rs_lsp",
  "tsgo_rs_orchestrator",
  "tsgo_rs",
] as const;

const delayMs = Number(process.env.CARGO_PUBLISH_DELAY_MS ?? "30000");
const startAt = process.env.CARGO_PUBLISH_START_AT?.trim() || undefined;
const cratesIoUserAgent = "tsgo-rs-release-script";

interface CargoMetadata {
  packages: Array<{
    name: string;
    version: string;
  }>;
}

interface CrateSpec {
  name: (typeof crateNames)[number];
  version: string;
}

function getPublicCrates(): CrateSpec[] {
  const metadata = assertCommandSucceeded(
    "cargo",
    ["metadata", "--format-version", "1", "--no-deps"],
    runCommandCapture("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
      cwd: rootDir,
    }),
  );
  const packages = (JSON.parse(metadata.stdout) as CargoMetadata).packages;
  return crateNames.map((name) => {
    const pkg = packages.find((candidate) => candidate.name === name);
    if (!pkg) {
      throw new Error(`Unable to find cargo metadata for ${name}`);
    }
    return { name, version: pkg.version };
  });
}

async function isCrateVersionPublished(crate: CrateSpec): Promise<boolean> {
  const response = await fetch(`https://crates.io/api/v1/crates/${crate.name}/${crate.version}`, {
    headers: {
      "user-agent": cratesIoUserAgent,
    },
  });
  if (response.status === 404) {
    return false;
  }
  if (!response.ok) {
    throw new Error(
      `Failed to query crates.io for ${crate.name}@${crate.version}: ${response.status} ${response.statusText}`,
    );
  }
  return true;
}

function parseRetryAfter(output: string): Date | null {
  const match = output.match(/Please try again after (.+?) and see https:\/\/crates\.io\/docs\/rate-limits/i);
  if (!match) {
    return null;
  }
  const retryAt = new Date(match[1]);
  return Number.isNaN(retryAt.getTime()) ? null : retryAt;
}

async function main(): Promise<void> {
  const crates = getPublicCrates();
  if (startAt && !crates.some((crate) => crate.name === startAt)) {
    throw new Error(`Unknown CARGO_PUBLISH_START_AT crate: ${startAt}`);
  }

  let started = !startAt;

  for (const [index, crate] of crates.entries()) {
    if (!started) {
      started = crate.name === startAt;
      if (!started) {
        continue;
      }
    }

    if (await isCrateVersionPublished(crate)) {
      console.log(`crates.io already has ${crate.name}@${crate.version}; skipping`);
      continue;
    }

    for (;;) {
      const result = runCommandCapture("cargo", ["publish", "--locked", "-p", crate.name], {
        cwd: rootDir,
      });
      if (result.status === 0) {
        break;
      }

      if (await isCrateVersionPublished(crate)) {
        console.log(`crates.io already has ${crate.name}@${crate.version}; skipping`);
        break;
      }

      const output = `${result.stdout}\n${result.stderr}`;
      const retryAt = parseRetryAfter(output);
      if (!retryAt) {
        assertCommandSucceeded("cargo", ["publish", "--locked", "-p", crate.name], result);
      }

      const waitMs = retryAt.getTime() - Date.now();
      const waitSeconds = Math.max(0, Math.ceil(waitMs / 1000));
      console.log(
        `crates.io rate-limited new crate publishes for ${crate.name}; retrying at ${retryAt.toISOString()} after waiting ${waitSeconds}s`,
      );
      if (waitMs > 0) {
        await sleep(waitMs);
      }
    }

    if (index + 1 < crates.length && delayMs > 0) {
      await sleep(delayMs);
    }
  }
}

await main().catch(fail);
