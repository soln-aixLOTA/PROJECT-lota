use vaultrs::api::secrets::kv::v2::requests::ReadSecretVersionRequest;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::sys::auth;

async fn get_nvidia_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let client = VaultClient::new(VaultClientSettingsBuilder::default().address("<vault_address>").token("<your_approle_token>").build()?)?;

    // ... authenticate if needed ...

    let response = client.read_secret_version(ReadSecretVersionRequest {
        mount_point: "secret",
        path: "nvidia-api-key",
        ..Default::default()
    }).await?;

    let api_key = response.data.data.get("value").ok_or("API key not found")?.to_string();
    Ok(api_key)
} 