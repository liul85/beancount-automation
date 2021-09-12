use http::StatusCode;
use std::error::Error;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request, Response};

fn main() -> Result<(), Box<dyn Error>> {
    Ok(lambda!(handler))
}

fn handler(_: Request) -> Result<impl IntoResponse, VercelError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body("API for beancount")
        .expect("Internal Service Error");
    Ok(response)
}
