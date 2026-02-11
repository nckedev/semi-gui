use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub status: String,
    pub background: bool,
    pub billing: Billing,
    pub completed_at: Option<i64>,
    pub error: Option<Error>,
    pub frequency_penalty: f64,
    pub incomplete_details: Option<String>,
    pub instructions: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
    pub parallel_tool_calls: bool,
    pub presence_penalty: f64,
    pub previous_response_id: Option<String>,
    pub prompt_cache_key: Option<String>,
    pub prompt_cache_retention: Option<String>,
    pub reasoning: Reasoning,
    pub safety_identifier: Option<String>,
    pub service_tier: String,
    pub store: bool,
    pub temperature: f64,
    pub text: TextConfig,
    pub tool_choice: String,
    pub tools: Vec<String>,
    pub top_logprobs: u32,
    pub top_p: f64,
    pub truncation: String,
    pub usage: Usage,
    pub user: Option<String>,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Billing {
    pub payer: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ResponseOutputItem {
//     pub id: Option<String>,
//     #[serde(rename = "type")]
//     pub item_type: String,
//     pub status: String,
//     pub content: Vec<ContentItem>,
//     pub role: String,
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseOutputItem {
    #[serde(rename = "message")]
    ResponseOutputMessage {
        id: String,
        content: Vec<ResponseOutputText>,
        role: String,
        status: String,
    },
    #[serde(rename = "file_search_call")]
    ResponseFileSearchToolCall {},
    #[serde(rename = "function_call")]
    ResponseFunctionToolCall {},
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ContentItem {
//     #[serde(rename = "type")]
//     pub content_type: String,
//     pub annotations: Vec<serde_json::Value>,
//     pub logprobs: Vec<serde_json::Value>,
//     pub text: String,
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseOutputText {
    #[serde(rename = "output_text")]
    OutputText {
        #[serde(skip)]
        annotations: Vec<serde_json::Value>,
        #[serde(skip)]
        logprobs: Vec<serde_json::Value>,
        text: String,
    },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reasoning {
    pub effort: String,
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextConfig {
    pub format: TextFormat,
    pub verbosity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub input_tokens_details: InputTokenDetails,
    pub output_tokens: u32,
    pub output_tokens_details: OutputTokenDetails,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputTokenDetails {
    pub cached_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputTokenDetails {
    pub reasoning_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    message: String,
    #[serde(rename = "type")]
    err_type: String,
    code: String,
    param: Option<String>,
}
