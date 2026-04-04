import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import {
  createBinaryPackageManifest,
  createRootBindingPublishManifest,
  getNodeBindingTargets,
  parseTargetTriple,
} from "../../scripts/npm_release_utils.ts";

const nodeBindingManifest = JSON.parse(
  readFileSync(resolve(process.cwd(), "src/bindings/nodejs/corsa_node/package.json"), "utf8"),
) as {
  files: string[];
  name: string;
  version: string;
};

describe("npm release utils", () => {
  it("includes the configured native binding targets", () => {
    expect(
      getNodeBindingTargets(nodeBindingManifest).map(
        (target: { platformArchABI: string }) => target.platformArchABI,
      ),
    ).toEqual(["win32-x64-msvc", "darwin-x64", "linux-x64-gnu", "darwin-arm64"]);
  });

  it("deduplicates repeated native binding targets without changing publish order", () => {
    expect(
      getNodeBindingTargets({
        ...nodeBindingManifest,
        napi: {
          triples: {
            additional: [
              "x86_64-unknown-linux-gnu",
              "aarch64-apple-darwin",
              "x86_64-unknown-linux-gnu",
            ],
          },
        },
      }).map((target: { platformArchABI: string }) => target.platformArchABI),
    ).toEqual(["win32-x64-msvc", "darwin-x64", "linux-x64-gnu", "darwin-arm64"]);
  });

  it("creates binary package manifests with libc metadata when needed", () => {
    const target = parseTargetTriple("x86_64-unknown-linux-gnu");
    expect(
      createBinaryPackageManifest(
        nodeBindingManifest,
        nodeBindingManifest.version,
        target,
        "corsa_node.linux-x64-gnu.node",
      ),
    ).toMatchObject({
      cpu: ["x64"],
      files: ["corsa_node.linux-x64-gnu.node"],
      libc: ["glibc"],
      main: "corsa_node.linux-x64-gnu.node",
      name: `${nodeBindingManifest.name}-linux-x64-gnu`,
      os: ["linux"],
      version: "0.1.0",
    });
  });

  it("keeps the root package JS-only and wires optional dependencies", () => {
    const manifest = createRootBindingPublishManifest(
      nodeBindingManifest,
      nodeBindingManifest.version,
      [parseTargetTriple("x86_64-unknown-linux-gnu"), parseTargetTriple("aarch64-apple-darwin")],
    );

    expect(manifest.files).not.toContain("*.node");
    expect(manifest.optionalDependencies).toEqual({
      [`${nodeBindingManifest.name}-darwin-arm64`]: "0.1.0",
      [`${nodeBindingManifest.name}-linux-x64-gnu`]: "0.1.0",
    });
  });
});
