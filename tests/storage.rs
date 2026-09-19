mod support;
use exomemory::{data::Storage, sqlite::Sqlite, types::*};
use support::{Run, draft};
#[test]
fn complete_record_reopens_and_enforces_access() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let record = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    assert_eq!(record.metadata.revision, 1);
    assert_eq!(record.metadata.penalty, 0);
    assert_eq!(
        record.metadata.read_token_estimate,
        serde_json::to_vec(&record).unwrap().len().div_ceil(4) as u64
    );
    assert_eq!(db.state_version().unwrap(), 1);
    drop(db);
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    assert_eq!(
        db.read(&run.policy, &run.context, &record.metadata.ghost_key)
            .unwrap(),
        record
    );
    let other = Run::new();
    assert_eq!(
        db.read(&run.policy, &other.context, &record.metadata.ghost_key)
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    let mut bad = draft(&run);
    bad.scope = Scope::Global;
    assert!(db.create(&run.policy, &run.context, bad).is_err());
    let mut bad = draft(&run);
    bad.triggers.clear();
    assert!(db.create(&run.policy, &run.context, bad).is_err());
    let mut bad = draft(&run);
    bad.backend.kind = "redis".into();
    assert_eq!(
        db.create(&run.policy, &run.context, bad).unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(db.state_version().unwrap(), 1);
}
#[test]
fn failure_after_body_insert_rolls_back_every_canonical_table() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let inspector = rusqlite::Connection::open(&path).unwrap();
    inspector.execute_batch("CREATE TRIGGER fail_publication BEFORE INSERT ON triggers BEGIN SELECT RAISE(ABORT,'injected trigger failure'); END;").unwrap();
    assert!(db.create(&run.policy, &run.context, draft(&run)).is_err());
    for table in ["memories", "revisions", "triggers"] {
        let count: i64 = inspector
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    assert_eq!(db.state_version().unwrap(), 0);
    inspector
        .execute_batch("DROP TRIGGER fail_publication")
        .unwrap();
    let record = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    assert_eq!(
        db.read(&run.policy, &run.context, &record.metadata.ghost_key)
            .unwrap(),
        record
    );
}
