use crate::chat::model::{ChatResponse, CompletionRequest};
use async_trait::async_trait;
use custom_logger as log;

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
        log::debug!("key {}", self.api_key);
        let client = reqwest::Client::builder()
            .http1_title_case_headers()
            .build()?;
        log::debug!("payload {}", json);

        let response = client
            .post(&self.base_url)
            .bearer_auth(self.api_key.to_owned().trim())
            .body(json)
            .send()
            .await;

        let res = response.unwrap();
        log::debug!("response {:?}", res.status());
        let content: ChatResponse = serde_json::from_slice(&res.bytes().await.unwrap())?;
        let result = content.choices[0].message.content.clone();
        // preserve origin content (ie no log decorations)
        println!("{}", result.clone());
        Ok(result.clone())
    }
}
