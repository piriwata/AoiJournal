use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        OllamaConfig {
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            timeout_secs: 60,
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedPosting {
    pub account: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedTransaction {
    pub date: String,
    pub narration: String,
    pub postings: Vec<ProposedPosting>,
    pub confidence: Option<f32>,
    pub notes: Option<String>,
}

pub async fn propose_transaction(
    config: &OllamaConfig,
    natural_language: &str,
    available_accounts: &[String],
) -> Result<Vec<ProposedTransaction>, String> {
    let accounts_list = available_accounts.join(", ");
    let prompt = format!(
        r#"You are a Japanese bookkeeping assistant. Convert the following natural language description into double-entry bookkeeping journal entries.

Available accounts: {accounts_list}

Rules:
- All amounts are in JPY (Japanese Yen, integers only)
- Debits are positive amounts, credits are negative amounts
- The sum of all postings in a transaction MUST equal zero
- Use the exact account names from the available accounts list
- Date format: YYYY-MM-DD
- Return JSON only, no explanation

Natural language input: {natural_language}

Return a JSON array of transactions:
[
  {{
    "date": "YYYY-MM-DD",
    "narration": "description",
    "postings": [
      {{"account": "account_name", "amount": 12345}},
      {{"account": "account_name", "amount": -12345}}
    ],
    "notes": "optional explanation"
  }}
]"#,
        accounts_list = accounts_list,
        natural_language = natural_language
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;

    let request = OllamaRequest {
        model: config.model.clone(),
        prompt,
        stream: false,
        format: None,
    };

    let url = format!("{}/api/generate", config.base_url);
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama connection error: {}", e))?;

    let ollama_resp: OllamaResponse = response
        .json()
        .await
        .map_err(|e| format!("Ollama response parse error: {}", e))?;

    // Parse JSON from the response
    let json_str = extract_json(&ollama_resp.response);
    let proposals: Vec<ProposedTransaction> = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse Ollama JSON response: {}", e))?;

    Ok(proposals)
}

fn extract_json(text: &str) -> String {
    // Try to extract JSON array from the text
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if start < end {
                return text[start..=end].to_string();
            }
        }
    }
    text.to_string()
}
