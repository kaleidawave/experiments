pub mod tools;
pub mod utilities;

use std::borrow::Cow;
use std::ffi::OsStr;

pub struct CommandRequest<'a> {
    pub program: Cow<'a, OsStr>,
    pub arguments: Vec<Cow<'a, OsStr>>,
}

/// TODO iterations, etc for time etc
pub struct ToolOptions {
    /// Save SDE file...
    pub keep: Option<String>,
    /// skip Rust internals
    pub skip_internals: bool,
}

#[non_exhaustive]
pub enum ToolOutput {
    SymbolInstructionCounts { symbols: Vec<Entry>, total: u32 },
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub symbol_name: String,
    pub total: u32,
    /// TODO maybe fixed
    pub entries: Vec<(String, u32)>,
}
