import { npmPackages, publishPackedTarball, sleep } from "./npm_release_utils.mjs";

const delayMs = Number(process.env.NPM_PUBLISH_DELAY_MS ?? "10000");
const distTag = process.env.NPM_DIST_TAG?.trim() || undefined;

for (const [index, pkg] of npmPackages.entries()) {
  publishPackedTarball(pkg, { tag: distTag });
  if (index + 1 < npmPackages.length && delayMs > 0) {
    await sleep(delayMs);
  }
}
