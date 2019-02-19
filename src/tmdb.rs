// Copyright 2019 James Chapman

use chrono::NaiveDate;
use reqwest::{Client, Url};
use serde::{Deserialize, Deserializer};

const BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Deserialize, GraphQLObject)]
pub struct SearchMovie {
    // pub id: u64,
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub overview: Option<String>,
    #[serde(deserialize_with = "deserialize_date_or_empty_string")]
    pub release_date: Option<NaiveDate>,
    // pub genre_ids: Vec<u16>,
    pub genre_ids: Vec<i32>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub popularity: f64,
    pub adult: bool,
}

#[derive(Debug, Deserialize, GraphQLObject)]
pub struct SearchMovieResults {
    // pub page: u8,
    pub page: i32,
    // pub total_pages: u8,
    pub total_pages: i32,
    // pub total_results: u8,
    pub total_results: i32,
    pub results: Vec<SearchMovie>,
}

pub struct TmdbClient {
    api_key: String,
    client: Client,
}

impl TmdbClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        TmdbClient { api_key, client }
    }

    pub fn search_movies(&self, title: &str) -> Result<SearchMovieResults, reqwest::Error> {
        let mut url = Url::parse(&format!("{}/search/movie", BASE_URL)).unwrap();
        url.query_pairs_mut()
            .append_pair("api_key", &self.api_key)
            .append_pair("query", &title);

        let mut response = self.client.get(url).send()?;
        response.json()
    }
}

fn deserialize_date_or_empty_string<'de, D>(deserialiser: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    let o = Option::<String>::deserialize(deserialiser)?;
    if let Some(s) = o {
        if !s.is_empty() {
            return s.parse()
                .map(Some)
                .map_err(|err| serde::de::Error::custom(format!("{}", err)));
        }
    }

    Ok(None)
}
