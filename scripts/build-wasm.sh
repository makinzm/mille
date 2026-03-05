#!/usr/bin/env bash
# Build mille for wasm32-wasip1 (WASI Preview 1) and distribute to packages/.
#
# Usage:
#   bash scripts/build-wasm.sh
#
# Requirements (handled automatically):
#   - Rust toolchain with wasm32-wasip1 target (via rustup)
#   - wasi-sdk-30 (downloaded to .wasi-sdk/ if not already present)
#
# NOTE: tree-sitter compiles C code via the `cc` crate. For wasm32-wasip1
#       cross-compilation, the Nix-wrapped clang (devbox) cannot be used
#       because it injects glibc paths incompatible with the WASI sysroot.
#       We therefore download wasi-sdk-30 which ships a self-contained clang
#       and WASI sysroot.
set -euo pipefail

WASM_TARGET="wasm32-wasip1"
RELEASE_WASM="target/${WASM_TARGET}/release/mille.wasm"

# ---------------------------------------------------------------------------
# wasi-sdk setup (auto-download if absent)
# ---------------------------------------------------------------------------
WASI_SDK_VERSION="30"
WASI_SDK_DIR=".wasi-sdk"

if [[ -n "${WASI_SDK_PATH:-}" ]]; then
    echo "→ Using WASI_SDK_PATH from environment: ${WASI_SDK_PATH}"
elif [[ -d "${WASI_SDK_DIR}" ]]; then
    WASI_SDK_PATH="$(pwd)/${WASI_SDK_DIR}"
    echo "→ Using cached wasi-sdk: ${WASI_SDK_PATH}"
else
    ARCH=$(uname -m)
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCHIVE="wasi-sdk-${WASI_SDK_VERSION}.0-${ARCH}-${OS}.tar.gz"
    URL="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_VERSION}/${ARCHIVE}"

    echo "→ Downloading wasi-sdk-${WASI_SDK_VERSION} from GitHub ..."
    mkdir -p "${WASI_SDK_DIR}"
    curl -fsSL "${URL}" | tar xz --strip-components=1 -C "${WASI_SDK_DIR}"
    WASI_SDK_PATH="$(pwd)/${WASI_SDK_DIR}"
    echo "→ wasi-sdk installed to ${WASI_SDK_PATH}"
fi

# NOTE: CC_<triple> / AR_<triple> / CFLAGS_<triple> are picked up by the
#       `cc` crate automatically when cross-compiling to wasm32-wasip1.
export CC_wasm32_wasip1="${WASI_SDK_PATH}/bin/clang"
export AR_wasm32_wasip1="${WASI_SDK_PATH}/bin/llvm-ar"
# NOTE: tree-sitter's ts_tree_print_dot_graph() calls dup() and fdopen() which
#       are POSIX-only and absent from WASI Preview 1 sysroot. That function is
#       dead code in mille (never invoked), so release-mode --gc-sections
#       eliminates it before link. We suppress the implicit-declaration error so
#       that the C compilation phase succeeds.
export CFLAGS_wasm32_wasip1="--sysroot=${WASI_SDK_PATH}/share/wasi-sysroot -Wno-implicit-function-declaration"

echo "→ Building mille for ${WASM_TARGET} ..."
rustup target add "${WASM_TARGET}" 2>/dev/null || true
cargo build --target "${WASM_TARGET}" --release

# ---------------------------------------------------------------------------
# Distribute .wasm to packages/
# ---------------------------------------------------------------------------
mkdir -p packages/wasm packages/go

cp "${RELEASE_WASM}" packages/wasm/mille.wasm
cp packages/wasm/mille.wasm packages/go/mille.wasm

echo "✓ packages/wasm/mille.wasm"
echo "✓ packages/go/mille.wasm"
