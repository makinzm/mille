// Package main is the Go wrapper for mille, embedding the WASI binary.
//
// Install:
//
//	go install github.com/makinzm/mille/packages/go@latest
//
// The mille.wasm file is embedded at compile time via //go:embed.
// No network access or external binary is needed at runtime.
package main

import (
	"context"
	_ "embed"
	"fmt"
	"os"
)

//go:embed mille.wasm
var milleWasm []byte

func main() {
	ctx := context.Background()

	cwd, err := os.Getwd()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}

	// NOTE: os.Args[0] is the Go binary name; runWasm prepends "mille" as the
	// WASI argv[0], so we pass only the user-facing arguments (os.Args[1:]).
	code := runWasm(ctx, milleWasm, cwd, os.Args[1:])
	os.Exit(code)
}
