mod db;
mod puzzle;
mod schema;

use db::Database;
use puzzle::{puzzle_date, puzzle_source};
use reqwest::ClientBuilder;
use schema::PuzzleList;
use std::env;

const DFAC_URL: &str = "https://api.foracross.com/api/puzzle_list";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    if dotenv::dotenv().is_err() {
        log::warn!("No .env file found; continuing with env variables");
    }

    let db_path = env::var("DB_PATH").expect("DB_PATH not set");
    let db = Database::new(db_path);
    db.init_schema().unwrap();

    let client = ClientBuilder::new().build().unwrap();
    let res = client
        .get(DFAC_URL)
        .query(&[
            ("page", "0"),
            ("pageSize", "50"),
            ("filter[nameOrTitleFilter]", ""),
            ("filter[sizeFilter][Mini]", "false"),
            ("filter[sizeFilter][Standard]", "true"),
        ])
        .send()
        .await
        .unwrap();

    res.json::<PuzzleList>()
        .await
        .unwrap()
        .iter()
        .filter_map(|puzzle| {
            puzzle_source(puzzle)
                .and_then(|source| puzzle_date(puzzle, source).map(|date| (puzzle, date, source)))
        })
        .for_each(|(puzzle, date, source)| {
            db.upsert_puzzle(puzzle, date, source).unwrap();
        });
}
