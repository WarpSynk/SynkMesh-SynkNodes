use crate::node::SynkNode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::Filter;

#[derive(Deserialize)]
struct StoreRequest {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct Status {
    node_id: String,
    tcp_port: u16,
    http_port: u16,
    keys: Vec<String>,
}

pub async fn run_api(node: Arc<SynkNode>) {
    let health = warp::path!("status").map({
        let node = node.clone();
        move || {
            let status = Status {
                node_id: node.id.clone(),
                tcp_port: node.tcp_port,
                http_port: node.http_port,
                keys: node.storage.list_keys(),
            };
            warp::reply::json(&status)
        }
    });

    let get_data = warp::path!("data" / String).map({
        let node = node.clone();
        move |key: String| {
            match node.storage.get(&key) {
                Some(v) => warp::reply::json(&serde_json::json!({ "key": key, "value": v })),
                None => warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND),
            }
        }
    });

    let put = warp::path!("store")
        .and(warp::post())
        .and(warp::body::json())
        .map({
            let node = node.clone();
            move |req: StoreRequest| {
                if let Err(e) = node.storage.set(&req.key, &req.value) {
                    log::error!("store error: {}", e);
                    return warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }
                warp::reply::with_status("OK", warp::http::StatusCode::CREATED).into_response()
            }
        });

    let routes = health.or(get_data).or(put);

    let addr = ([0, 0, 0, 0], node.http_port);
    log::info!("HTTP api listening on {}", node.http_port);
    warp::serve(routes).run(addr).await;
}