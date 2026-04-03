mod support;

use std::{fs, process::Command};

use corsa_bind_rs::{
    api::{ApiClient, ApiMode, ApiSpawnConfig, UpdateSnapshotParams},
    runtime::block_on,
};
use tempfile::tempdir;

#[test]
fn real_tsgo_reports_actual_type_errors() {
    let Some(binary) = support::resolved_real_tsgo_binary() else {
        return;
    };
    let project = fixture(&[
        (
            "tsconfig.json",
            r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
        ),
        (
            "src/index.ts",
            r#"const amount: number = "oops";
export const fixed = amount.toFixed(2);
"#,
        ),
    ]);
    let output = Command::new(binary)
        .arg("--pretty")
        .arg("false")
        .arg("-p")
        .arg(project.path().join("tsconfig.json"))
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected tsgo to reject the fixture"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "src/index.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'."
        ),
        "unexpected compiler output: {stdout}"
    );
}

#[test]
fn real_tsgo_infers_contextual_and_generic_types() {
    block_on(async {
        let Some(binary) = support::resolved_real_tsgo_binary() else {
            return;
        };
        let project = fixture(&[
            (
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "src/index.ts",
                r#"const numbers = [1, 2, 3];
const inferred = numbers.map((value) => value.toFixed(2));
"#,
            ),
        ]);
        let file = project.path().join("src/index.ts");
        let file_text = fs::read_to_string(&file).unwrap();
        let file_wire = file.display().to_string();
        let config_wire = project.path().join("tsconfig.json").display().to_string();
        let value_pos = u32::try_from(file_text.find("(value) =>").unwrap() + 1).unwrap();
        let inferred_pos = u32::try_from(file_text.find("inferred =").unwrap() + 1).unwrap();

        for mode in [ApiMode::SyncMsgpackStdio, ApiMode::AsyncJsonRpcStdio] {
            let client = ApiClient::spawn(
                ApiSpawnConfig::new(binary.clone())
                    .with_mode(mode)
                    .with_cwd(project.path()),
            )
            .await
            .unwrap();
            let snapshot = client
                .update_snapshot(UpdateSnapshotParams {
                    open_project: Some(config_wire.clone()),
                    file_changes: None,
                })
                .await
                .unwrap();
            let project = snapshot.projects[0].id.clone();

            let value_type = client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    file_wire.as_str(),
                    value_pos,
                )
                .await
                .unwrap()
                .unwrap();
            let value_rendered = client
                .type_to_string(
                    snapshot.handle.clone(),
                    project.clone(),
                    value_type.id,
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                value_rendered, "number",
                "unexpected callback parameter type for {mode:?}"
            );

            let inferred_type = client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    file_wire.as_str(),
                    inferred_pos,
                )
                .await
                .unwrap()
                .unwrap();
            let inferred_rendered = client
                .type_to_string(
                    snapshot.handle.clone(),
                    project.clone(),
                    inferred_type.id,
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                inferred_rendered, "string[]",
                "unexpected inferred array type for {mode:?}"
            );

            snapshot.release().await.unwrap();
            client.close().await.unwrap();
        }
    });
}

fn fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempdir().unwrap();
    for (relative, contents) in files {
        let path = project.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
    project
}
