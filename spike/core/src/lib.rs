//! Shared core for the agent-context spike. This crate is byte-identical
//! between the FFI and WASM tracks; only the outer binding differs.
//! Plain `std::fs` + the `glob` crate so the same source compiles natively
//! and to wasm32-wasip2 (where std::fs resolves against host preopens).

use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PrintResult {
    pub files: Vec<String>,
    pub text: String,
}

/// Expand `globs` (supports `**`) relative to `cwd` (default "."), read every
/// match as UTF-8, and concatenate in lexicographic relpath order with
/// `===== BEGIN <relpath> =====` / `===== END <relpath> =====` headers.
pub fn print(globs: &[String], cwd: Option<&str>) -> Result<PrintResult, String> {
    let base = cwd.unwrap_or(".");
    let base_path = Path::new(base);

    let mut rels: Vec<String> = Vec::new();
    for pattern in globs {
        let full = base_path.join(pattern);
        let full = full.to_str().ok_or("non-UTF-8 pattern")?;
        let paths = glob::glob(full).map_err(|e| format!("bad glob {pattern:?}: {e}"))?;
        for entry in paths {
            let path = entry.map_err(|e| format!("glob error: {e}"))?;
            if !path.is_file() {
                continue;
            }
            let rel = path
                .strip_prefix(base_path)
                .unwrap_or(&path)
                .to_str()
                .ok_or("non-UTF-8 path")?
                .replace('\\', "/");
            rels.push(rel);
        }
    }
    rels.sort();
    rels.dedup();

    let mut text = String::new();
    for rel in &rels {
        let content = fs::read_to_string(base_path.join(rel))
            .map_err(|e| format!("read {rel}: {e}"))?;
        text.push_str(&format!("===== BEGIN {rel} =====\n"));
        text.push_str(&content);
        if !content.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!("===== END {rel} =====\n"));
    }

    Ok(PrintResult { files: rels, text })
}

/// Upsert a sentinel-delimited block into the file at `path`. Creates the
/// file if absent, appends if the markers are missing, otherwise replaces
/// only the region between the markers. Returns whether the file changed
/// (re-running with identical content is a no-op).
pub fn write_block(path: &str, id: &str, content: &str) -> Result<bool, String> {
    let begin = format!("<!-- agent-context:begin {id} -->");
    let end = format!("<!-- agent-context:end {id} -->");
    let block = format!("{begin}\n{content}\n{end}\n");

    let old = match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("read {path}: {e}")),
    };

    let new = match &old {
        None => block,
        Some(existing) => match existing.find(&begin) {
            Some(b) => {
                let after = &existing[b..];
                let e = after
                    .find(&end)
                    .ok_or_else(|| format!("begin marker without end marker in {path}"))?;
                let region_end = b + e + end.len();
                let rest = existing[region_end..].trim_start_matches('\n');
                format!(
                    "{}{begin}\n{content}\n{end}\n{}",
                    &existing[..b],
                    rest
                )
            }
            None => {
                let mut s = existing.clone();
                if !s.is_empty() && !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(&block);
                s
            }
        },
    };

    let changed = old.as_deref() != Some(new.as_str());
    if changed {
        fs::write(path, &new).map_err(|e| format!("write {path}: {e}"))?;
    }
    Ok(changed)
}

/// Deliberately fault, to probe crash isolation per host.
/// "panic" unwinds; "segfault" writes through a null pointer (native only —
/// address 0 is plain linear memory under wasm32); "abort" hard-aborts.
pub fn crash(mode: &str) -> Result<(), String> {
    match mode {
        "panic" => panic!("agent-context deliberate panic"),
        "segfault" => unsafe {
            std::ptr::null_mut::<u8>().write_volatile(1);
            Ok(())
        },
        "abort" => std::process::abort(),
        other => Err(format!("unknown crash mode {other:?}")),
    }
}

#[derive(Parser)]
#[command(name = "agent-context", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Concatenate every file matching the globs, with filename headers.
    Print {
        globs: Vec<String>,
        /// Emit {"files": [...], "text": "..."} instead of raw text.
        #[arg(long)]
        json: bool,
        /// Resolve globs relative to this directory instead of ".".
        #[arg(long)]
        cwd: Option<String>,
    },
    /// Upsert a sentinel-delimited block into a file.
    WriteBlock {
        path: String,
        id: String,
        content: String,
    },
    /// Deliberately fault (panic | segfault | abort) — crash-isolation probe.
    Crash { mode: String },
}

/// Single-sourced CLI: every language's CLI is a one-line shim that forwards
/// argv (without the program name) here.
pub fn run<I>(argv: I) -> i32
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let args = std::iter::once("agent-context".to_string())
        .chain(argv.into_iter().map(Into::into));
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            let code = if e.exit_code() == 0 { 0 } else { 2 };
            let _ = e.print();
            return code;
        }
    };
    match cli.cmd {
        Cmd::Print { globs, json, cwd } => match print(&globs, cwd.as_deref()) {
            Ok(result) => {
                if json {
                    println!("{}", serde_json::to_string(&result).expect("serialize"));
                } else {
                    std::io::Write::write_all(
                        &mut std::io::stdout(),
                        result.text.as_bytes(),
                    )
                    .expect("stdout");
                }
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        },
        Cmd::WriteBlock { path, id, content } => match write_block(&path, &id, &content) {
            Ok(changed) => {
                println!("{}", if changed { "updated" } else { "unchanged" });
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        },
        Cmd::Crash { mode } => match crash(&mode) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        },
    }
}
