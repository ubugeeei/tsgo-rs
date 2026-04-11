package corsautils

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestUtilsBindings(t *testing.T) {
	if got := ClassifyTypeText("Promise<string> | null"); got != "nullish" {
		t.Fatalf("classify = %q", got)
	}
	split := SplitTypeText("string | Promise<any>")
	if len(split) != 2 || split[1] != "Promise<any>" {
		t.Fatalf("split = %#v", split)
	}
	if !IsErrorLikeTypeTexts([]string{"TypeError"}, nil) {
		t.Fatal("expected error-like detection")
	}
	if !HasUnsafeAnyFlow([]string{"Promise<any>"}, []string{"Promise<string>"}) {
		t.Fatal("expected unsafe any flow detection")
	}
}

func TestVirtualDocumentBindings(t *testing.T) {
	document, err := NewUntitledVirtualDocument("/demo.ts", "typescript", "const value = 1;")
	if err != nil {
		t.Fatalf("new untitled: %v", err)
	}
	defer document.Close()
	if err := document.Splice(0, 14, 0, 15, "2"); err != nil {
		t.Fatalf("splice: %v", err)
	}
	if got := document.Text(); got != "const value = 2;" {
		t.Fatalf("text = %q", got)
	}
	if got := document.Version(); got != 2 {
		t.Fatalf("version = %d", got)
	}
}

func TestApiClientCheckerPositionBindings(t *testing.T) {
	root, err := filepath.Abs("../../../..")
	if err != nil {
		t.Fatalf("workspace root: %v", err)
	}
	binary := filepath.Join(root, "target", "debug", "mock_tsgo")
	if _, err := os.Stat(binary); err != nil {
		t.Skip("mock_tsgo binary is not built")
	}
	client, err := NewApiClient(ApiClientOptions{
		Executable: binary,
		Cwd:        root,
		Mode:       ApiModeJSONRPC,
	})
	if err != nil {
		t.Fatalf("new api client: %v", err)
	}
	defer client.Close()

	snapshotJSON, err := client.UpdateSnapshotJSON(`{"openProject":"/workspace/tsconfig.json"}`)
	if err != nil {
		t.Fatalf("update snapshot: %v", err)
	}
	var snapshot struct {
		Snapshot string `json:"snapshot"`
		Projects []struct {
			ID string `json:"id"`
		} `json:"projects"`
	}
	if err := json.Unmarshal([]byte(snapshotJSON), &snapshot); err != nil {
		t.Fatalf("decode snapshot: %v", err)
	}
	project := snapshot.Projects[0].ID

	typeJSON, err := client.GetTypeAtPositionJSON(snapshot.Snapshot, project, "/workspace/src/index.ts", 1)
	if err != nil {
		t.Fatalf("get type at position: %v", err)
	}
	var typ struct {
		ID string `json:"id"`
	}
	if err := json.Unmarshal([]byte(typeJSON), &typ); err != nil {
		t.Fatalf("decode type: %v", err)
	}
	if typ.ID != "t0000000000000001" {
		t.Fatalf("type id = %q", typ.ID)
	}

	symbolJSON, err := client.GetSymbolAtPositionJSON(snapshot.Snapshot, project, "/workspace/src/index.ts", 1)
	if err != nil {
		t.Fatalf("get symbol at position: %v", err)
	}
	var symbol struct {
		Name string `json:"name"`
	}
	if err := json.Unmarshal([]byte(symbolJSON), &symbol); err != nil {
		t.Fatalf("decode symbol: %v", err)
	}
	if symbol.Name != "value" {
		t.Fatalf("symbol name = %q", symbol.Name)
	}
}
