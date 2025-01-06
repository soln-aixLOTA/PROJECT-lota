# Document Automation Service Usage Examples

This guide provides examples of how to interact with the Document Automation Service API using various programming languages and tools.

## Authentication

All requests (except health check) require a JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

## Examples

### 1. Upload Document

#### cURL

```bash
# Upload a document
curl -X POST http://localhost:8080/documents \
  -H "Authorization: Bearer <your_jwt_token>" \
  -F "file=@/path/to/document.pdf"
```

#### Python

```python
import requests

def upload_document(file_path, token):
    url = "http://localhost:8080/documents"
    headers = {
        "Authorization": f"Bearer {token}"
    }
    with open(file_path, "rb") as f:
        files = {"file": f}
        response = requests.post(url, headers=headers, files=files)

    if response.status_code == 201:
        return response.json()
    else:
        raise Exception(f"Upload failed: {response.text}")

# Usage
document = upload_document("document.pdf", "your_jwt_token")
print(f"Document ID: {document['id']}")
```

#### TypeScript/Node.js

```typescript
import axios from "axios";
import FormData from "form-data";
import fs from "fs";

async function uploadDocument(filePath: string, token: string) {
  const url = "http://localhost:8080/documents";
  const formData = new FormData();
  formData.append("file", fs.createReadStream(filePath));

  const response = await axios.post(url, formData, {
    headers: {
      ...formData.getHeaders(),
      Authorization: `Bearer ${token}`,
    },
  });

  return response.data;
}

// Usage
try {
  const document = await uploadDocument("document.pdf", "your_jwt_token");
  console.log("Document ID:", document.id);
} catch (error) {
  console.error("Upload failed:", error.response?.data || error.message);
}
```

#### Rust

```rust
use reqwest::multipart;
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug, Deserialize)]
struct Document {
    id: String,
    name: String,
    status: String,
    storage_path: String,
}

async fn upload_document(file_path: &str, token: &str) -> Result<Document, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let file = File::open(file_path).await?;
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_part = multipart::Part::stream(stream)
        .file_name("document.pdf")
        .mime_str("application/pdf")?;
    let form = multipart::Form::new().part("file", file_part);

    let response = client
        .post("http://localhost:8080/documents")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let document = response.json::<Document>().await?;
        Ok(document)
    } else {
        Err(format!("Upload failed: {}", response.text().await?).into())
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = upload_document("document.pdf", "your_jwt_token").await?;
    println!("Document ID: {}", document.id);
    Ok(())
}
```

### 2. List Documents

#### cURL

```bash
# List documents with pagination
curl http://localhost:8080/documents?limit=10&offset=0 \
  -H "Authorization: Bearer <your_jwt_token>"
```

#### Python

```python
import requests

def list_documents(token, limit=10, offset=0):
    url = "http://localhost:8080/documents"
    headers = {
        "Authorization": f"Bearer {token}"
    }
    params = {
        "limit": limit,
        "offset": offset
    }
    response = requests.get(url, headers=headers, params=params)

    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"List failed: {response.text}")

# Usage
documents = list_documents("your_jwt_token", limit=5)
for doc in documents:
    print(f"Document: {doc['name']} ({doc['id']})")
```

### 3. Get Document

#### cURL

```bash
# Download a document
curl http://localhost:8080/documents/{document_id} \
  -H "Authorization: Bearer <your_jwt_token>" \
  --output document.pdf
```

#### Python

```python
import requests

def get_document(document_id, token):
    url = f"http://localhost:8080/documents/{document_id}"
    headers = {
        "Authorization": f"Bearer {token}"
    }
    response = requests.get(url, headers=headers)

    if response.status_code == 200:
        return response.content
    else:
        raise Exception(f"Download failed: {response.text}")

# Usage
content = get_document("document_id", "your_jwt_token")
with open("downloaded_document.pdf", "wb") as f:
    f.write(content)
```

### 4. Delete Document

#### cURL

```bash
# Delete a document
curl -X DELETE http://localhost:8080/documents/{document_id} \
  -H "Authorization: Bearer <your_jwt_token>"
```

#### Python

```python
import requests

def delete_document(document_id, token):
    url = f"http://localhost:8080/documents/{document_id}"
    headers = {
        "Authorization": f"Bearer {token}"
    }
    response = requests.delete(url, headers=headers)

    if response.status_code == 204:
        return True
    else:
        raise Exception(f"Delete failed: {response.text}")

# Usage
success = delete_document("document_id", "your_jwt_token")
print("Document deleted successfully" if success else "Delete failed")
```

## Error Handling

The API returns structured error responses:

```json
{
  "error": "Document not found",
  "code": "NOT_FOUND"
}
```

Example error handling in Python:

```python
import requests

class DocumentServiceError(Exception):
    def __init__(self, message, code=None):
        self.message = message
        self.code = code
        super().__init__(self.message)

def handle_request(response):
    if response.status_code >= 400:
        error = response.json()
        raise DocumentServiceError(
            error.get("error", "Unknown error"),
            error.get("code")
        )
    return response.json()

# Usage
try:
    response = requests.get(url, headers=headers)
    result = handle_request(response)
except DocumentServiceError as e:
    print(f"Error ({e.code}): {e.message}")
```

## Best Practices

1. **Token Management**

   - Store tokens securely
   - Refresh tokens before expiration
   - Never expose tokens in client-side code

2. **Error Handling**

   - Implement proper error handling
   - Retry on transient failures
   - Log errors for debugging

3. **Resource Cleanup**

   - Close file handles
   - Clean up temporary files
   - Handle connection cleanup

4. **Performance**
   - Use connection pooling
   - Implement request timeouts
   - Consider batch operations

## Rate Limiting

The service implements rate limiting. Handle 429 responses appropriately:

```python
import time
import requests
from requests.exceptions import HTTPError

def make_request_with_retry(url, headers, max_retries=3, initial_wait=1):
    for attempt in range(max_retries):
        try:
            response = requests.get(url, headers=headers)
            response.raise_for_status()
            return response.json()
        except HTTPError as e:
            if e.response.status_code == 429:  # Too Many Requests
                if attempt < max_retries - 1:
                    wait_time = initial_wait * (2 ** attempt)  # Exponential backoff
                    time.sleep(wait_time)
                    continue
            raise
```
