CREATE TABLE attempt_settings (singleton INTEGER PRIMARY KEY CHECK(singleton=1), settings TEXT NOT NULL);
CREATE TABLE sessions (session TEXT PRIMARY KEY, last_event INTEGER NOT NULL);
CREATE TABLE attempts (session TEXT NOT NULL, context TEXT NOT NULL, ghost_key TEXT NOT NULL, PRIMARY KEY(session,context,ghost_key));
CREATE TABLE last_attempted (session TEXT NOT NULL, ghost_key TEXT NOT NULL, event INTEGER NOT NULL, PRIMARY KEY(session,ghost_key));
PRAGMA user_version=2;
