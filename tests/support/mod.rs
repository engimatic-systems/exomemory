#![allow(dead_code)]
use exomemory::{policy::Policy, types::*};
use std::{fs, path::PathBuf};
pub struct Run {
    pub root: PathBuf,
    pub scope: PathBuf,
    pub policy: Policy,
    pub context: Context,
}
impl Run {
    pub fn new() -> Self {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/.runtime");
        fs::create_dir_all(&parent).unwrap();
        let root = parent.join(format!(
            "r{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        fs::create_dir(&root).unwrap();
        let scope = root.join("scope");
        fs::create_dir(&scope).unwrap();
        for dir in ["logs", "harness"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
        let policy = Policy::test_profile(&root, &scope, "fixture-tenant".into()).unwrap();
        let context = Context {
            tenant: policy.tenant.clone(),
            session: "s1".into(),
            cwd: scope.clone(),
        };
        Self {
            root,
            scope,
            policy,
            context,
        }
    }
}
