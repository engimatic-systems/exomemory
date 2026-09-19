mod support;
use exomemory::{app::select, attempts::AttemptState, data::Storage, sqlite::Sqlite, types::*};
use support::*;
#[test]
fn feedback_changes_rank_not_matching_and_survives_overwrite_and_restart() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    for _ in 0..2 {
        db.create(&run.policy, &run.context, draft(&run)).unwrap();
    }
    let event = event(&run, 1);
    let before = db.candidates(&run.policy, &event).unwrap();
    let selected = select(
        &run.policy,
        &event,
        &before,
        &AttemptState::default(),
        &bounds(),
    )
    .unwrap();
    assert_eq!(selected.len(), 1);
    let key = &selected[0].ghost().ghost_key;
    let feedback = db.annoy(&run.policy, &run.context, key).unwrap();
    assert_eq!(
        (
            feedback.penalty,
            feedback.feedback_version,
            feedback.state_version
        ),
        (1, 1, 3)
    );
    assert_eq!(feedback.session, run.context.session);
    assert_eq!(feedback.cwd, run.scope);
    let after = db.candidates(&run.policy, &event).unwrap();
    assert_eq!(
        before
            .candidates
            .iter()
            .map(|c| &c.ghost.ghost_key)
            .collect::<Vec<_>>(),
        after
            .candidates
            .iter()
            .map(|c| &c.ghost.ghost_key)
            .collect::<Vec<_>>()
    );
    let selected_after = select(
        &run.policy,
        &event,
        &after,
        &AttemptState::default(),
        &bounds(),
    )
    .unwrap();
    assert_ne!(&selected_after[0].ghost().ghost_key, key);
    let record = db
        .overwrite(&run.policy, &run.context, key, 1, draft(&run))
        .unwrap();
    assert_eq!(
        (
            record.metadata.revision,
            record.metadata.penalty,
            record.metadata.feedback_version
        ),
        (2, 1, 1)
    );
    let expected = db.candidates(&run.policy, &event).unwrap();
    drop(db);
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    assert_eq!(db.candidates(&run.policy, &event).unwrap(), expected);
    // Saved snapshots remain sufficient to reproduce the earlier selection.
    assert_eq!(
        select(
            &run.policy,
            &event,
            &before,
            &AttemptState::default(),
            &bounds()
        )
        .unwrap()[0]
            .ghost()
            .ghost_key,
        *key
    );
    let inspector = rusqlite::Connection::open(path).unwrap();
    let attribution: (String, i64, i64) = inspector
        .query_row(
            "SELECT session,feedback_version,state_version FROM feedback_events",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(attribution, (run.context.session, 1, 3));
}
#[test]
fn concurrent_feedback_is_attributed_and_saturates_without_overflow() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let key = db
        .create(&run.policy, &run.context, draft(&run))
        .unwrap()
        .metadata
        .ghost_key;
    let barrier = std::sync::Barrier::new(8);
    let mut outcomes = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..8)
            .map(|i| {
                let mut db = Sqlite::open(&run.policy, &path).unwrap();
                let mut context = run.context.clone();
                context.session = format!("actor-{i}");
                let barrier = &barrier;
                let key = &key;
                let policy = &run.policy;
                scope.spawn(move || {
                    barrier.wait();
                    db.annoy(policy, &context, key).unwrap()
                })
            })
            .collect();
        jobs.into_iter()
            .map(|j| j.join().unwrap())
            .collect::<Vec<_>>()
    });
    outcomes.sort_by_key(|f| f.feedback_version);
    for (i, f) in outcomes.iter().enumerate() {
        assert_eq!(f.penalty as usize, i + 1);
        assert_eq!(f.feedback_version as usize, i + 1);
        assert_eq!(f.state_version as usize, i + 2);
    }
    let inspector = rusqlite::Connection::open(path).unwrap();
    let actors: i64 = inspector
        .query_row(
            "SELECT count(DISTINCT session) FROM feedback_events",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actors, 8);
    // Seed the boundary directly; billions of CLI increments would add no coverage.
    inspector
        .execute("UPDATE feedback SET penalty=?1", [u32::MAX - 1])
        .unwrap();
    for expected_version in [9, 10] {
        let f = db.annoy(&run.policy, &run.context, &key).unwrap();
        assert_eq!(f.penalty, u32::MAX);
        assert_eq!(f.feedback_version, expected_version);
    }
    assert_eq!(
        db.read(&run.policy, &run.context, &key)
            .unwrap()
            .metadata
            .revision,
        1
    );
    inspector.execute_batch("CREATE TRIGGER fail BEFORE INSERT ON feedback_events BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let before = db.state_version().unwrap();
    assert!(db.annoy(&run.policy, &run.context, &key).is_err());
    assert_eq!(db.state_version().unwrap(), before);
    assert_eq!(
        db.read(&run.policy, &run.context, &key)
            .unwrap()
            .metadata
            .feedback_version,
        10
    );
}
#[test]
fn feedback_enforces_scope_and_cli_rejects_arbitrary_penalties() {
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let (_, created) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["create", "--json", "-"],
        Some(&draft(&run)),
    );
    let key = created["result"]["Ok"]["metadata"]["ghost_key"]
        .as_str()
        .unwrap();
    let (status, value) = ctl(&run, &daemon.socket, &run.scope, &["annoy", key], None);
    assert_eq!(status, 0);
    assert_eq!(value["result"]["Ok"]["penalty"], 1);
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["annoy", key, "10"],
            None
        )
        .0,
        2
    );
    let other = Run::new();
    assert_eq!(
        ctl(&run, &daemon.socket, &other.scope, &["annoy", key], None).0,
        4
    );
    let mut db = Sqlite::open(&run.policy, &run.root.join("memory.sqlite")).unwrap();
    let mut other_context = run.context.clone();
    other_context.tenant = "other".into();
    assert_eq!(
        db.annoy(&run.policy, &other_context, key).unwrap_err().code,
        ErrorCode::Forbidden
    );
    assert_eq!(
        db.read(&run.policy, &run.context, key)
            .unwrap()
            .metadata
            .penalty,
        1
    );
}
#[test]
fn schema_one_records_migrate_without_changing_revision() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let mut record = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    drop(db);
    let inspector = rusqlite::Connection::open(&path).unwrap();
    // Reconstruct the exact old schema/JSON shape to exercise the supported migration.
    inspector
        .execute_batch("DROP TABLE feedback_events; DROP TABLE feedback; PRAGMA user_version=1;")
        .unwrap();
    let mut json = serde_json::to_value(&record).unwrap();
    json["metadata"]
        .as_object_mut()
        .unwrap()
        .remove("feedback_version");
    inspector
        .execute("UPDATE revisions SET record_json=?1", [json.to_string()])
        .unwrap();
    drop(inspector);
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let migrated = db
        .read(&run.policy, &run.context, &record.metadata.ghost_key)
        .unwrap();
    record.metadata.read_token_estimate = migrated.metadata.read_token_estimate;
    assert_eq!(record, migrated);
    assert_eq!(
        db.annoy(&run.policy, &run.context, &record.metadata.ghost_key)
            .unwrap()
            .feedback_version,
        1
    );
}
