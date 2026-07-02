//! napi-rs binding — thin glue only; all logic lives in agent-context-core.

use napi_derive::napi;

#[napi(object)]
pub struct PrintResult {
    pub files: Vec<String>,
    pub text: String,
}

#[napi]
pub fn print(globs: Vec<String>, cwd: Option<String>) -> napi::Result<PrintResult> {
    agent_context_core::print(&globs, cwd.as_deref())
        .map(|r| PrintResult {
            files: r.files,
            text: r.text,
        })
        .map_err(napi::Error::from_reason)
}

#[napi]
pub fn write_block(path: String, id: String, content: String) -> napi::Result<bool> {
    agent_context_core::write_block(&path, &id, &content).map_err(napi::Error::from_reason)
}

#[napi]
pub fn crash(mode: String) -> napi::Result<()> {
    agent_context_core::crash(&mode).map_err(napi::Error::from_reason)
}

#[napi]
pub fn run(argv: Vec<String>) -> i32 {
    agent_context_core::run(argv)
}
