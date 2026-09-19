use crate::{
    app::select,
    policy::Policy,
    render::{Bounds, RenderedGhost},
    types::*,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptState {
    pub attempted: BTreeSet<String>,
    pub last_attempted: BTreeMap<String, u64>,
    pub last_event: u64,
}

/// App-server bookkeeping for attempts, independent of the host's live context.
pub struct AttemptStore {
    connection: Connection,
    policy: Policy,
    bounds: Bounds,
}

fn snapshot(connection: &Connection, session: &str, context: &str) -> Result<AttemptState> {
    let last_event = connection
        .query_row(
            "SELECT last_event FROM sessions WHERE session=?1",
            [session],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0) as u64;
    let mut query = connection.prepare(
        "SELECT ghost_key FROM attempts WHERE session=?1 AND context=?2 ORDER BY ghost_key",
    )?;
    let attempted = query
        .query_map(params![session, context], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let mut query = connection.prepare(
        "SELECT ghost_key,event FROM last_attempted WHERE session=?1 ORDER BY ghost_key",
    )?;
    let last_attempted = query
        .query_map([session], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
        })?
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    Ok(AttemptState {
        attempted,
        last_attempted,
        last_event,
    })
}

impl AttemptStore {
    pub fn open(policy: &Policy, path: &Path, bounds: Bounds) -> Result<Self> {
        bounds.validate()?;
        let settings = serde_json::to_string(&(&bounds, &policy.tenant, &policy.permitted_root))?;
        let mut connection = Connection::open(policy.runtime_path(path)?)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: u32 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 0 {
            tx.execute_batch(include_str!("../migrations/002_attempts.sql"))?;
            tx.execute("INSERT INTO attempt_settings VALUES (1,?1)", [&settings])?;
        } else if version != 2 {
            // Old controlled-harness files cannot establish which failed sends were attempted.
            return Err(Error::new(
                ErrorCode::Unsupported,
                "legacy harness schema; use a new attempt database for a new context",
            ));
        }
        let stored: String = tx.query_row(
            "SELECT settings FROM attempt_settings WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        if stored != settings {
            return Err(Error::new(
                ErrorCode::Conflict,
                "attempt bounds or scope differ from persisted settings",
            ));
        }
        tx.commit()?;
        Ok(Self {
            connection,
            policy: policy.clone(),
            bounds,
        })
    }

    pub fn state(&mut self, context: &Context, context_id: &str) -> Result<AttemptState> {
        self.policy.context(context)?;
        if context_id.is_empty() {
            return Err(Error::new(ErrorCode::InvalidInput, "context id required"));
        }
        let tx = self.connection.transaction()?;
        let state = snapshot(&tx, &context.session, context_id)?;
        tx.commit()?;
        Ok(state)
    }

    /// Commit selected keys before returning bytes to send once; no delivery receipts or retries.
    pub fn process(
        &mut self,
        event: &Event,
        candidates: &CandidateSnapshot,
    ) -> Result<Vec<RenderedGhost>> {
        self.policy.context(&event.context)?;
        if event.context_id.is_empty() || event.number == 0 || event.number > i64::MAX as u64 {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "context id and positive SQLite-range event number required",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let state = snapshot(&tx, &event.context.session, &event.context_id)?;
        if event.number <= state.last_event {
            return Err(Error::new(
                ErrorCode::Conflict,
                "duplicate or out-of-order session event",
            ));
        }
        let selected = if event.kind == EventKind::CompactionCompleted {
            // Completion clears only this context; session cooldown and ordering survive.
            tx.execute(
                "DELETE FROM attempts WHERE session=?1 AND context=?2",
                params![event.context.session, event.context_id],
            )?;
            Vec::new()
        } else {
            let selected = select(&self.policy, event, candidates, &state, &self.bounds)?;
            for item in &selected {
                let key = &item.ghost().ghost_key;
                tx.execute(
                    "INSERT INTO attempts VALUES (?1,?2,?3)",
                    params![event.context.session, event.context_id, key],
                )?;
                tx.execute("INSERT INTO last_attempted VALUES (?1,?2,?3) ON CONFLICT(session,ghost_key) DO UPDATE SET event=excluded.event", params![event.context.session, key, event.number as i64])?;
            }
            selected
        };
        tx.execute("INSERT INTO sessions VALUES (?1,?2) ON CONFLICT(session) DO UPDATE SET last_event=excluded.last_event", params![event.context.session, event.number as i64])?;
        tx.commit()?;
        Ok(selected)
    }
}
