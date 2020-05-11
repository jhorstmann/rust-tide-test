use tide::*;
use serde::*;
use serde::{Serialize, Deserialize};
use serde::ser::SerializeSeq;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use serde_json::json;
use std::convert::TryInto;
use std::fmt::Debug;
use serde::export::Formatter;
use std::future::Future;

#[derive(Debug, Serialize, Deserialize)]
struct QueryResponse {
    headers: Vec<String>,
    data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryRequest {
    query: String
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    status: u16,
    message: String
}

impl Into<Response> for QueryResponse {
    fn into(self) -> Response {
        Response::new(StatusCode::Ok)
            .body_json(&self)
            .unwrap_or(Response::new(StatusCode::InternalServerError))
    }
}


impl ErrorBody {
    fn new(message: String) -> Self {
        ErrorBody::with_status(500, message)
    }

    fn with_status(status: u16, message: String) -> Self {
        Self {
            status,
            message
        }

    }
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct PrettyErrorMiddleware();

impl PrettyErrorMiddleware {
    pub fn new() -> Self {
        Self()
    }

    async fn handle_error<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> Result {
        match next.run(ctx).await {
            Ok(res) => Ok(res),
            Err(err) => {
                let msg = err.to_string();
                let status = err.status() as u16;
                let body = ErrorBody {
                    status,
                    message: msg
                };
                Ok(Response::new(StatusCode::InternalServerError).body_json(&body).or_else(|serde_err| {
                    println!("Could not serialize error message {}", err);
                    Result::Err(err)
                })?)
            }
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for PrettyErrorMiddleware {
    fn handle<'a>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Result> {
        Box::pin(async move { self.handle_error(ctx, next).await })
    }
}



async fn sample_data(mut req: Request<()>) -> Result {

    let query : QueryRequest = req.body_json().await?;
    dbg!(query.query);

    let response = QueryResponse {
        headers: vec!["foo".to_owned(), "bar".to_owned()],
        data: vec![vec![json!(1.0), json!("choice")]]
    };

    Ok(Response::new(StatusCode::Ok)
           .body_json(&response)?)
}

#[async_std::main]
async fn main()  {
    let mut app = tide::new();
    app.middleware(PrettyErrorMiddleware::new());

    app.at("/").post(sample_data);

    app.listen("127.0.0.1:8080").await.unwrap();
}
