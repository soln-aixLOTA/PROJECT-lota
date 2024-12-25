use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const AZURE_TRANSLATOR_ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com";

#[derive(Debug, Serialize)]
struct TranslationRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct TranslationResponse {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    text: String,
    to: String,
}

/// Translates text to a specific language using Azure Translator API
///
/// # Arguments
///
/// * `text` - The text to translate
/// * `target_language` - The target language code (e.g., "es" for Spanish)
///
/// # Returns
///
/// Returns the translated text as a String, or the original text if translation fails
pub async fn translate_text(text: &str, target_language: &str) -> Result<String, Box<dyn Error>> {
    let api_key = env::var("AZURE_TRANSLATOR_KEY")
        .map_err(|_| "AZURE_TRANSLATOR_KEY environment variable not set")?;
    let region = env::var("AZURE_TRANSLATOR_REGION")
        .map_err(|_| "AZURE_TRANSLATOR_REGION environment variable not set")?;

    let client = Client::new();
    let url = format!(
        "{}/translate?api-version=3.0&to={}",
        AZURE_TRANSLATOR_ENDPOINT, target_language
    );

    let request = vec![TranslationRequest {
        text: text.to_string(),
    }];

    let response = client
        .post(&url)
        .header("Ocp-Apim-Subscription-Key", &api_key)
        .header("Ocp-Apim-Subscription-Region", &region)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(text.to_string()); // Return original text on error
    }

    let translation_response: Vec<TranslationResponse> = response.json().await?;
    Ok(translation_response
        .first()
        .and_then(|resp| resp.translations.first())
        .map(|translation| translation.text.clone())
        .unwrap_or_else(|| text.to_string()))
}

// Synchronous wrapper for compatibility with existing code
pub fn translate_text_sync(text: &str, language: &str) -> String {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime
        .block_on(translate_text(text, language))
        .unwrap_or_else(|_| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_translate_text_success() {
        let mock_server = MockServer::start().await;

        let mock_response = r#"[{
            "translations": [{
                "text": "¡Hola mundo!",
                "to": "es"
            }]
        }]"#;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(header("Ocp-Apim-Subscription-Key", "test_key"))
            .and(header("Ocp-Apim-Subscription-Region", "test_region"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_response))
            .mount(&mock_server)
            .await;

        env::set_var("AZURE_TRANSLATOR_KEY", "test_key");
        env::set_var("AZURE_TRANSLATOR_REGION", "test_region");

        let result = translate_text("Hello world", "es").await.unwrap();
        assert_eq!(result, "¡Hola mundo!");
    }

    #[tokio::test]
    async fn test_translate_text_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        env::set_var("AZURE_TRANSLATOR_KEY", "test_key");
        env::set_var("AZURE_TRANSLATOR_REGION", "test_region");

        let result = translate_text("Hello world", "es").await.unwrap();
        // Should return original text on error
        assert_eq!(result, "Hello world");
    }

    #[tokio::test]
    async fn test_translate_text_missing_env_vars() {
        env::remove_var("AZURE_TRANSLATOR_KEY");
        env::remove_var("AZURE_TRANSLATOR_REGION");

        let result = translate_text("Hello world", "es").await;
        assert!(result.is_err());
    }
}
