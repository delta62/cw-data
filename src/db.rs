use chrono::NaiveDate;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, Result};
use std::path::Path;

use crate::{puzzle::PuzzleSource, schema::Puzzle};

pub type Connection = PooledConnection<SqliteConnectionManager>;

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let manager = if let Some(":memory:") = path.as_ref().to_str() {
            log::debug!("Opening in-memory sqlite database");
            SqliteConnectionManager::memory()
        } else {
            log::debug!("Opening sqlite database at {:?}", path.as_ref());
            SqliteConnectionManager::file(path)
        };

        let pool = Pool::new(manager).unwrap();

        Self { pool }
    }

    pub fn init_schema(&self) -> Result<()> {
        let conn = self.connection();
        conn.execute_batch(
            r#"
        CREATE TABLE sources (
            id      INTEGER     NOT NULL    PRIMARY KEY,
            name    TEXT        NOT NULL    UNIQUE
        );

        CREATE TABLE puzzles (
            id      INTEGER     NOT NULL    PRIMARY KEY,
            dfac_id TEXT        NOT NULL    UNIQUE,
            title   TEXT        NOT NULL,
            date    INTEGER     NOT NULL,
            source  INTEGER     NOT NULL,
            FOREIGN KEY (source) REFERENCES sources (id)
                ON UPDATE NO ACTION
                ON DELETE NO ACTION,
            UNIQUE (date, source)
        );

        CREATE TABLE answers (
            id      INTEGER     NOT NULL    PRIMARY KEY,
            text    TEXT        NOT NULL    UNIQUE
        );

        CREATE TABLE puzzle_answers (
            puzzle_id   INTEGER     NOT NULL,
            answer_id   INTEGER     NOT NULL,
            FOREIGN KEY (puzzle_id) REFERENCES puzzles (id)
                ON UPDATE NO ACTION
                ON DELETE NO ACTION,
            FOREIGN KEY (answer_id) REFERENCES answers (id)
                ON UPDATE NO ACTION
                ON DELETE NO ACTION
        );

        INSERT INTO sources (name)
        VALUES ("New York Times"), ("Los Angeles Times");
    "#,
        )
    }

    pub fn connection(&self) -> Connection {
        self.pool.get().unwrap()
    }

    pub fn upsert_puzzle(
        &self,
        puzzle: &Puzzle,
        date: NaiveDate,
        source: PuzzleSource,
    ) -> Result<()> {
        let conn = self.connection();
        let db_source_name = if source == PuzzleSource::NewYorkTimes {
            "New York Times"
        } else {
            "Los Angeles Times"
        };

        let timestamp = date.and_hms_opt(0, 0, 0).unwrap().timestamp();
        let source_id: i64 = conn.query_row(
            "SELECT id FROM sources WHERE name = :name",
            named_params! { ":name": db_source_name},
            |row| row.get("id"),
        )?;

        let puzzle_id = conn.execute(
            "INSERT OR IGNORE INTO puzzles (dfac_id, title, date, source) VALUES (:dfac_id, :title, :date, :source)",
            named_params! {
                ":dfac_id": puzzle.id(),
                ":title": puzzle.title(),
                ":date": timestamp,
                ":source": source_id,
            },
        )?;

        for word in puzzle.answers().iter() {
            let answer_id = conn.execute(
                "INSERT OR IGNORE INTO answers (text) VALUES (:text)",
                named_params! { ":text": word },
            )?;

            conn.execute("INSERT OR IGNORE INTO puzzle_answers (puzzle_id, answer_id) VALUES (:puzzle_id, :answer_id)", named_params! { ":puzzle_id": puzzle_id, ":answer_id": answer_id})?;
        }

        Ok(())
    }
}
