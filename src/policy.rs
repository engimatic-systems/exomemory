use crate::types::*;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[default]
    Test,
    Local,
}
impl Profile {
    pub fn policy(self, runtime: &Path, scope: &Path, tenant: String) -> Result<Policy> {
        match self {
            Self::Test => Policy::test_profile(runtime, scope, tenant),
            Self::Local => Policy::local_profile(runtime, scope, tenant),
        }
    }
}
/// Explicit runtime and directory boundaries; all roots must already exist.
#[derive(Debug, Clone)]
pub struct Policy {
    pub runtime: PathBuf,
    pub permitted_root: PathBuf,
    pub tenant: String,
}
fn forbidden(message: &str) -> Error {
    Error::new(ErrorCode::Forbidden, message)
}
fn clean_absolute(path: &Path) -> Result<()> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(forbidden("path must be absolute without parent traversal"));
    }
    Ok(())
}
fn canonical(path: &Path) -> Result<PathBuf> {
    clean_absolute(path)?;
    fs::canonicalize(path).map_err(|_| forbidden("path cannot be resolved"))
}
impl Policy {
    pub fn test_profile(runtime: &Path, permitted_root: &Path, tenant: String) -> Result<Self> {
        let tests = canonical(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"))?;
        let runtime_parent = canonical(&tests.join(".runtime"))?;
        if !runtime_parent.starts_with(&tests) {
            return Err(forbidden("test runtime parent escapes tests"));
        }
        let runtime = canonical(runtime)?;
        let permitted_root = canonical(permitted_root)?;
        if runtime == runtime_parent
            || !runtime.starts_with(&runtime_parent)
            || !permitted_root.starts_with(&runtime)
            || tenant.trim().is_empty()
            || !runtime.is_dir()
            || !permitted_root.is_dir()
        {
            return Err(forbidden("invalid isolated runtime, scope root, or tenant"));
        }
        Ok(Self {
            runtime,
            permitted_root,
            tenant,
        })
    }
    pub fn local_profile(runtime: &Path, permitted_root: &Path, tenant: String) -> Result<Self> {
        use std::os::unix::fs::PermissionsExt;
        let runtime = canonical(runtime)?;
        let permitted_root = canonical(permitted_root)?;
        if !runtime.is_dir()
            || !permitted_root.is_dir()
            || tenant.trim().is_empty()
            || fs::metadata(&runtime)?.permissions().mode() & 0o077 != 0
        {
            return Err(forbidden(
                "local runtime must be a private directory; scope and tenant are required",
            ));
        }
        Ok(Self {
            runtime,
            permitted_root,
            tenant,
        })
    }
    pub fn runtime_path(&self, path: &Path) -> Result<PathBuf> {
        clean_absolute(path)?;
        let resolved = if path.exists() {
            canonical(path)?
        } else {
            let parent = path.parent().ok_or_else(|| forbidden("missing parent"))?;
            // Reject dangling symlinks too: they are not a safe new file.
            if fs::symlink_metadata(path).is_ok() {
                return Err(forbidden("unresolved runtime symlink"));
            }
            canonical(parent)?.join(
                path.file_name()
                    .ok_or_else(|| forbidden("missing filename"))?,
            )
        };
        if !resolved.starts_with(&self.runtime) || resolved == self.runtime {
            return Err(forbidden("runtime path escapes run"));
        }
        Ok(resolved)
    }
    pub fn context(&self, context: &Context) -> Result<PathBuf> {
        if context.tenant != self.tenant || context.session.is_empty() {
            return Err(forbidden("invalid tenant or session"));
        }
        let cwd = canonical(&context.cwd)?;
        if !cwd.is_dir() || !cwd.starts_with(&self.permitted_root) {
            return Err(forbidden("context outside permitted root"));
        }
        Ok(cwd)
    }
    /// Shared by creation and complete replacement; global scope is forbidden in tests.
    pub fn write_scope(&self, context: &Context, scope: &Scope) -> Result<Scope> {
        let cwd = self.context(context)?;
        let Scope::Dir { root } = scope else {
            return Err(forbidden("global scope is not enabled in this profile"));
        };
        let root = canonical(root)?;
        if !root.is_dir() || !root.starts_with(&self.permitted_root) || !cwd.starts_with(&root) {
            return Err(forbidden("memory scope outside permitted context"));
        }
        Ok(Scope::Dir { root })
    }
    pub fn access(&self, context: &Context, tenant: &str, scope: &Scope) -> Result<()> {
        if tenant != self.tenant {
            return Err(forbidden("tenant mismatch"));
        }
        self.write_scope(context, scope).map(|_| ())
    }
}
