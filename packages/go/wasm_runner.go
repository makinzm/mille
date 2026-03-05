package main

import (
	"context"
	"os"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
	"github.com/tetratelabs/wazero/sys"
)

// runWasm executes the mille WASI module (wasmBytes) with the given host
// directory (dir) premounted as the WASI root "/", and forwards args as
// WASI argv[1:] (argv[0] is always "mille").
//
// Returns the process exit code:
//
//	0  — no violations
//	1  — at least one error-severity violation
//	3  — configuration or runtime error
func runWasm(ctx context.Context, wasmBytes []byte, dir string, args []string) int {
	rt, closeRT := newRuntime(ctx, compilationCacheDir())
	defer closeRT()

	// NOTE: mille.wasm targets wasm32-wasip1 (WASI Preview 1).
	//       MustInstantiate registers the wasi_snapshot_preview1 host module
	//       so that the WASI syscalls (fd_read, path_open, proc_exit, …) are
	//       satisfied without any additional configuration.
	wasi_snapshot_preview1.MustInstantiate(ctx, rt)

	// Mount the host CWD as "/" inside WASI so that paths like "mille.toml"
	// and "src/domain/**" resolve correctly relative to the project root.
	fsCfg := wazero.NewFSConfig().WithDirMount(dir, "/")

	// NOTE: os.Args[0] is the Go binary name; prepend "mille" as the WASI
	//       argv[0] so that clap's binary-name detection works as expected.
	wasiArgs := append([]string{"mille"}, args...)

	cfg := wazero.NewModuleConfig().
		WithStdin(os.Stdin).
		WithStdout(os.Stdout).
		WithStderr(os.Stderr).
		WithArgs(wasiArgs...).
		WithFSConfig(fsCfg).
		WithSysNanosleep()

	compiled, err := rt.CompileModule(ctx, wasmBytes)
	if err != nil {
		return 3
	}

	_, err = rt.InstantiateModule(ctx, compiled, cfg)
	if err != nil {
		if exitErr, ok := err.(*sys.ExitError); ok {
			return int(exitErr.ExitCode())
		}
		return 3
	}
	return 0
}
