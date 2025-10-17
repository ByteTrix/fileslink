use std::convert::Infallible;
use std::collections::HashMap;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{self, State, Query},
    response::{Html, Response},
    routing::{get, Router},
};
use http::{header::CONTENT_TYPE, StatusCode};
use log::{debug, error, info, warn};
use teloxide::net::Download;
use teloxide::prelude::Requester;
use base64::Engine;

use shared::file_storage::{get_file_metadata, list_all_files, FileMetadata};
use crate::config::Config;
use shared::link_utils::extract_id_from_path;

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

/// Proxy large file download to FastTelethon service
async fn proxy_to_fasttelethon(metadata: &FileMetadata) -> Result<Response<Body>, Infallible> {
    let config = Config::instance().await;
    let fasttelethon_url = config.fasttelethon_url();
    
    // Get storage channel ID from config
    let channel_id = match config.storage_channel_id() {
        Ok(id) => id.to_string(),
        Err(_) => {
            error!("STORAGE_CHANNEL_ID not configured");
            return Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("Large file download service not configured"))
                .unwrap());
        }
    };
    
    // Get message_id from metadata (needed for FastTelethon MTProto download)
    let message_id = match metadata.message_id {
        Some(id) => id,
        None => {
            // Fall back to Bot API for files without message_id (uploaded before FastTelethon integration)
            warn!("File {} has no message_id, cannot use FastTelethon", metadata.unique_id);
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File was uploaded before FastTelethon integration and cannot be downloaded via MTProto. Please re-upload the file."))
                .unwrap());
        }
    };
    
    // Build FastTelethon download URL
    let download_url = format!("{}/download/{}/{}", fasttelethon_url, channel_id, message_id);
    
    info!("Proxying large file download to FastTelethon: {}", download_url);
    
    // Make HTTP request to FastTelethon service
    match reqwest::get(&download_url).await {
        Ok(response) => {
            if response.status().is_success() {
                // Get headers from FastTelethon response (clone to avoid borrow issues)
                let content_type = response.headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                
                let content_disposition = response.headers()
                    .get("content-disposition")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("attachment; filename=\"{}\"", metadata.file_name));
                
                // Stream the response body
                let bytes = response.bytes().await.unwrap_or_default();
                
                info!("Successfully proxied {} bytes from FastTelethon", bytes.len());
                
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, content_type)
                    .header("Content-Disposition", content_disposition)
                    .header("X-Content-Type-Options", "nosniff")
                    .body(bytes.into())
                    .unwrap())
            } else {
                error!("FastTelethon returned error: {}", response.status());
                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Body::from("Failed to retrieve file from download service"))
                    .unwrap())
            }
        }
        Err(e) => {
            error!("Failed to connect to FastTelethon: {:?}", e);
            Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("Download service temporarily unavailable"))
                .unwrap())
        }
    }
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
    extract::Path(id): extract::Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response<Body>, Infallible> {
    debug!("Requested file with path: {}", id);
    
    // Extract unique ID from path (format: abc123_filename.ext) using shared util
    let unique_id = extract_id_from_path(&id);
    
    debug!("Extracted unique ID: {}", unique_id);

    // Get file metadata from storage
    let metadata = match get_file_metadata(unique_id).await {
        Some(m) => m,
        None => {
            warn!("File not found with ID: {}", unique_id);
            let body = not_found_handler().await;
            return Ok((
                StatusCode::NOT_FOUND,
                [(CONTENT_TYPE, "text/html")],
                body,
            ).into_response());
        }
    };

    info!("Found file: {} (Telegram ID: {})", metadata.file_name, metadata.telegram_file_id);

    // Try to get file from Telegram, but if it's too big, proxy to FastTelethon
    let file_info = match state.bot.get_file(&metadata.telegram_file_id).await {
        Ok(info) => Some(info),
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if error_msg.contains("file is too big") || error_msg.contains("Bad Request") {
                warn!("File too large for bot API, proxying to FastTelethon: {}", metadata.file_name);
                
                // Proxy to FastTelethon service for large files
                return proxy_to_fasttelethon(&metadata).await;
            } else {
                error!("Failed to get file info from Telegram: {:?}", e);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to retrieve file from storage"))
                    .unwrap());
            }
        }
    };

    // Download file from Telegram if possible
    let mut file_bytes = Vec::new();
    if let Some(file_info) = file_info {
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
    }

    // Check if auto-close is requested via ?close=1
    let auto_close = params.get("close").is_some();
    
    // Determine content type, allow force download via ?dl=1
    let force_download = params.get("dl").is_some();
    let content_type = if force_download {
        "application/octet-stream".to_string()
    } else {
        metadata.mime_type
            .unwrap_or_else(|| "application/octet-stream".to_string())
    };

    // Use original filename from metadata, force download as attachment
    let content_disposition = format!("attachment; filename=\"{}\"", metadata.file_name);

    info!("Serving file: {} ({} bytes) with content type: {}", 
          metadata.file_name, file_bytes.len(), content_type);

    // If auto-close requested, return HTML with auto-download and close script
    if auto_close {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Downloading {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        .loader {{ border: 5px solid #f3f3f3; border-top: 5px solid #3498db; 
                   border-radius: 50%; width: 50px; height: 50px; 
                   animation: spin 1s linear infinite; margin: 20px auto; }}
        @keyframes spin {{ 0% {{ transform: rotate(0deg); }} 100% {{ transform: rotate(360deg); }} }}
    </style>
</head>
<body>
    <h2>Downloading {}</h2>
    <div class="loader"></div>
    <p>Your download will begin shortly...</p>
    <p><small>This window will close automatically.</small></p>
    <script>
        // Create blob from base64 data and trigger download
        const base64Data = "{}";
        const byteCharacters = atob(base64Data);
        const byteNumbers = new Array(byteCharacters.length);
        for (let i = 0; i < byteCharacters.length; i++) {{
            byteNumbers[i] = byteCharacters.charCodeAt(i);
        }}
        const byteArray = new Uint8Array(byteNumbers);
        const blob = new Blob([byteArray], {{ type: '{}' }});
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = "{}";
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        
        // Close window after 2 seconds
        setTimeout(function() {{
            window.close();
        }}, 2000);
    </script>
</body>
</html>"#,
            metadata.file_name,
            metadata.file_name,
            base64::engine::general_purpose::STANDARD.encode(&file_bytes),
            content_type,
            metadata.file_name
        );
        
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html; charset=utf-8")
            .body(html.into())
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header("Content-Disposition", content_disposition)
        .header("X-Content-Type-Options", "nosniff")
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