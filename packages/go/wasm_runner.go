package main

import "context"

// runWasm executes the mille WASI module (wasmBytes) in the given host
// directory (dir), forwarding args as WASI argv (without argv[0]).
// It returns the process exit code: 0 = clean, 1 = violations, 3 = config error.
func runWasm(_ context.Context, _ []byte, _ string, _ []string) int {
	panic("not implemented")
}
