use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub const PROTOCOL_VERSION: u32 = 1;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    Unavailable,
    Forbidden,
    Unsupported,
    NotFound,
    Conflict,
    Internal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}
impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub fn exit_status(&self) -> i32 {
        match self.code {
            ErrorCode::InvalidInput => 2,
            ErrorCode::Unavailable => 3,
            ErrorCode::Forbidden => 4,
            ErrorCode::Unsupported => 5,
            ErrorCode::NotFound => 6,
            ErrorCode::Conflict => 7,
            ErrorCode::Internal => 8,
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorCode::Internal, e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorCode::InvalidInput, e.to_string())
    }
}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::new(ErrorCode::Internal, e.to_string())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    Global,
    Dir { root: PathBuf },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    Untrusted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub origin: String,
    pub actor: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceId {
    pub backend: String,
    pub trigger: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backend {
    pub kind: String,
    pub instance: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    Regex { id: String, pattern: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub body: String,
    pub blurb: String,
    pub uses: Vec<String>,
    pub topics: Vec<String>,
    pub triggers: Vec<Trigger>,
    pub backend: Backend,
    pub scope: Scope,
    pub provenance: Provenance,
    pub trust: Trust,
    pub overwritable: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub ghost_key: String,
    pub revision: u64,
    pub tenant: String,
    pub blurb: String,
    pub uses: Vec<String>,
    pub topics: Vec<String>,
    pub triggers: Vec<Trigger>,
    pub backend: Backend,
    pub scope: Scope,
    pub provenance: Provenance,
    pub trust: Trust,
    pub overwritable: bool,
    pub penalty: u32,
    pub read_token_estimate: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub metadata: Metadata,
    pub body: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ghost {
    pub ghost_key: String,
    pub revision: u64,
    pub blurb: String,
    pub uses: Vec<String>,
    pub topics: Vec<String>,
    pub read_token_estimate: u64,
    pub source_id: SourceId,
    pub provenance: Provenance,
    pub trust: Trust,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub ghost: Ghost,
    pub tenant: String,
    pub scope: Scope,
    pub relevance_band: u32,
    pub backend_rank: u64,
    pub penalty: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateSnapshot {
    pub state_version: u64,
    pub candidates: Vec<Candidate>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub tenant: String,
    pub session: String,
    pub cwd: PathBuf,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Prompt,
    ToolCall,
    ToolResponse,
    Permission,
    SessionStart,
    SessionEnd,
    SubagentStart,
    SubagentEnd,
    Compaction,
    TurnComplete,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub context: Context,
    pub live_context: String,
    pub number: u64,
    pub presence_version: u64,
    pub kind: EventKind,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    ListTriggerTypes,
    Create { draft: Draft },
    Read { key: String },
    Meta { key: String },
    Candidates { event: Event },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub context: Context,
    pub operation: Operation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub version: u32,
    pub result: std::result::Result<serde_json::Value, Error>,
}
