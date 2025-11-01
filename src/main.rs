use crate::chat::client::OpenAIClient;
use crate::chat::process::ChatSession;
use clap::Parser;
use custom_logger as log;
use std::sync::Arc;
use std::{fs, str::FromStr};

mod chat;
mod cli;
mod service;

// local modules
use cli::schema::*;

// example curl call
/*
    curl -v --location 'https://api.cerebras.ai/v1/chat/completions' --header 'Content-Type: application/json' --header "Authorization: Bearer xxx" --data '{
        "model": "qwen-3-235b-a22b-instruct-2507",
        "stream": true,
        "max_tokens": 20000,
        "temperature": 0.2,
        "top_p": 0.8,
        "messages": [
            {
                "role": "user",
                "content": "what is a qubit?"
            }
        ]
    }'
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();

    // Setup logging
    let log_level = log::LevelFilter::from_str(&args.loglevel)
        .map_err(|_| format!("Invalid log level: {}", args.loglevel))?;
    log::Logging::new().with_level(log_level).init()?;

    // Read config
    let config_data = fs::read_to_string(&args.config)
        .map_err(|e| format!("Failed to read config file '{}': {}", args.config, e))?;
    let cfg: ApplicationConfig = serde_json::from_str(&config_data)
        .map_err(|e| format!("Invalid JSON in config file: {}", e))?;

    // Validate config
    if cfg.spec.api_url.is_empty() {
        return Err("api_url cannot be empty".into());
    }
    if cfg.spec.model.is_empty() {
        return Err("model cannot be empty".into());
    }

    // Read and trim API key
    let api_key = fs::read_to_string(&cfg.spec.api_key_path)
        .map_err(|e| {
            format!(
                "Failed to read API key file '{}': {}",
                cfg.spec.api_key_path, e
            )
        })?
        .trim()
        .to_string();

    log::info!("application : {}", env!("CARGO_PKG_NAME"));
    log::info!("author      : {}", env!("CARGO_PKG_AUTHORS"));
    log::info!("version     : {}", env!("CARGO_PKG_VERSION"));

    log::debug!("Using model: {}", cfg.spec.model);
    log::debug!("Connecting to API: {}", cfg.spec.api_url);

    let client = Arc::new(OpenAIClient::new(api_key, cfg.spec.api_url.clone()));
    let mut session = ChatSession::new(client, cfg);

    // Use args system prompt or fallback
    session.add_system_prompt(args.system_prompt);

    // Run chat
    if let Err(e) = session.chat().await {
        log::error!("Chat session error: {}", e);
        return Err(Box::from(e.to_string()));
    }

    Ok(())
}
