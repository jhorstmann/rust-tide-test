use async_std::*;
use tide::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_json::json;

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

type Result<T> = std::result::Result<T, ErrorBody>;

fn handle_error(e: impl std::error::Error) -> Response {
    let body = ErrorBody::with_status(400, e.to_string());

    Response::new(400)
        .body_json(&body)
        .unwrap_or_else(|e| Response::new(500))
}

impl std::convert::From<std::io::Error> for ErrorBody {
    fn from(e: std::io::Error) -> Self {
        ErrorBody::new(e.to_string())
    }
}

impl IntoResponse for ErrorBody {
    fn into_response(self) -> Response {
        Response::new(self.status)
            .body_json(&self)
            .unwrap_or_else(|e| Response::new(500))
    }
}

macro_rules! try_or_400 {
    ( $expression:expr ) => {
        match $expression {
            Ok(x) => x,
            Err(e) => return handle_error(e)
        }
    }
}



async fn sample_data(mut req: Request<()>) -> Response {

    /*
    match req.body_json::<QueryRequest>().await {
        Ok(query) => dbg!(query.query),
        Err(e) => return handle_error(e)
    };
    */
    let query : QueryRequest = try_or_400!(req.body_json().await);
    dbg!(query.query);


    try_or_400!(Response::new(200).body_json(&QueryResponse {
        headers: vec!["foo".to_owned(), "bar".to_owned()],
        data: vec![vec![json!(1.0), json!("choice")]]
    }))
}

#[async_std::main]
async fn main()  {
    let mut app = tide::new();
    app.at("/").post(sample_data);

    app.listen("127.0.0.1:8080").await.unwrap();
}
