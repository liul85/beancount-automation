use anyhow::Result;
use bot_message::telegram::{ResponseBody, Update};
use http::StatusCode;
use log::{error, info};
use parser::Parser;
use repository::github_store::GithubStore;
use repository::Store;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

#[allow(dead_code)]
fn main() -> Result<()> {
    env_logger::init();
    Ok(lambda!(handler))
}

#[allow(dead_code)]
fn handler(request: Request) -> Result<impl IntoResponse, VercelError> {
    let body = String::from_utf8_lossy(request.body());
    info!("request body is {}", body);

    let update: Update =
        serde_json::from_str(&body).or_else(|e| Err(VercelError::new(e.to_string().as_str())))?;

    let parser = Parser::new().or_else(|e| {
        Err(VercelError::new(
            format!("Failed to create parser: {}", e).as_str(),
        ))
    })?;

    let transaction = match parser.parse(&update.message.text) {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to parse input: {}", e.to_string());
            let response_body = ResponseBody {
                method: "sendMessage".into(),
                chat_id: update.message.chat.id,
                text: format!("⚠️\n==============================\nFailed to parse input: {}", e.to_string()),
                reply_to_message_id: update.message.message_id,
            };
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&response_body).unwrap())?);
        }
    };

    info!("parsed transaction is {:?}", transaction);

    let store = GithubStore::new().or_else(|e| {
        Err(VercelError::new(
            format!("Failed to create github store: {}", e).as_str(),
        ))
    })?;

    match store.save(transaction.to_beancount()) {
        Ok(_) => {
            info!("Successfully saved transaction!");
            let response_body = ResponseBody {
                method: "sendMessage".into(),
                chat_id: update.message.chat.id,
                text: transaction.into(),
                reply_to_message_id: update.message.message_id,
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&response_body).unwrap())?)
        }
        Err(e) => {
            error!("Failed to save transaction: {}", e.to_string());
            Err(VercelError::new(&e.to_string()))
        }
    }
}
