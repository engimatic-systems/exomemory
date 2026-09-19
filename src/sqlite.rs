use crate::{
    data::{Capabilities, Storage},
    policy::Policy,
    types::*,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::{collections::BTreeSet, path::Path, time::Duration};
pub const MAX_RECORD_BYTES: usize = 1024 * 1024;
pub struct Sqlite {
    pub(crate) connection: Connection,
}
impl Sqlite {
    pub fn open(policy: &Policy, path: &Path) -> Result<Self> {
        let path = policy.runtime_path(path)?;
        let mut connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
        )?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: u32 = transaction.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match version {
            0 => transaction.execute_batch(include_str!("../migrations/001_initial.sql"))?,
            1 => {}
            _ => {
                return Err(Error::new(
                    ErrorCode::Unsupported,
                    "unsupported SQLite schema version",
                ));
            }
        }
        transaction.commit()?;
        Ok(Self { connection })
    }
    pub fn state_version(&self) -> Result<u64> {
        Ok(self.connection.query_row(
            "SELECT version FROM store_state WHERE singleton=1",
            [],
            |r| r.get::<_, i64>(0),
        )? as u64)
    }
}
pub(crate) fn validate_draft(policy: &Policy, context: &Context, draft: &mut Draft) -> Result<()> {
    draft.scope = policy.write_scope(context, &draft.scope)?;
    if draft.backend.kind != "sqlite" || draft.backend.instance != "local" {
        return Err(Error::new(
            ErrorCode::Unsupported,
            "only backend sqlite/local is registered",
        ));
    }
    if draft.blurb.trim().is_empty()
        || draft.provenance.origin.trim().is_empty()
        || draft.provenance.actor.trim().is_empty()
        || draft.triggers.is_empty()
    {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "blurb, provenance and non-empty triggers required",
        ));
    }
    let mut ids = BTreeSet::new();
    for Trigger::Regex { id, pattern } in &draft.triggers {
        if id.is_empty() || !ids.insert(id) {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "trigger ids must be nonempty and unique per revision",
            ));
        }
        regex::RegexBuilder::new(pattern)
            .size_limit(1024 * 1024)
            .build()
            .map_err(|e| Error::new(ErrorCode::InvalidInput, format!("invalid regex: {e}")))?;
    }
    if serde_json::to_vec(draft)?.len() > MAX_RECORD_BYTES {
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "record exceeds supported size",
        ));
    }
    Ok(())
}
pub(crate) fn decode_record(json: &str) -> Result<Record> {
    serde_json::from_str(json)
        .map_err(|e| Error::new(ErrorCode::Internal, format!("invalid stored record: {e}")))
}
impl Storage for Sqlite {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            read: true,
            create: true,
            overwrite: false,
            feedback: false,
            candidates: true,
        }
    }
    fn create(&mut self, policy: &Policy, context: &Context, mut draft: Draft) -> Result<Record> {
        validate_draft(policy, context, &mut draft)?;
        let key = uuid::Uuid::new_v4().to_string();
        let metadata = Metadata {
            ghost_key: key.clone(),
            revision: 1,
            tenant: context.tenant.clone(),
            blurb: draft.blurb,
            uses: draft.uses,
            topics: draft.topics,
            triggers: draft.triggers,
            backend: draft.backend,
            scope: draft.scope,
            provenance: draft.provenance,
            trust: draft.trust,
            overwritable: draft.overwritable,
            penalty: 0,
            read_token_estimate: 0,
        };
        let mut record = Record {
            metadata,
            body: draft.body,
        };
        // Deterministic UTF-8 byte/4 approximation of full JSON retrieval.
        // Iterate to account for the estimate field's own decimal width.
        loop {
            let estimate = serde_json::to_vec(&record)?.len().div_ceil(4) as u64;
            if estimate == record.metadata.read_token_estimate {
                break;
            }
            record.metadata.read_token_estimate = estimate;
        }
        let serialized = serde_json::to_string(&record)?;
        if serialized.len() > MAX_RECORD_BYTES {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "complete record exceeds supported size",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO memories VALUES (?1,1)", params![key])?;
        tx.execute(
            "INSERT INTO revisions VALUES (?1,1,?2)",
            params![key, serialized],
        )?;
        for Trigger::Regex { id, pattern } in &record.metadata.triggers {
            tx.execute(
                "INSERT INTO triggers VALUES (?1,1,?2,?3)",
                params![key, id, pattern],
            )?;
        }
        tx.execute(
            "UPDATE store_state SET version=version+1 WHERE singleton=1",
            [],
        )?;
        tx.commit()?;
        Ok(record)
    }
    fn read(&mut self, policy: &Policy, context: &Context, key: &str) -> Result<Record> {
        policy.context(context)?;
        let json:Option<String>=self.connection.query_row("SELECT r.record_json FROM memories m JOIN revisions r ON r.ghost_key=m.ghost_key AND r.revision=m.current_revision WHERE m.ghost_key=?1",params![key],|r|r.get(0)).optional()?;
        let record = decode_record(
            &json.ok_or_else(|| Error::new(ErrorCode::NotFound, "memory not found"))?,
        )?;
        policy.access(context, &record.metadata.tenant, &record.metadata.scope)?;
        Ok(record)
    }

    fn candidates(&mut self, policy: &Policy, event: &Event) -> Result<CandidateSnapshot> {
        policy.context(&event.context)?;
        if event.text.len() > MAX_RECORD_BYTES {
            return Err(Error::new(ErrorCode::InvalidInput, "event text too large"));
        }
        let text = event.text.replace("\r\n", "\n").replace('\r', "\n");
        let tx = self.connection.transaction()?;
        // This first SELECT pins the snapshot for the entire matching/query pass.
        let state_version = tx.query_row(
            "SELECT version FROM store_state WHERE singleton=1",
            [],
            |r| r.get::<_, i64>(0),
        )? as u64;
        let triggers = {
            let mut query = tx.prepare("SELECT t.ghost_key,t.trigger_id,t.pattern FROM triggers t JOIN memories m ON m.ghost_key=t.ghost_key AND m.current_revision=t.revision JOIN revisions r ON r.ghost_key=t.ghost_key AND r.revision=t.revision WHERE json_extract(r.record_json,'$.metadata.tenant')=?1 ORDER BY t.ghost_key,t.trigger_id")?;
            query
                .query_map(params![event.context.tenant], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let mut candidates = Vec::new();
        let mut ranks = std::collections::BTreeMap::<SourceId, u64>::new();
        for (key, trigger, pattern) in triggers {
            let regex = regex::RegexBuilder::new(&pattern)
                .size_limit(1024 * 1024)
                .build()
                .map_err(|e| {
                    Error::new(ErrorCode::Internal, format!("stored regex invalid: {e}"))
                })?;
            if !regex.is_match(&text) {
                continue;
            }
            // Built-in read-only template. Neither event text nor a pattern is SQL syntax.
            let json: String = tx.query_row("SELECT r.record_json FROM revisions r JOIN memories m ON m.ghost_key=r.ghost_key AND m.current_revision=r.revision JOIN triggers t ON t.ghost_key=r.ghost_key AND t.revision=r.revision WHERE r.ghost_key=?1 AND t.trigger_id=?2 AND json_extract(r.record_json,'$.metadata.tenant')=?3 ORDER BY r.ghost_key,r.revision", params![key,trigger,event.context.tenant], |r| r.get(0))?;
            let metadata = decode_record(&json)?.metadata;
            if policy
                .access(&event.context, &metadata.tenant, &metadata.scope)
                .is_err()
            {
                continue;
            }
            let source = SourceId {
                backend: format!("{}/{}", metadata.backend.kind, metadata.backend.instance),
                trigger,
            };
            let rank = ranks.entry(source.clone()).or_default();
            let ghost = Ghost {
                ghost_key: metadata.ghost_key,
                revision: metadata.revision,
                blurb: metadata.blurb,
                uses: metadata.uses,
                topics: metadata.topics,
                read_token_estimate: metadata.read_token_estimate,
                source_id: source,
                provenance: metadata.provenance,
                trust: metadata.trust,
            };
            candidates.push(Candidate {
                ghost,
                tenant: metadata.tenant,
                scope: metadata.scope,
                relevance_band: 0,
                backend_rank: *rank,
                penalty: metadata.penalty,
            });
            *rank += 1;
        }
        tx.commit()?;
        Ok(CandidateSnapshot {
            state_version,
            candidates,
        })
    }
}
