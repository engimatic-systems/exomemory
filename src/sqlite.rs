/// Compiled-in SQLite boundary; transaction implementation follows in exo-b12v.
pub struct Sqlite {
    pub(crate) connection: rusqlite::Connection,
}
impl Sqlite {
    pub fn sqlite_version(&self) -> Result<String, rusqlite::Error> {
        self.connection
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
    }
}
