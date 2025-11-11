use crate::chat::process::ChatSession;
use crate::cli::schema::ApplicationConfig;
use crate::stt::process::execute;
use clap::Parser;
use custom_logger as log;
use std::{fs, str::FromStr};

mod chat;
mod cli;
mod prompt;
mod service;
mod stt;

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
        .map_err(|_| format!("[main] invalid log level: {}", args.loglevel))?;
    log::Logging::new().with_level(log_level).init()?;

    // Read config
    let config_data = fs::read_to_string(&args.config)
        .map_err(|e| format!("[main] failed to read config file '{}': {}", args.config, e))?;
    let cfg: ApplicationConfig = serde_json::from_str(&config_data)
        .map_err(|e| format!("[main] invalid JSON in config file: {}", e))?;

    // Validate config
    if cfg.spec.api_url.is_empty() {
        return Err("[main] api_url cannot be empty".into());
    }
    if cfg.spec.model.is_empty() {
        return Err("[main] model cannot be empty".into());
    }

    log::info!("[main] application : {}", env!("CARGO_PKG_NAME"));
    log::info!("[main] author      : {}", env!("CARGO_PKG_AUTHORS"));
    log::info!("[main] version     : {}", env!("CARGO_PKG_VERSION"));

    // clean up all staging entries
    fs::remove_dir_all(format!("{}/staging", cfg.spec.working_dir))?;
    fs::create_dir_all(format!("{}/staging", cfg.spec.working_dir))?;

    if args.stt {
        let _res = execute(cfg).await;
    } else {
        log::debug!("[main] using model: {}", cfg.spec.model);
        log::trace!("[main] connecting to API: {}", cfg.spec.api_url);

        let mut session = ChatSession::new(cfg);

        // Run chat
        if let Err(e) = session.chat().await {
            log::error!("[main] chat session error: {}", e);
            return Err(Box::from(e.to_string()));
        }
    }
    Ok(())
}
