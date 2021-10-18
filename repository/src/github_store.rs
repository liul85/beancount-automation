use crate::Store;
use base64::{decode, encode};
use chrono::prelude::Local;
use log::{error, info};
use reqwest::{blocking::Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

pub struct GithubStore {
    owner: String,
    repo: String,
    path: String,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileContent {
    #[serde(rename = "type")]
    content_type: String,
    encoding: String,
    size: u64,
    name: String,
    path: String,
    content: String,
    sha: String,
    url: String,
    git_url: String,
    html_url: String,
    download_url: String,
    _links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
struct Links {
    git: String,
    #[serde(rename = "self")]
    celf: String,
    html: String,
}

#[derive(Serialize, Debug)]
struct UpdateRequest {
    message: String,
    content: String,
    sha: String,
}

impl GithubStore {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let owner = env::var("GITHUB_OWNER")?;
        let repo = env::var("GITHUB_REPO")?;
        let path = format!("{}.bean", Local::now().format("%Y").to_string());
        let github_token = env::var("GITHUB_TOKEN")?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        let mut token = header::HeaderValue::from_str(&format!("token {}", github_token))?;
        token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, token);

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .user_agent("beancount-automation/0.1.0")
            .build()?;
        Ok(GithubStore {
            owner,
            repo,
            path,
            client,
        })
    }
}

impl Store for GithubStore {
    fn save(&self, update: String) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, self.path
        );

        let content_response = self.client.get(&url).send()?;
        match content_response.status() {
            StatusCode::OK => (),
            _ => {
                error!("Failed to get file!");
                error!("Response status was {}", content_response.status());
                error!("Response body was {}", content_response.text()?);
                return Err("Failed to get file content".into());
            }
        };

        let file_content: FileContent = content_response.json()?;
        let decoded_value = decode(&file_content.content.replace("\n", ""))?;
        let content = String::from_utf8_lossy(&decoded_value);

        let update_request = UpdateRequest {
            message: "updated content".to_string(),
            content: encode(format!("{}\n{}", content, update)),
            sha: file_content.sha,
        };

        let body = serde_json::to_string(&update_request)?;
        let rb = self.client.put(url).body(body);
        let response = rb.send()?;
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                info!(
                    "Successfully created/updated file {} in repo {}.",
                    self.path, self.repo
                );
                Ok(())
            }
            _ => {
                error!("Failed to save transaction!");
                error!(
                    "github api response status code was [{}]",
                    response.status()
                );
                error!("github api response body was {}", response.text()?);
                Err("Failed to save transaction!".into())
            }
        }
    }
}
