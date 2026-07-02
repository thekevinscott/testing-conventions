// WASM-track Node SDK. The transpiled component's std::fs goes through
// WASI preopens; `sandbox(dir)` maps the host dir to /work inside the guest.
// NOTE: the preview2-shim DEFAULT preopens "/" to the whole host filesystem.
import { _setPreopens } from '@bytecodealliance/preview2-shim/filesystem';
import { tool } from './dist/agent_context_wasm.js';

export function sandbox(hostDir) {
  _setPreopens({ '/work': hostDir });
  return '/work';
}

export const { print, writeBlock, crash, run } = tool;
