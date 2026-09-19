mod support;
use exomemory::{
    attempts::AttemptStore,
    render::WRAPPER,
    transport::{call, frame},
    types::*,
};
use std::{fs, io::Write, os::unix::net::UnixStream};
use support::*;
#[test]
fn create_surface_read_compact_and_resurface_through_running_service() {
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/m1.json")).unwrap();
    value["scope"] = serde_json::to_value(Scope::Dir {
        root: run.scope.clone(),
    })
    .unwrap();
    let draft: Draft = serde_json::from_value(value).unwrap();
    let (status, created) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["create", "--json", "-"],
        Some(&draft),
    );
    assert_eq!(status, 0, "{created}");
    let record: Record = serde_json::from_value(created["result"]["Ok"].clone()).unwrap();
    let key = &record.metadata.ghost_key;
    assert_eq!(record.body, draft.body);
    assert_eq!(record.metadata.revision, 1);
    assert_eq!(record.metadata.provenance, draft.provenance);
    let mut config = bounds();
    config.max_attempts = 1;
    let mut harness = AttemptStore::open(
        &run.policy,
        &run.root.join("harness/context.sqlite"),
        config.clone(),
    )
    .unwrap();
    let query = |event: &Event| -> CandidateSnapshot {
        serde_json::from_value(
            call(
                &run.policy,
                &daemon.socket,
                &Request {
                    version: 1,
                    context: event.context.clone(),
                    operation: Operation::Candidates {
                        event: event.clone(),
                    },
                },
            )
            .unwrap(),
        )
        .unwrap()
    };
    let first = event(&run, 1);
    let snapshot = query(&first);
    assert_eq!(snapshot.state_version, 1);
    assert_eq!(snapshot.candidates.len(), 2);
    assert!(snapshot.candidates.iter().all(|c| c.ghost.ghost_key == *key
        && c.ghost.revision == 1
        && c.ghost.provenance == draft.provenance));
    assert_eq!(query(&first), snapshot);
    let selected = harness.process(&first, &snapshot).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].ghost().ghost_key, *key);
    // The controlled receiver sees bytes only after the attempt has been committed.
    let context = format!("{}{}", WRAPPER, selected[0].serialized());
    assert!(!context.contains(&record.body));
    assert!(context.len() <= config.interval_envelope_bytes);
    let (status, read) = ctl(&run, &daemon.socket, &run.scope, &[key], None);
    assert_eq!(status, 0);
    assert_eq!(
        serde_json::from_value::<Record>(read["result"]["Ok"].clone()).unwrap(),
        record
    );
    let (status, meta) = ctl(&run, &daemon.socket, &run.scope, &["meta", key], None);
    assert_eq!(status, 0);
    assert_eq!(
        serde_json::from_value::<Metadata>(meta["result"]["Ok"].clone()).unwrap(),
        record.metadata
    );
    // Expired cooldown does not allow a second attempt in the same interval.
    let again = event(&run, 10);
    let suppressed = harness.process(&again, &query(&again)).unwrap();
    assert!(suppressed.is_empty());
    let mut compact = event(&run, 11);
    compact.kind = EventKind::CompactionCompleted;
    harness.process(&compact, &query(&compact)).unwrap();
    assert!(
        harness
            .state(&run.context, "main")
            .unwrap()
            .attempted
            .is_empty()
    );
    let later = event(&run, 12);
    let resurfaced = harness.process(&later, &query(&later)).unwrap();
    assert_eq!(resurfaced, selected);
    // Bypass client checks to independently exercise service-side access on reads,
    // metadata and event retrieval outside the permitted directory.
    let outside = Run::new();
    let mut outside_event = event(&run, 13);
    outside_event.context.cwd = outside.scope.clone();
    for operation in [
        Operation::Read { key: key.clone() },
        Operation::Meta { key: key.clone() },
        Operation::Candidates {
            event: outside_event.clone(),
        },
    ] {
        let mut stream = UnixStream::connect(&daemon.socket).unwrap();
        let request = Request {
            version: 1,
            context: outside_event.context.clone(),
            operation,
        };
        writeln!(stream, "{}", serde_json::to_string(&request).unwrap()).unwrap();
        let result: Response = serde_json::from_slice(&frame(stream).unwrap()).unwrap();
        assert_eq!(result.result.unwrap_err().code, ErrorCode::Forbidden);
    }
    assert_eq!(ctl(&run, &daemon.socket, &outside.scope, &[key], None).0, 4);
    // Reversing the candidate input preserves deterministic selection in a fresh store.
    let mut replay = AttemptStore::open(
        &run.policy,
        &run.root.join("harness/replay.sqlite"),
        config.clone(),
    )
    .unwrap();
    for (input, expected) in [(&first, &selected), (&again, &suppressed)] {
        let mut candidates = query(input);
        candidates.candidates.reverse();
        assert_eq!(&replay.process(input, &candidates).unwrap(), expected);
    }
    replay.process(&compact, &query(&compact)).unwrap();
    assert_eq!(replay.process(&later, &query(&later)).unwrap(), resurfaced);
    assert_eq!(
        replay.state(&run.context, "main").unwrap(),
        harness.state(&run.context, "main").unwrap()
    );
    let evidence = serde_json::json!({"record":record,"candidate_snapshot":snapshot,"events":[first,again,compact,later],"replay_identical":true,"first_delivery":context,"attempt_state":harness.state(&run.context,"main").unwrap(),"delivery_policy":"best_effort_no_retry"});
    fs::write(
        run.root.join("harness/m1-evidence.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    println!(
        "M1 passed: create → ghost → exoctl read/meta → attempt suppression → compaction → resurfacing. Evidence: {}",
        run.root.display()
    );
}
