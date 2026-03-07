// Package main is the Go wrapper for mille, embedding mille.wasm directly.
//
// Install:
//
//	go install github.com/makinzm/mille/packages/go@latest
//
// mille.wasm is embedded at build time via //go:embed.
// No network access or external binary is needed at runtime.
package main

import (
	"context"
	_ "embed"
	"fmt"
	"io"
	"os"
	"runtime/debug"
	"strings"
)

//go:embed mille.wasm
var milleWasm []byte

// getVersion returns the module version from build info (e.g. "0.0.9" for
// go install @v0.0.9). Falls back to "dev" in local builds.
func getVersion() string {
	if info, ok := debug.ReadBuildInfo(); ok {
		v := info.Main.Version
		if v != "" && v != "(devel)" {
			return strings.TrimPrefix(v, "v")
		}
	}
	return "dev"
}

// handleVersionFlag intercepts --version / -V before the args reach the WASM.
// Writes "mille <version>\n" to w and returns true when intercepted.
// Stops scanning at "--" or the first non-flag argument (standard POSIX convention).
func handleVersionFlag(args []string, w io.Writer) bool {
	for _, arg := range args {
		if arg == "--version" || arg == "-V" {
			fmt.Fprintf(w, "mille %s\n", getVersion())
			return true
		}
		if arg == "--" || !strings.HasPrefix(arg, "-") {
			break
		}
	}
	return false
}

func main() {
	// NOTE: Intercept --version/-V here so the Go module version is shown
	// (e.g. "v0.0.9") rather than the version baked into the WASM binary
	// at WASM-build time (which would be the Cargo.toml value at that time).
	if handleVersionFlag(os.Args[1:], os.Stdout) {
		os.Exit(0)
	}

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
