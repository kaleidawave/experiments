pub mod tools;
pub mod utilities;

pub struct CommandRequest {
    pub program: String,
    pub arguments: Vec<String>,
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
