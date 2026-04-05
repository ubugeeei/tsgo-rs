import { assertReleaseTagMatchesWorkspace } from "./release_manifest.ts";
import { fail } from "./shared.ts";

function main(): void {
  const tag = process.argv[2] ?? process.env.RELEASE_TAG ?? process.env.GITHUB_REF_NAME;
  if (!tag) {
    throw new Error("Expected a release tag via argv, RELEASE_TAG, or GITHUB_REF_NAME");
  }

  const version = assertReleaseTagMatchesWorkspace(tag);
  console.log(`release tag ${tag} matches workspace version ${version}`);
}

try {
  main();
} catch (error) {
  fail(error);
}
