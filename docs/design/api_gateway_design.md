1|# API Gateway Design
2|
3|This document details the design of the API Gateway for the LotaBots platform. The API Gateway serves as the single entry point for all client requests and is responsible for routing, authentication, authorization, rate limiting, and other cross-cutting concerns.
4|
5|## Goals
6|- Provide a unified entry point for all client requests.
7|- Implement authentication and authorization for all API endpoints.
8|- Handle routing of requests to the appropriate backend services.
9|- Implement rate limiting to protect backend services.
10|- Provide a layer of abstraction between clients and backend services.
11|- Implement request logging and monitoring.
12|
13|## Architecture
14|The API Gateway is implemented using Rust with the Actix Web framework. It consists of the following key components:
15|
16|- **Reverse Proxy:** Routes incoming requests to the appropriate backend service based on the request path.
17|- **Authentication Middleware:** Verifies the authenticity of client requests using JWT tokens.
18|- **Authorization Middleware:** Enforces access control policies based on user roles and permissions.
19|- **Rate Limiting Middleware:** Limits the number of requests from a single client within a given time window.
20|- **Request Logging Middleware:** Logs all incoming requests and their processing time.
21|- **Metrics Collection Middleware:** Collects metrics on request latency, error rates, etc.
22|
23|## Request Flow
24|1. Client sends a request to the API Gateway.
25|2. The API Gateway receives the request.
26|3. **Authentication Middleware** verifies the JWT token in the request headers.
27|4. **Authorization Middleware** checks if the user has the necessary permissions to access the requested resource.
28|5. **Rate Limiting Middleware** checks if the client has exceeded the allowed request rate.
29|6. **Request Logging Middleware** logs the request.
30|7. The **Reverse Proxy** routes the request to the appropriate backend service.
31|8. The backend service processes the request and returns a response to the API Gateway.
32|9. The API Gateway receives the response.
33|10. **Metrics Collection Middleware** records metrics about the response.
34|11. The API Gateway sends the response back to the client.
35|
36|## Authentication and Authorization
37|- Authentication is based on JWT (JSON Web Tokens). Clients must include a valid JWT in the `Authorization` header.
38|- The API Gateway verifies the signature of the JWT and extracts user information.
39|- Authorization is role-based. User roles and permissions are stored in the User Management Service.
40|- The Authorization Middleware checks if the user's roles grant access to the requested endpoint.
41|
42|## Rate Limiting
43|- Rate limiting is implemented using a token bucket algorithm.
44|- Each client is assigned a bucket with a certain number of tokens.
45|- Each request consumes a token.
46|- Tokens are replenished at a নির্দিষ্ট rate.
47|- If a client's bucket is empty, subsequent requests are rejected.
48|
49|## API Endpoints
50|The API Gateway exposes the following endpoints:
51|
52|- `/api/v1/users`: Routes to the User Management Service.
53|- `/api/v1/models`: Routes to the Training Service.
54|- `/api/v1/infer`: Routes to the Inference Service.
55|
56|## Error Handling
57|The API Gateway implements centralized error handling. When a backend service returns an error, the API Gateway transforms it into a consistent error response format before sending it back to the client. (See `docs/error-handling.md`) 