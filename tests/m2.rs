mod support;
use exomemory::{app::select, attempts::AttemptState, transport, types::*};
use serde_json::{Value, json};
use support::*;
fn session(run: &Run, name: &str) -> Run {
    let mut context = run.context.clone();
    context.session = name.into();
    Run {
        root: run.root.clone(),
        scope: run.scope.clone(),
        policy: run.policy.clone(),
        context,
    }
}
fn query(run: &Run, daemon: &Daemon) -> CandidateSnapshot {
    serde_json::from_value(
        transport::call(
            &run.policy,
            &daemon.socket,
            &Request {
                version: PROTOCOL_VERSION,
                context: run.context.clone(),
                operation: Operation::Candidates {
                    event: event(run, 1),
                },
            },
        )
        .unwrap(),
    )
    .unwrap()
}
fn selected(run: &Run, snapshot: &CandidateSnapshot) -> String {
    let selected = select(
        &run.policy,
        &event(run, 1),
        snapshot,
        &AttemptState::default(),
        &bounds(),
    )
    .unwrap();
    assert_eq!(selected.len(), 1); // Both fixtures share one trigger source and its quota is one.
    selected[0].ghost().ghost_key.clone()
}
#[test]
fn concurrent_sessions_reconcile_then_feedback_and_restart_reproduce_selection() {
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let a = session(&run, "session-a");
    let b = session(&run, "session-b");
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
    for client in [&a, &b] {
        let (status, read) = ctl(client, &daemon.socket, &client.scope, &[key], None);
        assert_eq!(status, 0);
        assert_eq!(read, created);
    }
    let barrier = std::sync::Barrier::new(2);
    let outcomes = std::thread::scope(|scope| {
        let jobs: Vec<_> = [(&a, 15), (&b, 18)]
            .into_iter()
            .map(|(client, seconds)| {
                let mut replacement = draft(client);
                replacement.body = format!("The test suite took {seconds} seconds.");
                replacement.blurb = format!("Test suite: {seconds} seconds");
                let barrier = &barrier;
                let socket = &daemon.socket;
                scope.spawn(move || {
                    barrier.wait();
                    ctl(
                        client,
                        socket,
                        &client.scope,
                        &["overwrite", key, "--if-revision", "1", "--json", "-"],
                        Some(&replacement),
                    )
                })
            })
            .collect();
        jobs.into_iter()
            .map(|j| j.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(outcomes.iter().filter(|r| r.0 == 0).count(), 1);
    assert_eq!(outcomes.iter().filter(|r| r.0 == 7).count(), 1);
    let winner = &outcomes.iter().find(|r| r.0 == 0).unwrap().1["result"]["Ok"];
    let conflict = &outcomes.iter().find(|r| r.0 == 7).unwrap().1["result"]["Err"];
    assert_eq!(conflict["current"], winner["metadata"]);
    let loser = if outcomes[0].0 == 7 { &a } else { &b };
    let (status, current) = ctl(loser, &daemon.socket, &loser.scope, &[key], None);
    assert_eq!(status, 0);
    assert_eq!(&current["result"]["Ok"], winner);
    let mut reconciliation = draft(&run);
    reconciliation.body = format!(
        "{} Later measurement: 20 seconds.",
        winner["body"].as_str().unwrap()
    );
    reconciliation.blurb = "Test suite: latest measurement 20 seconds".into();
    let (status, reconciled) = ctl(
        loser,
        &daemon.socket,
        &loser.scope,
        &["overwrite", key, "--if-revision", "2", "--json", "-"],
        Some(&reconciliation),
    );
    assert_eq!(status, 0);
    assert_eq!(reconciled["result"]["Ok"]["metadata"]["revision"], 3);
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
    let before = query(&run, &daemon);
    assert_eq!(before.candidates.len(), 2);
    let chosen = selected(&run, &before);
    let (status, feedback) = ctl(&b, &daemon.socket, &b.scope, &["annoy", &chosen], None);
    assert_eq!(status, 0);
    assert_eq!(feedback["result"]["Ok"]["session"], "session-b");
    let after = query(&run, &daemon);
    assert_eq!(after.candidates.len(), 2);
    assert_ne!(selected(&run, &after), chosen);
    drop(daemon);
    let daemon = Daemon::start(&run);
    assert_eq!(query(&run, &daemon), after);
    let (status, read) = ctl(&a, &daemon.socket, &a.scope, &[key], None);
    assert_eq!(status, 0);
    assert_eq!(read["result"]["Ok"]["body"], reconciliation.body);
    assert_eq!(
        read["result"]["Ok"]["metadata"]["blurb"],
        reconciliation.blurb
    );
    let evidence = json!({"concurrent_outcomes":outcomes,"reconciled":reconciled,"feedback":feedback,"before":before,"after":after,"selected_before":chosen,"selected_after":selected(&run,&after)});
    let artifact = run.root.join("harness/m2-evidence.json");
    std::fs::write(&artifact, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    let replay: Value = serde_json::from_slice(&std::fs::read(artifact).unwrap()).unwrap();
    let saved: CandidateSnapshot = serde_json::from_value(replay["before"].clone()).unwrap();
    assert_eq!(selected(&run, &saved), chosen);
    println!("M2 evidence: {}", run.root.display());
}
