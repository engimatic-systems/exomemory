mod support;
use exomemory::{
    app::{ranking, select},
    harness::Presence,
    types::*,
};
use std::collections::BTreeMap;
use support::*;
fn candidate(run: &Run, key: &str, source: &str) -> Candidate {
    let mut ghost = ghost(run);
    ghost.ghost_key = key.into();
    ghost.source_id.trigger = source.into();
    Candidate {
        ghost,
        tenant: run.policy.tenant.clone(),
        scope: Scope::Dir {
            root: run.scope.clone(),
        },
        relevance_band: 0,
        backend_rank: 0,
        penalty: 0,
    }
}
fn empty() -> Presence {
    Presence {
        version: 0,
        live: BTreeMap::new(),
        last_shown: BTreeMap::new(),
        last_event: 0,
    }
}
fn keys(
    run: &Run,
    event: &Event,
    candidates: Vec<Candidate>,
    presence: &Presence,
    config: &exomemory::render::Bounds,
) -> Vec<String> {
    select(
        &run.policy,
        event,
        &CandidateSnapshot {
            state_version: 1,
            candidates,
        },
        presence,
        config,
    )
    .unwrap()
    .iter()
    .map(|g| g.ghost().ghost_key.clone())
    .collect()
}
#[test]
fn every_lexicographic_field_and_never_shown_sentinel_have_specified_priority() {
    let run = Run::new();
    let mut event = event(&run, 10, 0);
    let nested = run.scope.join("nested");
    std::fs::create_dir(&nested).unwrap();
    event.context.cwd = nested.clone();
    let config = bounds();
    let a = candidate(&run, "a", "a");
    let b = candidate(&run, "b", "b");
    // Each earlier field wins even when all later fields oppose it.
    for field in 0..6 {
        let mut winner = b.clone();
        let mut loser = a.clone();
        let mut presence = empty();
        winner.penalty = 100;
        winner.backend_rank = 100;
        presence.last_shown.insert("b".into(), 8);
        match field {
            0 => {
                loser.relevance_band = 1;
                loser.scope = Scope::Dir {
                    root: nested.clone(),
                };
            }
            1 => {
                winner.scope = Scope::Dir {
                    root: nested.clone(),
                };
            }
            2 => {
                winner.penalty = 0;
                loser.penalty = 1;
            }
            3 => {
                winner.penalty = 0;
                winner.backend_rank = 0;
                loser.backend_rank = 1;
            }
            4 => {
                winner.penalty = 0;
                winner.backend_rank = 0;
                presence.last_shown.clear();
                presence.last_shown.insert("a".into(), 1);
            }
            5 => {
                winner = a.clone();
                loser = b.clone();
                presence.last_shown.clear();
            }
            _ => unreachable!(),
        }
        let expected = winner.ghost.ghost_key.clone();
        assert_eq!(
            keys(
                &run,
                &event,
                vec![loser.clone(), winner.clone()],
                &presence,
                &config
            )[0],
            expected,
            "field {field}"
        );
        assert_eq!(
            keys(&run, &event, vec![winner, loser], &presence, &config)[0],
            expected
        );
    }
    let mut global = a.clone();
    global.scope = Scope::Global;
    assert!(
        ranking(&a, &event.context.cwd, 0).unwrap()
            < ranking(&global, &event.context.cwd, 0).unwrap()
    );
}
#[test]
fn eligibility_precedes_duplicate_collapse_and_source_quotas() {
    let run = Run::new();
    let event = event(&run, 10, 0);
    let mut config = bounds();
    config.max_live = 2;
    let a = candidate(&run, "a", "broad");
    let b = candidate(&run, "b", "broad");
    let c = candidate(&run, "c", "other");
    let mut oversized = a.clone();
    oversized.relevance_band = 0;
    oversized.ghost.blurb = "<".repeat(1000);
    oversized.ghost.source_id.trigger = "earlier".into();
    let mut duplicate = a.clone();
    duplicate.backend_rank = 10;
    duplicate.ghost.source_id.trigger = "other".into();
    let inputs = vec![c.clone(), b.clone(), oversized, duplicate, a.clone()];
    let expected = vec!["a".to_string(), "c".into()];
    for shift in 0..inputs.len() {
        let mut permutation = inputs.clone();
        permutation.rotate_left(shift);
        assert_eq!(keys(&run, &event, permutation, &empty(), &config), expected);
    }
    let mut tied = a.clone();
    tied.ghost.source_id.trigger = "aaa".into();
    let selection = select(
        &run.policy,
        &event,
        &CandidateSnapshot {
            state_version: 1,
            candidates: vec![a.clone(), tied],
        },
        &empty(),
        &config,
    )
    .unwrap();
    assert_eq!(selection[0].ghost().source_id.trigger, "aaa");
    let mut presence = empty();
    presence.live.insert("old".into(), "a".into());
    assert_eq!(
        keys(
            &run,
            &event,
            vec![a.clone(), b.clone(), c.clone()],
            &presence,
            &config
        ),
        vec!["b"]
    );
    presence.live.insert("old2".into(), "z".into());
    assert!(keys(&run, &event, vec![b.clone(), c.clone()], &presence, &config).is_empty());
    presence = empty();
    presence.last_shown.insert("a".into(), 9);
    assert_eq!(
        keys(&run, &event, vec![a, b, c], &presence, &config),
        vec!["b", "c"]
    );
}
#[test]
fn scope_trust_schema_and_stale_presence_are_rejected() {
    let run = Run::new();
    let event = event(&run, 1, 0);
    let mut inaccessible = candidate(&run, "a", "a");
    inaccessible.tenant = "other".into();
    let mut malformed = candidate(&run, "b", "b");
    malformed.ghost.revision = 0;
    let good = candidate(&run, "c", "c");
    assert_eq!(
        keys(
            &run,
            &event,
            vec![inaccessible, malformed, good.clone()],
            &empty(),
            &bounds()
        ),
        vec!["c"]
    );
    let mut json = serde_json::to_value(&good).unwrap();
    json["ghost"]["trust"] = "developer".into();
    assert!(serde_json::from_value::<Candidate>(json).is_err());
    let mut stale = empty();
    stale.version = 1;
    assert!(
        select(
            &run.policy,
            &event,
            &CandidateSnapshot {
                state_version: 1,
                candidates: vec![good]
            },
            &stale,
            &bounds()
        )
        .is_err()
    );
}
