package main

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	millewasm "github.com/makinzm/mille/packages/wasm"
)

// TestRunWasm_WasmBytesEmbedded verifies the //go:embed in the millewasm
// module worked and the .wasm file is non-empty.
func TestRunWasm_WasmBytesEmbedded(t *testing.T) {
	// Minimal valid WebAssembly module is 8 bytes (magic + version).
	if len(millewasm.Wasm) < 8 {
		t.Fatalf("millewasm.Wasm too small (%d bytes) — run `bash scripts/build-wasm.sh` first", len(millewasm.Wasm))
	}
}

// TestRunWasm_MissingConfig checks that a non-existent config path causes
// mille to exit with a non-zero code (3 = config error).
func TestRunWasm_MissingConfig(t *testing.T) {
	dir := t.TempDir() // empty dir → no mille.toml
	ctx := context.Background()

	code := runWasm(ctx, millewasm.Wasm, dir, []string{"check", "--config", "nonexistent.toml"})
	if code == 0 {
		t.Errorf("expected non-zero exit code for missing config, got 0")
	}
}

// TestRunWasm_SelfCheck runs mille check against packages/go itself.
// Expects exit code 0 (no violations).
func TestRunWasm_SelfCheck(t *testing.T) {
	dir, err := findDirWithMilleToml(".")
	if err != nil {
		t.Skip("cannot find a directory with mille.toml:", err)
	}

	ctx := context.Background()
	code := runWasm(ctx, millewasm.Wasm, dir, []string{"check"})
	if code != 0 {
		t.Errorf("expected exit code 0 (no violations), got %d", code)
	}
}

// findDirWithMilleToml searches start and its parents for a mille.toml file.
func findDirWithMilleToml(start string) (string, error) {
	abs, err := filepath.Abs(start)
	if err != nil {
		return "", err
	}
	for {
		if _, err := os.Stat(filepath.Join(abs, "mille.toml")); err == nil {
			return abs, nil
		}
		parent := filepath.Dir(abs)
		if parent == abs {
			break
		}
		abs = parent
	}
	return "", os.ErrNotExist
}
