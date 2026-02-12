use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::{ai::openai::client::OpenAIClient, strings};

const SUPPORTED_MODELS: &[Model] = &[Model {
    vendor: "OpenAI",
    models: &["gpt-5.2", "gpt-5.2-mini"],
}];

#[async_trait]
pub trait AiClient: Send {
    async fn send(&self, content: Request) -> Result<Response, AiErr>;
    fn clone_box(&self) -> Box<dyn AiClient>;
}

pub struct ClientBuilder {
    key: Option<String>,
}

impl ClientBuilder {
    pub fn openai() -> Result<Box<dyn AiClient>, AiErr> {
        let key = dotenv::var(strings::OPENAI_KEY).map_err(|_| AiErr::MissingKey)?;
        let mut headers = HeaderMap::new();
        let mut auth =
            HeaderValue::from_str(&format!("Bearer {}", &key)).map_err(|_| AiErr::HeaderErr)?;
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);
        // headers.insert(
        //     "OpenAI-Organization",
        //     HeaderValue::from_str("Personal").map_err(|_| AiErr::HeaderErr)?,
        // );
        // headers.insert(
        //     "OpenAI-Project",
        //     HeaderValue::from_str("SG").map_err(|_| AiErr::HeaderErr)?,
        // );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| AiErr::ClientBuilderError)?;

        let client = OpenAIClient {
            key,
            http_client: client,
        };
        Ok(Box::new(client))
    }
}

#[derive(Debug)]
pub struct Request {
    pub model: String,
    pub input: String,
    pub format: ResponseFormat,
    pub tools: Vec<Tool>,
    pub prev_id: Option<String>,
}

#[derive(Debug)]
pub struct Tool {
    name: String,
    description: String,
}

impl Request {
    pub fn new(input: String) -> Self {
        Self {
            input,
            model: "gpt-5.2".to_string(),
            format: ResponseFormat::Text,
            tools: vec![],
            prev_id: None,
        }
    }

    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn with_prev_id(mut self, id: String) -> Self {
        self.prev_id = Some(id);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub id: String,
    pub content: String,
}

#[derive(Debug)]
pub enum ResponseFormat {
    Text,
    Json,
}

pub enum AiVendor {
    Openai,
}

#[derive(Debug)]
pub enum AiErr {
    MissingKey,
    ClientBuilderError,
    HeaderErr,
    Deserr,
    Custom(String),
}

pub struct Model {
    vendor: &'static str,
    models: &'static [&'static str],
}
