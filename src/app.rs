use crate::{
    attempts::AttemptState,
    policy::Policy,
    render::{Bounds, RenderedGhost, render},
    types::*,
};
use std::{collections::BTreeMap, path::Path};
/// Ascending lexicographic order, exactly as specified. Never attempted uses event 0;
/// valid logical events start at 1. Source is only a final duplicate tie-break.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ranking {
    pub relevance_band: u32,
    pub scope_distance: usize,
    pub penalty: u32,
    pub backend_rank: u64,
    pub last_attempted: u64,
    pub ghost_key: String,
    pub source: SourceId,
}
pub fn ranking(candidate: &Candidate, cwd: &Path, last_attempted: u64) -> Result<Ranking> {
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
        last_attempted,
        ghost_key: candidate.ghost.ghost_key.clone(),
        source: candidate.ghost.source_id.clone(),
    })
}
pub fn select(
    policy: &Policy,
    event: &Event,
    snapshot: &CandidateSnapshot,
    state: &AttemptState,
    bounds: &Bounds,
) -> Result<Vec<RenderedGhost>> {
    bounds.validate()?;
    policy.context(&event.context)?;
    if event.number <= state.last_event || event.context_id.is_empty() {
        return Err(Error::new(
            ErrorCode::Conflict,
            "selection has stale event identity",
        ));
    }
    let remaining = bounds
        .max_attempts
        .checked_sub(state.attempted.len())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::Conflict,
                "attempt set already exceeds interval bound",
            )
        })?;
    let mut unique = BTreeMap::<String, (Ranking, RenderedGhost)>::new();
    for candidate in &snapshot.candidates {
        if policy
            .access(&event.context, &candidate.tenant, &candidate.scope)
            .is_err()
            || state.attempted.contains(&candidate.ghost.ghost_key)
        {
            continue;
        }
        let shown = state
            .last_attempted
            .get(&candidate.ghost.ghost_key)
            .copied();
        if shown.is_some_and(|n| event.number.saturating_sub(n) < bounds.cooldown_events) {
            continue;
        }
        let Ok(rendered) = render(&candidate.ghost, bounds) else {
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
