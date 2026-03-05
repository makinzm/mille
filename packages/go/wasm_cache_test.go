package main

import (
	"strings"
	"testing"
)

// TestCompilationCacheDir_ContainsMille verifies that compilationCacheDir
// returns a path that includes "mille" as a subdirectory component.
func TestCompilationCacheDir_ContainsMille(t *testing.T) {
	dir := compilationCacheDir()
	if dir == "" {
		t.Skip("UserCacheDir not available in this environment")
	}
	if !strings.Contains(dir, "mille") {
		t.Errorf("compilationCacheDir() = %q, want path containing \"mille\"", dir)
	}
}

// TestCompilationCacheDir_ContainsWazero verifies the cache is namespaced
// under a wazero subdirectory to avoid conflicts with other tools.
func TestCompilationCacheDir_ContainsWazero(t *testing.T) {
	dir := compilationCacheDir()
	if dir == "" {
		t.Skip("UserCacheDir not available in this environment")
	}
	if !strings.Contains(dir, "wazero") {
		t.Errorf("compilationCacheDir() = %q, want path containing \"wazero\"", dir)
	}
}
