use reqwest::Client;
use tokio_stream::StreamExt;
use serde_json::Value;
use std::error::Error;

async fn stream_llm_request(api_url: &str, payload: &Value) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let response = client.post(api_url)
        .header("Content-Type", "application/json")
        .json(payload)
        .send()
        .await?;

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8(bytes.to_vec())?;
                println!("Received chunk: {}", chunk);
            }
            Err(e) => {
                eprintln!("Error while streaming: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_stream_llm_request() -> Result<(), Box<dyn Error>> {
        let payload = serde_json::json!({"messages": [{"role": "user", "content": "Hello, world!"}], "stream": true});
        stream_llm_request("http://127.0.0.1:8080/v1/chat/completions", &payload).await
    }
}