use anyhow::Result;
use beancount::parser::BeancountParser;
use beancount::settings::Settings;
use bot_message::telegram::{ResponseBody, Update};
use http::StatusCode;
use log::{error, info, warn};
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

    let update: Update = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => {
            warn!("Failed to deserialize request body: {}", body);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body("Failed to deserialize request body".into())?);
        }
    };

    let message = match update.message {
        Some(v) => v,
        None => match update.edited_message {
            Some(v) => v,
            None => {
                warn!("Could not get message or edited_message from request");
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body("Could not get message or edited_message from request".into())?);
            }
        },
    };

    let settings =
        Settings::load_from_env().map_err(|e| VercelError::new(e.to_string().as_str()))?;
    let parser = BeancountParser::new(settings);

    let ok_response = |text| {
        let response_body = ResponseBody {
            method: "sendMessage".into(),
            chat_id: message.chat.id,
            text,
            reply_to_message_id: message.message_id,
        };

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&response_body).unwrap())?)
    };

    let transaction = match parser.parse(&message.text) {
        Ok(transaction) => transaction,
        Err(e) => {
            error!("Failed to parse input: {}", e.to_string());
            return ok_response(format!(
                "⚠️\n==============================\nFailed to parse input: {}",
                e.to_string()
            ));
        }
    };

    info!("parsed transaction is {:?}", transaction);

    let store = GithubStore::new()
        .map_err(|e| VercelError::new(format!("Failed to create github store: {}", e).as_str()))?;

    match store.save(transaction) {
        Ok(text) => {
            info!("Successfully saved transaction!");
            ok_response(text)
        }
        Err(e) => {
            error!("Failed to save transaction: {}", e.to_string());
            Err(VercelError::new(&e.to_string()))
        }
    }
}
