use crate::policy::Profile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const USER_CONFIG_RELATIVE: &str = ".config/exomemory/exoctl.json";
pub const CODEX_LAUNCHER_CONFIG: &str = "codex-launcher.json";

/// Persistent connection and policy values for an installed `exoctl`.
/// Session and working-directory identity are intentionally resolved per process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    pub profile: Profile,
    pub runtime: PathBuf,
    pub scope_root: PathBuf,
    pub socket: PathBuf,
    pub tenant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodexLauncherConfig {
    pub upstream: PathBuf,
    pub host_socket: PathBuf,
}
