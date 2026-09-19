-- Feedback evolves independently of immutable body revisions.
CREATE TABLE feedback (
    ghost_key TEXT PRIMARY KEY REFERENCES memories(ghost_key),
    penalty INTEGER NOT NULL CHECK(penalty BETWEEN 0 AND 4294967295),
    version INTEGER NOT NULL CHECK(version >= 0)
);
INSERT INTO feedback SELECT m.ghost_key,json_extract(r.record_json,'$.metadata.penalty'),0
FROM memories m JOIN revisions r ON r.ghost_key=m.ghost_key AND r.revision=m.current_revision;
CREATE TABLE feedback_events (
    state_version INTEGER PRIMARY KEY CHECK(state_version > 0),
    ghost_key TEXT NOT NULL REFERENCES memories(ghost_key),
    feedback_version INTEGER NOT NULL CHECK(feedback_version > 0),
    tenant TEXT NOT NULL,
    session TEXT NOT NULL,
    cwd TEXT NOT NULL,
    penalty INTEGER NOT NULL CHECK(penalty BETWEEN 0 AND 4294967295),
    UNIQUE(ghost_key, feedback_version)
);
PRAGMA user_version = 2;
