use crate::chat::client::OpenAIClient;
use crate::cli::schema::ApplicationConfig;
use crate::prompt::parser::PromptParser;
use crate::service::execute::{Execute, ExecuteInterface};
use custom_logger as log;
use std::fs;
use std::io::{self, Write};
use std::sync::Arc;

#[allow(unused)]
pub struct ChatSession {
    config: ApplicationConfig,
}

impl ChatSession {
    pub fn new(config: ApplicationConfig) -> Self {
        Self { config }
    }

    pub async fn chat(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("[chat] welcome!! input your question at the prompt");
        println!();
        log::info!("menu :");
        log::info!("     : type 'execute <service>'  to execute a service");
        log::info!("     : type 'show current' to console print current session content");
        log::info!("     : type 'exit' to quit");
        println!();

        // Read and trim API key
        let api_key = fs::read_to_string(self.config.spec.openai_key_path.clone())
            .map_err(|e| format!("[chat] failed to read API key file : {}", e))?
            .trim()
            .to_string();

        let client = Arc::new(OpenAIClient::new(api_key, self.config.spec.api_url.clone()));
        let mut ep = Execute::new(client, self.config.clone());

        loop {
            print!("prompt> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();

            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                log::info!("[chat] exiting session");
                break;
            }

            let parsed_command = PromptParser::parse(self.config.spec.working_dir.clone(), input)?;
            let res = ep.process_task(parsed_command).await;
            // we don't want to crash so lets handle the error
            match res {
                Ok(_data) => {}
                Err(err) => {
                    log::error!("[chat] {}", err.to_string());
                }
            }
            println!();
        }
        Ok(())
    }
}
