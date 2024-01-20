use askama::Template;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use log::info;
use search::query::QueryProcessor;
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{Arc, Mutex},
    time::Instant,
};

struct AppState {
    query_processor: Mutex<QueryProcessor>,
    index_path: String,
}

#[tokio::main]
async fn main() {
    // logger
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --bin client <base_path>");
        return;
    }

    let base_path = &args[1];
    let index_path = format!("{}/index/index", base_path);
    let tokenizer_path = format!("{}/tokenizer/bert-base-uncased", base_path);

    let state = Arc::new(AppState {
        query_processor: Mutex::new(QueryProcessor::build_query_processor(
            &index_path,
            &tokenizer_path,
        )),
        index_path: base_path.clone(),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/query", post(post_query))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Application started on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

// utility struct to render templates
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),

            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

// homepage
#[derive(Template)]
#[template(path = "index.html")]
struct Root {
    index_path: String,
}

async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Root request");
    HtmlTemplate(Root {
        index_path: state.index_path.clone(),
    })
}

// query handler

#[derive(Deserialize, Debug)]
struct QueryRequest {
    query: String,
    // limit: usize,
}

#[derive(Template)]
#[template(path = "query.html")]
struct QueryResponse {
    time_ms: u128,
    documents: Vec<Document>,
}

#[derive(Serialize, Deserialize)]
struct Document {
    id: u32,
    score: f32,
    path: String,
}

async fn post_query(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    info!("Query request: {:?}", payload);

    let mut q = state.query_processor.lock().unwrap();

    let start_time = Instant::now();
    let query_result = q.query(&payload.query, 10);
    let time_ms = start_time.elapsed().as_millis();

    let documents = query_result
        .iter()
        .map(|r| Document {
            id: r.id,
            score: r.score,
            path: r.path.clone(),
        })
        .collect();

    HtmlTemplate(QueryResponse { time_ms, documents })
}
