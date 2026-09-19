mod support;
use exomemory::{harness::ControlledHarness, render::render, types::*};
use std::collections::BTreeSet;
use support::*;
#[test]
fn delivery_retry_compaction_and_cooldown_use_actual_live_occurrences() {
    let run = Run::new();
    let path = run.root.join("harness/context.sqlite");
    let config = bounds();
    let mut harness = ControlledHarness::open(&run.policy, &path, config.clone()).unwrap();
    let one = render(&ghost(&run), "one", &config).unwrap();
    let first = event(&run, 1, 0);
    assert!(
        harness
            .accept(&first, std::slice::from_ref(&one), false)
            .is_err()
    );
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .last_shown
            .is_empty()
    );
    let receipt = harness
        .accept(&first, std::slice::from_ref(&one), true)
        .unwrap();
    assert_eq!(
        receipt,
        harness
            .accept(&first, std::slice::from_ref(&one), true)
            .unwrap()
    );
    assert!(
        harness
            .accept(&event(&run, 9, 1), std::slice::from_ref(&one), true)
            .is_err()
    );
    let mut compact = event(&run, 2, 1);
    compact.kind = EventKind::Compaction;
    harness
        .compact(&compact, &BTreeSet::from(["one".into()]))
        .unwrap();
    assert_eq!(
        harness.presence(&run.context, "main").unwrap().live.len(),
        1
    );
    compact.number = 3;
    compact.presence_version = 2;
    harness.compact(&compact, &BTreeSet::new()).unwrap();
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .live
            .is_empty()
    );
    // Replaying old delivery does not restore a compacted occurrence.
    harness
        .accept(&first, std::slice::from_ref(&one), true)
        .unwrap();
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .live
            .is_empty()
    );
    let two = render(&ghost(&run), "two", &config).unwrap();
    harness.accept(&event(&run, 4, 3), &[two], true).unwrap();
    drop(harness);
    let mut harness = ControlledHarness::open(&run.policy, &path, config).unwrap();
    let presence = harness.presence(&run.context, "main").unwrap();
    assert_eq!(presence.live.get("two"), Some(&"fixture-key".into()));
    assert_eq!(presence.last_shown["fixture-key"], 4);
    assert_eq!(
        harness
            .live_context(&run.context, "main")
            .unwrap()
            .lines()
            .count(),
        2
    );
}
#[test]
fn concurrent_stale_views_cannot_duplicate_or_overfill_and_sessions_are_independent() {
    let run = Run::new();
    let path = run.root.join("harness/context.sqlite");
    let mut config = bounds();
    config.max_live = 1;
    let mut a = ControlledHarness::open(&run.policy, &path, config.clone()).unwrap();
    let mut b = ControlledHarness::open(&run.policy, &path, config.clone()).unwrap();
    let one = render(&ghost(&run), "one", &config).unwrap();
    let mut other = ghost(&run);
    other.ghost_key = "other".into();
    let two = render(&other, "two", &config).unwrap();
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            a.accept(&event(&run, 1, 0), std::slice::from_ref(&one), true)
        });
        let second = scope.spawn(|| {
            barrier.wait();
            b.accept(&event(&run, 2, 0), std::slice::from_ref(&two), true)
        });
        assert_ne!(
            first.join().unwrap().is_ok(),
            second.join().unwrap().is_ok()
        );
    });
    assert_eq!(a.presence(&run.context, "main").unwrap().live.len(), 1);
    let mut child = event(&run, 3, 0);
    child.live_context = "child".into();
    a.accept(&child, &[], true).unwrap();
    assert!(a.presence(&run.context, "child").unwrap().live.is_empty());
    let mut independent = event(&run, 1, 0);
    independent.context.session = "s2".into();
    a.accept(&independent, &[one], true).unwrap();
    assert_eq!(
        a.presence(&independent.context, "main").unwrap().live.len(),
        1
    );
}
#[test]
fn cooldown_and_invalid_compaction_leave_ledger_unchanged() {
    let run = Run::new();
    let mut config = bounds();
    config.cooldown_events = 10;
    let mut harness = ControlledHarness::open(
        &run.policy,
        &run.root.join("harness/context.sqlite"),
        config.clone(),
    )
    .unwrap();
    let one = render(&ghost(&run), "one", &config).unwrap();
    harness
        .accept(&event(&run, 1, 0), std::slice::from_ref(&one), true)
        .unwrap();
    let mut compact = event(&run, 2, 1);
    compact.kind = EventKind::Compaction;
    assert!(
        harness
            .compact(&compact, &BTreeSet::from(["unknown".into()]))
            .is_err()
    );
    assert_eq!(harness.presence(&run.context, "main").unwrap().version, 1);
    harness.compact(&compact, &BTreeSet::new()).unwrap();
    assert!(
        harness
            .accept(&event(&run, 3, 2), std::slice::from_ref(&one), true)
            .is_err()
    );
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .live
            .is_empty()
    );
    harness.accept(&event(&run, 11, 2), &[one], true).unwrap();
}
#[test]
fn batch_failure_is_atomic_and_reopen_cannot_change_tenant() {
    let run = Run::new();
    let config = bounds();
    let path = run.root.join("harness/context.sqlite");
    let mut harness = ControlledHarness::open(&run.policy, &path, config.clone()).unwrap();
    let one = render(&ghost(&run), "one", &config).unwrap();
    let duplicate = render(&ghost(&run), "two", &config).unwrap();
    assert!(
        harness
            .accept(&event(&run, 1, 0), &[one, duplicate], true)
            .is_err()
    );
    let presence = harness.presence(&run.context, "main").unwrap();
    assert_eq!(presence.version, 0);
    assert!(presence.live.is_empty());
    assert!(presence.last_shown.is_empty());
    let mut policy = run.policy.clone();
    policy.tenant = "different-tenant".into();
    assert!(ControlledHarness::open(&policy, &path, config).is_err());
}
