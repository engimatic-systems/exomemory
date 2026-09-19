mod support;
use exomemory::{data::Storage, sqlite::Sqlite, types::*};
use support::*;
#[test]
fn concurrent_cas_has_one_winner_and_only_complete_revisions() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let old = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    let key = old.metadata.ghost_key.clone();
    let barrier = std::sync::Barrier::new(3);
    let outcomes = std::thread::scope(|scope| {
        let writers: Vec<_> = (0..2)
            .map(|i| {
                let mut connection = Sqlite::open(&run.policy, &path).unwrap();
                let mut replacement = draft(&run);
                replacement.body = format!("writer-{i}");
                replacement.blurb = replacement.body.clone();
                replacement.triggers = vec![Trigger::Regex {
                    id: "replacement".into(),
                    pattern: "updated".into(),
                }];
                let barrier = &barrier;
                let key = &key;
                let run = &run;
                scope.spawn(move || {
                    let mut context = run.context.clone();
                    context.session = format!("writer-{i}");
                    barrier.wait();
                    connection.overwrite(&run.policy, &context, key, 1, replacement)
                })
            })
            .collect();
        barrier.wait();
        for _ in 0..20 {
            let record = db.read(&run.policy, &run.context, &key).unwrap();
            if record.metadata.revision == 1 {
                assert_eq!(record, old);
            } else {
                assert_eq!(record.metadata.revision, 2);
                assert_eq!(record.body, record.metadata.blurb);
            }
            let mut query = event(&run, 1);
            query.text = "test updated".into();
            let candidates = db.candidates(&run.policy, &query).unwrap().candidates;
            assert_eq!(candidates.len(), 1);
            let candidate = &candidates[0];
            assert_eq!(
                candidate.ghost.source_id.trigger,
                if candidate.ghost.revision == 1 {
                    "test-duration"
                } else {
                    "replacement"
                }
            );
        }
        writers
            .into_iter()
            .map(|w| w.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
    let winner = db.read(&run.policy, &run.context, &key).unwrap();
    let conflict = outcomes.into_iter().find_map(Result::err).unwrap();
    assert_eq!(conflict.code, ErrorCode::Conflict);
    assert_eq!(*conflict.current.unwrap(), winner.metadata);
    assert_eq!(db.state_version().unwrap(), 2);
    assert!(
        db.candidates(&run.policy, &event(&run, 1))
            .unwrap()
            .candidates
            .is_empty()
    );
}
#[test]
fn invalid_unauthorized_and_failed_publication_preserve_the_old_revision() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let old = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    let key = &old.metadata.ghost_key;
    let mut bad = draft(&run);
    bad.triggers.clear();
    assert!(
        db.overwrite(&run.policy, &run.context, key, 1, bad)
            .is_err()
    );
    let mut bad = draft(&run);
    bad.scope = Scope::Global;
    assert_eq!(
        db.overwrite(&run.policy, &run.context, key, 1, bad)
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    let other = Run::new();
    assert_eq!(
        db.overwrite(&run.policy, &other.context, key, 1, draft(&run))
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    let mut other_tenant = run.context.clone();
    other_tenant.tenant = "other".into();
    assert_eq!(
        db.overwrite(&run.policy, &other_tenant, key, 1, draft(&run))
            .unwrap_err()
            .code,
        ErrorCode::Forbidden
    );
    let inspector = rusqlite::Connection::open(&path).unwrap();
    inspector.execute_batch("CREATE TRIGGER fail BEFORE INSERT ON triggers WHEN NEW.revision=2 BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.overwrite(&run.policy, &run.context, key, 1, draft(&run))
            .is_err()
    );
    assert_eq!(db.read(&run.policy, &run.context, key).unwrap(), old);
    assert_eq!(db.state_version().unwrap(), 1);
    let count: i64 = inspector
        .query_row("SELECT count(*) FROM revisions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let mut immutable = draft(&run);
    immutable.overwritable = false;
    let immutable = db.create(&run.policy, &run.context, immutable).unwrap();
    assert_eq!(
        db.overwrite(
            &run.policy,
            &run.context,
            &immutable.metadata.ghost_key,
            1,
            draft(&run)
        )
        .unwrap_err()
        .code,
        ErrorCode::Forbidden
    );
}
#[test]
fn cli_requires_revision_and_returns_current_metadata_on_conflict() {
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let (status, created) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["create", "--json", "-"],
        Some(&draft(&run)),
    );
    assert_eq!(status, 0);
    let key = created["result"]["Ok"]["metadata"]["ghost_key"]
        .as_str()
        .unwrap();
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["overwrite", key, "--json", "-"],
            None
        )
        .0,
        2
    );
    let (status, updated) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["overwrite", key, "--if-revision", "1", "--json", "-"],
        Some(&draft(&run)),
    );
    assert_eq!(status, 0);
    let (status, stale) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["overwrite", key, "--if-revision", "1", "--json", "-"],
        Some(&draft(&run)),
    );
    assert_eq!(status, 7);
    assert_eq!(
        stale["result"]["Err"]["current"],
        updated["result"]["Ok"]["metadata"]
    );
}
