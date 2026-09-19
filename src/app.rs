use crate::{
    harness::Presence,
    policy::Policy,
    render::{Bounds, RenderedGhost, render},
    types::*,
};
use std::{collections::BTreeMap, path::Path};
/// Ascending lexicographic order, exactly as specified. Never shown uses event 0;
/// valid logical events start at 1. Source is only a final duplicate tie-break.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ranking {
    pub relevance_band: u32,
    pub scope_distance: usize,
    pub penalty: u32,
    pub backend_rank: u64,
    pub last_shown: u64,
    pub ghost_key: String,
    pub source: SourceId,
}
pub fn ranking(candidate: &Candidate, cwd: &Path, last_shown: u64) -> Result<Ranking> {
    let scope_distance = match &candidate.scope {
        Scope::Global => usize::MAX,
        Scope::Dir { root } => {
            let root = std::fs::canonicalize(root)?;
            let cwd = std::fs::canonicalize(cwd)?;
            if !cwd.starts_with(&root) {
                return Err(Error::new(ErrorCode::Forbidden, "scope not applicable"));
            }
            cwd.components().count() - root.components().count()
        }
    };
    Ok(Ranking {
        relevance_band: candidate.relevance_band,
        scope_distance,
        penalty: candidate.penalty,
        backend_rank: candidate.backend_rank,
        last_shown,
        ghost_key: candidate.ghost.ghost_key.clone(),
        source: candidate.ghost.source_id.clone(),
    })
}
pub fn select(
    policy: &Policy,
    event: &Event,
    snapshot: &CandidateSnapshot,
    presence: &Presence,
    bounds: &Bounds,
) -> Result<Vec<RenderedGhost>> {
    bounds.validate()?;
    policy.context(&event.context)?;
    if event.presence_version != presence.version
        || event.number <= presence.last_event
        || event.live_context.is_empty()
    {
        return Err(Error::new(
            ErrorCode::Conflict,
            "selection has stale presence or event identity",
        ));
    }
    let remaining = bounds
        .max_live
        .checked_sub(presence.live.len())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::Conflict,
                "authoritative context already exceeds count bound",
            )
        })?;
    let mut unique = BTreeMap::<String, (Ranking, RenderedGhost)>::new();
    for candidate in &snapshot.candidates {
        if policy
            .access(&event.context, &candidate.tenant, &candidate.scope)
            .is_err()
            || presence
                .live
                .values()
                .any(|key| key == &candidate.ghost.ghost_key)
        {
            continue;
        }
        let shown = presence.last_shown.get(&candidate.ghost.ghost_key).copied();
        if shown.is_some_and(|n| event.number.saturating_sub(n) < bounds.cooldown_events) {
            continue;
        }
        let occurrence = format!("{}:{}", event.number, candidate.ghost.ghost_key);
        let Ok(rendered) = render(&candidate.ghost, &occurrence, bounds) else {
            continue;
        };
        let rank = ranking(candidate, &event.context.cwd, shown.unwrap_or(0))?;
        // Encoding is only a last-resort tie-break for contradictory equal-rank
        // input from a plugin; canonical SQLite duplicates have identical fields.
        let replace = unique
            .get(&candidate.ghost.ghost_key)
            .is_none_or(|(old, old_rendered)| {
                (&rank, rendered.serialized()) < (old, old_rendered.serialized())
            });
        if replace {
            unique.insert(candidate.ghost.ghost_key.clone(), (rank, rendered));
        }
    }
    let mut ordered: Vec<_> = unique.into_values().collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sources = BTreeMap::<SourceId, usize>::new();
    let mut selected = Vec::new();
    for (_, rendered) in ordered {
        if selected.len() == remaining {
            break;
        }
        let used = sources
            .entry(rendered.ghost().source_id.clone())
            .or_default();
        if *used == bounds.per_source {
            continue;
        }
        *used += 1;
        selected.push(rendered);
    }
    Ok(selected)
}
