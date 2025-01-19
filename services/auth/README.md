# LotaBots Authentication Service

[![Build Status](https://github.com/your-org/lotabots/actions/workflows/auth-tests.yml/badge.svg)](https://github.com/your-org/lotabots/actions/workflows/auth-tests.yml)
[![codecov](https://codecov.io/gh/your-org/lotabots/branch/main/graph/badge.svg?token=your-token)](https://codecov.io/gh/your-org/lotabots)

This service handles user authentication and authorization for the LotaBots platform. It provides endpoints for user registration, login, and JWT token management.

## Features

- User registration with email and password
- User login with JWT token generation
- Password hashing using bcrypt
- PostgreSQL for user data storage
- Input validation and error handling
- Secure JWT token generation and validation

## API Endpoints

### POST /auth/register

Register a new user.

**Request Body:**
```json
{
    "username": "string",
    "email": "string",
    "password": "string"
}
```

**Response:**
```json
{
    "id": "uuid",
    "username": "string",
    "email": "string",
    "created_at": "datetime"
}
```

### POST /auth/login

Login with username and password.

**Request Body:**
```json
{
    "username": "string",
    "password": "string"
}
```

**Response:**
```json
{
    "token": "string",
    "user_id": "uuid",
    "username": "string"
}
```

## Configuration

The service requires the following environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT token generation
- `SERVER_ADDR`: Server address (default: "127.0.0.1:8080")

## Development Setup

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Set up environment variables (create a `.env` file):
   ```
   DATABASE_URL=postgres://user:password@localhost:5432/lotabots
   JWT_SECRET=  # Generate with: openssl rand -hex 32
   SERVER_ADDR=127.0.0.1:8080
   ```

3. Run database migrations:
   ```bash
   sqlx migrate run
   ```

4. Start the service:
   ```bash
   cargo run
   ```

## Testing

Run the test suite:
```bash
cargo test
```

## Security Considerations

- Passwords are hashed using bcrypt with a work factor of 12
- JWT tokens expire after 24 hours
- All endpoints use HTTPS in production
- Input validation prevents common injection attacks
- Rate limiting should be configured at the API gateway level
