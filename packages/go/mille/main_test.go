package main

import (
	"bytes"
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestRunWasm_WasmBytesEmbedded verifies the //go:embed in main.go
// worked and the .wasm file is non-empty.
func TestRunWasm_WasmBytesEmbedded(t *testing.T) {
	// Minimal valid WebAssembly module is 8 bytes (magic + version).
	if len(milleWasm) < 8 {
		t.Fatalf("milleWasm too small (%d bytes) — run `bash scripts/build-wasm.sh` first", len(milleWasm))
	}
}

// TestRunWasm_MissingConfig checks that a non-existent config path causes
// mille to exit with a non-zero code (3 = config error).
func TestRunWasm_MissingConfig(t *testing.T) {
	dir := t.TempDir() // empty dir → no mille.toml
	ctx := context.Background()

	code := runWasm(ctx, milleWasm, dir, []string{"check", "--config", "nonexistent.toml"})
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
	code := runWasm(ctx, milleWasm, dir, []string{"check"})
	if code != 0 {
		t.Errorf("expected exit code 0 (no violations), got %d", code)
	}
}

// TestHandleVersionFlag_Version checks that --version is intercepted and outputs
// a non-empty "mille <version>" line without forwarding to WASM.
func TestHandleVersionFlag_Version(t *testing.T) {
	var buf bytes.Buffer
	handled := handleVersionFlag([]string{"--version"}, &buf)
	if !handled {
		t.Fatal("expected handleVersionFlag to return true for --version")
	}
	out := buf.String()
	if !strings.HasPrefix(out, "mille ") {
		t.Errorf("expected output starting with 'mille ', got: %q", out)
	}
}

// TestHandleVersionFlag_ShortV checks that -V is also intercepted.
func TestHandleVersionFlag_ShortV(t *testing.T) {
	var buf bytes.Buffer
	handled := handleVersionFlag([]string{"-V"}, &buf)
	if !handled {
		t.Fatal("expected handleVersionFlag to return true for -V")
	}
}

// TestHandleVersionFlag_NotVersion verifies non-version args are not intercepted.
func TestHandleVersionFlag_NotVersion(t *testing.T) {
	var buf bytes.Buffer
	handled := handleVersionFlag([]string{"check"}, &buf)
	if handled {
		t.Error("expected handleVersionFlag to return false for 'check'")
	}
	if buf.Len() != 0 {
		t.Errorf("expected no output for 'check', got: %q", buf.String())
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
