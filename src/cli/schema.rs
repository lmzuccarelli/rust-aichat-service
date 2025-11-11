use clap::Parser;
use serde_derive::{Deserialize, Serialize};

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A simple chat service (aimed an openai schema)", long_about = None)]
#[command(
    help_template = "{author-with-newline} {about-section}Version: {version} \n {usage-heading} {usage} \n {all-args} {tab}"
)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: String,

    /// set the loglevel. Valid arguments are info, debug, trace
    #[arg(value_enum, long, value_name = "loglevel", default_value = "info")]
    pub loglevel: String,

    /// use speech-to-text service for prompting
    #[arg(long, value_name = "stt", default_value_t = false)]
    pub stt: bool,
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
    #[serde(rename = "openai_key_path")]
    pub openai_key_path: String,
    #[serde(rename = "deepgram_key_path")]
    pub deepgram_key_path: String,
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
    #[serde(rename = "working_dir")]
    pub working_dir: String,
    #[serde(rename = "folders")]
    pub folders: Vec<String>,
    #[serde(rename = "system_prompt")]
    pub system_prompt: String,
}
