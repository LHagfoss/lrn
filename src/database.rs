use regex::Regex;
use rusqlite::{Connection, Result, params};
use std::fs;
use std::path::Path;

pub struct DbEngine {
    conn: Connection,
}

impl DbEngine {
    pub fn init() -> Result<Self> {
        let conn = Connection::open("lrn_vault.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
                source_id INTEGER,
                target_title TEXT,
                PRIMARY KEY (source_id, target_title),
                FOREIGN KEY(source_id) REFERENCES files(id) ON DELETE CASCADE
            );",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn index_vault(&mut self, vault_path: &Path) -> Result<()> {
        // Clear all tables so deleted files are removed from the DB.
        self.conn.execute("DELETE FROM links", [])?;
        self.conn.execute("DELETE FROM files", [])?;

        let re = Regex::new(r"\[\[(.*?)\]\]").unwrap();
        let entries = fs::read_dir(vault_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let path_str = path.to_string_lossy().to_string();
                let title = path.file_name().unwrap().to_string_lossy().to_string();

                self.conn.execute(
                    "INSERT OR IGNORE INTO files (path, title) VALUES (?1, ?2)",
                    params![path_str, title],
                )?;

                let source_id: i64 = self.conn.query_row(
                    "SELECT id FROM files WHERE path = ?1",
                    params![path_str],
                    |row| row.get(0),
                )?;

                self.conn
                    .execute("DELETE FROM links WHERE source_id = ?1", params![source_id])?;

                if let Ok(content) = fs::read_to_string(&path) {
                    for cap in re.captures_iter(&content) {
                        let mut target_title = cap[1].trim().to_string();
                        if !target_title.ends_with(".md") {
                            target_title.push_str(".md");
                        }

                        self.conn.execute(
                            "INSERT OR IGNORE INTO links (source_id, target_title) VALUES (?1, ?2)",
                            params![source_id, target_title],
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_all_files(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, title FROM files ORDER BY title ASC")?;
        let file_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut results = Vec::new();
        for file in file_iter {
            results.push(file?);
        }
        Ok(results)
    }

    pub fn get_backlinks(&self, file_title: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.title FROM files f
             JOIN links l ON f.id = l.source_id
             WHERE l.target_title = ?1
             ORDER BY f.title ASC",
        )?;

        let rows = stmt.query_map(params![file_title], |row| row.get::<_, String>(0))?;
        let mut backlinks = Vec::new();
        for row in rows {
            backlinks.push(row?);
        }
        Ok(backlinks)
    }
}
