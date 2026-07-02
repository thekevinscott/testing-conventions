//! WASM component binding — thin glue only; all logic lives in
//! agent-context-core, whose std::fs resolves against host preopens.

wit_bindgen::generate!({
    path: "../wit",
    world: "agent-context",
});

use exports::spike::agent_context::tool::{Guest, PrintResult};

struct Component;

impl Guest for Component {
    fn print(globs: Vec<String>, cwd: Option<String>) -> Result<PrintResult, String> {
        agent_context_core::print(&globs, cwd.as_deref()).map(|r| PrintResult {
            files: r.files,
            text: r.text,
        })
    }

    fn write_block(path: String, id: String, content: String) -> Result<bool, String> {
        agent_context_core::write_block(&path, &id, &content)
    }

    fn crash(mode: String) -> Result<(), String> {
        agent_context_core::crash(&mode)
    }

    fn run(argv: Vec<String>) -> i32 {
        agent_context_core::run(argv)
    }
}

export!(Component);
