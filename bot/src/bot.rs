use log::{debug, error, info};
use reqwest::{Client, Url};
use shared::chat_config::PermissionsConfig;
use shared::config::Config;
use std::sync::Arc;
use std::time::Duration;
use teloxide::prelude::{Message, Requester};
use teloxide::utils::command::{BotCommands, ParseError};
use tokio::sync::Mutex;
use crate::process_message::process_message;
use crate::queue::{FileQueueType, get_queue_snapshot, clear_queue_all};
use shared::file_storage::{list_all_files, delete_file_metadata, save_file_metadata};
use teloxide::types::ChatId;

pub trait Bot {
    fn new(config: Arc<Config>, permissions: Arc<Mutex<PermissionsConfig>>, queue: FileQueueType) -> Result<Self, String> where Self: Sized;
    fn run(&self, tx: tokio::sync::mpsc::Sender<()>) -> impl std::future::Future<Output=()> + Send;
}

#[derive(Debug, Clone)]
pub struct TeloxideBot {
    permissions: Arc<Mutex<PermissionsConfig>>,
    queue: FileQueueType,
    teloxide_bot: Arc<teloxide::Bot>,
}

impl TeloxideBot {
    pub fn get_teloxide_bot(&self) -> Arc<teloxide::Bot> {
        self.teloxide_bot.clone()
    }
}

impl Bot for TeloxideBot {
    fn new(config: Arc<Config>, permissions: Arc<Mutex<PermissionsConfig>>, queue: FileQueueType) -> Result<Self, String> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(300))
            .tcp_nodelay(true)
            .build()
            .unwrap_or_else(|e| {
                error!("Failed to create client: {}", e);
                Client::new()
            });

        let token = match config.bot_token() {
            Ok(t) => { t }
            Err(_) => {
                error!("Failed to get bot token");

                return Err("Failed to get bot token".to_owned());
            }
        };

        let mut bot = teloxide::Bot::with_client(token, client);

        bot = bot.set_api_url(Url::parse(config.telegram_api_url().as_str()).unwrap());

        let bot_ref = Arc::new(bot);

        Ok(TeloxideBot {
            teloxide_bot: bot_ref,
            permissions,
            queue,
        })
    }

    async fn run(&self, tx: tokio::sync::mpsc::Sender<()>) {
        let file_queue = Arc::clone(&self.queue);
        let permissions = Arc::clone(&self.permissions);
        let bot = self.teloxide_bot.clone();

    teloxide::repl(bot.clone(), move |msg: Message| {
            debug!("Received message: {:?}", msg);

            let bot = Arc::clone(&bot);
            let bot_clone = Arc::clone(&bot);
            let permissions = Arc::clone(&permissions);
            let file_queue = Arc::clone(&file_queue);
            let tx = tx.clone();

            async move {
                let permissions = permissions.lock().await;

                let from = match msg.from() {
                    Some(from) => from,
                    None => {
                        info!("Message does not have a sender");
                        return Ok(());
                    }
                };

                if !permissions.user_has_access(msg.chat.id.to_string(), &from.id.to_string()) {
                    info!(
                        "User {} does not have access to chat {}",
                        msg.from().unwrap().id,
                        msg.clone().chat.id
                    );

                    return Ok(());
                }

                info!(
                    "User {} has access to chat {}",
                    msg.from().unwrap().id,
                    msg.clone().chat.id
                );

                // Try to parse commands first
                if let Some(text) = msg.text() {
                    let chat_id = msg.chat.id;
                    // Allow spaces in /find query
                    if let Some(rest) = text.strip_prefix("/find ") {
                        handle_command(bot_clone.clone(), chat_id, file_queue.clone(), Command::Find { query: rest.trim().to_string() }).await;
                        return Ok(());
                    }
                    // Handle /list with optional page number
                    if text.starts_with("/list") {
                        let page = text
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<usize>().ok());
                        handle_list_command(bot_clone.clone(), chat_id, page).await;
                        return Ok(());
                    }
                    if let Ok(cmd) = Command::parse(text, "") {
                        handle_command(bot_clone.clone(), chat_id, file_queue.clone(), cmd).await;
                        return Ok(());
                    }
                }

                if let Err(e) = process_message(bot_clone.clone(), msg.clone(), file_queue, tx).await {
                    error!("Failed to process message: {}", e);
                }

                Ok(())
            }
        }).await;
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "show this help message")]
    Help,
    #[command(description = "list recent file links. Usage: /list or /list <page>")]
    List,
    #[command(description = "show current queue")]
    ShowQueue,
    #[command(description = "clear the processing queue")]
    ClearQueue,
    #[command(description = "delete a file by id (prefix of the link)")]
    Delete(String),
    #[command(description = "edit filename: /edit <id> <new_name.ext>", parse_with = split)]
    Edit { id: String, new_name: String },
    #[command(description = "search files by name: /find <query>")]
    Find { query: String },
}

// Custom argument parser for `/edit <id> <new_name>`
fn split(s: String) -> Result<(String, String), ParseError> {
    let mut parts = s.splitn(2, char::is_whitespace).filter(|p| !p.is_empty());
    let id = parts.next().ok_or_else(|| ParseError::Custom("missing id".into()))?;
    let new_name = parts.next().ok_or_else(|| ParseError::Custom("missing new filename".into()))?;
    Ok((id.to_string(), new_name.to_string()))
}

async fn handle_list_command(bot: Arc<teloxide::Bot>, chat_id: ChatId, page: Option<usize>) {
    let mut files = list_all_files().await;
    if files.is_empty() {
        let _ = bot.send_message(chat_id, "No files found").await;
        return;
    }
    files.sort_by_key(|m| m.uploaded_at);
    let per_page: usize = 10;
    let total = files.len();
    let total_pages = (total + per_page - 1) / per_page;
    let p = page.unwrap_or(1).max(1).min(total_pages.max(1));

    let start_from_end = (p - 1) * per_page;
    let slice: Vec<_> = files.into_iter().rev().skip(start_from_end).take(per_page).collect();
    let cfg = Config::instance().await;
    let domain = cfg.file_domain();
    let mut lines = Vec::new();
    lines.push(format!("Page {}/{} ({} total)", p, total_pages.max(1), total));
    for f in slice {
        let url_name = f.file_name.replace(' ', "_");
        lines.push(format!("- {} ({} bytes)\n{}{}_{}", f.file_name, f.file_size, domain, f.unique_id, url_name));
    }
    if total_pages > 1 {
        lines.push("\nTip: use /list <page>".to_string());
    }
    let _ = bot.send_message(chat_id, lines.join("\n")).await;
}

async fn handle_command(bot: Arc<teloxide::Bot>, chat_id: ChatId, queue: FileQueueType, cmd: Command) {
    match cmd {
        Command::Help => {
            let _ = bot.send_message(chat_id, Command::descriptions().to_string()).await;
        }
        Command::ShowQueue => {
            let (total, items) = get_queue_snapshot(&queue, 10).await;
            let mut text = format!("Queue size: {}\n", total);
            if items.is_empty() { text.push_str("(empty)"); } else { text.push_str(&items.join("\n")); }
            let _ = bot.send_message(chat_id, text).await;
        }
        Command::ClearQueue => {
            let n = clear_queue_all(&queue).await;
            let _ = bot.send_message(chat_id, format!("Cleared {} item(s) from queue", n)).await;
        }
        Command::List => {
            handle_list_command(bot.clone(), chat_id, None).await;
        }
        Command::Delete(id) => {
            let _ = delete_file_metadata(&id).await;
            let _ = bot.send_message(chat_id, format!("Deleted mapping for id: {}", id)).await;
        }
        Command::Edit { id, new_name } => {
            // Load all, update one, and save via save_file_metadata
            let files = list_all_files().await;
            if let Some(mut meta) = files.into_iter().find(|m| m.unique_id == id) {
                meta.file_name = new_name.clone();
                let _ = save_file_metadata(meta).await;
                let _ = bot.send_message(chat_id, format!("Updated filename for {}", id)).await;
            } else {
                let _ = bot.send_message(chat_id, format!("File id not found: {}", id)).await;
            }
        }
        Command::Find { query } => {
            let q = query.to_lowercase();
            if q.is_empty() {
                let _ = bot.send_message(chat_id, "Usage: /find <query>").await;
                return;
            }
            let mut files = list_all_files().await;
            files.sort_by_key(|m| m.uploaded_at);
            let matches: Vec<_> = files
                .into_iter()
                .filter(|m| m.file_name.to_lowercase().contains(&q))
                .rev()
                .take(10)
                .collect();
            if matches.is_empty() {
                let _ = bot.send_message(chat_id, "No matches found").await;
                return;
            }
            let cfg = Config::instance().await;
            let domain = cfg.file_domain();
            let mut lines = Vec::new();
            for f in matches {
                let url_name = f.file_name.replace(' ', "_");
                lines.push(format!("- {} ({} bytes)\n{}{}_{}", f.file_name, f.file_size, domain, f.unique_id, url_name));
            }
            if lines.len() == 10 {
                lines.push("(showing first 10 results)".to_string());
            }
            let _ = bot.send_message(chat_id, lines.join("\n")).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bot::{Bot, TeloxideBot};
    use shared::chat_config::PermissionsConfig;
    use shared::config::Config;
    use std::env;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_teloxide_bot_new() {
        env::set_var("BOT_TOKEN", "test_token");

        let config = Arc::new(Config::new());
        let permissions = Arc::new(Mutex::new(PermissionsConfig::init_allow_all()));
        let queue = Arc::new(Mutex::new(Vec::new()));

        let bot = match TeloxideBot::new(config, permissions, queue) {
            Ok(b) => { b }
            Err(_) => {
                panic!("Failed to create bot");
            }
        };

        assert_eq!(bot.get_teloxide_bot().token(), "test_token");
        assert_eq!(bot.get_teloxide_bot().api_url().as_str(), "https://api.telegram.org/");

        env::remove_var("BOT_TOKEN")
    }
}