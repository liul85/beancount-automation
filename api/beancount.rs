use http::StatusCode;
use log::info;
use std::error::Error;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    Ok(lambda!(handler))
}

fn handler(request: Request) -> Result<impl IntoResponse, VercelError> {
    let body = String::from_utf8_lossy(request.body());
    info!("request body is {}", body);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("API for beancount")
        .expect("Internal Service Error");
    Ok(response)
}
