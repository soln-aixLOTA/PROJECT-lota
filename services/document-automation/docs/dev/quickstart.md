# Quick Start Guide

This guide will help you get started with the Document Automation Service in a development environment.

## Prerequisites

- Rust 1.70+ (`rustup update stable`)
- Docker and Docker Compose
- PostgreSQL client tools
- Git

## 1. Clone and Setup

```bash
# Clone the repository
git clone https://github.com/your-org/document-automation.git
cd document-automation

# Create development configuration
cp config/default.example.toml config/default.toml
```

Edit `config/default.toml` with your development settings:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgresql://postgres:postgres@localhost:5432/docautomation"

[storage]
provider = "local"  # Use local storage for development
path = "./data/documents"

[security]
jwt_secret = "dev-secret-change-me-in-production"
token_expiration_hours = 24
```

## 2. Development Environment

Start the development database:

```bash
# Start PostgreSQL container
docker compose up -d db

# Run database migrations
cargo sqlx migrate run
```

## 3. Build and Run

```bash
# Build the service
cargo build

# Run in development mode
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

## 4. Hello World Example

### 1. Create an Authentication Token

```bash
# Generate a development JWT token
cargo run --example generate_token

# Or use the provided helper script
./scripts/dev-token.sh
```

Save the token for use in subsequent requests:

```bash
export AUTH_TOKEN="your.jwt.token"
```

### 2. Upload a Document

```bash
# Create a sample document
echo "Hello, Document Automation!" > hello.txt

# Upload using curl
curl -X POST http://localhost:8080/documents \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -F "file=@hello.txt"
```

Expected response:

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "hello.txt",
  "content_type": "text/plain",
  "size": 28,
  "status": "completed",
  "metadata": {
    "author": "dev-user",
    "tags": [],
    "classification": null,
    "security_level": "internal"
  }
}
```

### 3. List Documents

```bash
# List uploaded documents
curl http://localhost:8080/documents \
  -H "Authorization: Bearer $AUTH_TOKEN"
```

### 4. Retrieve Document

```bash
# Download the document
curl http://localhost:8080/documents/123e4567-e89b-12d3-a456-426614174000 \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  --output downloaded.txt

# Verify content
cat downloaded.txt
```

## 5. Development Tools

### Code Generation

Generate OpenAPI client:

```bash
# Install openapi-generator
cargo install openapi-generator-cli

# Generate TypeScript client
openapi-generator generate -i docs/api/openapi.yaml -g typescript-fetch -o clients/typescript
```

### Testing

```bash
# Run unit tests
cargo test

# Run specific test
cargo test upload_document

# Run with logging
RUST_LOG=debug cargo test
```

### Database Operations

```bash
# Connect to development database
psql postgresql://postgres:postgres@localhost:5432/docautomation

# View document table
SELECT * FROM documents;

# Reset development database
cargo sqlx database reset
```

## 6. Example Application

Here's a complete example using the TypeScript client:

```typescript
import { Configuration, DocumentsApi } from "./clients/typescript";

async function main() {
  // Initialize client
  const config = new Configuration({
    basePath: "http://localhost:8080",
    headers: {
      Authorization: `Bearer ${process.env.AUTH_TOKEN}`,
    },
  });
  const api = new DocumentsApi(config);

  try {
    // Upload document
    const file = new File(["Hello, World!"], "hello.txt", {
      type: "text/plain",
    });
    const formData = new FormData();
    formData.append("file", file);
    const uploadResult = await api.uploadDocument(formData);
    console.log("Uploaded:", uploadResult);

    // List documents
    const documents = await api.listDocuments();
    console.log("Documents:", documents);

    // Download document
    const content = await api.getDocument(uploadResult.id);
    console.log("Content:", await content.text());
  } catch (error) {
    console.error("Error:", error);
  }
}

main();
```

## 7. Development Best Practices

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check dependencies
cargo audit
```

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add document classification"

# Run tests before pushing
cargo test
git push origin feature/my-feature
```

### Debugging

1. Enable debug logging:

```bash
RUST_LOG=debug,sqlx=debug cargo run
```

2. Use VS Code launch configuration:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Service",
      "cargo": {
        "args": ["build"]
      },
      "args": [],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

## 8. Next Steps

1. **Explore Advanced Features**

   - Document classification
   - OCR processing
   - Custom workflows

2. **Integration Examples**

   - Web application integration
   - Batch processing
   - Event handling

3. **Production Deployment**
   - See `deployment.md` for production setup
   - Configure monitoring
   - Set up backups

## Common Issues

### Database Connection

If you see database connection errors:

```bash
# Check database is running
docker compose ps

# Reset database
docker compose down -v
docker compose up -d db
cargo sqlx database reset
```

### Storage Issues

For local storage issues:

```bash
# Check permissions
ls -l data/documents

# Create storage directory
mkdir -p data/documents
chmod 755 data/documents
```

### JWT Token

If authentication fails:

```bash
# Check token is valid
echo $AUTH_TOKEN | jwt decode -

# Generate new token
./scripts/dev-token.sh
```

## Support

For development questions:

1. Check the troubleshooting guide
2. Search existing issues
3. Open a new issue with:
   - Environment details
   - Steps to reproduce
   - Expected vs actual behavior
