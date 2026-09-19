CREATE TABLE harness_settings (singleton INTEGER PRIMARY KEY CHECK(singleton=1), bounds TEXT NOT NULL);
CREATE TABLE sessions (session TEXT PRIMARY KEY, last_event INTEGER NOT NULL);
CREATE TABLE contexts (session TEXT NOT NULL, context TEXT NOT NULL, version INTEGER NOT NULL, PRIMARY KEY(session,context));
CREATE TABLE live_items (session TEXT NOT NULL, context TEXT NOT NULL, occurrence TEXT NOT NULL, ghost_key TEXT NOT NULL, serialized TEXT NOT NULL, PRIMARY KEY(session,context,occurrence), UNIQUE(session,context,ghost_key));
CREATE TABLE last_shown (session TEXT NOT NULL, ghost_key TEXT NOT NULL, event INTEGER NOT NULL, PRIMARY KEY(session,ghost_key));
CREATE TABLE receipts (session TEXT NOT NULL, event INTEGER NOT NULL, payload TEXT NOT NULL, receipt TEXT NOT NULL, PRIMARY KEY(session,event));
PRAGMA user_version=1;
