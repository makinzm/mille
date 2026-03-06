#!/usr/bin/env node
'use strict';

// Suppress the WASI experimental warning so that mille's own stderr output
// is not mixed with Node.js internals noise.
process.on('warning', (w) => {
  if (w.name === 'ExperimentalWarning' && /WASI/i.test(w.message)) return;
  process.stderr.write(w.stack + '\n');
});

const { WASI } = require('node:wasi');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');

async function run() {
  const wasmPath = join(__dirname, 'mille.wasm');
  let wasmBuffer;
  try {
    wasmBuffer = readFileSync(wasmPath);
  } catch {
    process.stderr.write(
      'mille: mille.wasm not found. Please reinstall the package.\n'
    );
    process.exit(3);
  }

  // NOTE: Mount the host CWD as "/" inside WASI so that paths like
  //       "mille.toml" and "src/domain/**" resolve correctly relative
  //       to the project root — same as the Go/wazero wrapper.
  const wasi = new WASI({
    version: 'preview1',
    args: ['mille', ...process.argv.slice(2)],
    env: process.env,
    preopens: { '/': process.cwd() },
  });

  const { instance } = await WebAssembly.instantiate(wasmBuffer, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });

  // NOTE: wasi.start() calls process.exit() directly when the WASI module
  //       invokes proc_exit. Exit codes:
  //         0 — no violations
  //         1 — at least one error-severity violation
  //         3 — configuration or runtime error
  wasi.start(instance);
}

run().catch((err) => {
  process.stderr.write('mille: ' + err.message + '\n');
  process.exit(3);
});
