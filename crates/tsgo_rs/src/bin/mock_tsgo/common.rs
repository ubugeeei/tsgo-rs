use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

pub fn project(config_file_name: &str) -> Value {
    json!({
        "id": "p./workspace/tsconfig.json",
        "configFileName": config_file_name,
        "compilerOptions": { "strict": true, "module": "esnext" },
        "rootFiles": ["/workspace/src/index.ts"],
    })
}

pub fn snapshot(config_file_name: &str) -> Value {
    json!({
        "snapshot": "n0000000000000001",
        "projects": [project(config_file_name)],
        "changes": {
            "changedProjects": {
                "p./workspace/tsconfig.json": {
                    "changedFiles": ["/workspace/src/index.ts"],
                    "deletedFiles": []
                }
            },
            "removedProjects": []
        }
    })
}

pub fn symbol(name: &str) -> Value {
    json!({
        "id": "s0000000000000001",
        "name": name,
        "flags": 2,
        "checkFlags": 0,
        "declarations": ["1.3.80./workspace/src/index.ts"],
        "valueDeclaration": "1.3.80./workspace/src/index.ts",
    })
}

pub fn type_response(id: &str) -> Value {
    json!({
        "id": id,
        "flags": 262144,
        "objectFlags": 16,
        "symbol": "s0000000000000001",
        "texts": ["type-text"],
    })
}

pub fn signature() -> Value {
    json!({
        "id": "g0000000000000001",
        "flags": 1,
        "declaration": "1.3.80./workspace/src/index.ts",
        "typeParameters": ["t0000000000000002"],
        "parameters": ["s0000000000000001"],
        "thisParameter": "s0000000000000002",
    })
}

pub fn type_predicate() -> Value {
    json!({
        "kind": 1,
        "parameterIndex": 0,
        "parameterName": "value",
        "type": type_response("t0000000000000003"),
    })
}

pub fn index_info() -> Value {
    json!({
        "keyType": type_response("t0000000000000004"),
        "valueType": type_response("t0000000000000005"),
        "isReadonly": true,
    })
}

pub fn encoded(bytes: &[u8]) -> Value {
    json!({ "data": STANDARD.encode(bytes) })
}
