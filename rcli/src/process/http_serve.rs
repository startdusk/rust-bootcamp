use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Serving {:?} on port {}", path, port);
    let state = HttpServeState { path: path.clone() };
    let dir_service = ServeDir::new(path)
        // 增加压缩
        .append_index_html_on_directories(true)
        .precompressed_br()
        .precompressed_gzip()
        .precompressed_deflate();
    // axum router
    let router = Router::new()
        .route("/*path", get(file_handler))
        // .nest_service("/", dir_service)
        .route_service("/tower", dir_service)
        .with_state(Arc::new(state));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

// 对比 axum 和 tower-http 的静态文件服务
// tower-http 提供的能力更多 更强
async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(key): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(key);
    info!("Reading file {:?}", p);
    if !p.exists() {
        return (
            StatusCode::NOT_FOUND,
            format!("File {} note found", p.display()),
        );
    }

    // TODO: test p is a directory
    // if it is a directory, list all files/subdirectories
    // as <li><a href="/path/to/file">file name</a></li>
    // <html><body><ul>..</ul></body></html>
    match tokio::fs::read_to_string(p).await {
        Ok(content) => (StatusCode::OK, content),
        Err(err) => {
            warn!("Error reading file: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });
        let (status, content) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        assert!(content.trim().starts_with("[package]"));
    }
}
