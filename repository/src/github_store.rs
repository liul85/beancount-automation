use crate::Store;
use anyhow::{anyhow, Result};
use base64::{decode, encode};
use log::{error, info};
use parser::Transaction;
use reqwest::{blocking::Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

pub struct GithubStore {
    owner: String,
    repo: String,
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
    pub fn new() -> Result<Self> {
        let owner = env::var("GITHUB_OWNER")?;
        let repo = env::var("GITHUB_REPO")?;
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
            client,
        })
    }
}

impl Store for GithubStore {
    fn save(&self, transaction: Transaction) -> Result<String> {
        let path = format!("{}.bean", transaction.year());
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        );

        let mut content_response = self.client.get(&url).send()?;
        match content_response.status() {
            StatusCode::OK => (),
            StatusCode::NOT_FOUND => {
                self.create_file(path)?;
                content_response = self.client.get(&url).send()?;
            }
            _ => {
                error!("Failed to get file!");
                error!("Response status was {}", content_response.status());
                error!("Response body was {}", content_response.text()?);
                return Err(anyhow!("Failed to get file content"));
            }
        };

        let file_content: FileContent = content_response.json()?;
        let decoded_value = decode(&file_content.content.replace('\n', ""))?;
        let content = String::from_utf8_lossy(&decoded_value);
        let transaction_year = transaction.year();
        let transaction_text = String::from(transaction);

        let update_request = UpdateRequest {
            message: "updated content".to_string(),
            content: encode(format!("{}\n{}", content, transaction_text)),
            sha: file_content.sha,
        };

        let body = serde_json::to_string(&update_request)?;
        let rb = self.client.put(url).body(body);
        let response = rb.send()?;
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                info!(
                    "Successfully created/updated file {} in repo {}.",
                    transaction_year, self.repo
                );
                Ok(transaction_text)
            }
            _ => {
                error!("Failed to save transaction!");
                error!(
                    "github api response status code was [{}]",
                    response.status()
                );
                error!("github api response body was {}", response.text()?);
                Err(anyhow!("Failed to save transaction!"))
            }
        }
    }
}

impl GithubStore {
    fn create_file(&self, path: String) -> Result<()> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.owner, self.repo, path
        );
        let mut body = HashMap::new();
        body.insert("message", format!("created file {}", path));
        body.insert("content", "".into());
        let response = self.client.post(&url).json(&body).send()?;
        match response.status() {
            StatusCode::CREATED | StatusCode::OK => Ok(()),
            _ => {
                error!("Failed to create new file {}", path);
                error!(
                    "github api response status code was [{}]",
                    response.status()
                );
                error!("github api response body was {}", response.text()?);
                Err(anyhow!("Failed to create new file {}", path))
            }
        }
    }
}
