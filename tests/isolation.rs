mod support;
use exomemory::{policy::Policy, types::*};
use std::fs;
use support::Run;
#[test]
fn isolated_paths_and_scope_boundaries() {
    let run = Run::new();
    let other = Run::new();
    assert_ne!(run.root, other.root);
    assert!(
        run.policy
            .runtime_path(&run.root.join("memory.sqlite"))
            .is_ok()
    );
    assert!(
        run.policy
            .runtime_path(&other.root.join("memory.sqlite"))
            .is_err()
    );
    assert!(
        run.policy
            .runtime_path(&run.root.join("../escape"))
            .is_err()
    );
    assert!(
        run.policy
            .write_scope(&run.context, &Scope::Global)
            .is_err()
    );
    for root in [&other.scope, &run.root] {
        assert!(
            run.policy
                .write_scope(&run.context, &Scope::Dir { root: root.clone() })
                .is_err()
        );
    }
    let scope = Scope::Dir {
        root: run.scope.clone(),
    };
    assert!(run.policy.write_scope(&run.context, &scope).is_ok());
    let child = run.scope.join("nested");
    fs::create_dir(&child).unwrap();
    let mut context = run.context.clone();
    context.cwd = child;
    assert!(
        run.policy
            .access(&context, &run.policy.tenant, &scope)
            .is_ok()
    );
    context.cwd = run.root.join("scope-sibling");
    fs::create_dir(&context.cwd).unwrap();
    assert!(
        run.policy
            .access(&context, &run.policy.tenant, &scope)
            .is_err()
    );
    context.cwd = run.scope.join("nested/..");
    assert!(run.policy.context(&context).is_err());
    context = run.context.clone();
    context.tenant = "other".into();
    assert!(run.policy.context(&context).is_err());
    assert!(Policy::test_profile(&run.root, &other.scope, "t".into()).is_err());
}
#[cfg(unix)]
#[test]
fn symlink_escape_and_dangling_runtime_are_rejected() {
    use std::os::unix::fs::symlink;
    let run = Run::new();
    let other = Run::new();
    let escape = run.scope.join("escape");
    symlink(&other.scope, &escape).unwrap();
    let scope = Scope::Dir {
        root: escape.clone(),
    };
    assert!(run.policy.write_scope(&run.context, &scope).is_err());
    let mut context = run.context.clone();
    context.cwd = escape.clone();
    assert!(run.policy.context(&context).is_err());
    assert!(run.policy.runtime_path(&escape.join("db")).is_err());
    let dangling = run.root.join("dangling");
    symlink(other.root.join("absent"), &dangling).unwrap();
    assert!(run.policy.runtime_path(&dangling).is_err());
}
