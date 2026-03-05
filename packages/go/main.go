// Package main is the Go wrapper for mille, using the WASI binary embedded in
// the millewasm module.
//
// Install:
//
//	go install github.com/makinzm/mille/packages/go@latest
//
// mille.wasm is managed in the github.com/makinzm/mille/packages/wasm module
// and embedded there via //go:embed. No copies are committed here.
// No network access or external binary is needed at runtime.
package main

import (
	"context"
	"fmt"
	"os"

	millewasm "github.com/makinzm/mille/packages/wasm"
)

func main() {
	ctx := context.Background()

	cwd, err := os.Getwd()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}

	// NOTE: os.Args[0] is the Go binary name; runWasm prepends "mille" as the
	// WASI argv[0], so we pass only the user-facing arguments (os.Args[1:]).
	code := runWasm(ctx, millewasm.Wasm, cwd, os.Args[1:])
	os.Exit(code)
}
