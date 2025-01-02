import hvac

# ...

def get_db_password():
    client = hvac.Client(url='<vault_address>', token='<your_approle_token>') # Or use approle_id and secret_id
    
    # Check if client is authenticated
    if not client.is_authenticated():
        raise Exception("Vault client not authenticated")

    try:
        read_response = client.secrets.kv.v2.read_secret_version(path='database-credentials', mount_point='secret')
        db_password = read_response['data']['data']['password']
        return db_password
    except Exception as e:
        print(f"Error reading secret: {e}")
        raise

# ... use the retrieved password in your database connection ... 