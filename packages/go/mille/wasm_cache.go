package main

import (
	"context"
	"os"
	"path/filepath"

	"github.com/tetratelabs/wazero"
)

// compilationCacheDir returns the directory used for the wazero compilation
// cache. Returns an empty string if the user cache directory cannot be
// determined.
//
// Typical paths:
//
//	Linux:  ~/.cache/mille/wazero/
//	macOS:  ~/Library/Caches/mille/wazero/
func compilationCacheDir() string {
	base, err := os.UserCacheDir()
	if err != nil {
		return ""
	}
	return filepath.Join(base, "mille", "wazero")
}

// newRuntime creates a wazero runtime. If cacheDir is non-empty and a
// file-based CompilationCache can be created there, the cache is attached to
// the runtime so that subsequent runs reuse the compiled machine code.
//
// The returned closeFn must be called when the runtime is no longer needed
// (it closes both the runtime and, if present, the cache).
func newRuntime(ctx context.Context, cacheDir string) (rt wazero.Runtime, closeFn func()) {
	if cacheDir != "" {
		if err := os.MkdirAll(cacheDir, 0o755); err == nil {
			// NOTE: cache must outlive the runtime — defer order matters.
			//       cache.Close is deferred first so it runs last (LIFO).
			if cache, err := wazero.NewCompilationCacheWithDir(cacheDir); err == nil {
				rt = wazero.NewRuntimeWithConfig(ctx, wazero.NewRuntimeConfig().WithCompilationCache(cache))
				return rt, func() {
					rt.Close(ctx)
					cache.Close(ctx)
				}
			}
		}
	}
	rt = wazero.NewRuntime(ctx)
	return rt, func() { rt.Close(ctx) }
}
