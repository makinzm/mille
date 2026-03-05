module github.com/makinzm/mille/packages/go

go 1.24.0

require (
	github.com/makinzm/mille/packages/wasm v0.0.1
	github.com/tetratelabs/wazero v1.11.0
)

require golang.org/x/sys v0.38.0 // indirect

// NOTE: This replace directive enables local development and CI builds without
//       requiring packages/wasm to be published to the Go module proxy.
//       go.work (workspace mode) also covers this, but the replace here ensures
//       `go mod tidy` works correctly (it ignores go.work).
//       When packages/wasm is published at v0.0.1, this directive should be removed.
replace github.com/makinzm/mille/packages/wasm v0.0.1 => ../wasm
