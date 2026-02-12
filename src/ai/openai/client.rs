use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    ai::{
        ai::{AiClient, AiErr, Request, Response},
        openai::types::{self, ResponseOutputItem, ResponseOutputText},
    },
    strings,
};

#[derive(Clone)]
pub(crate) struct OpenAIClient {
    pub(crate) key: String,
    pub(crate) http_client: reqwest::Client,
}

#[async_trait]
impl AiClient for OpenAIClient {
    async fn send(&self, content: Request) -> Result<Response, AiErr> {
        tracing::info!("prev id: {}", &content.input);
        let oc = OpenAiRequest {
            model: "gpt-5.2".to_string(),
            input: content.input,
            previous_response_id: content.prev_id,
        };

        let res = self
            .http_client
            .post(strings::OPENAI_URL)
            .json(&oc)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .map_err(|e| {
                tracing::error!("{}", e);
                AiErr::Custom(format!("{}", e))
            })?
            // .text()
            .json::<types::Response>()
            .await
            .map_err(|e| {
                tracing::error!("{}", e);
                AiErr::Custom(format!("{}", e))
            })?;

        tracing::debug!("response {:?}", res);

        let res_id = res.id.clone();
        let r = match &res.output[0] {
            types::ResponseOutputItem::ResponseOutputMessage { content, .. } => {
                match content.as_slice() {
                    [ResponseOutputText::OutputText { text, .. }, ..] => Response {
                        id: res_id,
                        content: text.clone(),
                    },
                    [ResponseOutputText::Refusal { refusal }, ..] => Response {
                        id: res_id,
                        content: refusal.clone(),
                    },
                    _ => Err(AiErr::Custom("failed to parse".to_string()))?,
                }
            }
            _ => Err(AiErr::Custom("Custom".to_string()))?,
        };
        Ok(r)
    }

    fn clone_box(&self) -> Box<dyn AiClient> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn AiClient> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Serialize, Deserialize)]
struct OpenAiRequest {
    model: String,
    input: String,
    previous_response_id: Option<String>,
}
