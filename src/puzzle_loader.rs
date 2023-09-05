use crate::{
    client::Client,
    error::{Error, Result},
    puzzle::{puzzle_date, puzzle_source, RemotePuzzle},
};

pub struct PuzzleLoader {
    client: Client,
}

impl PuzzleLoader {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    pub async fn history_until(&self, history_count: usize) -> Result<Vec<RemotePuzzle>> {
        let mut puzzles = Vec::new();
        let mut page_number = 0;

        while puzzles.len() < history_count {
            log::debug!("Requesting page {page_number}");
            let page = self
                .client
                .get_page(page_number)
                .await
                .map_err(Error::Network)?;

            page.into_iter()
                .filter_map(|puzzle| {
                    puzzle_source(&puzzle).and_then(|source| {
                        if let Some(date) = puzzle_date(&puzzle, source) {
                            Some(RemotePuzzle {
                                date,
                                source,
                                puzzle,
                            })
                        } else {
                            log::warn!("puzzle date cannot be parsed: {}", puzzle.title());
                            None
                        }
                    })
                })
                .for_each(|puz| puzzles.push(puz));

            page_number += 1;
        }

        Ok(puzzles)
    }
}
