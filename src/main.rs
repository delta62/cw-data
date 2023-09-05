mod client;
mod db;
mod error;
mod puzzle;
mod puzzle_loader;
mod schema;

use db::Database;
use error::{Error, Result};
use puzzle::RemotePuzzle;
use puzzle_loader::PuzzleLoader;

macro_rules! var {
    ($name:expr) => {
        std::env::var($name)
            .map_err(|_| crate::error::Error::Environment($name))
            .unwrap()
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init();

    if let Err(err) = try_run().await {
        log::error!("{err}");
    }
}

async fn try_run() -> Result<()> {
    let db_path = var!("DB_PATH");
    let db = Database::new(db_path);

    db.init_schema().map_err(Error::Database)?;

    let loader = PuzzleLoader::new();
    let puzzles = loader.history_until(1_000).await?;

    for RemotePuzzle {
        puzzle,
        date,
        source,
    } in puzzles
    {
        log::info!("Adding {source:?} - {date:?}");
        db.upsert_puzzle(&puzzle, date, source)
            .map_err(Error::Database)?;
    }

    Ok(())
}

fn init() {
    env_logger::init();
    log::info!("Booting up");

    if dotenv::dotenv().is_err() {
        log::warn!("No .env file found; continuing with env variables");
    }
}
