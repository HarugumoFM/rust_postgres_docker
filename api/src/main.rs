use axum::{
    extract::{Json, State}, http::response, response::IntoResponse, routing::{get, post}, Router
};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, vec};
use axum::http::StatusCode;
use tokio_postgres::{NoTls, Row, Statement};
use bb8::{Pool};
use bb8_postgres::PostgresConnectionManager;
use hyper::header::CONTENT_TYPE;
use tower_http::cors::{CorsLayer, Any, AllowOrigin};

#[derive(Clone)]
struct AppState {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Deserialize)]
struct SearchQuery {
    word: String,
    descending: Option<bool>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct InsertQuery {
    title: String,
    vector: Vec<f32>,
}

#[derive(Serialize)]
struct SearchResult {
    id: i32,
    title: String,
    distance: f64,
}

#[derive(Serialize)]
struct Word {
    id: i32,
    title: String,
}

#[derive(Serialize)]
struct Response<T> {
    message: String,
    data: Vec<T>
}

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    dotenvy::dotenv().ok();
    
    // データベース接続
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)
        .expect("Failed to create PostgresConnectionManager");
    let pool = Pool::builder()
        .max_size(16) // 最大接続数を設定
        .connection_timeout(std::time::Duration::from_secs(30)) // タイムアウトを設定
        .build(manager)
        .await
        .expect("Failed to create connection pool");
    
    // アプリケーション状態の作成
    let state = Arc::new(AppState { pool });
    let allowed_origins = vec![
        "http://localhost:3000".parse().unwrap(),
        "http://127.0.0.1:3000".parse().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods(Any)
        .allow_headers(vec![CONTENT_TYPE]);
    // ルーターの設定
    let app = Router::new()
        .route("/", get(hello))
        .route("/connect", get(get_connect))
        .route("/documents", get(get_documents))
        .route("/docunemts/insert", post(insert_document))
        .with_state(state)
        .layer(cors); // CORSレイヤーを追加
    
    // サーバーの起動
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    println!("listening on {:?}", listener);
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn get_connect(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    println!("{}", std::env::var("DATABASE_URL").unwrap());
    match state.pool.get().await {
        Ok(_) => (StatusCode::OK, "Success").into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn get_documents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = state.pool.get().await.unwrap();
    let stmt = client.prepare("SELECT id, title FROM documents").await.unwrap();
    
    match client.query(&stmt, &[]).await {
        Ok(rows) => {
            let words = rows.iter().map(|row| {
                Word {
                    id: row.get("id"),
                    title: row.get("title"),
                }
            }).collect::<Vec<_>>();
            
            let response = Response {
                message: "Success".to_string(),
                data: words,
            };
            
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(err) => {
            let response:Response<Word> = Response {
                message: err.to_string(),
                data: vec![],
            };
            
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn insert_document(
    State(state): State<Arc<AppState>>,
    Json(query): Json<InsertQuery>,
) -> impl IntoResponse {
    let client = state.pool.get().await.unwrap();
    let vector = Vector::from(query.vector);
    let stmt = client.prepare("INSERT INTO documents (title,embedding) VALUES ($1, $2)").await.unwrap();
    
    match client.query(&stmt, &[&query.title, &vector]).await {
        Ok(_) => (StatusCode::CREATED, Json("Success")).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}