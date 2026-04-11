import Foundation
import Testing
@testable import CorsaUtils

@Test func smoke() async throws {
    #expect(CorsaUtils.classifyTypeText("Promise<string> | null") == "nullish")
    #expect(CorsaUtils.splitTypeText("string | Promise<any>") == ["string", "Promise<any>"])
    #expect(CorsaUtils.isErrorLikeTypeTexts(["TypeError"]))
    #expect(CorsaUtils.hasUnsafeAnyFlow(["Promise<any>"], targetTexts: ["Promise<string>"]))
    let document = try CorsaVirtualDocument.untitled(path: "/demo.ts", languageID: "typescript", text: "const value = 1;")
    try document.splice(startLine: 0, startCharacter: 14, endLine: 0, endCharacter: 15, text: "2")
    #expect(document.text == "const value = 2;")
    #expect(document.version == 2)
}

@Test func apiClientCheckerPositionBindings() async throws {
    let root = URL(fileURLWithPath: "../../../..", relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath)).standardizedFileURL
    let binary = root.appending(path: "target/debug/mock_tsgo").path
    guard FileManager.default.fileExists(atPath: binary) else {
        return
    }
    let client = try CorsaTsgoApiClient(options: CorsaTsgoApiClientOptions(
        executable: binary,
        cwd: root.path,
        mode: .jsonrpc
    ))
    defer {
        try? client.close()
    }

    let snapshotJSON = try client.updateSnapshotJSON(paramsJSON: #"{"openProject":"/workspace/tsconfig.json"}"#)
    let snapshot = try JSONSerialization.jsonObject(with: Data(snapshotJSON.utf8)) as! [String: Any]
    let snapshotID = snapshot["snapshot"] as! String
    let projects = snapshot["projects"] as! [[String: Any]]
    let projectID = projects[0]["id"] as! String

    let typeJSON = try client.getTypeAtPositionJSON(
        snapshot: snapshotID,
        project: projectID,
        file: "/workspace/src/index.ts",
        position: 1
    )
    let type = try JSONSerialization.jsonObject(with: Data(typeJSON.utf8)) as! [String: Any]
    #expect(type["id"] as? String == "t0000000000000001")

    let symbolJSON = try client.getSymbolAtPositionJSON(
        snapshot: snapshotID,
        project: projectID,
        file: "/workspace/src/index.ts",
        position: 1
    )
    let symbol = try JSONSerialization.jsonObject(with: Data(symbolJSON.utf8)) as! [String: Any]
    #expect(symbol["name"] as? String == "value")
}
