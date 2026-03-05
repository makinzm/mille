// Package millewasm exports the embedded mille WASI binary.
//
// This is a separate Go module so that any language wrapper can declare it as
// a versioned dependency. Only one copy of mille.wasm is committed to the
// repository (here). Other language packages (npm, pypi) bundle the same file
// into their distribution archives at publish time — no runtime download.
package millewasm

import _ "embed"

//go:embed mille.wasm
var Wasm []byte
