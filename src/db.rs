use chrono::NaiveDate;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, DropBehavior, Result};
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
        CREATE TABLE IF NOT EXISTS sources (
            id      INTEGER     NOT NULL    PRIMARY KEY,
            name    TEXT        NOT NULL    UNIQUE
        );

        CREATE TABLE IF NOT EXISTS puzzles (
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

        CREATE TABLE IF NOT EXISTS answers (
            id      INTEGER     NOT NULL    PRIMARY KEY,
            text    TEXT        NOT NULL    UNIQUE
        );

        CREATE TABLE IF NOT EXISTS puzzle_answers (
            puzzle_id   INTEGER     NOT NULL,
            answer_id   INTEGER     NOT NULL,
            PRIMARY KEY (puzzle_id, answer_id),
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

    fn source_name(source: PuzzleSource) -> &'static str {
        match source {
            PuzzleSource::LosAngelesTimes => "Los Angeles Times",
            PuzzleSource::NewYorkTimes => "New York Times",
        }
    }

    pub fn upsert_puzzle(
        &self,
        puzzle: &Puzzle,
        date: NaiveDate,
        source: PuzzleSource,
    ) -> Result<()> {
        let mut conn = self.connection();
        let mut tx = conn.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);

        let db_source_name = Self::source_name(source);
        let timestamp = date.and_hms_opt(0, 0, 0).unwrap().timestamp();

        let source_id: i64 = tx.query_row(
            "SELECT id FROM sources WHERE name = :name",
            named_params! { ":name": db_source_name},
            |row| row.get("id"),
        )?;

        tx.execute(
            "INSERT OR IGNORE INTO puzzles (dfac_id, title, date, source) VALUES (:dfac_id, :title, :date, :source)",
            named_params! {
                ":dfac_id": puzzle.id(),
                ":title": puzzle.title(),
                ":date": timestamp,
                ":source": source_id,
            },
        )?;

        let puzzle_id = tx.last_insert_rowid();

        for word in puzzle.answers().iter() {
            tx.execute(
                "INSERT OR IGNORE INTO answers (text) VALUES (:text)",
                named_params! { ":text": word },
            )?;

            let answer_id = tx.last_insert_rowid();

            tx.execute("INSERT OR IGNORE INTO puzzle_answers (puzzle_id, answer_id) VALUES (:puzzle_id, :answer_id)", named_params! { ":puzzle_id": puzzle_id, ":answer_id": answer_id})?;
        }

        Ok(())
    }
}
