mod support;
use exomemory::{
    app::select,
    harness::ControlledHarness,
    render::WRAPPER,
    transport::{call, frame},
    types::*,
};
use std::{collections::BTreeSet, fs, io::Write, os::unix::net::UnixStream};
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
    config.max_live = 1;
    let mut harness = ControlledHarness::open(
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
    let first = event(&run, 1, 0);
    let snapshot = query(&first);
    assert_eq!(snapshot.state_version, 1);
    assert_eq!(snapshot.candidates.len(), 2);
    assert!(snapshot.candidates.iter().all(|c| c.ghost.ghost_key == *key
        && c.ghost.revision == 1
        && c.ghost.provenance == draft.provenance));
    assert_eq!(query(&first), snapshot);
    let presence = harness
        .presence(&first.context, &first.live_context)
        .unwrap();
    let selected = select(&run.policy, &first, &snapshot, &presence, &config).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].ghost().ghost_key, *key);
    let mut reverse = snapshot.clone();
    reverse.candidates.reverse();
    assert_eq!(
        select(&run.policy, &first, &reverse, &presence, &config).unwrap(),
        selected
    );
    assert!(harness.accept(&first, &selected, false).is_err());
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .live
            .is_empty()
    );
    let receipt = harness.accept(&first, &selected, true).unwrap();
    assert_eq!(receipt, harness.accept(&first, &selected, true).unwrap());
    let context = harness.live_context(&run.context, "main").unwrap();
    assert_eq!(context, format!("{}{}", WRAPPER, selected[0].serialized()));
    assert!(!context.contains(&record.body));
    assert!(context.len() <= config.context_envelope_bytes);
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
    // Cooldown has expired at event 10, yet the live occurrence still suppresses it.
    let again = event(&run, 10, 1);
    let presence = harness.presence(&run.context, "main").unwrap();
    let suppressed = select(&run.policy, &again, &query(&again), &presence, &config).unwrap();
    assert!(suppressed.is_empty());
    harness.accept(&again, &suppressed, true).unwrap();
    let mut compact = event(&run, 11, 2);
    compact.kind = EventKind::Compaction;
    harness.compact(&compact, &BTreeSet::new()).unwrap();
    assert!(
        harness
            .presence(&run.context, "main")
            .unwrap()
            .live
            .is_empty()
    );
    let later = event(&run, 12, 3);
    let resurfaced = select(
        &run.policy,
        &later,
        &query(&later),
        &harness.presence(&run.context, "main").unwrap(),
        &config,
    )
    .unwrap();
    assert_eq!(resurfaced.len(), 1);
    assert_ne!(selected[0].occurrence_id(), resurfaced[0].occurrence_id());
    harness.accept(&later, &resurfaced, true).unwrap();
    // Bypass client checks to independently exercise service-side access on reads,
    // metadata and event retrieval outside the permitted directory.
    let outside = Run::new();
    let mut outside_event = event(&run, 13, 4);
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
    // Replay the recorded events against the same committed memory snapshot in
    // a fresh live context. Every selection and final live byte sequence agrees.
    let mut replay = ControlledHarness::open(
        &run.policy,
        &run.root.join("harness/replay.sqlite"),
        config.clone(),
    )
    .unwrap();
    for (input, expected) in [(&first, &selected), (&again, &suppressed)] {
        let view = replay.presence(&run.context, "main").unwrap();
        let actual = select(&run.policy, input, &query(input), &view, &config).unwrap();
        assert_eq!(&actual, expected);
        replay.accept(input, &actual, true).unwrap();
    }
    replay.compact(&compact, &BTreeSet::new()).unwrap();
    let view = replay.presence(&run.context, "main").unwrap();
    let actual = select(&run.policy, &later, &query(&later), &view, &config).unwrap();
    assert_eq!(actual, resurfaced);
    replay.accept(&later, &actual, true).unwrap();
    assert_eq!(
        replay.live_context(&run.context, "main").unwrap(),
        harness.live_context(&run.context, "main").unwrap()
    );
    let evidence = serde_json::json!({"record":record,"candidate_snapshot":snapshot,"events":[first,again,compact,later],"replay_identical":true,"first_occurrence":selected[0].occurrence_id(),"resurfaced_occurrence":resurfaced[0].occurrence_id(),"live_context":harness.live_context(&run.context,"main").unwrap()});
    fs::write(
        run.root.join("harness/m1-evidence.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    println!(
        "M1 passed: create → ghost → exoctl read/meta → live suppression → compaction → resurfacing. Evidence: {}",
        run.root.display()
    );
}
