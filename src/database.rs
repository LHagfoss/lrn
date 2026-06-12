use rusqlite::{Connection, Result};
use std::fs;
use std::path::Path;

pub struct DbEngine {
    conn: Connection,
}

impl DbEngine {
    pub fn init() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                filepath TEXT UNIQUE,
                title TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS links (
                source_file_id INTEGER,
                target_file_id INTEGER,
                FOREIGN KEY(source_file_id) REFERENCES files(id),
                FOREIGN KEY(target_file_id) REFERENCES files(id)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn index_vault(&mut self, vault_path: &Path) -> Result<()> {
        self.conn.execute("DELETE FROM files", [])?;

        if let Ok(entries) = fs::read_dir(vault_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        let filepath_str = path.to_string_lossy().to_string();

                        let _ = self.conn.execute(
                            "INSERT OR IGNORE INTO files (filepath, title) VALUES (?, ?)",
                            [&filepath_str, filename],
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_all_files(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT filepath, title FROM files ORDER BY title ASC")?;
        let file_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut files = Vec::new();
        for file in file_iter {
            if let Ok(f) = file {
                files.push(f);
            }
        }

        Ok(files)
    }
}
