use crate::BeancountStore;
use chrono::prelude::Local;
use reqwest::{header, Client};
use std::env;

struct GithubStore {
    owner: String,
    repo: String,
    path: String,
    client: Client,
}

impl GithubStore {
    pub fn new() -> Self {
        let owner = env::var("GITHUB_OWNER").unwrap();
        let repo = env::var("GITHUB_REPO").unwrap();
        let path = Local::now().format("%Y").to_string();
        let github_token = env::var("GITHUB_TOKEN").unwrap();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        let mut token = header::HeaderValue::from_str(&github_token).unwrap();
        token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, token);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        GithubStore {
            owner,
            repo,
            path,
            client,
        }
    }
}

impl BeancountStore for GithubStore {
    fn save(content: String) -> Result<(), String> {
        Err("Not implemented!".to_string())
    }
}
