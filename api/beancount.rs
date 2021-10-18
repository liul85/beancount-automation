use bot_message::telegram::Update;
use http::StatusCode;
use log::{error, info};
use parser::parse;
use repository::github_store::GithubStore;
use repository::Store;
use std::error::Error;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    Ok(lambda!(handler))
}

fn handler(request: Request) -> Result<impl IntoResponse, VercelError> {
    let body = String::from_utf8_lossy(request.body());
    info!("request body is {}", body);

    let update: Update = serde_json::from_str(&body).unwrap();

    let transaction = parse(&update.message.text).unwrap();
    info!("parsed transaction is {:?}", transaction);

    let store: &dyn Store =
        &GithubStore::new().or_else(|_| Err(VercelError::new("Failed to create github store!")))?;
    let result = store.save(transaction.to_beancount());
    match result {
        Ok(_) => {
            info!("Successfully saved transaction!");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(transaction.to_beancount())?)
        }
        Err(e) => {
            error!("Failed to save transaction: {}", e.to_string());
            Err(VercelError::new(&e.to_string()))
        }
    }
}
