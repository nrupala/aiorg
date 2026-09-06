use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        let conn = Connection::open(root.join("aiorg.db"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS runs (id TEXT PRIMARY KEY, project TEXT NOT NULL, brief TEXT NOT NULL, status TEXT NOT NULL, created INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS events (seq INTEGER PRIMARY KEY AUTOINCREMENT, run_id TEXT NOT NULL, kind TEXT NOT NULL, payload TEXT NOT NULL, prev_hash TEXT NOT NULL, hash TEXT NOT NULL); CREATE TABLE IF NOT EXISTS artifacts (run_id TEXT NOT NULL, name TEXT NOT NULL, path TEXT NOT NULL, hash TEXT NOT NULL, producer TEXT NOT NULL, verifier TEXT NOT NULL, PRIMARY KEY(run_id,name));")?;
        Ok(Self { conn })
    }
    pub fn begin_run(&self, id: &str, project: &Path, brief: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO runs VALUES (?1, ?2, ?3, 'running', unixepoch())",
            params![id, project.to_string_lossy(), brief],
        )?;
        self.event(
            id,
            "run.started",
            &serde_json::json!({"project":project,"brief":brief}),
        )
    }
    pub fn event(&self, run_id: &str, kind: &str, payload: &Value) -> Result<()> {
        let prev: String = self
            .conn
            .query_row(
                "SELECT hash FROM events WHERE run_id=?1 ORDER BY seq DESC LIMIT 1",
                params![run_id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        let payload_text = serde_json::to_string(payload)?;
        let hash = digest(format!("{run_id}|{kind}|{payload_text}|{prev}").as_bytes());
        self.conn.execute(
            "INSERT INTO events (run_id,kind,payload,prev_hash,hash) VALUES (?1,?2,?3,?4,?5)",
            params![run_id, kind, payload_text, prev, hash],
        )?;
        Ok(())
    }
    pub fn artifact(
        &self,
        run_id: &str,
        name: &str,
        path: &Path,
        producer: &str,
        verifier: &str,
    ) -> Result<()> {
        let bytes =
            std::fs::read(path).with_context(|| format!("read artifact {}", path.display()))?;
        let hash = digest(&bytes);
        self.conn.execute(
            "INSERT OR REPLACE INTO artifacts VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                run_id,
                name,
                path.to_string_lossy(),
                hash,
                producer,
                verifier
            ],
        )?;
        self.event(
            run_id,
            "artifact.recorded",
            &serde_json::json!({"name":name,"hash":hash,"producer":producer,"verifier":verifier}),
        )
    }
    pub fn close_run(&self, run_id: &str, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE runs SET status=?2 WHERE id=?1",
            params![run_id, status],
        )?;
        self.event(run_id, "run.closed", &serde_json::json!({"status":status}))
    }
    pub fn verify_chain(&self, run_id: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT kind,payload,prev_hash,hash FROM events WHERE run_id=?1 ORDER BY seq",
        )?;
        let mut rows = stmt.query(params![run_id])?;
        let mut prior = String::new();
        while let Some(row) = rows.next()? {
            let kind: String = row.get(0)?;
            let payload: String = row.get(1)?;
            let prev: String = row.get(2)?;
            let hash: String = row.get(3)?;
            if prev != prior
                || digest(format!("{run_id}|{kind}|{payload}|{prev}").as_bytes()) != hash
            {
                return Ok(false);
            }
            prior = hash;
        }
        Ok(true)
    }
}

pub fn digest(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wal_chain_detects_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        store.begin_run("r1", dir.path(), "test").unwrap();
        store
            .event("r1", "stage", &serde_json::json!({"ok":true}))
            .unwrap();
        assert!(store.verify_chain("r1").unwrap());
        store
            .conn
            .execute(
                "UPDATE events SET payload='tampered' WHERE kind='stage'",
                [],
            )
            .unwrap();
        assert!(!store.verify_chain("r1").unwrap());
    }
}
