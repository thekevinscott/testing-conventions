#!/usr/bin/env node
// One-line CLI shim: parsing lives in the Rust core (core::run).
const { run } = require('./index.js');
process.exit(run(process.argv.slice(2)));
