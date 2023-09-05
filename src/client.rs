use crate::schema::PuzzleList;
use reqwest::Result;

const DFAC_URL: &str = "https://api.foracross.com/api/puzzle_list";

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    pub async fn get_page(&self, page_num: usize) -> Result<PuzzleList> {
        let res = self
            .client
            .get(DFAC_URL)
            .query(&[
                ("page", page_num.to_string().as_str()),
                ("pageSize", "50"),
                ("filter[nameOrTitleFilter]", ""),
                ("filter[sizeFilter][Mini]", "false"),
                ("filter[sizeFilter][Standard]", "true"),
            ])
            .send()
            .await?;

        res.json().await
    }
}
