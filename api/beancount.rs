use bot_message::telegram::Update;
use http::StatusCode;
use log::info;
use parser::parse;
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

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("API for beancount")
        .expect("Internal Service Error");
    Ok(response)
}
