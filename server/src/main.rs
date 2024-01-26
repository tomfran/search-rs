use askama::Template;
use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use log::info;
use search::query::Processor;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::read_to_string,
    mem::replace,
    sync::{Arc, Mutex},
};

struct AppState {
    index_path: String,
    query_processor: Mutex<Processor>,
    cached_query: Mutex<String>,
    cached_result: Mutex<Option<QueryResponse>>,
}

#[tokio::main]
async fn main() {
    // logger
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --bin server <base_path>");
        return;
    }

    let base_path = &args[1];
    let index_path = format!("{base_path}/index/idx");

    let state = Arc::new(AppState {
        index_path: base_path.clone(),
        query_processor: Mutex::new(Processor::build_query_processor(&index_path)),
        cached_query: Mutex::new(String::new()),
        cached_result: Mutex::new(None),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/query", post(post_query))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Application started on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

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
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

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

#[derive(Deserialize, Debug)]
struct QueryRequest {
    query: String,
}

#[derive(Template, Clone)]
#[template(path = "query.html")]
struct QueryResponse {
    tokens: Vec<String>,
    time_ms: u128,
    documents: Vec<Document>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Document {
    id: u32,
    score: f64,
    path: String,
    content: String,
}

#[debug_handler]
async fn post_query(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<QueryRequest>,
) -> impl IntoResponse {
    info!("Query request: {}", payload.query);

    let mut cq = state.cached_query.lock().unwrap();
    let mut cqr = state.cached_result.lock().unwrap();

    if *cq == payload.query {
        info!("Cache hit for query: {}", payload.query);
        return HtmlTemplate(cqr.clone().unwrap());
    }

    let mut q = state.query_processor.lock().unwrap();

    let query_result = q.query(&payload.query, 100);

    let documents = query_result
        .documents
        .iter()
        .map(|r| Document {
            id: r.id,
            score: r.score,
            path: r.path.clone(),
            content: read_file_content(r.path.clone()),
        })
        .collect();

    let response = QueryResponse {
        tokens: query_result.tokens,
        documents,
        time_ms: query_result.time_ms,
    };

    info!("Replacing cache with query: {}", payload.query);
    let _ = replace(&mut *cq, payload.query);
    let _ = replace(&mut *cqr, Some(response.clone()));

    HtmlTemplate(response)
}

fn read_file_content(path: String) -> String {
    read_to_string(path).expect("error while reading file")
}
