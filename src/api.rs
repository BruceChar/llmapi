use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, error::Error};
use tokio_stream::StreamExt;


struct Api {
    base_url: String,
    routes: HashMap<String, Route>,
}

struct Route {
    path: String,
    method: Option<String>,
    body: Option<Value>,
    auth: Option<Auth>,
    next: Option<Box<Route>>,
}

enum Auth {
    Bearer(String),
    Basic(String, String),
    None,
}

struct Header {
    content_type: HeaderContentType,
    authorization: String,
}

enum HeaderContentType {
    ApplicationOctetStream,
    ApplicationJson,
    ApplicationFormUrlEncoded,
    ApplicationMultipartFormData,
}

#[async_trait]
pub trait LLMApi {
    async fn request(&self, payload: &Value, stream: bool) -> Result<Response, Box<dyn Error>>;
}

pub struct ApiClient {
    api_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self { api_url, api_key }
    }
}

pub enum Response {
    Text(String),
    Stream(Box<dyn Stream<Item = Result<String, Box<dyn Error>>> + Unpin + Send>),
}

#[async_trait]
impl LLMApi for ApiClient {
    async fn request(&self, payload: &Value, stream: bool) -> Result<Response, Box<dyn Error>> {
        let client = Client::new();
        let mut request = client.post(&self.api_url)
            .header("Content-Type", "application/json")
            .json(payload);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await?;

        if stream {
            let stream = response.bytes_stream().map(|item| {
                item.map_err(|e| Box::new(e) as Box<dyn Error>)
                    .and_then(|bytes| {
                        String::from_utf8(bytes.to_vec())
                            .map_err(|e| Box::new(e) as Box<dyn Error>)
                    })
            });
            Ok(Response::Stream(Box::new(stream)))
        } else {
            let text = response.text().await?;
            Ok(Response::Text(text))
        }
    }
}