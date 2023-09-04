use crate::schema::Puzzle;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;

// e.g. Wed, Aug 30, 2023
const SHORT_DATE_FMT: &str = "%a, %b %e, %Y";

// e.g. Wednesday, August 30, 2023
const LONG_DATE_FMT: &str = "%A, %B %e, %Y";

lazy_static! {
    static ref LAT_REGEX: Regex = Regex::new("^LA Times, (.*)$").unwrap();
    static ref NYT_REGEX: Regex = Regex::new("^NY Times, (.*)$").unwrap();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PuzzleSource {
    NewYorkTimes,
    LosAngelesTimes,
}

pub fn puzzle_source(puzzle: &Puzzle) -> Option<PuzzleSource> {
    if NYT_REGEX.is_match(puzzle.title()) {
        Some(PuzzleSource::NewYorkTimes)
    } else if LAT_REGEX.is_match(puzzle.title()) {
        Some(PuzzleSource::LosAngelesTimes)
    } else {
        None
    }
}

pub fn puzzle_date(puzzle: &Puzzle, source: PuzzleSource) -> Option<NaiveDate> {
    match source {
        PuzzleSource::NewYorkTimes => NYT_REGEX
            .captures(puzzle.title())
            .and_then(|capt| capt.get(1))
            .and_then(|date_part| {
                NaiveDate::parse_from_str(date_part.as_str(), SHORT_DATE_FMT)
                    .ok()
                    .or_else(|| NaiveDate::parse_from_str(date_part.as_str(), LONG_DATE_FMT).ok())
            }),
        PuzzleSource::LosAngelesTimes => LAT_REGEX
            .captures(puzzle.title())
            .and_then(|capt| capt.get(1))
            .and_then(|date_part| {
                NaiveDate::parse_from_str(date_part.as_str(), SHORT_DATE_FMT)
                    .ok()
                    .or_else(|| NaiveDate::parse_from_str(date_part.as_str(), LONG_DATE_FMT).ok())
            }),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::schema::test::make_puzzle;

    #[test]
    fn parses_lat_date() {
        let title = "LA Times, Wed, Aug 30, 2023";
        let puzzle = make_puzzle(title);

        assert_eq!(
            puzzle_date(&puzzle, PuzzleSource::LosAngelesTimes),
            NaiveDate::from_ymd_opt(2023, 8, 30)
        );
    }

    #[test]
    fn parses_nyt_date() {
        let title = "NY Times, Wednesday, August 30, 2023";
        let puzzle = make_puzzle(title);

        assert_eq!(
            puzzle_date(&puzzle, PuzzleSource::NewYorkTimes),
            NaiveDate::from_ymd_opt(2023, 8, 30)
        )
    }
}
