import os
import hvac

client = hvac.Client(url=os.environ['VAULT_ADDR'], token=os.environ['VAULT_TOKEN'])

if not client.is_authenticated():
    raise Exception("Failed to authenticate with Vault")

secret_path = 'secret/data/attestation-service-config'
read_response = client.secrets.kv.v2.read_secret_version(
    path=secret_path,
    mount_point='secret',
)

if read_response and 'data' in read_response and 'data' in read_response['data']:
    config_data = read_response['data']['data']
    api_key = config_data.get('api_key')
    if not api_key:
        raise ValueError(f"API key not found in Vault at {secret_path}")
else:
    raise ValueError(f"Could not retrieve secret from Vault at {secret_path}")

print(f"Successfully retrieved API key from Vault.") 