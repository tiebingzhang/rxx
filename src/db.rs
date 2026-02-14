use anyhow::Result;
use rand::Rng;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS registrations (
                id TEXT PRIMARY KEY,
                ipv6 TEXT NOT NULL,
                nonce TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(Database { conn })
    }

    pub fn register(&self, id: &str, ipv6: &str) -> Result<(bool, String)> {
        let id_lower = id.to_lowercase();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let nonce: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        match self.conn.execute(
            "INSERT INTO registrations (id, ipv6, nonce, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![id_lower, ipv6, nonce, now],
        ) {
            Ok(_) => Ok((true, nonce)),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Ok((false, String::new()))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn update(&self, id: &str, ipv6: &str, nonce: &str) -> Result<bool> {
        let id_lower = id.to_lowercase();

        // Check current IP and nonce
        let mut stmt = self
            .conn
            .prepare("SELECT ipv6, nonce FROM registrations WHERE id = ?1")?;
        let result = stmt.query_row(params![id_lower], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        });

        match result {
            Ok((current_ipv6, stored_nonce)) => {
                if stored_nonce != nonce {
                    return Ok(false);
                }
                if current_ipv6 == ipv6 {
                    println!("IP unchanged for {}, skipping DB write", id_lower);
                    return Ok(true);
                }

                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
                self.conn.execute(
                    "UPDATE registrations SET ipv6 = ?1, updated_at = ?2 WHERE id = ?3",
                    params![ipv6, now, id_lower],
                )?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    pub fn get_ipv6(&self, id: &str) -> Result<Option<String>> {
        let id_lower = id.to_lowercase();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let one_year_ago = now - (365 * 24 * 60 * 60);

        let mut stmt = self
            .conn
            .prepare("SELECT ipv6 FROM registrations WHERE id = ?1 AND updated_at > ?2")?;

        let result = stmt.query_row(params![id_lower, one_year_ago], |row| row.get(0));

        match result {
            Ok(ipv6) => Ok(Some(ipv6)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
