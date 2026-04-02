import {
  assertPublishablePackageManifest,
  getPackageVersion,
  isNpmPackageVersionPublished,
  publishPackedTarball,
  sleep,
  typescriptOxlintPackage,
  withStagedNodeBindingPackages,
} from "./npm_release_utils.ts";
import { fail } from "./shared.ts";

const delayMs = Number(process.env.NPM_PUBLISH_DELAY_MS ?? "10000");
const distTag = process.env.NPM_DIST_TAG?.trim() || undefined;
const artifactsDir = process.env.NAPI_ARTIFACTS_DIR?.trim() || undefined;
const requireAllTargets = process.env.NAPI_REQUIRE_ALL_TARGETS?.trim() === "0" ? false : true;
const startAt = process.env.NPM_PUBLISH_START_AT?.trim() || undefined;

async function main(): Promise<void> {
  await withStagedNodeBindingPackages(
    { artifactsDir, requireAllTargets },
    async ({ binaryPackages, rootPackage }) => {
      const releasePackages = [...binaryPackages, rootPackage, typescriptOxlintPackage];
      if (startAt && !releasePackages.some((pkg) => pkg.name === startAt)) {
        throw new Error(`Unknown NPM_PUBLISH_START_AT package: ${startAt}`);
      }

      let started = !startAt;

      for (const [index, pkg] of releasePackages.entries()) {
        if (!started) {
          started = pkg.name === startAt;
          if (!started) {
            continue;
          }
        }

        assertPublishablePackageManifest(pkg);
        const version = getPackageVersion(pkg);

        if (await isNpmPackageVersionPublished(pkg, version)) {
          console.log(`npm package ${pkg.name}@${version} already exists; skipping`);
        } else {
          try {
            publishPackedTarball(pkg, { tag: distTag });
          } catch (error) {
            if (await isNpmPackageVersionPublished(pkg, version)) {
              console.log(`npm package ${pkg.name}@${version} was published concurrently; skipping`);
            } else {
              throw error;
            }
          }
        }

        if (index + 1 < releasePackages.length && delayMs > 0) {
          await sleep(delayMs);
        }
      }
    },
  );
}

await main().catch(fail);
