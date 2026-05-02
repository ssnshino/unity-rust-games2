use axum::{
    extract::Path,
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::{json, Map, Value};
use std::{net::SocketAddr, path::{Component, PathBuf}};

const AIRRACE_ROUNDS_JSON: &str = include_str!("airrace/airrace_rounds.json");
const ROUND_WORLD_CATALOG_JSON: &str = include_str!("airrace3d/StreamingAssets/AirRace/round_world_catalog.json");
const AIRCRAFT_PREFAB_CATALOG_JSON: &str = include_str!("airrace3d/StreamingAssets/AirRace/aircraft_prefab_catalog.json");
const WORLD_OBJECT_CATALOG_JSON: &str = include_str!("airrace3d/StreamingAssets/AirRace/world_object_catalog.json");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/airrace3d", get(airrace3d_index))
        .route("/airrace3d/", get(airrace3d_index))
        .route("/airrace3d/*path", get(static_airrace3d))
        .route("/api/airrace3d/round-index", get(round_index))
        .route("/api/airrace3d/course/:round", get(course))
        .route("/api/airrace3d/field-catalog", get(field_catalog))
        .route("/api/airrace3d/world-object-catalog", get(world_object_catalog))
        .route("/api/airrace3d/round-world-catalog", get(round_world_catalog))
        .route("/api/airrace3d/aircraft-catalog", get(aircraft_catalog))
        .route("/api/airrace3d/aircraft-models", get(aircraft_models))
        .route("/api/airrace3d/aircraft-prefab-catalog", get(aircraft_prefab_catalog));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("unity-rust-games2 listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>ゲームセンターしのみ屋２号店</title>
  <style>
    body{margin:0;min-height:100vh;background:#0d1624;color:#f7e38b;font-family:system-ui,sans-serif;display:grid;place-items:center}
    main{width:min(900px,92vw);padding:40px;border:1px solid #f0b42955;border-radius:24px;background:#111d2dcc}
    h1{font-size:44px;margin:0 0 8px;letter-spacing:.06em}
    p{color:#d7e7ff;margin:0 0 28px}
    a{display:inline-block;padding:18px 28px;border:1px solid #f0b429;border-radius:18px;color:#f7e38b;text-decoration:none;font-size:22px;background:#17283d}
  </style>
</head>
<body>
  <main>
    <h1>ゲームセンターしのみ屋２号店</h1>
    <a href="/airrace3d/">AIRRACE3D2</a>
  </main>
</body>
</html>"#,
    )
}

async fn airrace3d_index() -> Html<&'static str> {
    Html(include_str!("airrace3d/index.html"))
}

async fn round_index() -> Response {
    match serde_json::from_str::<Value>(AIRRACE_ROUNDS_JSON) {
        Ok(Value::Object(map)) => {
            let mut rounds: Vec<u32> = map.keys().filter_map(|k| k.parse::<u32>().ok()).collect();
            rounds.sort_unstable();
            let final_round = rounds.iter().copied().max().unwrap_or(0);
            json_response(json!({ "finalRound": final_round, "rounds": rounds }))
        }
        _ => error_response(StatusCode::INTERNAL_SERVER_ERROR, "invalid round catalog"),
    }
}

async fn course(Path(round): Path<u32>) -> Response {
    match serde_json::from_str::<Value>(AIRRACE_ROUNDS_JSON) {
        Ok(Value::Object(map)) => match map.get(&round.to_string()) {
            Some(course) => json_response(course.clone()),
            None => error_response(StatusCode::NOT_FOUND, "round not found"),
        },
        _ => error_response(StatusCode::INTERNAL_SERVER_ERROR, "invalid round catalog"),
    }
}

async fn field_catalog() -> Response {
    json_response(json!({
        "version": "airrace3d-field-v1",
        "globalField": {
            "minX": -32000,
            "maxX": 32000,
            "minZ": -32000,
            "maxZ": 32000,
            "margin": 640
        },
        "rounds": []
    }))
}

async fn world_object_catalog() -> Response {
    json_text_response(WORLD_OBJECT_CATALOG_JSON)
}

async fn round_world_catalog() -> Response {
    json_text_response(ROUND_WORLD_CATALOG_JSON)
}

async fn aircraft_catalog() -> Response {
    json_response(json!({
        "version": "airrace3d-aircraft-v3",
        "defaultAircraftId": "skylancer",
        "aircrafts": [
            {"id":"skylancer","name":"スカイランサー号","nameEn":"Skylancer","summary":"平均的な優等生。初見コースの基準機。","summaryEn":"Balanced all-rounder. Best baseline for new courses.","speedMul":1.00,"boostMul":1.00,"turnMul":1.00,"climbMul":1.00,"color":"rgba(125, 211, 252, 0.78)","stroke":"#e0f2fe","shape":"standard"},
            {"id":"thunderbolt","name":"サンダーボルト号","nameEn":"Thunderbolt","summary":"きびきび旋回。テクニカル向け。","summaryEn":"Sharp turning. Great for technical sections.","speedMul":0.94,"boostMul":0.94,"turnMul":1.24,"climbMul":1.08,"color":"rgba(250, 204, 21, 0.78)","stroke":"#fef3c7","shape":"wide"},
            {"id":"shootingstar","name":"シューティングスター号","nameEn":"Shooting Star","summary":"高速番長。直線と大カーブで強い。","summaryEn":"Top speed specialist. Dominates straights and long bends.","speedMul":1.08,"boostMul":1.16,"turnMul":0.82,"climbMul":0.94,"color":"rgba(248, 113, 113, 0.80)","stroke":"#fee2e2","shape":"dart"},
            {"id":"spiralfang","name":"スパイラルファング号","nameEn":"Spiral Fang","summary":"ピーキーな軽量機。反応最速。","summaryEn":"Twitchy lightweight. Fastest response.","speedMul":0.98,"boostMul":0.98,"turnMul":1.32,"climbMul":1.14,"color":"rgba(192, 132, 252, 0.82)","stroke":"#f3e8ff","shape":"fang"},
            {"id":"ironhawk","name":"アイアンホーク号","nameEn":"Iron Hawk","summary":"重いけど安定。崩れにくい。","summaryEn":"Heavy but stable. Hard to destabilize.","speedMul":1.03,"boostMul":1.08,"turnMul":0.90,"climbMul":0.92,"color":"rgba(74, 222, 128, 0.76)","stroke":"#dcfce7","shape":"heavy"}
        ]
    }))
}

async fn aircraft_models() -> Response {
    json_response(json!({ "version": "airrace3d-aircraft-models-v1", "models": [] }))
}

async fn aircraft_prefab_catalog() -> Response {
    json_text_response(AIRCRAFT_PREFAB_CATALOG_JSON)
}

async fn static_airrace3d(Path(path): Path<String>) -> Response {
    let Some(safe_path) = safe_relative_path(&path) else {
        return error_response(StatusCode::BAD_REQUEST, "invalid path");
    };
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/airrace3d").join(safe_path);
    match tokio::fs::read(&full_path).await {
        Ok(bytes) => bytes_response(bytes, content_type(&full_path), full_path.extension().and_then(|s| s.to_str()) == Some("gz")),
        Err(_) => error_response(StatusCode::NOT_FOUND, "not found"),
    }
}

fn safe_relative_path(path: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(part) => out.push(part),
            _ => return None,
        }
    }
    Some(out)
}

fn json_text_response(text: &'static str) -> Response {
    match serde_json::from_str::<Value>(text) {
        Ok(value) => json_response(value),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "invalid embedded json"),
    }
}

fn json_response(value: Value) -> Response {
    let body = serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec());
    bytes_response(body, "application/json; charset=utf-8", false)
}

fn error_response(status: StatusCode, message: &'static str) -> Response {
    let mut map = Map::new();
    map.insert("error".to_string(), Value::String(message.to_string()));
    let mut response = json_response(Value::Object(map));
    *response.status_mut() = status;
    response
}

fn bytes_response(bytes: Vec<u8>, content_type: &'static str, gzip: bool) -> Response {
    let mut response = bytes.into_response();
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    if gzip {
        response.headers_mut().insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    }
    response
}

fn content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "data" => "application/octet-stream",
        "json" => "application/json; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "gz" => match path.file_stem().and_then(|s| std::path::Path::new(s).extension()).and_then(|s| s.to_str()).unwrap_or_default() {
            "js" => "application/javascript; charset=utf-8",
            "wasm" => "application/wasm",
            "data" => "application/octet-stream",
            _ => "application/octet-stream",
        },
        _ => "application/octet-stream",
    }
}
