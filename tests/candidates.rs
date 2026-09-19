mod support;
use exomemory::{data::Storage, sqlite::Sqlite, types::*};
use support::*;
#[test]
fn deterministic_parameterized_snapshot_candidates_and_scope() {
    let run = Run::new();
    let mut db = Sqlite::open(&run.policy, &run.root.join("memory.sqlite")).unwrap();
    let mut input = draft(&run);
    input.triggers.push(Trigger::Regex {
        id: "second".into(),
        pattern: "test".into(),
    });
    let first = db.create(&run.policy, &run.context, input).unwrap();
    db.create(&run.policy, &run.context, draft(&run)).unwrap();
    let query = event(&run, 1, 0);
    let expected = db.candidates(&run.policy, &query).unwrap();
    assert_eq!(expected.state_version, 2);
    assert_eq!(expected.candidates.len(), 3);
    for _ in 0..10 {
        assert_eq!(db.candidates(&run.policy, &query).unwrap(), expected);
    }
    let duplicates: Vec<_> = expected
        .candidates
        .iter()
        .filter(|c| c.ghost.ghost_key == first.metadata.ghost_key)
        .collect();
    assert_eq!(duplicates.len(), 2);
    assert_ne!(duplicates[0].ghost.source_id, duplicates[1].ghost.source_id);
    let other = Run::new();
    let mut outside = query.clone();
    outside.context.cwd = other.scope;
    assert_eq!(
        db.candidates(&run.policy, &outside).unwrap_err().code,
        ErrorCode::Forbidden
    );
    let mut nonmatch = query.clone();
    nonmatch.text = "' OR 1=1; DROP TABLE memories; --".into();
    assert!(
        db.candidates(&run.policy, &nonmatch)
            .unwrap()
            .candidates
            .is_empty()
    );
    assert_eq!(db.state_version().unwrap(), 2);
    let mut bad = draft(&run);
    bad.triggers = vec![Trigger::Regex {
        id: "invalid".into(),
        pattern: "[".into(),
    }];
    assert!(db.create(&run.policy, &run.context, bad).is_err());
    assert_eq!(db.candidates(&run.policy, &query).unwrap(), expected);
    let nested = run.scope.join("nested");
    std::fs::create_dir(&nested).unwrap();
    let mut nested_context = run.context.clone();
    nested_context.cwd = nested.clone();
    let mut scoped = draft(&run);
    scoped.scope = Scope::Dir { root: nested };
    db.create(&run.policy, &nested_context, scoped).unwrap();
    assert_eq!(
        db.candidates(&run.policy, &query).unwrap().candidates.len(),
        3
    );
}
#[test]
fn normalization_and_rollback_never_advertise_partial_records() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let inspector = rusqlite::Connection::open(path).unwrap();
    inspector.execute_batch("CREATE TRIGGER fail BEFORE INSERT ON triggers BEGIN SELECT RAISE(ABORT,'failure'); END;").unwrap();
    assert!(db.create(&run.policy, &run.context, draft(&run)).is_err());
    assert!(
        db.candidates(&run.policy, &event(&run, 1, 0))
            .unwrap()
            .candidates
            .is_empty()
    );
    inspector.execute_batch("DROP TRIGGER fail").unwrap();
    let mut input = draft(&run);
    input.triggers = vec![Trigger::Regex {
        id: "lines".into(),
        pattern: "first\\nsecond".into(),
    }];
    db.create(&run.policy, &run.context, input).unwrap();
    let mut query = event(&run, 1, 0);
    query.text = "first\r\nsecond".into();
    let expected = db.candidates(&run.policy, &query).unwrap();
    assert_eq!(expected.candidates.len(), 1);
    query.text = "first\rsecond".into();
    assert_eq!(db.candidates(&run.policy, &query).unwrap(), expected);
}
