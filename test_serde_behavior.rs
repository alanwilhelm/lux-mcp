use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ChatRequest {
    message: String,
    #[serde(default)]
    model: Option<String>,
    // Note: max_tokens field is NOT defined here
}

fn main() {
    // JSON with extra field that doesn't exist in struct
    let json_with_extra = r#"{
        "message": "Hello",
        "model": "gpt-5",
        "max_tokens": 2000,
        "another_unknown_field": "ignored"
    }"#;
    
    // This will work - unknown fields are ignored
    match serde_json::from_str::<ChatRequest>(json_with_extra) {
        Ok(request) => {
            println!("✅ Deserialization successful!");
            println!("Message: {}", request.message);
            println!("Model: {:?}", request.model);
            println!("Note: max_tokens and another_unknown_field were silently ignored");
        }
        Err(e) => {
            println!("❌ Deserialization failed: {}", e);
        }
    }
}