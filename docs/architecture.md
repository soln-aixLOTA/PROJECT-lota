# LotaBots Architecture

LotaBots is built using a microservices architecture, with each service responsible for a specific function. The main components of the architecture are:

## API Gateway

The API Gateway is the entry point for all client requests. It is responsible for:

-   Authentication and authorization.
-   Request routing.
-   Rate limiting.
-   Usage tracking.

The API Gateway is implemented using Actix Web, a high-performance web framework for Rust.

## User Authentication Service

The User Authentication Service is responsible for:

-   User registration.
-   User login.
-   Password management.
-   Token generation.

The User Authentication Service is implemented using Rust and SQLx for database interaction.

## Inference Service

The Inference Service is responsible for:

-   Processing user requests.
-   Generating responses using AI models.
-   Scaling to handle different load levels.

The Inference Service is implemented using Python and NVIDIA AI Enterprise for GPU acceleration.

### Scaling Strategy

The Inference Service is designed to scale horizontally to handle different load levels. We use the following strategies:

-   **Kubernetes Resource Quotas and Limits:** We use Kubernetes resource quotas and limits to allocate different amounts of CPU, memory, and GPU resources to different subscription tiers.
-   **Horizontal Scaling:** We implement horizontal scaling for the Inference Service, allowing it to handle increased load by adding more replicas.
-   **Load Balancing:** We use a load balancer to distribute traffic evenly across Inference Service replicas.

## Usage Tracking System

The usage tracking system is responsible for:

-   Tracking API requests and message volume.
-   Storing usage data in a dedicated database.
-   Exposing usage metrics through a monitoring system.
-   Integrating with billing systems.

The usage tracking system is integrated with the API Gateway and the database.

## Levels of Autonomy

The LotaBots platform supports different levels of autonomy in human-AI interaction:

-   **No AI:** The user is fully in control, and the AI system is not involved.
-   **Assisted:** The AI system provides assistance to the user, but the user remains in control.
-   **Collaborator:** The AI system and the user work together as partners, sharing control and responsibility.
-   **Expert:** The AI system provides expert advice and guidance to the user, but the user retains the final decision-making authority.
-   **Agent:** The AI system operates autonomously, making decisions and taking actions on behalf of the user.

Users can configure the level of autonomy for different tasks and contexts. 