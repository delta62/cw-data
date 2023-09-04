use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleInfo {
    #[serde(rename = "type")]
    pub puzzle_type: String,
    pub title: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleClues {
    pub down: Vec<Option<String>>,
    pub across: Vec<Option<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleContent {
    pub private: bool,
    pub info: PuzzleInfo,
    pub clues: PuzzleClues,
    // shades: Vec<()>,
    pub circles: Vec<usize>,
    pub grid: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleStats {
    pub num_solves: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Puzzle {
    pub pid: String,
    pub content: PuzzleContent,
    pub stats: PuzzleStats,
}

impl Puzzle {
    pub fn title(&self) -> &str {
        &self.content.info.title
    }

    pub fn id(&self) -> &str {
        &self.pid
    }

    pub fn answers(&self) -> Vec<String> {
        let mut answers = Vec::new();

        let rows = self.content.grid.iter();
        for row in rows {
            let s = row.iter().fold(String::new(), |mut acc, x| {
                acc.push_str(x);
                acc
            });

            answers.append(&mut words(&s));
        }

        let num_rows = self.content.grid.len();
        let num_cols = self
            .content
            .grid
            .first()
            .map(|row| row.len())
            .unwrap_or_default();

        for col_num in 0..num_cols {
            let mut s = String::new();
            for row_num in 0..num_rows {
                let c = &self.content.grid[row_num][col_num];
                s.push_str(c);
            }

            answers.append(&mut words(&s));
        }

        answers
    }
}

fn words(s: &str) -> Vec<String> {
    s.split('.')
        .filter_map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_owned())
            }
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuzzleList {
    puzzles: Vec<Puzzle>,
}

impl PuzzleList {
    pub fn iter(&self) -> impl Iterator<Item = &Puzzle> {
        self.puzzles.iter()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn make_puzzle(title: &str) -> Puzzle {
        Puzzle {
            pid: "123abc".to_owned(),
            content: PuzzleContent {
                info: PuzzleInfo {
                    title: title.to_owned(),
                    puzzle_type: "Daily Puzzle".to_owned(),
                    author: "Auth Thor".to_owned(),
                    description: "Description of the puzzle".to_owned(),
                },
                grid: vec![],
                private: false,
                clues: PuzzleClues {
                    across: vec![],
                    down: vec![],
                },
                circles: vec![],
            },
            stats: PuzzleStats { num_solves: 42 },
        }
    }
}
