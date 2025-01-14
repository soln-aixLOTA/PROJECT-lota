# LotaBots API Documentation

## Authentication

The API uses JWT (JSON Web Token) for authentication. To access protected endpoints, include the JWT token in the Authorization header:

```
Authorization: Bearer <your_jwt_token>
```

## Endpoints

### Users

#### GET /users
Get all users (requires admin role)

**Response**
```json
[
  {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "created_at": "datetime",
    "updated_at": "datetime"
  }
]
```

#### POST /users
Create a new user

**Request Body**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Response**
```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

#### GET /users/{id}
Get user by ID (requires authentication)

**Response**
```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

### Authentication

#### POST /auth/login
Login with username/email and password

**Request Body**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response**
```json
{
  "token": "string",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Roles and Permissions

#### GET /roles
Get all roles (requires admin role)

**Response**
```json
[
  {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "permissions": [
      {
        "id": "uuid",
        "name": "string",
        "description": "string"
      }
    ]
  }
]
```

#### POST /roles
Create a new role (requires admin role)

**Request Body**
```json
{
  "name": "string",
  "description": "string",
  "permissions": ["permission_id1", "permission_id2"]
}
```

## Error Responses

The API uses standard HTTP status codes and returns error messages in the following format:

```json
{
  "error": {
    "code": "string",
    "message": "string"
  }
}
```

Common error codes:
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 409: Conflict
- 500: Internal Server Error

## Rate Limiting

The API implements rate limiting per IP address. By default, clients are limited to:
- 60 requests per minute for most endpoints
- 10 requests per minute for authentication endpoints

Rate limit headers are included in the response:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1500000000
``` 