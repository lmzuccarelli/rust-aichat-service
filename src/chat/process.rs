use crate::chat::{client::ChatClient, model::CompletionRequest, model::InputMessage};
use crate::cli::schema::ApplicationConfig;
use custom_logger as log;
use std::fs;
use std::{
    io::{self, Write},
    sync::Arc,
};

#[allow(unused)]
pub struct ChatSession {
    client: Arc<dyn ChatClient>,
    messages: Vec<InputMessage>,
    config: ApplicationConfig,
}

impl ChatSession {
    pub fn new(client: Arc<dyn ChatClient>, config: ApplicationConfig) -> Self {
        Self {
            client,
            messages: Vec::new(),
            config,
        }
    }

    pub fn add_system_prompt(&mut self, prompt: impl ToString) {
        self.messages.push(InputMessage::system(prompt));
    }

    pub async fn chat(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("welcome!! input your question at the prompt");
        println!("");
        log::info!("menu :");
        log::info!("     : type 'read <filename>' to read a file");
        log::info!("     : type 'save <filename>' to save content to file");
        log::info!("     : type 'call <service>'  to execute a service");
        log::info!("     : type 'exit' to quit");
        println!("");
        let mut result = String::new();
        let mut filecontents = String::new();

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
                break;
            }

            // 0 should be role = system
            match self.messages.get(1) {
                Some(_) => {
                    if filecontents.len() > 0 {
                        self.messages[1].content = format!("{} {}", input.clone(), filecontents);
                    } else {
                        self.messages[1].content = input.clone();
                    }
                }
                None => self.messages.push(InputMessage::user(input.clone())),
            }

            let request = CompletionRequest {
                model: self.config.spec.model.clone(),
                messages: self.messages.clone(),
                top_p: self.config.spec.top_p,
                temperature: Some(self.config.spec.temperature),
                stream: self.config.spec.stream,
                max_tokens: self.config.spec.max_tokens,
            };

            if input.clone().contains("save") && result.len() > 0 {
                let res_file = input.split(" ").nth(1);
                match res_file {
                    Some(filename) => {
                        fs::write(filename, result).expect("should save file");
                        log::info!("succesfully saved {} to disk", filename);
                        result = String::new();
                    }
                    None => {
                        log::warn!("please specify a filname");
                    }
                }
                println!("");
                continue;
            }

            if input.clone().contains("read") {
                let res_file = input.split(" ").nth(1);
                match res_file {
                    Some(filename) => {
                        filecontents = fs::read_to_string(filename).expect("should read a file");
                        log::info!("succesfully read {} from disk", filename);
                    }
                    None => {
                        log::warn!("please specify a filname");
                    }
                }
                println!("");
                continue;
            }

            // send request
            result = self.client.complete(request).await?;
        }
        Ok(())
    }
}
