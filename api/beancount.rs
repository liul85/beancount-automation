use anyhow::Result;
use bot_message::telegram::{ResponseBody, Update};
use http::StatusCode;
use log::{error, info};
use parser::parse;
use repository::github_store::GithubStore;
use repository::Store;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn main() -> Result<()> {
    env_logger::init();
    Ok(lambda!(handler))
}

fn handler(request: Request) -> Result<impl IntoResponse, VercelError> {
    let body = String::from_utf8_lossy(request.body());
    info!("request body is {}", body);

    let update: Update = serde_json::from_str(&body).unwrap();

    let transaction = parse(&update.message.text).unwrap();
    info!("parsed transaction is {:?}", transaction);

    let store =
        GithubStore::new().or_else(|_| Err(VercelError::new("Failed to create github store!")))?;
    let result = store.save(transaction.to_beancount());
    match result {
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
