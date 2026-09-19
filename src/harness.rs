use crate::{
    policy::Policy,
    render::{Bounds, RenderedGhost, WRAPPER, render},
    types::*,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presence {
    pub version: u64,
    pub live: BTreeMap<String, String>,
    pub last_shown: BTreeMap<String, u64>,
    pub last_event: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub presence_version: u64,
    pub occurrences: Vec<String>,
}
/// The controlled harness's actual live context consists of its SQLite live_items.
/// No transcript or cooldown inference is used. This is not a real Codex adapter.
pub struct ControlledHarness {
    connection: Connection,
    policy: Policy,
    bounds: Bounds,
}
fn conflict(message: &str) -> Error {
    Error::new(ErrorCode::Conflict, message)
}
fn signed(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| {
        Error::new(
            ErrorCode::InvalidInput,
            "logical counter exceeds supported range",
        )
    })
}
fn snapshot(connection: &Connection, session: &str, context: &str) -> Result<Presence> {
    let version = connection
        .query_row(
            "SELECT version FROM contexts WHERE session=?1 AND context=?2",
            params![session, context],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0) as u64;
    let last_event = connection
        .query_row(
            "SELECT last_event FROM sessions WHERE session=?1",
            params![session],
            |r| r.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0) as u64;
    let mut query=connection.prepare("SELECT occurrence,ghost_key FROM live_items WHERE session=?1 AND context=?2 ORDER BY occurrence")?;
    let live = query
        .query_map(params![session, context], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    let mut query = connection
        .prepare("SELECT ghost_key,event FROM last_shown WHERE session=?1 ORDER BY ghost_key")?;
    let last_shown = query
        .query_map(params![session], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
        })?
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    Ok(Presence {
        version,
        live,
        last_shown,
        last_event,
    })
}
impl ControlledHarness {
    pub fn open(policy: &Policy, path: &Path, bounds: Bounds) -> Result<Self> {
        bounds.validate()?;
        let settings = serde_json::to_string(&(&bounds, &policy.tenant, &policy.permitted_root))?;
        let mut connection = Connection::open(policy.runtime_path(path)?)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: u32 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 0 {
            tx.execute_batch(include_str!("../migrations/001_harness.sql"))?;
            tx.execute(
                "INSERT INTO harness_settings VALUES (1,?1)",
                params![settings],
            )?;
        } else if version != 1 {
            return Err(Error::new(
                ErrorCode::Unsupported,
                "unsupported harness schema",
            ));
        }
        let stored: String = tx.query_row(
            "SELECT bounds FROM harness_settings WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        if stored != settings {
            return Err(conflict(
                "harness bounds or scope differ from persisted context contract",
            ));
        }
        tx.commit()?;
        Ok(Self {
            connection,
            policy: policy.clone(),
            bounds,
        })
    }
    pub fn presence(&mut self, context: &Context, live_context: &str) -> Result<Presence> {
        self.policy.context(context)?;
        if live_context.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "live context id required",
            ));
        }
        let tx = self.connection.transaction()?;
        let presence = snapshot(&tx, &context.session, live_context)?;
        tx.commit()?;
        Ok(presence)
    }
    pub fn live_context(&mut self, context: &Context, live_context: &str) -> Result<String> {
        self.policy.context(context)?;
        let mut result = WRAPPER.to_owned();
        let mut query = self.connection.prepare(
            "SELECT serialized FROM live_items WHERE session=?1 AND context=?2 ORDER BY occurrence",
        )?;
        for row in query.query_map(params![context.session, live_context], |r| {
            r.get::<_, String>(0)
        })? {
            result.push_str(&row?);
        }
        Ok(result)
    }
    /// Rejected transport delivery changes no state. A successful retry returns its
    /// historical receipt without reinserting, even if compaction has since removed it.
    pub fn accept(
        &mut self,
        event: &Event,
        ghosts: &[RenderedGhost],
        accepted: bool,
    ) -> Result<Receipt> {
        self.policy.context(&event.context)?;
        if !accepted {
            return Err(Error::new(
                ErrorCode::Unavailable,
                "harness rejected delivery",
            ));
        }
        for item in ghosts {
            if render(item.ghost(), item.occurrence_id(), &self.bounds)? != *item {
                return Err(Error::new(
                    ErrorCode::InvalidInput,
                    "rendered representation differs",
                ));
            }
        }
        let payload = serde_json::to_string(&(
            "delivery",
            event,
            ghosts.iter().map(|g| g.serialized()).collect::<Vec<_>>(),
        ))?;
        self.mutate(event, &payload, Some(ghosts), None)
    }
    /// The retained set comes from this harness's actual context transition.
    pub fn compact(&mut self, event: &Event, retained: &BTreeSet<String>) -> Result<Receipt> {
        self.policy.context(&event.context)?;
        if event.kind != EventKind::Compaction {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "compaction event required",
            ));
        }
        let payload = serde_json::to_string(&("compaction", event, retained))?;
        self.mutate(event, &payload, None, Some(retained))
    }
    fn mutate(
        &mut self,
        event: &Event,
        payload: &str,
        ghosts: Option<&[RenderedGhost]>,
        retained: Option<&BTreeSet<String>>,
    ) -> Result<Receipt> {
        if event.live_context.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "live context id required",
            ));
        }
        let number = signed(event.number)?;
        let session = &event.context.session;
        let context = &event.live_context;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let prior: Option<(String, String)> = tx
            .query_row(
                "SELECT payload,receipt FROM receipts WHERE session=?1 AND event=?2",
                params![session, number],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((stored, receipt)) = prior {
            if stored != payload {
                return Err(conflict("logical event retry has different payload"));
            }
            return Ok(serde_json::from_str(&receipt)?);
        }
        let presence = snapshot(&tx, session, context)?;
        if presence.version != event.presence_version || event.number <= presence.last_event {
            return Err(conflict(
                "stale presence version or non-increasing session event",
            ));
        }
        let next = presence
            .version
            .checked_add(1)
            .ok_or_else(|| conflict("presence version exhausted"))?;
        let next_signed = signed(next)?;
        let mut occurrences = Vec::new();
        if let Some(items) = ghosts {
            if presence
                .live
                .len()
                .checked_add(items.len())
                .is_none_or(|n| n > self.bounds.max_live)
            {
                return Err(conflict("live ghost capacity exceeded"));
            }
            let mut keys: BTreeSet<_> = presence.live.values().cloned().collect();
            let mut ids: BTreeSet<_> = presence.live.keys().cloned().collect();
            for item in items {
                let key = &item.ghost().ghost_key;
                if !keys.insert(key.clone()) || !ids.insert(item.occurrence_id().into()) {
                    return Err(conflict("duplicate live ghost or occurrence"));
                }
                if presence.last_shown.get(key).is_some_and(|shown| {
                    event.number.saturating_sub(*shown) < self.bounds.cooldown_events
                }) {
                    return Err(conflict("ghost cooldown has not elapsed"));
                }
                tx.execute(
                    "INSERT INTO live_items VALUES (?1,?2,?3,?4,?5)",
                    params![
                        session,
                        context,
                        item.occurrence_id(),
                        key,
                        item.serialized()
                    ],
                )?;
                tx.execute("INSERT INTO last_shown VALUES (?1,?2,?3) ON CONFLICT(session,ghost_key) DO UPDATE SET event=excluded.event",params![session,key,number])?;
                occurrences.push(item.occurrence_id().to_owned());
            }
        }
        if let Some(retained) = retained {
            if retained.iter().any(|id| !presence.live.contains_key(id)) {
                return Err(conflict("retained occurrence is not live in this context"));
            }
            for occurrence in presence.live.keys() {
                if !retained.contains(occurrence) {
                    tx.execute(
                        "DELETE FROM live_items WHERE session=?1 AND context=?2 AND occurrence=?3",
                        params![session, context, occurrence],
                    )?;
                }
            }
            occurrences = retained.iter().cloned().collect();
        }
        tx.execute("INSERT INTO contexts VALUES (?1,?2,?3) ON CONFLICT(session,context) DO UPDATE SET version=excluded.version",params![session,context,next_signed])?;
        tx.execute("INSERT INTO sessions VALUES (?1,?2) ON CONFLICT(session) DO UPDATE SET last_event=excluded.last_event",params![session,number])?;
        let receipt = Receipt {
            presence_version: next,
            occurrences,
        };
        tx.execute(
            "INSERT INTO receipts VALUES (?1,?2,?3,?4)",
            params![session, number, payload, serde_json::to_string(&receipt)?],
        )?;
        tx.commit()?;
        Ok(receipt)
    }
}
