use crate::chat::model::{ChatResponse, CompletionRequest};
use async_trait::async_trait;
use custom_logger as log;
use http::StatusCode;
use std::time::Duration;

#[async_trait]
pub trait ChatClient: Send + Sync {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

pub struct OpenAIClient {
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, url: String) -> Self {
        Self {
            api_key,
            base_url: url,
        }
    }
}

#[allow(unused_macros)]
macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            let mut h = stdout();
            write!(h, $($t)* ).unwrap();
            h.flush().unwrap();
        }
    }
}

#[async_trait]
impl ChatClient for OpenAIClient {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&request).unwrap();
        log::debug!("url {}", self.base_url);
        let client_res = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .http1_title_case_headers()
            .timeout(Duration::new(1200, 0))
            .build();

        let client = match client_res {
            Ok(client) => {
                log::debug!("[complete] llm openapi client created");
                client
            }
            Err(e) => {
                return Err(Box::from(format!("[complete] llm openapi {} ", e)));
            }
        };

        log::debug!("payload {}", json);

        let response = client
            .post(self.base_url.clone())
            .bearer_auth(self.api_key.trim())
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await;

        let result = match response {
            Ok(result) => {
                let status = result.status();
                log::debug!("[complete] llm openapi response status {}", status);
                match status {
                    StatusCode::OK => {
                        let contents = result.bytes().await?;
                        log::trace!(
                            "[complete] llm openapi client response {}",
                            String::from_utf8(contents.to_vec()).unwrap()
                        );
                        let chat_response: ChatResponse = serde_json::from_slice(&contents)?;
                        chat_response.choices[0].message.content.clone()
                    }
                    _ => {
                        let contents = result.bytes().await?;
                        return Err(Box::from(format!(
                            "[complete] llm openapi {}",
                            String::from_utf8(contents.to_vec())
                                .unwrap_or("could not parse error".to_string())
                        )));
                    }
                }
            }
            Err(e) => {
                return Err(Box::from(format!("[complete] llm openapi error {}", e)));
            }
        };

        //let res = response;
        //log::debug!("[complete] response {:?}", response.status());
        //let content: ChatResponse = serde_json::from_slice(&response.bytes().await?)?;
        //let result = content.choices[0].message.content.clone();
        // preserve origin content (ie no log decorations)
        println!("{}", result);
        Ok(result)
    }
}
