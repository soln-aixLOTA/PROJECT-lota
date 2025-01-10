# Document Automation Service

A microservice for handling document automation workflows, including storage, OCR, classification, and processing.

## Features

- Document upload and retrieval
- Secure storage with multiple backend options (S3, Local, GCS planned)
- Document workflow management
- OCR processing (planned)
- Document classification (planned)
- Security and compliance features

## Prerequisites

- Rust 1.70 or later
- PostgreSQL 13 or later
- Tesseract OCR (for OCR features)
- AWS credentials (for S3 storage)

## Setup

1. Install dependencies:

   ```bash
   # Install Tesseract OCR
   sudo apt-get install tesseract-ocr libtesseract-dev

   # Install PostgreSQL
   sudo apt-get install postgresql postgresql-contrib
   ```

2. Configure the environment:

   ```bash
   # Copy the example config
   cp config/default.example.toml config/default.toml

   # Edit the configuration
   nano config/default.toml
   ```

3. Build the service:
   ```bash
   cargo build --release
   ```

## Running

1. Start the service:

   ```bash
   cargo run --release
   ```

2. The service will be available at `http://localhost:8080`

## API Endpoints

### Health Check

```
GET /health
```

### Upload Document

```
POST /documents
Content-Type: multipart/form-data

Parameters:
- file: The document file to upload
```

### Get Document

```
GET /documents/:id
```

## Configuration

The service can be configured using:

- Configuration files in `config/`
- Environment variables with the prefix `APP_`

Key configuration options:

- `server.port`: HTTP server port
- `storage.provider`: Storage backend (local, s3)
- `database.url`: PostgreSQL connection string
- `security.jwt_secret`: JWT signing key

## Development

1. Run tests:

   ```bash
   cargo test
   ```

2. Run with development settings:
   ```bash
   RUN_MODE=development cargo run
   ```

## Security

- All documents are stored with encryption at rest
- JWT-based authentication
- Role-based access control
- Audit logging

## License

This project is licensed under the MIT License - see the LICENSE file for details.
