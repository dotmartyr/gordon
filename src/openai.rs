use anyhow::{anyhow, Result};
use reqwest::Client as ReqwestClient;
use serde::Serialize;
use serde_json::{json, Value};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_MODEL: &str = "gpt-4o-mini";

pub struct OpenAIClient {
    client: ReqwestClient,
    api_key: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIMessage {
    role: String,
    content: String,
}

impl OpenAIMessage {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: ReqwestClient::new(),
            api_key,
        }
    }

    pub async fn ask(&self, messages: Vec<OpenAIMessage>) -> Result<String> {
        let request_body = self.create_request_body(&messages);
        let response = self.send_request(&request_body).await?;
        self.parse_response(response).await
    }

    fn create_request_body(&self, messages: &[OpenAIMessage]) -> Value {
        json!({
            "model": OPENAI_MODEL,
            "messages": messages,
            "max_tokens": 4096,
            "temperature": 0.7,
            "top_p": 1.0,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.0
        })
    }

    async fn send_request(&self, request_body: &Value) -> Result<reqwest::Response> {
        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(request_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(anyhow!(
                "Failed to get response from OpenAI API: {}",
                response.status()
            ))
        }
    }

    async fn parse_response(&self, response: reqwest::Response) -> Result<String> {
        let response_body: Value = response.json().await?;

        let reply = response_body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("I'm sorry, I couldn't understand that.")
            .to_string();
        Ok(reply)
    }
}
