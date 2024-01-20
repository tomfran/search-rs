use actix_web::{post, web, App, HttpServer, Responder, Result};
use search::query::QueryProcessor;
use serde::{Deserialize, Serialize};
use std::{env, sync::Mutex, time::Instant};

#[derive(Deserialize, Debug)]
struct QueryRequest {
    query: String,
    limit: usize,
}

#[derive(Serialize)]
struct QueryResponse {
    num_results: u32,
    time_ms: u128,
    documents: Vec<QueryDocumentResponse>,
}

#[derive(Serialize)]
struct QueryDocumentResponse {
    id: u32,
    score: f32,
    path: String,
}

#[post("/query")]
async fn query(
    r: web::Json<QueryRequest>,
    q: web::Data<Mutex<QueryProcessor>>,
) -> Result<impl Responder> {
    println!("query: {:?}", r);

    let mut local_q = q.lock().unwrap();

    let start_time = Instant::now();
    let result = local_q.query(&r.query, r.limit);
    let elapsed_time = start_time.elapsed();

    let response = QueryResponse {
        num_results: result.len() as u32,
        time_ms: elapsed_time.as_millis(),
        documents: result
            .iter()
            .map(|e| QueryDocumentResponse {
                id: e.id,
                score: e.score,
                path: e.path.clone(),
            })
            .collect(),
    };

    Ok(web::Json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --bin client <base_path>");
        return Ok(());
    }

    let base_path = &args[1];
    let index_path = format!("{}/index/index", base_path);
    let tokenizer_path = format!("{}/tokenizer/bert-base-uncased", base_path);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(
                QueryProcessor::build_query_processor(&index_path, &tokenizer_path),
            )))
            .service(query)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
