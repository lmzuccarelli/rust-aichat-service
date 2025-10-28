use clap::Parser;
use serde_derive::{Deserialize, Serialize};

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = "rust-aichat-service")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.1.0")]
#[command(about = "A simple chat service (aimed an openai schema)", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: String,

    /// set the loglevel. Valid arguments are info, debug, trace
    #[arg(value_enum, long, value_name = "loglevel", default_value = "info")]
    pub loglevel: String,

    /// set the system prompt
    #[arg(
        value_enum,
        long,
        value_name = "system-prompt",
        default_value = "You are a helpful assistant. Use the context to help the user."
    )]
    pub system_prompt: String,
}

/// Application configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationConfig {
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec {
    #[serde(rename = "api_key_path")]
    pub api_key_path: String,
    #[serde(rename = "api_url")]
    pub api_url: String,
    #[serde(rename = "api_port")]
    pub api_port: i32,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "temperature")]
    pub temperature: f32,
    #[serde(rename = "top_p")]
    pub top_p: f32,
    #[serde(rename = "top_k")]
    pub top_k: usize,
    #[serde(rename = "max_tokens")]
    pub max_tokens: usize,
    #[serde(rename = "stream")]
    pub stream: bool,
    #[serde(rename = "n_keep")]
    pub n_keep: usize,
    #[serde(rename = "n_predict")]
    pub n_predict: usize,
    #[serde(rename = "cache_prompt")]
    pub cache_prompt: bool,
}
