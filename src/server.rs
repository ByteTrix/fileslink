use std::convert::Infallible;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{self, State},
    response::{Html, Response},
    routing::{get, Router},
};
use http::{header::CONTENT_TYPE, StatusCode};
use log::{debug, error, info, warn};
use teloxide::net::Download;
use teloxide::prelude::Requester;

use shared::file_storage::{get_file_metadata, list_all_files};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub bot: Arc<teloxide::Bot>,
}

pub async fn create_app(bot: Arc<teloxide::Bot>) -> Router {
    let enable_files_route = Config::instance().await.enable_files_route();

    let state = AppState { bot };

    let mut router = Router::new()
        .route("/", get(root))
        .route("/files/:id", get(files_id))
        .with_state(state);

    if enable_files_route {
        router = router.route("/files", get(files_list));
    }

    router.fallback(not_found_handler)
}

/// Lists all files from the file storage metadata
async fn files_list() -> Result<Response<Body>, Infallible> {
    info!("Files list accessed");

    let files = list_all_files().await;

    if files.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html")
            .body(Body::from("<h1>Files in storage</h1><p>No files uploaded yet.</p>"))
            .unwrap());
    }

    let mut html = String::from("<h1>Files in storage</h1><ul>");

    for file in files {
        html.push_str(&format!(
            "<li><a href=\"/files/{}\">{}</a> ({} bytes)</li>",
            file.unique_id, file.file_name, file.file_size
        ));
    }

    html.push_str("</ul>");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap())
}

async fn files_id(
    State(state): State<AppState>,
    extract::Path(id): extract::Path<String>
) -> Result<Response<Body>, Infallible> {
    debug!("Requested file with unique ID: {}", id);

    // Get file metadata from storage
    let metadata = match get_file_metadata(&id).await {
        Some(m) => m,
        None => {
            warn!("File not found with ID: {}", id);
            let body = not_found_handler().await;
            return Ok((
                StatusCode::NOT_FOUND,
                [(CONTENT_TYPE, "text/html")],
                body,
            ).into_response());
        }
    };

    info!("Found file: {} (Telegram ID: {})", metadata.file_name, metadata.telegram_file_id);

    // Get file from Telegram
    let file_info = match state.bot.get_file(&metadata.telegram_file_id).await {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to get file info from Telegram: {:?}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to retrieve file from storage"))
                .unwrap());
        }
    };

    // Download file from Telegram
    let mut file_bytes = Vec::new();
    match state.bot.download_file(&file_info.path, &mut file_bytes).await {
        Ok(_) => {
            info!("Successfully downloaded {} bytes from Telegram", file_bytes.len());
        }
        Err(e) => {
            error!("Failed to download file from Telegram: {:?}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to download file from storage"))
                .unwrap());
        }
    }

    // Determine content type
    let content_type = metadata.mime_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let content_disposition = format!("attachment; filename=\"{}\"", metadata.file_name);

    info!("Serving file: {} ({} bytes) with content type: {}", 
          metadata.file_name, file_bytes.len(), content_type);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header("Content-Disposition", content_disposition)
        .body(file_bytes.into())
        .unwrap())
}

async fn root() -> Html<&'static str> {
    info!("Root path accessed");

    Html("\
    <h1>Server working</h1>\
    <div><a href=\"https://github.com/bytetrix/fileslink\">GitHub</a></div>\
    ")
}

async fn not_found_handler() -> Html<&'static str> {
    Html("\
    <h1>404 Not Found</h1>\
    <p>The page you are looking for does not exist.</p>\
    <a href=\"/\">Go back to the homepage</a>\
    ")
}