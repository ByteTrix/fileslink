use crate::bot::TeloxideBot;
use log::{debug, error, info, warn};
use nanoid::nanoid;
use shared::config::Config;
use shared::file_storage::{save_file_metadata, FileMetadata};
use shared::link_utils::build_url_path;
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use teloxide::payloads::{EditMessageTextSetters, SendDocumentSetters, SendPhotoSetters, SendVideoSetters, SendAnimationSetters};
use teloxide::prelude::{Message, Requester};
use teloxide::types::{ChatId, InputFile, ParseMode};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct FileQueueItem {
    message: Arc<Message>,
    queue_message: Arc<Message>,
    file_id: Option<String>,
    file_name: Option<String>,
    url: Option<String>,
}

impl FileQueueItem {
    pub fn new(
        message: Arc<Message>,
        queue_message: Arc<Message>,
        file_id: Option<String>,
        file_name: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            message,
            queue_message,
            file_id,
            file_name,
            url,
        }
    }
}

impl Display for FileQueueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileQueueItem {{ message: {:?}, queue_message: {:?}, file_id: {:?}, file_name: {:?}, url: {:?} }}", self.message, self.queue_message, self.file_id, self.file_name, self.url)
    }
}

pub type FileQueueType = Arc<Mutex<Vec<FileQueueItem>>>;

impl FileQueueItem {
    /// Human-readable summary for queue display
    pub fn summary(&self) -> String {
        if let Some(name) = &self.file_name {
            return format!("{}", name);
        }
        if let Some(url) = &self.url {
            let short = if url.len() > 48 { format!("{}…", &url[..48]) } else { url.clone() };
            return format!("URL: {}", short);
        }
        if let Some(fid) = &self.file_id {
            return format!("file_id: {}", fid);
        }
        "<unknown>".to_string()
    }
}

/// Get a snapshot of the current queue as display strings
pub async fn get_queue_snapshot(queue: &FileQueueType, limit: usize) -> (usize, Vec<String>) {
    let q = queue.lock().await;
    let total = q.len();
    let mut items = Vec::new();
    for (i, item) in q.iter().take(limit).enumerate() {
        items.push(format!("{}. {}", i + 1, item.summary()));
    }
    (total, items)
}

/// Clear the entire queue, returning number of items removed
pub async fn clear_queue_all(queue: &FileQueueType) -> usize {
    let mut q = queue.lock().await;
    let n = q.len();
    q.clear();
    n
}


pub async fn process_queue(
    bot: Arc<TeloxideBot>,
    file_queue: FileQueueType,
    mut rx: Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    Ok(while let Some(()) = rx.recv().await {
        let queue_item = {
            let queue = file_queue.lock().await;

            if let Some(item) = queue.first() {
                item.clone()
            } else {
                continue;
            }
        };

        debug!("Processing file: {:?}", queue_item);

        const MAX_ATTEMPTS: u32 = 3;

        for attempt in 1..=MAX_ATTEMPTS {
            match bot.get_teloxide_bot().edit_message_text(
                queue_item.message.chat.id,
                queue_item.queue_message.id,
                "Processing file...",
            ).await {
                Ok(_) => break,
                Err(e) => {
                    if attempt == MAX_ATTEMPTS {
                        warn!("Failed to edit message text after {} attempts: {:?}", MAX_ATTEMPTS, e);
                    } else {
                        let delay = Duration::from_secs(2_u64.pow(attempt - 1));

                        warn!("Attempt to edit message {} failed, retrying in {:?}... Error: {:?}", attempt, delay, e);

                        sleep(delay).await;
                    }
                }
            }
        }

        if let Err(e) = if let Some(url) = &queue_item.url {
            download_and_store_file_from_url(
                bot.clone(),
                queue_item.clone(),
                url,
            ).await
        } else if let Some(file_id) = &queue_item.file_id {
            forward_file_to_storage_channel(
                bot.clone(),
                queue_item.clone(),
                file_id,
            ).await
        } else {
            Err("No file_id or url found".to_string())
        } {
            error!("Failed to process file: {}", e);
            continue;
        }

        let mut queue = file_queue.lock().await;

        queue.remove(0);

        if let Some(front) = queue.first() {
            let queue_item = front.clone();

            bot.get_teloxide_bot().edit_message_text(
                queue_item.queue_message.chat.id,
                queue_item.queue_message.id,
                format!("File processed. Remaining files in queue: {}", queue.len()),
            ).await.expect("Failed to edit message");
        }

        info!("Removed item from queue. Remaining items in queue: {}", queue.len());
    })
}


async fn forward_file_to_storage_channel(
    bot: Arc<TeloxideBot>,
    queue_item: FileQueueItem,
    file_id: &String,
) -> Result<(), String> {
    info!("Forwarding file to storage channel. File ID: {}", file_id);

    // Get storage channel ID from config
    let storage_channel_id = Config::instance().await.storage_channel_id()
        .map_err(|e| format!("Storage channel not configured: {}", e))?;

    // Generate unique ID for this file
    let unique_id = nanoid!(8);

    // Determine filename, mime type, and file size from the message itself (avoids get_file for large files)
    let file_size: u32;
    let mime_type: Option<String>;
    let mut final_file_name: Option<String> = queue_item.file_name.clone();

    // Forward the message to storage channel by copying the file
    let forwarded_msg = if let Some(doc) = queue_item.message.document() {
        // Prefer original filename for documents if not provided explicitly
        if final_file_name.is_none() {
            if let Some(name) = doc.file_name.clone() { final_file_name = Some(name); }
        }
        file_size = doc.file.size;
        mime_type = doc.mime_type.as_ref().map(|m| m.to_string());
        bot.get_teloxide_bot()
            .send_document(ChatId(storage_channel_id), InputFile::file_id(&doc.file.id))
            .caption(&unique_id)
            .await
            .map_err(|e| format!("Failed to forward document: {}", e))?
    } else if let Some(photo) = queue_item.message.photo() {
        let largest_photo = photo.last().ok_or("No photo found")?;
        // Telegram photos are JPEG; generate a meaningful name if not provided
        if final_file_name.is_none() {
            final_file_name = Some(format!("photo_{}.jpg", unique_id));
        }
        file_size = largest_photo.file.size;
        mime_type = Some("image/jpeg".to_string());
        bot.get_teloxide_bot()
            .send_photo(ChatId(storage_channel_id), InputFile::file_id(&largest_photo.file.id))
            .caption(&unique_id)
            .await
            .map_err(|e| format!("Failed to forward photo: {}", e))?
    } else if let Some(video) = queue_item.message.video() {
        if final_file_name.is_none() {
            // Most Telegram videos are MP4
            final_file_name = Some(format!("video_{}.mp4", unique_id));
        }
        file_size = video.file.size;
        mime_type = video.mime_type.as_ref().map(|m| m.to_string()).or(Some("video/mp4".to_string()));
        bot.get_teloxide_bot()
            .send_video(ChatId(storage_channel_id), InputFile::file_id(&video.file.id))
            .caption(&unique_id)
            .await
            .map_err(|e| format!("Failed to forward video: {}", e))?
    } else if let Some(animation) = queue_item.message.animation() {
        if final_file_name.is_none() {
            // Animation could be GIF or MP4; attempt to infer
            let ext = match animation.mime_type.as_ref().map(|m| m.essence_str()) {
                Some("image/gif") => "gif",
                _ => "mp4",
            };
            final_file_name = Some(format!("animation_{}.{}", unique_id, ext));
        }
        file_size = animation.file.size;
        mime_type = animation.mime_type.as_ref().map(|m| m.to_string()).or(Some("video/mp4".to_string()));
        bot.get_teloxide_bot()
            .send_animation(ChatId(storage_channel_id), InputFile::file_id(&animation.file.id))
            .caption(&unique_id)
            .await
            .map_err(|e| format!("Failed to forward animation: {}", e))?
    } else {
        return Err("Unsupported file type".to_string());
    };

    // Get the new file_id from the forwarded message
    let stored_file_id = if let Some(doc) = forwarded_msg.document() {
        doc.file.id.clone()
    } else if let Some(photo) = forwarded_msg.photo() {
        photo.last().ok_or("No photo in forwarded message")?.file.id.clone()
    } else if let Some(video) = forwarded_msg.video() {
        video.file.id.clone()
    } else if let Some(animation) = forwarded_msg.animation() {
        animation.file.id.clone()
    } else {
        return Err("Could not get file_id from forwarded message".to_string());
    };

    info!("File stored in channel with ID: {}", stored_file_id);

    // Capture the message ID for FastTelethon downloads
    let message_id = forwarded_msg.id.0;
    info!("Stored message ID: {}", message_id);

    // Finalize filename and mime type
    let file_name = final_file_name.unwrap_or_else(|| format!("file_{}", unique_id));
    let mime_type = mime_type.or_else(|| mime_guess::from_path(&file_name).first().map(|m| m.to_string()));

    // Save metadata to our mapping storage
    let metadata = FileMetadata {
        unique_id: unique_id.clone(),
        telegram_file_id: stored_file_id,
        file_name: file_name.clone(),
        mime_type,
        file_size,
        uploaded_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        message_id: Some(message_id),
    };

    save_file_metadata(metadata).await
        .map_err(|e| format!("Failed to save file metadata: {}", e))?;

    info!("File metadata saved successfully");

    // Send the download link to the user
    edit_message_with_file_link(bot, &queue_item, &unique_id, &file_name, file_size).await
}

async fn download_and_store_file_from_url(
    bot: Arc<TeloxideBot>,
    queue_item: FileQueueItem,
    url: &String,
) -> Result<(), String> {
    info!("Downloading file from URL: {}", url);

    // Get storage channel ID from config
    let storage_channel_id = Config::instance().await.storage_channel_id()
        .map_err(|e| format!("Storage channel not configured: {}", e))?;

    // Download the file
    let response = reqwest::get(url).await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    // Determine filename
    let content_disposition = response.headers().get(reqwest::header::CONTENT_DISPOSITION);
    let file_name = content_disposition
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split("filename=").nth(1))
        .map(|v| v.trim_matches('"').to_string())
        .or_else(|| url.split('/').last().map(|name| name.to_string()))
        .filter(|name| !name.is_empty())
        .ok_or("Could not determine file name")?;

    // Get file content
    let file_bytes = response.bytes().await
        .map_err(|e| format!("Failed to read file bytes: {}", e))?;

    let file_size = file_bytes.len() as u32;

    info!("Downloaded {} bytes from URL", file_size);

    // Generate unique ID
    let unique_id = nanoid!(8);

    let mime_type = mime_guess::from_path(&file_name)
        .first()
        .map(|m| m.to_string());

    // Upload to storage channel
    let uploaded_msg = bot.get_teloxide_bot()
        .send_document(
            ChatId(storage_channel_id),
            InputFile::memory(file_bytes).file_name(file_name.clone())
        )
        .caption(&unique_id)
        .await
        .map_err(|e| format!("Failed to upload to storage channel: {}", e))?;

    // Get the file_id from the uploaded message
    let stored_file_id = uploaded_msg.document()
        .ok_or("No document in uploaded message")?
        .file.id.clone();

    info!("File stored in channel with ID: {}", stored_file_id);

    // Capture the message ID for FastTelethon downloads
    let message_id = uploaded_msg.id.0;
    info!("Stored message ID: {}", message_id);

    // Save metadata
    let metadata = FileMetadata {
        unique_id: unique_id.clone(),
        telegram_file_id: stored_file_id,
        file_name: file_name.clone(),
        mime_type,
        file_size,
        uploaded_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        message_id: Some(message_id),
    };

    save_file_metadata(metadata).await
        .map_err(|e| format!("Failed to save file metadata: {}", e))?;

    info!("File metadata saved successfully");

    // Send the download link to the user
    edit_message_with_file_link(bot, &queue_item, &unique_id, &file_name, file_size).await
}


#[allow(dead_code)]
async fn generate_final_file_name(queue_item: &FileQueueItem, file_path_or_name: &str) -> String {
    let id = nanoid!(5);
    let name = queue_item.file_name.as_ref().map(|name| name.to_string().replace(' ', "_"));
    match name {
        Some(name) => format!("{}_{}", id, name),
        None => {
            let file_name = file_path_or_name.split('/').last().unwrap_or("file");
            format!("{}_{}", id, file_name)
        }
    }
}

/// Get file info from Telegram
///
/// # Arguments
/// * `bot` - Bot instance
/// * `id` - File ID
/// # Returns
/// * `Result` containing a tuple of file path and file size
/// * `String` containing an error message
#[allow(dead_code)]
async fn get_file_info(bot: Arc<TeloxideBot>, id: &String) -> Result<(String, u32), String> {
    const MAX_ATTEMPTS: u32 = 3;

    for attempt in 1..=MAX_ATTEMPTS {
        match bot.get_teloxide_bot().get_file(id).await {
            Ok(info) => return Ok((info.clone().path, info.size)),
            Err(e) => {
                if attempt == MAX_ATTEMPTS {
                    error!("Failed to get file info after {} attempts: {:?}", MAX_ATTEMPTS, e);

                    return Err("Failed to get file info".to_owned());
                } else {
                    warn!("Attempt {} failed, retrying... Error: {:?}", attempt, e);

                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    unreachable!()
}

// #[derive(BotCommands, Clone)]
// #[command(rename_rule = "lowercase", description = "These commands are supported:")]
// enum Command {
//     #[command(description = "display this text.")]
//     Help,
//     #[command(description = "download a file from the URL.")]
//     Url(String),
// }
//

fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{} bytes", bytes),
    }
}

async fn edit_message_with_file_link(
    bot: Arc<TeloxideBot>,
    queue_item: &FileQueueItem,
    unique_id: &str,
    file_name: &str,
    file_size: u32,
) -> Result<(), String> {
    let file_domain = Config::instance().await.file_domain();
    // Build full URL path using shared util (id + url-safe filename)
    let full_path = build_url_path(unique_id, file_name);
    info!("Generated download link: {}{}", file_domain, full_path);
    let size_str = human_size(file_size as u64);
    let edit_result = bot.get_teloxide_bot().edit_message_text(
        queue_item.message.chat.id,
        queue_item.queue_message.id,
        format!(
            "✅ <b>File uploaded successfully!</b>\n\n📁 <b>File:</b> {}\n📊 <b>Size:</b> {}\n\n🔗 <b>Download Link:</b>\n<a href=\"{}{}\">{}{}</a>",
            file_name,
            size_str,
            file_domain,
            full_path,
            file_domain,
            full_path
        ),
    )
        .parse_mode(ParseMode::Html)
        .await;

    if edit_result.is_err() {
        error!("Failed to edit message");
        return Err("Failed to edit message".to_owned());
    }

    Ok(())
}
