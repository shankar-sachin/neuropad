use crate::CoreResult;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct MetadataStore {
    conn: Connection,
}

impl MetadataStore {
    pub fn open<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> CoreResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notebook_index (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS recent_open (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                opened_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_notebook_index(
        &self,
        path: &str,
        title: &str,
        updated_at_iso: &str,
    ) -> CoreResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO notebook_index(path, title, updated_at)
            VALUES(?1, ?2, ?3)
            ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                updated_at = excluded.updated_at
            "#,
            params![path, title, updated_at_iso],
        )?;
        Ok(())
    }

    pub fn mark_recent_open(&self, path: &str, opened_at_iso: &str) -> CoreResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO recent_open(path, opened_at)
            VALUES(?1, ?2)
            ON CONFLICT(path) DO UPDATE SET
                opened_at = excluded.opened_at
            "#,
            params![path, opened_at_iso],
        )?;
        Ok(())
    }
}
