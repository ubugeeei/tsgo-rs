import { TsgoApiClient } from "@corsa-bind/node";

import { assertExists, isMain, realBinary, realDataset, workspaceRoot } from "../shared.ts";

export function runRealSnapshotExample() {
  assertExists(realBinary, "real tsgo binary", "run `vp run -w build_tsgo` first");
  assertExists(realDataset, "pinned tsgo dataset", "run `vp run -w sync_origin` first");

  const client = TsgoApiClient.spawn({
    executable: realBinary,
    cwd: workspaceRoot,
    mode: "msgpack",
  });

  let snapshotHandle: string | undefined;

  try {
    client.initialize();
    const config = client.parseConfigFile(realDataset);
    const snapshot = client.updateSnapshot({ openProject: realDataset });
    snapshotHandle = snapshot.snapshot;
    const project = snapshot.projects[0];
    if (!project) {
      throw new Error("real tsgo example did not return a project");
    }

    const primaryFile =
      config.fileNames.find((fileName) => !fileName.endsWith(".d.ts")) ?? config.fileNames[0];
    if (!primaryFile) {
      throw new Error("real tsgo example did not find a source file");
    }

    const sourceFile = client.getSourceFile(snapshot.snapshot, project.id, primaryFile);
    const stringType = client.getStringType(snapshot.snapshot, project.id);

    return {
      fileName: primaryFile,
      projectId: project.id,
      sourceLength: sourceFile?.byteLength ?? 0,
      stringTypeText: client.typeToString(snapshot.snapshot, project.id, stringType.id),
    };
  } finally {
    if (snapshotHandle) {
      client.releaseHandle(snapshotHandle);
    }
    client.close();
  }
}

if (isMain(import.meta.url)) {
  console.log(JSON.stringify(runRealSnapshotExample(), null, 2));
}
