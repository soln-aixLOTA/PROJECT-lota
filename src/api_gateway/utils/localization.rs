use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

const GOOGLE_TRANSLATE_ENDPOINT: &str = "https://translation.googleapis.com/v3/projects";

#[derive(Debug, Serialize)]
struct TranslationRequest {
    contents: Vec<String>,
    target_language_code: String,
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct TranslationResponse {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    translated_text: String,
}

/// Translates text to a specific language using Google Cloud Translation API
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
    let project_id = env::var("GOOGLE_CLOUD_PROJECT")
        .map_err(|_| "GOOGLE_CLOUD_PROJECT environment variable not set")?;

    let client = Client::new();
    let url = format!(
        "{}/{}/locations/global:translateText",
        GOOGLE_TRANSLATE_ENDPOINT, project_id
    );

    let request = TranslationRequest {
        contents: vec![text.to_string()],
        target_language_code: target_language.to_string(),
        mime_type: "text/plain".to_string(),
    };

    let response = client
        .post(&url)
        .bearer_auth(env::var("GOOGLE_APPLICATION_CREDENTIALS")?)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(text.to_string()); // Return original text on error
    }

    let translation_response: TranslationResponse = response.json().await?;
    Ok(translation_response
        .translations
        .first()
        .map(|translation| translation.translated_text.clone())
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

        let mock_response = r#"{
            "translations": [{
                "translated_text": "¡Hola mundo!"
            }]
        }"#;

        Mock::given(method("POST"))
            .and(path("/projects/test-project/locations/global:translateText"))
            .and(header("Authorization", "Bearer test_creds"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_response))
            .mount(&mock_server)
            .await;

        env::set_var("GOOGLE_CLOUD_PROJECT", "test-project");
        env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "test_creds");

        let result = translate_text("Hello world", "es").await.unwrap();
        assert_eq!(result, "¡Hola mundo!");
    }

    #[tokio::test]
    async fn test_translate_text_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/projects/test-project/locations/global:translateText"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        env::set_var("GOOGLE_CLOUD_PROJECT", "test-project");
        env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "test_creds");

        let result = translate_text("Hello world", "es").await.unwrap();
        // Should return original text on error
        assert_eq!(result, "Hello world");
    }

    #[tokio::test]
    async fn test_translate_text_missing_env_vars() {
        env::remove_var("GOOGLE_CLOUD_PROJECT");
        env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");

        let result = translate_text("Hello world", "es").await;
        assert!(result.is_err());
    }
}
