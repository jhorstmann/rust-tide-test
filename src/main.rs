use serde::export::Formatter;
use serde::ser::SerializeSeq;
use serde::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::TryInto;
use std::fmt::Debug;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tide::utils::After;
use tide::*;

#[derive(Debug, Serialize, Deserialize)]
struct QueryResponse {
    headers: Vec<String>,
    data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryRequest {
    query: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    status: u16,
    message: String,
}

impl Into<Response> for QueryResponse {
    fn into(self) -> Response {
        let mut res = Response::new(StatusCode::Ok);
        match Body::from_json(&self) {
            Ok(body) => {
                res.set_body(body);
                res
            }
            Err(e) => Response::new(StatusCode::InternalServerError),
        }
    }
}

impl ErrorBody {
    fn new(message: String) -> Self {
        ErrorBody::with_status(500, message)
    }

    fn with_status(status: u16, message: String) -> Self {
        Self { status, message }
    }
}

async fn sample_data(mut req: Request<()>) -> Result {
    let query: QueryRequest = req.body_json().await?;
    dbg!(query.query);

    let body = Body::from_json(&QueryResponse {
        headers: vec!["foo".to_owned(), "bar".to_owned()],
        data: vec![vec![json!(1.0), json!("choice")]],
    })?;

    Ok(body.into())
}

async fn test_error(_req: Request<()>) -> Result {
    Err(Error::from_str(
        StatusCode::ImATeapot,
        "Error Test: I am a teapot",
    ))
}

async fn pretty_error_middleware(mut response: Response) -> Result<Response> {
    if let Some(e) = response.error() {
        let msg = e.to_string();
        let status = e.status() as u16;
        let body = ErrorBody {
            status,
            message: msg,
        };
        let mut res = Response::new(status);
        match Body::from_json(&body) {
            Ok(body) => res.set_body(body),
            Err(e) => {
                res.set_status(StatusCode::InternalServerError);
                res.set_body("Error while serializing error message");
            }
        }
        Ok(res)
    } else {
        Ok(response)
    }
}

#[async_std::main]
async fn main() {
    let mut app = tide::new();
    app.with(After(pretty_error_middleware));

    app.at("/query").post(sample_data);
    app.at("/error").get(test_error);

    app.listen("127.0.0.1:8080").await.unwrap();
}
