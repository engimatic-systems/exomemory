mod support;
// Compile the real storage source with test-only commit barriers; production omits them.
pub use exomemory::{data, policy, types};
#[path = "../src/sqlite.rs"]
mod instrumented;
use data::Storage;
use exomemory::{sqlite::Sqlite, transport};
use std::{
    fs,
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::*;
use types::*;
fn checkpoint(point: &str) {
    if std::env::var("EXO_CRASH_POINT").unwrap() == point {
        fs::write(std::env::var("EXO_CRASH_READY").unwrap(), point).unwrap();
        loop {
            std::thread::park();
        }
    }
}
#[test]
fn crash_worker() {
    let Ok(root) = std::env::var("EXO_CRASH_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let policy =
        policy::Policy::test_profile(&root, &root.join("scope"), "fixture-tenant".into()).unwrap();
    let context: Context =
        serde_json::from_slice(&fs::read(root.join("harness/context.json")).unwrap()).unwrap();
    if std::env::var("EXO_CRASH_POINT").unwrap() == "after_attempt_mark" {
        let input: CandidateSnapshot =
            serde_json::from_slice(&fs::read(root.join("harness/candidates.json")).unwrap())
                .unwrap();
        let event: Event =
            serde_json::from_slice(&fs::read(root.join("harness/event.json")).unwrap()).unwrap();
        let mut attempts = exomemory::attempts::AttemptStore::open(
            &policy,
            &root.join("harness/attempts.sqlite"),
            bounds(),
        )
        .unwrap();
        assert_eq!(attempts.process(&event, &input).unwrap().len(), 1);
        checkpoint("after_attempt_mark");
        return;
    }
    let draft: Draft =
        serde_json::from_slice(&fs::read(root.join("harness/draft.json")).unwrap()).unwrap();
    let mut db = instrumented::Sqlite::open(&policy, &root.join("memory.sqlite")).unwrap();
    assert!(db.capabilities().overwrite);
    assert!(db.state_version().unwrap() <= 1);
    db.fault = Some(checkpoint);
    if let Ok(key) = std::env::var("EXO_CRASH_KEY") {
        db.overwrite(&policy, &context, &key, 1, draft).unwrap();
    } else {
        db.create(&policy, &context, draft).unwrap();
    }
    panic!("fault point was not reached");
}
#[test]
fn killed_processes_recover_complete_state_on_both_sides_of_publication() {
    for operation in ["create", "overwrite"] {
        for side in ["before", "after"] {
            let run = Run::new();
            let path = run.root.join("memory.sqlite");
            let mut db = Sqlite::open(&run.policy, &path).unwrap();
            let old = if operation == "overwrite" {
                Some(db.create(&run.policy, &run.context, draft(&run)).unwrap())
            } else {
                None
            };
            drop(db);
            let mut replacement = draft(&run);
            replacement.body = "replacement body".into();
            replacement.blurb = "replacement preview".into();
            replacement.triggers = vec![Trigger::Regex {
                id: "new-trigger".into(),
                pattern: "replacement".into(),
            }];
            fs::write(
                run.root.join("harness/context.json"),
                serde_json::to_vec(&run.context).unwrap(),
            )
            .unwrap();
            fs::write(
                run.root.join("harness/draft.json"),
                serde_json::to_vec(&replacement).unwrap(),
            )
            .unwrap();
            let ready = run.root.join("harness/ready");
            let point = format!("{side}_{operation}_commit");
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "crash_worker", "--nocapture"])
                .env("EXO_CRASH_ROOT", &run.root)
                .env("EXO_CRASH_READY", &ready)
                .env("EXO_CRASH_POINT", &point)
                .stdout(Stdio::from(
                    fs::File::create(run.root.join("logs/crash-worker.log")).unwrap(),
                ))
                .stderr(Stdio::null());
            if let Some(record) = &old {
                command.env("EXO_CRASH_KEY", &record.metadata.ghost_key);
            }
            let mut child = command.spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !ready.exists() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "worker exited before {point}"
                );
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("fault point timeout");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            child.kill().unwrap();
            child.wait().unwrap();
            let daemon = Daemon::start(&run);
            let inspector = rusqlite::Connection::open(&path).unwrap();
            let integrity: String = inspector
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .unwrap();
            assert_eq!(integrity, "ok");
            let count: i64 = inspector
                .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
                .unwrap();
            let absent = operation == "create" && side == "before";
            assert_eq!(count, if absent { 0 } else { 1 });
            for table in ["revisions", "triggers", "feedback"] {
                let count: i64 = inspector
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                    .unwrap();
                let expected = if absent {
                    0
                } else if operation == "overwrite" && side == "after" && table != "feedback" {
                    2
                } else {
                    1
                };
                assert_eq!(count, expected, "{point}/{table}");
            }
            let mut query = event(&run, 1);
            query.text = "test replacement".into();
            let snapshot: CandidateSnapshot = serde_json::from_value(
                transport::call(
                    &run.policy,
                    &daemon.socket,
                    &Request {
                        version: 1,
                        context: run.context.clone(),
                        operation: Operation::Candidates { event: query },
                    },
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(snapshot.candidates.len(), usize::from(!absent));
            if !absent {
                let candidate = &snapshot.candidates[0];
                let (status, record) = ctl(
                    &run,
                    &daemon.socket,
                    &run.scope,
                    &[&candidate.ghost.ghost_key],
                    None,
                );
                assert_eq!(status, 0);
                let record: Record =
                    serde_json::from_value(record["result"]["Ok"].clone()).unwrap();
                assert_eq!(candidate.ghost.revision, record.metadata.revision);
                assert_eq!(candidate.ghost.blurb, record.metadata.blurb);
                if side == "before" {
                    assert_eq!(record, old.unwrap());
                } else {
                    assert_eq!(record.body, replacement.body);
                    assert_eq!(record.metadata.triggers, replacement.triggers);
                }
            }
            fs::write(
                run.root.join("harness/recovery-evidence.json"),
                serde_json::to_vec_pretty(
                    &serde_json::json!({"point":point,"integrity":integrity,"snapshot":snapshot}),
                )
                .unwrap(),
            )
            .unwrap();
            println!("Crash recovery {point}: {}", run.root.display());
        }
    }
}
#[test]
fn response_loss_after_commit_is_uncertain_and_stale_retry_conflicts() {
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
    let proxy = run.root.join("proxy.sock");
    let listener = UnixListener::bind(&proxy).unwrap();
    std::thread::scope(|scope| {
        let proxy_job = scope.spawn(|| {
            let (client, _) = listener.accept().unwrap();
            let request = transport::frame(&client).unwrap();
            let mut server = UnixStream::connect(&daemon.socket).unwrap();
            server.write_all(&request).unwrap();
            let reply: Response =
                serde_json::from_slice(&transport::frame(server).unwrap()).unwrap();
            assert_eq!(reply.result.unwrap()["metadata"]["revision"], 2);
            // Drop the connection only after the real daemon has acknowledged its commit.
            drop(client);
        });
        let request = Request {
            version: 1,
            context: run.context.clone(),
            operation: Operation::Overwrite {
                key: key.into(),
                if_revision: 1,
                draft: draft(&run),
            },
        };
        let error = transport::call(&run.policy, &proxy, &request).unwrap_err();
        assert_eq!(error.code, ErrorCode::Unavailable);
        assert!(error.message.contains("may have committed"));
        proxy_job.join().unwrap();
        assert_eq!(
            transport::call(&run.policy, &daemon.socket, &request)
                .unwrap_err()
                .code,
            ErrorCode::Conflict
        );
    });
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &[key], None).1["result"]["Ok"]["metadata"]["revision"],
        2
    );
}
#[test]
fn mixed_clients_preserve_accepted_updates_and_busy_wait_is_bounded() {
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
    let barrier = std::sync::Barrier::new(4);
    let winners = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    assert_eq!(ctl(&run, &daemon.socket, &run.scope, &[key], None).0, 0);
                    assert_eq!(
                        ctl(
                            &run,
                            &daemon.socket,
                            &run.scope,
                            &["create", "--json", "-"],
                            Some(&draft(&run))
                        )
                        .0,
                        0
                    );
                    assert_eq!(
                        ctl(&run, &daemon.socket, &run.scope, &["annoy", key], None).0,
                        0
                    );
                    let code = ctl(
                        &run,
                        &daemon.socket,
                        &run.scope,
                        &["overwrite", key, "--if-revision", "1", "--json", "-"],
                        Some(&draft(&run)),
                    )
                    .0;
                    assert!([0, 7].contains(&code));
                    usize::from(code == 0)
                })
            })
            .collect();
        jobs.into_iter().map(|j| j.join().unwrap()).sum::<usize>()
    });
    assert_eq!(winners, 1);
    let metadata = &ctl(&run, &daemon.socket, &run.scope, &["meta", key], None).1["result"]["Ok"];
    assert_eq!(metadata["penalty"], 4);
    assert_eq!(metadata["revision"], 2);
    let inspector = rusqlite::Connection::open(run.root.join("memory.sqlite")).unwrap();
    inspector.execute_batch("BEGIN IMMEDIATE").unwrap();
    let start = Instant::now();
    let (status, result) = ctl(&run, &daemon.socket, &run.scope, &["annoy", key], None);
    assert_eq!(status, 3, "{result}");
    assert!(start.elapsed() >= Duration::from_secs(1));
    assert!(start.elapsed() < Duration::from_secs(4));
    inspector.execute_batch("ROLLBACK").unwrap();
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &["meta", key], None).1["result"]["Ok"]["penalty"],
        4
    );
    drop(daemon);
    let daemon = Daemon::start(&run);
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &["meta", key], None).1["result"]["Ok"]["feedback_version"],
        4
    );
    let count: i64 = inspector
        .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn endpoint_startup_rejects_active_and_regular_files_without_replacing_them() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let inode = fs::metadata(&daemon.socket).unwrap().ino();
    let start = || {
        Command::new(env!("CARGO_BIN_EXE_exod"))
            .args([
                "--runtime",
                run.root.to_str().unwrap(),
                "--scope-root",
                run.scope.to_str().unwrap(),
                "--tenant",
                &run.policy.tenant,
                "--database",
                run.root.join("memory.sqlite").to_str().unwrap(),
                "--socket",
                run.root.join("exo.sock").to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    assert_eq!(start().status.code(), Some(3));
    assert_eq!(fs::metadata(&daemon.socket).unwrap().ino(), inode);
    assert_eq!(
        fs::metadata(&daemon.socket).unwrap().permissions().mode() & 0o777,
        0o600
    );
    drop(daemon);
    let daemon = Daemon::start(&run);
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["list-trigger-types"],
            None
        )
        .0,
        0
    );
    drop(daemon);
    fs::remove_file(run.root.join("exo.sock")).unwrap();
    fs::write(run.root.join("exo.sock"), "preserve me").unwrap();
    assert_eq!(start().status.code(), Some(3));
    assert_eq!(
        fs::read_to_string(run.root.join("exo.sock")).unwrap(),
        "preserve me"
    );
}

#[test]
fn kill_after_attempt_commit_never_retries_on_process_restart() {
    use exomemory::attempts::AttemptStore;
    let run = Run::new();
    let input = CandidateSnapshot {
        state_version: 1,
        candidates: vec![Candidate {
            ghost: ghost(&run),
            tenant: run.policy.tenant.clone(),
            scope: draft(&run).scope,
            relevance_band: 0,
            backend_rank: 0,
            penalty: 0,
            feedback_version: 0,
        }],
    };
    fs::write(
        run.root.join("harness/context.json"),
        serde_json::to_vec(&run.context).unwrap(),
    )
    .unwrap();
    fs::write(
        run.root.join("harness/event.json"),
        serde_json::to_vec(&event(&run, 1)).unwrap(),
    )
    .unwrap();
    fs::write(
        run.root.join("harness/candidates.json"),
        serde_json::to_vec(&input).unwrap(),
    )
    .unwrap();
    let ready = run.root.join("harness/ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_worker", "--nocapture"])
        .env("EXO_CRASH_ROOT", &run.root)
        .env("EXO_CRASH_READY", &ready)
        .env("EXO_CRASH_POINT", "after_attempt_mark")
        .stdout(Stdio::from(
            fs::File::create(run.root.join("logs/crash-worker.log")).unwrap(),
        ))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.exists() {
        assert!(child.try_wait().unwrap().is_none());
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("attempt mark timeout");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let mut attempts = AttemptStore::open(
        &run.policy,
        &run.root.join("harness/attempts.sqlite"),
        bounds(),
    )
    .unwrap();
    assert!(
        attempts
            .process(&event(&run, 10), &input)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        attempts.state(&run.context, "main").unwrap().last_attempted["fixture-key"],
        1
    );
    let mut compact = event(&run, 11);
    compact.kind = EventKind::CompactionCompleted;
    attempts.process(&compact, &input).unwrap();
    assert_eq!(attempts.process(&event(&run, 12), &input).unwrap().len(), 1);
    assert!(attempts.process(&compact, &input).is_err());
    assert!(
        attempts
            .process(&event(&run, 13), &input)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn exhausted_snapshot_version_rolls_back_every_kind_of_write() {
    let run = Run::new();
    let path = run.root.join("memory.sqlite");
    let mut db = Sqlite::open(&run.policy, &path).unwrap();
    let record = db.create(&run.policy, &run.context, draft(&run)).unwrap();
    let key = &record.metadata.ghost_key;
    let inspector = rusqlite::Connection::open(path).unwrap();
    inspector
        .execute("UPDATE store_state SET version=?1", [i64::MAX])
        .unwrap();
    assert_eq!(
        db.create(&run.policy, &run.context, draft(&run))
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    assert_eq!(
        db.overwrite(&run.policy, &run.context, key, 1, draft(&run))
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    assert_eq!(
        db.annoy(&run.policy, &run.context, key).unwrap_err().code,
        ErrorCode::Conflict
    );
    assert_eq!(db.read(&run.policy, &run.context, key).unwrap(), record);
    assert_eq!(db.state_version().unwrap(), i64::MAX as u64);
    for table in ["memories", "revisions", "triggers", "feedback"] {
        let count: i64 = inspector
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
    let count: i64 = inspector
        .query_row("SELECT count(*) FROM feedback_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
