mod support;
use exomemory::{attempts::AttemptStore, types::*};
use support::*;

fn candidates(run: &Run, keys: &[&str]) -> CandidateSnapshot {
    CandidateSnapshot {
        state_version: 1,
        candidates: keys
            .iter()
            .map(|key| {
                let mut ghost = ghost(run);
                ghost.ghost_key = (*key).into();
                ghost.source_id.trigger = (*key).into();
                Candidate {
                    ghost,
                    tenant: run.policy.tenant.clone(),
                    scope: draft(run).scope,
                    relevance_band: 0,
                    backend_rank: 0,
                    penalty: 0,
                }
            })
            .collect(),
    }
}

#[test]
fn abandoned_send_stays_suppressed_across_restart_until_completed_compaction() {
    let run = Run::new();
    let path = run.root.join("harness/attempts.sqlite");
    let input = candidates(&run, &["a"]);
    let mut store = AttemptStore::open(&run.policy, &path, bounds()).unwrap();
    assert_eq!(store.process(&event(&run, 1), &input).unwrap().len(), 1);
    // Drop the committed bytes without delivering them, as with a crash before sending.
    drop(store);
    let mut store = AttemptStore::open(&run.policy, &path, bounds()).unwrap();
    assert!(store.process(&event(&run, 10), &input).unwrap().is_empty());
    assert_eq!(
        store.state(&run.context, "main").unwrap().last_attempted["a"],
        1
    );
    assert!(store.process(&event(&run, 1), &input).is_err());
    let mut compact = event(&run, 11);
    compact.kind = EventKind::CompactionStarted;
    assert!(store.process(&compact, &input).unwrap().is_empty());
    assert!(
        store
            .state(&run.context, "main")
            .unwrap()
            .attempted
            .contains("a")
    );
    compact.kind = EventKind::CompactionCompleted;
    compact.number = 12;
    assert!(store.process(&compact, &input).unwrap().is_empty());
    assert!(
        store
            .state(&run.context, "main")
            .unwrap()
            .attempted
            .is_empty()
    );
    assert_eq!(store.process(&event(&run, 13), &input).unwrap().len(), 1);
    // A duplicate old completion must not clear attempts from the new interval.
    assert!(store.process(&compact, &input).is_err());
    assert!(store.process(&event(&run, 14), &input).unwrap().is_empty());
}

#[test]
fn concurrent_attempts_cannot_duplicate_or_exceed_the_interval_cap() {
    for same_key in [true, false] {
        let run = Run::new();
        let path = run.root.join("harness/attempts.sqlite");
        let mut config = bounds();
        config.max_attempts = if same_key { 3 } else { 1 };
        let mut a = AttemptStore::open(&run.policy, &path, config.clone()).unwrap();
        let mut b = AttemptStore::open(&run.policy, &path, config).unwrap();
        let barrier = std::sync::Barrier::new(2);
        let attempted = std::thread::scope(|scope| {
            let first = scope.spawn(|| {
                barrier.wait();
                a.process(&event(&run, 1), &candidates(&run, &["a"]))
            });
            let second = scope.spawn(|| {
                barrier.wait();
                b.process(
                    &event(&run, 2),
                    &candidates(&run, &[if same_key { "a" } else { "b" }]),
                )
            });
            [first.join().unwrap(), second.join().unwrap()]
                .into_iter()
                .map(|r| match r {
                    Ok(batch) => batch.len(),
                    Err(e) => {
                        assert_eq!(e.code, ErrorCode::Conflict);
                        0
                    }
                })
                .sum::<usize>()
        });
        assert_eq!(attempted, 1);
        assert_eq!(a.state(&run.context, "main").unwrap().attempted.len(), 1);
    }
}

#[test]
fn reset_is_context_local_and_cooldown_and_session_order_survive() {
    let run = Run::new();
    let path = run.root.join("harness/attempts.sqlite");
    let mut store = AttemptStore::open(&run.policy, &path, bounds()).unwrap();
    let input = candidates(&run, &["a"]);
    store.process(&event(&run, 1), &input).unwrap();
    let mut child = event(&run, 2);
    child.context_id = "child".into();
    assert!(store.process(&child, &input).unwrap().is_empty());
    child.number = 3;
    assert_eq!(store.process(&child, &input).unwrap().len(), 1);
    let mut compact = event(&run, 4);
    compact.kind = EventKind::CompactionCompleted;
    store.process(&compact, &input).unwrap();
    assert!(
        store
            .state(&run.context, "main")
            .unwrap()
            .attempted
            .is_empty()
    );
    assert!(
        store
            .state(&run.context, "child")
            .unwrap()
            .attempted
            .contains("a")
    );
    assert_eq!(store.process(&event(&run, 5), &input).unwrap().len(), 1);
    let mut separate = event(&run, 1);
    separate.context.session = "another-session".into();
    assert_eq!(store.process(&separate, &input).unwrap().len(), 1);
}

#[test]
fn transaction_failure_and_outside_scope_leave_attempts_unchanged() {
    let run = Run::new();
    let path = run.root.join("harness/attempts.sqlite");
    let mut store = AttemptStore::open(&run.policy, &path, bounds()).unwrap();
    // Fail the second selected insertion to test rollback of the first and its cooldown.
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TRIGGER fault BEFORE INSERT ON attempts WHEN NEW.ghost_key='b' BEGIN SELECT RAISE(ABORT,'probe'); END;").unwrap();
    assert!(
        store
            .process(&event(&run, 1), &candidates(&run, &["a", "b"]))
            .is_err()
    );
    assert_eq!(
        store.state(&run.context, "main").unwrap(),
        Default::default()
    );
    let outside = Run::new();
    let mut invalid = event(&run, 1);
    invalid.context.cwd = outside.scope;
    assert_eq!(
        store
            .process(&invalid, &candidates(&run, &["a"]))
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    invalid = event(&run, 0);
    assert!(store.process(&invalid, &candidates(&run, &["a"])).is_err());
    assert_eq!(
        store.state(&run.context, "main").unwrap(),
        Default::default()
    );
    let mut other_policy = run.policy.clone();
    other_policy.tenant = "other".into();
    assert!(AttemptStore::open(&other_policy, &path, bounds()).is_err());
    let old = run.root.join("harness/legacy.sqlite");
    rusqlite::Connection::open(&old)
        .unwrap()
        .execute_batch(include_str!("../migrations/001_harness.sql"))
        .unwrap();
    assert!(matches!(
        AttemptStore::open(&run.policy, &old, bounds()),
        Err(Error {
            code: ErrorCode::Unsupported,
            ..
        })
    ));
}
