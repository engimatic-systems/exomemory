CREATE TABLE store_state (singleton INTEGER PRIMARY KEY CHECK(singleton = 1), version INTEGER NOT NULL CHECK(version >= 0));
INSERT INTO store_state VALUES (1, 0);
CREATE TABLE memories (ghost_key TEXT PRIMARY KEY, current_revision INTEGER NOT NULL CHECK(current_revision > 0));
CREATE TABLE revisions (
    ghost_key TEXT NOT NULL REFERENCES memories(ghost_key),
    revision INTEGER NOT NULL CHECK(revision > 0),
    record_json TEXT NOT NULL CHECK(json_valid(record_json)),
    PRIMARY KEY (ghost_key, revision)
);
CREATE TABLE triggers (
    ghost_key TEXT NOT NULL,
    revision INTEGER NOT NULL,
    trigger_id TEXT NOT NULL CHECK(length(trigger_id) > 0),
    pattern TEXT NOT NULL,
    PRIMARY KEY (ghost_key, revision, trigger_id),
    FOREIGN KEY (ghost_key, revision) REFERENCES revisions(ghost_key, revision)
);
PRAGMA user_version = 1;
