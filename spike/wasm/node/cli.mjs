#!/usr/bin/env node
// One-line CLI shim over the WASM component (default preopen: host "/").
import { tool } from './dist/agent_context_wasm.js';
process.exit(tool.run(process.argv.slice(2)));
